//! Commands for the learning side: profile, catalog, lesson page, home and the three
//! progress screens, and the result of an attempt.

use std::collections::{BTreeSet, HashSet};

use chrono::{Duration, Local, NaiveDate, NaiveDateTime, Timelike};
use fude_core::library::{LessonEntry, Library};
use fude_core::model::{Accent, Lesson, Skill};
use serde::Serialize;
use tauri::State;

use crate::db::{Attempt, JournalEntry, Profile};
use crate::error::{AppError, AppResult};
use crate::progress::{self, Achievement, Day, LessonInfo, Rank, SkillScore, Streak};
use crate::state::{AppState, lock, picture};

pub fn now() -> NaiveDateTime {
    Local::now().naive_local()
}

// ------------------------------------------------------------------ shared views

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonCard {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub category: String,
    pub category_title: String,
    pub kanji: Option<String>,
    pub accent: Accent,
    pub difficulty: u8,
    pub difficulty_label: String,
    pub minutes: u16,
    pub number: usize,
    pub of: usize,
    pub skills: Vec<Skill>,
    /// `passed`, `progress`, `open` or `locked`.
    pub status: &'static str,
    /// The lesson that has to be passed first, when locked.
    pub after: Option<String>,
    pub stage: Option<String>,
    pub editable: bool,
}

struct Ctx {
    library: Library,
    attempts: Vec<Attempt>,
    passed: HashSet<String>,
    stages: Vec<(String, String)>,
}

fn ctx(state: &AppState) -> AppResult<Ctx> {
    let library = state.library();
    let db = lock(&state.db);
    let attempts = db.attempts()?;
    let passed = progress::passed(&attempts);
    let stages = db.latest_stage()?.into_iter().map(|s| (s.lesson, s.stage)).collect();
    Ok(Ctx { library, attempts, passed, stages })
}

fn card(c: &Ctx, entry: &LessonEntry) -> LessonCard {
    let l = &entry.lesson;
    let category = c.library.category(l.category_id());
    let siblings = category.map_or(0, |cat| cat.lessons.len());
    let number = category.and_then(|cat| cat.lessons.iter().position(|x| x.lesson.id == l.id)).map_or(0, |i| i + 1);
    let stage = c.stages.iter().find(|(id, _)| *id == l.id).map(|(_, s)| s.clone());
    let unlocked = l.requires.iter().all(|r| c.passed.contains(r));
    let status = if c.passed.contains(&l.id) {
        "passed"
    } else if !unlocked {
        "locked"
    } else if stage.is_some() || c.attempts.iter().any(|a| a.lesson == l.id) {
        "progress"
    } else {
        "open"
    };
    let after = l.requires.iter().find(|r| !c.passed.contains(*r)).and_then(|r| c.library.lesson(r)).map(|e| e.lesson.title.clone());
    LessonCard {
        id: l.id.clone(),
        title: l.title.clone(),
        summary: l.summary.clone(),
        category: l.category_id().into(),
        category_title: category.map(|cat| cat.category.title.clone()).unwrap_or_default(),
        kanji: category.and_then(|cat| cat.category.kanji.clone()),
        accent: category.map(|cat| cat.category.accent).unwrap_or_default(),
        difficulty: l.difficulty,
        difficulty_label: l.difficulty_label().into(),
        minutes: l.minutes,
        number,
        of: siblings,
        skills: l.skills.clone(),
        status,
        after,
        stage,
        editable: entry.editable,
    }
}

// ------------------------------------------------------------------ profile

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Boot {
    pub profile: Profile,
    pub first_run: bool,
    pub editor: Option<String>,
    pub version: &'static str,
    pub problems: usize,
    pub rank: Rank,
    pub streak: Streak,
    pub done_today: u32,
}

#[tauri::command]
pub fn boot(state: State<'_, AppState>) -> AppResult<Boot> {
    let c = ctx(&state)?;
    let profile = lock(&state.db).profile()?;
    let today = now().date();
    let days = progress::days(&c.attempts);
    let active: BTreeSet<NaiveDate> = days.keys().copied().collect();
    Ok(Boot {
        first_run: profile.name.is_empty(),
        profile,
        editor: state.executable.as_ref().map(|p| p.display().to_string()),
        version: env!("CARGO_PKG_VERSION"),
        problems: c.library.problems.len(),
        rank: progress::rank(progress::total_xp(&c.attempts)),
        streak: progress::streak(&active, today),
        done_today: days.get(&today).map_or(0, |d| d.tasks),
    })
}

#[tauri::command]
pub fn profile_save(state: State<'_, AppState>, name: String, daily_goal: u32) -> AppResult<Profile> {
    if name.trim().is_empty() {
        return Err(AppError::msg("Как к вам обращаться? Имя не может быть пустым."));
    }
    let db = lock(&state.db);
    db.set_profile(&name, daily_goal)?;
    db.profile()
}

// ------------------------------------------------------------------ catalog

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryView {
    pub id: String,
    pub title: String,
    pub kanji: Option<String>,
    pub description: String,
    pub accent: Accent,
    pub user: bool,
    pub passed: usize,
    pub minutes: u32,
    pub difficulty: (u8, u8),
    pub lessons: Vec<LessonCard>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub categories: Vec<CategoryView>,
    pub total: usize,
    pub passed: usize,
}

#[tauri::command]
pub fn catalog(state: State<'_, AppState>) -> AppResult<Catalog> {
    let c = ctx(&state)?;
    let categories: Vec<CategoryView> = c
        .library
        .categories
        .iter()
        .map(|cat| {
            let lessons: Vec<LessonCard> = cat.lessons.iter().map(|e| card(&c, e)).collect();
            let difficulty =
                (cat.lessons.iter().map(|e| e.lesson.difficulty).min().unwrap_or(1), cat.lessons.iter().map(|e| e.lesson.difficulty).max().unwrap_or(1));
            CategoryView {
                id: cat.category.id.clone(),
                title: cat.category.title.clone(),
                kanji: cat.category.kanji.clone(),
                description: cat.category.description.clone(),
                accent: cat.category.accent,
                user: cat.user_dir.is_some(),
                passed: lessons.iter().filter(|l| l.status == "passed").count(),
                minutes: cat.lessons.iter().map(|e| u32::from(e.lesson.minutes)).sum(),
                difficulty,
                lessons,
            }
        })
        .collect();
    let total = categories.iter().map(|c| c.lessons.len()).sum();
    let passed = categories.iter().map(|c| c.passed).sum();
    Ok(Catalog { categories, total, passed })
}

// ------------------------------------------------------------------ lesson page

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuideStepView {
    pub caption: String,
    pub highlight: Option<String>,
    pub pause: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonView {
    pub card: LessonCard,
    pub lesson: Lesson,
    pub theory: String,
    pub dir: String,
    pub guide: Vec<GuideStepView>,
    pub guide_ms: u64,
    /// Absolute path of the uploaded video, for the webview's asset protocol.
    pub video: Option<String>,
    pub poster: Option<String>,
    pub task_image: Option<String>,
    pub reference_image: Option<String>,
    pub next: Option<LessonCard>,
}

#[tauri::command]
pub fn lesson(state: State<'_, AppState>, id: String) -> AppResult<LessonView> {
    let c = ctx(&state)?;
    let entry = c.library.lesson(&id).ok_or_else(|| AppError::msg(format!("урок «{id}» не найден")))?;
    let l = &entry.lesson;
    let dir = &entry.dir;
    let theory = std::fs::read_to_string(dir.join(&l.theory.file)).unwrap_or_default();
    let script = l.guide.script.as_ref().and_then(|s| fude_core::guide::load(&dir.join(s)).ok());
    let guide = script
        .as_ref()
        .map(|s| s.steps.iter().map(|st| GuideStepView { caption: st.caption.clone(), highlight: st.highlight.clone(), pause: st.pause_ms() }).collect())
        .unwrap_or_default();
    let all: Vec<&LessonEntry> = c.library.lessons().collect();
    let next = all.iter().position(|e| e.lesson.id == id).and_then(|i| all.get(i + 1)).map(|e| card(&c, e));
    Ok(LessonView {
        card: card(&c, entry),
        theory,
        dir: dir.display().to_string(),
        guide,
        guide_ms: script.as_ref().map_or(0, |s| s.duration_ms()),
        video: l.guide.video.as_ref().and_then(|v| fude_core::paths::resolve(dir, v).ok()).filter(|p| p.is_file()).map(|p| p.display().to_string()),
        poster: l.guide.poster.as_ref().and_then(|p| picture(dir, p)),
        task_image: l.task.open.as_ref().and_then(|p| picture(dir, p)),
        reference_image: l.task.reference.as_ref().and_then(|p| picture(dir, p)),
        next,
        lesson: l.clone(),
    })
}

#[tauri::command]
pub fn lesson_stage(state: State<'_, AppState>, id: String, stage: String) -> AppResult<()> {
    let db = lock(&state.db);
    let fresh = db.latest_stage()?.is_none_or(|s| s.lesson != id);
    db.set_stage(&id, &stage, now())?;
    if fresh && stage == "guide" {
        let title = state.library().lesson(&id).map(|e| e.lesson.title.clone()).unwrap_or(id);
        db.log(now(), "start", &format!("Начат урок «{title}»"), "")?;
    }
    Ok(())
}

// ------------------------------------------------------------------ home

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Home {
    pub greeting: &'static str,
    pub name: String,
    pub date: String,
    pub goal: u32,
    pub done_today: u32,
    pub today: Vec<JournalEntry>,
    pub current: Option<LessonCard>,
    pub recommended: Vec<LessonCard>,
    pub streak: Streak,
    pub rank: Rank,
    pub first_day: bool,
}

#[tauri::command]
pub fn home(state: State<'_, AppState>) -> AppResult<Home> {
    let c = ctx(&state)?;
    let db = lock(&state.db);
    let profile = db.profile()?;
    let t = now();
    let today = t.date();
    let days = progress::days(&c.attempts);
    let active: BTreeSet<NaiveDate> = days.keys().copied().collect();
    let prefix = today.format("%Y-%m-%d").to_string();
    let today_log: Vec<JournalEntry> = db.journal(50)?.into_iter().filter(|j| j.at.starts_with(&prefix)).collect();
    let current = db.latest_stage()?.and_then(|s| c.library.lesson(&s.lesson)).map(|e| card(&c, e)).or_else(|| {
        c.library.lessons().find(|e| !c.passed.contains(&e.lesson.id) && e.lesson.requires.iter().all(|r| c.passed.contains(r))).map(|e| card(&c, e))
    });
    let mut recommended = Vec::new();
    for cat in &c.library.categories {
        if let Some(e) = cat.lessons.iter().find(|e| !c.passed.contains(&e.lesson.id) && e.lesson.requires.iter().all(|r| c.passed.contains(r))) {
            if current.as_ref().is_none_or(|cur| cur.id != e.lesson.id) {
                recommended.push(card(&c, e));
            }
        }
    }
    recommended.truncate(3);
    Ok(Home {
        greeting: progress::greeting(t.hour()),
        name: profile.name,
        date: russian_date(today),
        goal: profile.daily_goal,
        done_today: days.get(&today).map_or(0, |d| d.tasks),
        today: today_log,
        current,
        recommended,
        streak: progress::streak(&active, today),
        rank: progress::rank(progress::total_xp(&c.attempts)),
        first_day: c.attempts.is_empty(),
    })
}

pub fn russian_date(d: NaiveDate) -> String {
    use chrono::Datelike;
    const MONTHS: [&str; 12] = ["января", "февраля", "марта", "апреля", "мая", "июня", "июля", "августа", "сентября", "октября", "ноября", "декабря"];
    const DAYS: [&str; 7] = ["Понедельник", "Вторник", "Среда", "Четверг", "Пятница", "Суббота", "Воскресенье"];
    format!("{}, {} {}", DAYS[d.weekday().num_days_from_monday() as usize], d.day(), MONTHS[d.month0() as usize])
}

// ------------------------------------------------------------------ achievements

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Achievements {
    pub rank: Rank,
    pub list: Vec<Achievement>,
    pub unlocked: usize,
}

/// Achievements now, storing any newly reached ones; returns them and the new ones.
pub fn refresh_achievements(state: &AppState) -> AppResult<(Vec<Achievement>, Vec<Achievement>)> {
    let library = state.library();
    let db = lock(&state.db);
    let attempts = db.attempts()?;
    let lessons = progress::lessons(&library);
    let user_lessons = lessons.iter().filter(|l| l.editable).count() as u32;
    let facts = progress::Facts { attempts: &attempts, lessons: &lessons, today: now().date(), user_lessons };
    let mut fresh = Vec::new();
    for a in progress::achievements(&facts, &db.unlocked()?) {
        if a.reached() && a.unlocked.is_none() && db.unlock(a.id, now())? {
            db.log(now(), "achievement", &format!("Получено достижение «{}»", a.title), &a.description)?;
            fresh.push(a);
        }
    }
    let list = progress::achievements(&facts, &db.unlocked()?);
    Ok((list, fresh))
}

#[tauri::command]
pub fn achievements(state: State<'_, AppState>) -> AppResult<Achievements> {
    let (list, _) = refresh_achievements(&state)?;
    let attempts = lock(&state.db).attempts()?;
    Ok(Achievements { rank: progress::rank(progress::total_xp(&attempts)), unlocked: list.iter().filter(|a| a.unlocked.is_some()).count(), list })
}

// ------------------------------------------------------------------ activity

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    /// The last 53 weeks, Monday first, ending this week.
    pub days: Vec<Day>,
    pub today: NaiveDate,
    pub streak: Streak,
    pub active_days: usize,
    pub tasks: u32,
    pub minutes: u32,
    pub week: Vec<Day>,
    pub goal_minutes: u32,
    pub journal: Vec<JournalEntry>,
}

#[tauri::command]
pub fn activity(state: State<'_, AppState>) -> AppResult<Activity> {
    use chrono::Datelike;
    let db = lock(&state.db);
    let attempts = db.attempts()?;
    let map = progress::days(&attempts);
    let today = now().date();
    let monday = today - Duration::days(i64::from(today.weekday().num_days_from_monday()));
    let start = monday - Duration::weeks(52);
    let day = |d: NaiveDate| map.get(&d).cloned().unwrap_or(Day { date: d, tasks: 0, minutes: 0, no_hints: false });
    let days: Vec<Day> = (0..(53 * 7)).map(|i| day(start + Duration::days(i))).collect();
    let year: Vec<&Day> = days.iter().filter(|d| d.date <= today && d.tasks > 0).collect();
    let active: BTreeSet<NaiveDate> = map.keys().copied().collect();
    Ok(Activity {
        streak: progress::streak(&active, today),
        active_days: year.len(),
        tasks: year.iter().map(|d| d.tasks).sum(),
        minutes: year.iter().map(|d| d.minutes).sum(),
        week: (0..7).map(|i| day(monday + Duration::days(i))).collect(),
        goal_minutes: 15 * db.profile()?.daily_goal,
        journal: db.journal(12)?,
        days,
        today,
    })
}

// ------------------------------------------------------------------ statistics

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub skills: Vec<SkillScore>,
    pub weekly: Vec<(NaiveDate, u32)>,
    pub total_xp: u32,
    pub period_xp: u32,
    pub independence: Option<f64>,
    pub hints: u32,
    pub passed: usize,
    pub advice: Option<Advice>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Advice {
    pub text: String,
    pub detail: String,
    pub lesson: Option<String>,
}

/// `period`: `week`, `quarter` or `all`.
#[tauri::command]
pub fn stats(state: State<'_, AppState>, period: String) -> AppResult<Stats> {
    let library = state.library();
    let attempts = lock(&state.db).attempts()?;
    let lessons = progress::lessons(&library);
    let t = now();
    let since = match period.as_str() {
        "week" => Some(t - Duration::days(7)),
        "quarter" => Some(t - Duration::days(91)),
        _ => None,
    };
    let skills = progress::skills(&attempts, &lessons, since);
    let weeks = if period == "week" { 4 } else { 13 };
    let in_period = |a: &&Attempt| since.is_none_or(|s| a.finished.is_some_and(|f| f >= s));
    Ok(Stats {
        weekly: progress::weekly_xp(&attempts, t.date(), weeks),
        total_xp: progress::total_xp(&attempts),
        period_xp: attempts.iter().filter(in_period).map(|a| a.xp).sum(),
        independence: progress::independence(&attempts),
        hints: attempts.iter().filter(in_period).map(|a| a.hints).sum(),
        passed: progress::passed(&attempts).len(),
        advice: advice(&attempts, &lessons, &skills),
        skills,
    })
}

/// A suggestion from the sensei based on real data: repeat the lesson that needed the most
/// hints, or train the weakest skill.
fn advice(attempts: &[Attempt], lessons: &[LessonInfo], skills: &[SkillScore]) -> Option<Advice> {
    let passed = progress::passed(attempts);
    let hinted = attempts
        .iter()
        .filter(|a| a.passed && a.hints > 0)
        .filter(|a| !attempts.iter().any(|b| b.lesson == a.lesson && b.passed && b.hints == 0))
        .max_by_key(|a| a.hints);
    if let Some(a) = hinted {
        let title = lessons.iter().find(|l| l.id == a.lesson).map_or(a.lesson.as_str(), |l| l.title.as_str());
        return Some(Advice {
            text: format!("Больше всего подсказок понадобилось в уроке «{title}»."),
            detail: "Пройдите его ещё раз без подсказок: самостоятельный проход даёт в полтора раза больше опыта и очков навыка.".into(),
            lesson: Some(a.lesson.clone()),
        });
    }
    let weakest = skills.iter().filter(|s| lessons.iter().any(|l| l.skills.contains(&s.skill))).min_by(|a, b| a.score.total_cmp(&b.score))?;
    let lesson = lessons.iter().find(|l| l.skills.contains(&weakest.skill) && !passed.contains(&l.id) && progress::unlocked(l, &passed));
    Some(Advice {
        text: format!("Слабее всего пока «{}».", weakest.label),
        detail: match lesson {
            Some(l) => format!("Следующий шаг — урок «{}».", l.title),
            None => "Откройте уроки, которые развивают этот навык.".into(),
        },
        lesson: lesson.map(|l| l.id.clone()),
    })
}

// ------------------------------------------------------------------ result

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultView {
    pub attempt: i64,
    pub card: Option<LessonCard>,
    pub passed: bool,
    pub seconds: u32,
    pub norm_minutes: u16,
    pub hints: u32,
    pub steps: usize,
    pub xp: u32,
    pub similarity: Option<f64>,
    pub canvas: Option<String>,
    pub reference: Option<String>,
    pub skills: Vec<(String, u32)>,
    pub rank: Rank,
    pub streak: Streak,
    pub next: Option<LessonCard>,
    pub achievements: Vec<Achievement>,
}

#[tauri::command]
pub fn result(state: State<'_, AppState>, attempt: i64) -> AppResult<ResultView> {
    let (_, fresh) = refresh_achievements(&state)?;
    let c = ctx(&state)?;
    let a = c.attempts.iter().find(|a| a.id == attempt).cloned().ok_or_else(|| AppError::msg("попытка не найдена"))?;
    let entry = c.library.lesson(&a.lesson);
    let steps = entry.map_or(0, |e| e.lesson.task.steps.len());
    let canvas = picture(&state.attempts_dir(), &format!("attempt-{attempt}.png"));
    let reference = entry.and_then(|e| e.lesson.task.reference.as_ref().and_then(|r| picture(&e.dir, r)));
    let skills = entry
        .map(|e| e.lesson.skills.iter().map(|s| (s.label().to_string(), 10 * u32::from(e.lesson.difficulty) * if a.hints == 0 { 3 } else { 2 } / 2)).collect())
        .unwrap_or_default();
    let all: Vec<&LessonEntry> = c.library.lessons().collect();
    let next = all.iter().position(|e| e.lesson.id == a.lesson).and_then(|i| all.get(i + 1)).map(|e| card(&c, e));
    let active: BTreeSet<NaiveDate> = progress::days(&c.attempts).into_keys().collect();
    Ok(ResultView {
        attempt,
        card: entry.map(|e| card(&c, e)),
        passed: a.passed,
        seconds: a.seconds,
        norm_minutes: entry.map_or(0, |e| e.lesson.minutes),
        hints: a.hints,
        steps,
        xp: a.xp,
        similarity: a.similarity,
        canvas,
        reference,
        skills,
        rank: progress::rank(progress::total_xp(&c.attempts)),
        streak: progress::streak(&active, now().date()),
        next,
        achievements: fresh,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    #[test]
    fn dates_read_in_russian() {
        assert_eq!(russian_date(NaiveDate::from_ymd_opt(2026, 10, 25).unwrap()), "Воскресенье, 25 октября");
        assert_eq!(russian_date(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()), "Четверг, 1 января");
    }

    #[test]
    fn advice_points_at_the_most_hinted_lesson_or_the_weakest_skill() {
        let t = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
        let info = |id: &str, skill: Skill, requires: &[&str]| LessonInfo {
            id: id.into(),
            title: format!("Урок {id}"),
            category: "x".into(),
            difficulty: 1,
            minutes: 5,
            skills: vec![skill],
            requires: requires.iter().map(|s| s.to_string()).collect(),
            editable: false,
        };
        let lessons = vec![info("x.a", Skill::Tools, &[]), info("x.b", Skill::Light, &[])];
        let pass = |lesson: &str, hints: u32| Attempt {
            id: 1,
            lesson: lesson.into(),
            started: t,
            finished: Some(t),
            passed: true,
            hints,
            seconds: 60,
            xp: 10,
            similarity: None,
        };
        let attempts = vec![pass("x.a", 3)];
        let skills = progress::skills(&attempts, &lessons, None);
        let a = advice(&attempts, &lessons, &skills).unwrap();
        assert_eq!(a.lesson.as_deref(), Some("x.a"));
        let attempts = vec![pass("x.a", 3), pass("x.a", 0)];
        let a = advice(&attempts, &lessons, &progress::skills(&attempts, &lessons, None)).unwrap();
        assert_eq!(a.lesson.as_deref(), Some("x.b"), "redone cleanly: train the weakest skill instead");
        assert!(a.text.contains("Свет и тень"));
    }

    #[test]
    fn a_fresh_profile_starts_on_the_first_day() {
        let t = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
        let db = Db::in_memory(t).unwrap();
        assert!(db.profile().unwrap().name.is_empty());
        assert!(progress::passed(&db.attempts().unwrap()).is_empty());
    }
}
