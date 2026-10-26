//! What the progress screens show, computed from the attempts in the database: experience
//! and kyu/dan ranks, day streaks, the activity calendar, skill growth, independence from
//! hints and achievements. Everything here is a pure function of its inputs.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike};
use fude_core::library::Library;
use fude_core::model::Skill;
use serde::Serialize;

use crate::db::Attempt;

/// What progress needs to know about a lesson.
#[derive(Debug, Clone, PartialEq)]
pub struct LessonInfo {
    pub id: String,
    pub title: String,
    pub category: String,
    pub difficulty: u8,
    pub minutes: u16,
    pub skills: Vec<Skill>,
    pub requires: Vec<String>,
    pub editable: bool,
}

pub fn lessons(library: &Library) -> Vec<LessonInfo> {
    library
        .lessons()
        .map(|e| LessonInfo {
            id: e.lesson.id.clone(),
            title: e.lesson.title.clone(),
            category: e.lesson.category_id().to_string(),
            difficulty: e.lesson.difficulty,
            minutes: e.lesson.minutes,
            skills: e.lesson.skills.clone(),
            requires: e.lesson.requires.clone(),
            editable: e.editable,
        })
        .collect()
}

// ------------------------------------------------------------------ experience and ranks

/// Experience for a finished task: harder lessons give more, a task done without hints
/// gives half as much again, and repeating a passed lesson gives a quarter.
pub fn xp_for(difficulty: u8, hints: u32, first_pass: bool) -> u32 {
    let base = 40 + 20 * u32::from(difficulty.clamp(1, 5));
    let base = if hints == 0 { base * 3 / 2 } else { base };
    if first_pass { base } else { base / 4 }
}

/// Total experience needed for each rank, from 10 kyu to 1 dan.
pub const RANKS: [u32; 11] = [0, 150, 350, 600, 900, 1250, 1650, 2100, 2600, 3150, 3800];

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rank {
    /// 0 is 10 kyu, 10 is 1 dan.
    pub index: usize,
    /// «4 кю», «1 дан».
    pub name: String,
    pub title: String,
    pub xp: u32,
    pub floor: u32,
    /// Experience of the next rank, `None` at the top.
    pub next: Option<u32>,
}

pub fn rank_name(index: usize) -> String {
    if index >= 10 { "1 дан".into() } else { format!("{} кю", 10 - index) }
}

pub fn rank(xp: u32) -> Rank {
    let index = RANKS.iter().rposition(|&floor| xp >= floor).unwrap_or(0);
    let title = match index {
        0..=3 => "Новичок",
        4..=6 => "Ученик кисти",
        7..=9 => "Подмастерье",
        _ => "Мастер",
    };
    Rank { index, name: rank_name(index), title: title.into(), xp, floor: RANKS[index], next: RANKS.get(index + 1).copied() }
}

pub fn total_xp(attempts: &[Attempt]) -> u32 {
    attempts.iter().map(|a| a.xp).sum()
}

/// Lessons passed at least once.
pub fn passed(attempts: &[Attempt]) -> HashSet<String> {
    attempts.iter().filter(|a| a.passed).map(|a| a.lesson.clone()).collect()
}

/// Whether a lesson is open: everything it requires is passed.
pub fn unlocked(lesson: &LessonInfo, passed: &HashSet<String>) -> bool {
    lesson.requires.iter().all(|r| passed.contains(r))
}

// ------------------------------------------------------------------ days and streaks

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Day {
    pub date: NaiveDate,
    /// Tasks finished (passed) that day.
    pub tasks: u32,
    pub minutes: u32,
    /// Every finished task of the day was done without hints.
    pub no_hints: bool,
}

/// Finished tasks per day.
pub fn days(attempts: &[Attempt]) -> BTreeMap<NaiveDate, Day> {
    let mut map: BTreeMap<NaiveDate, Day> = BTreeMap::new();
    for a in attempts.iter().filter(|a| a.passed) {
        let Some(at) = a.finished else { continue };
        let date = at.date();
        let day = map.entry(date).or_insert_with(|| Day { date, tasks: 0, minutes: 0, no_hints: true });
        day.tasks += 1;
        day.minutes += a.seconds.div_ceil(60);
        day.no_hints &= a.hints == 0;
    }
    map
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Streak {
    /// Days in a row ending today, or yesterday if today has nothing yet.
    pub current: u32,
    pub best: u32,
    /// The last day of the best streak.
    pub best_end: Option<NaiveDate>,
}

pub fn streak(active: &BTreeSet<NaiveDate>, today: NaiveDate) -> Streak {
    let mut best = 0;
    let mut best_end = None;
    let mut run = 0;
    let mut prev: Option<NaiveDate> = None;
    for &d in active {
        run = if prev.is_some_and(|p| p.succ_opt() == Some(d)) { run + 1 } else { 1 };
        if run >= best {
            best = run;
            best_end = Some(d);
        }
        prev = Some(d);
    }
    let mut current = 0;
    let start = if active.contains(&today) { Some(today) } else { today.pred_opt().filter(|y| active.contains(y)) };
    let mut day = start;
    while let Some(d) = day.filter(|d| active.contains(d)) {
        current += 1;
        day = d.pred_opt();
    }
    Streak { current, best, best_end }
}

// ------------------------------------------------------------------ skills

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillScore {
    pub skill: Skill,
    pub label: String,
    /// 0–100: share of everything this skill can earn in the library.
    pub score: f64,
    /// The same at the start of the period.
    pub before: f64,
    pub points: u32,
    pub level: u32,
}

/// Points a passed lesson gives each of its skills.
fn skill_points(difficulty: u8, no_hints: bool) -> u32 {
    let base = 10 * u32::from(difficulty.clamp(1, 5));
    if no_hints { base * 3 / 2 } else { base }
}

/// Skill scores now and at `since` (the start of the statistics period).
pub fn skills(attempts: &[Attempt], lessons: &[LessonInfo], since: Option<NaiveDateTime>) -> Vec<SkillScore> {
    let by_id: HashMap<&str, &LessonInfo> = lessons.iter().map(|l| (l.id.as_str(), l)).collect();
    let earned = |until: Option<NaiveDateTime>| {
        // Best result per lesson: any pass without hints beats a pass with them.
        let mut best: HashMap<&str, bool> = HashMap::new();
        for a in attempts.iter().filter(|a| a.passed && until.is_none_or(|u| a.finished.is_some_and(|f| f < u))) {
            let clean = best.entry(a.lesson.as_str()).or_insert(false);
            *clean |= a.hints == 0;
        }
        let mut points: HashMap<Skill, u32> = HashMap::new();
        for (id, clean) in best {
            if let Some(l) = by_id.get(id) {
                for s in &l.skills {
                    *points.entry(*s).or_default() += skill_points(l.difficulty, clean);
                }
            }
        }
        points
    };
    let mut possible: HashMap<Skill, u32> = HashMap::new();
    for l in lessons {
        for s in &l.skills {
            *possible.entry(*s).or_default() += skill_points(l.difficulty, true);
        }
    }
    let now = earned(None);
    let then = since.map(|s| earned(Some(s))).unwrap_or_default();
    Skill::ALL
        .iter()
        .map(|&skill| {
            let max = f64::from(possible.get(&skill).copied().unwrap_or(0).max(1));
            let points = now.get(&skill).copied().unwrap_or(0);
            SkillScore {
                skill,
                label: skill.label().into(),
                score: (f64::from(points) / max * 100.0).min(100.0),
                before: (f64::from(then.get(&skill).copied().unwrap_or(0)) / max * 100.0).min(100.0),
                points,
                level: points / 30 + 1,
            }
        })
        .collect()
}

/// Share (0–1) of passed tasks done without hints, or `None` before the first one.
pub fn independence(attempts: &[Attempt]) -> Option<f64> {
    let passed: Vec<&Attempt> = attempts.iter().filter(|a| a.passed).collect();
    (!passed.is_empty()).then(|| passed.iter().filter(|a| a.hints == 0).count() as f64 / passed.len() as f64)
}

/// Experience per week (Monday-based), oldest first, for the last `weeks` weeks.
pub fn weekly_xp(attempts: &[Attempt], today: NaiveDate, weeks: u32) -> Vec<(NaiveDate, u32)> {
    let monday = today - Duration::days(i64::from(today.weekday().num_days_from_monday()));
    let mut out: Vec<(NaiveDate, u32)> = (0..weeks).rev().map(|w| (monday - Duration::weeks(i64::from(w)), 0)).collect();
    for a in attempts {
        let Some(f) = a.finished else { continue };
        let d = f.date();
        if let Some(slot) = out.iter_mut().rev().find(|(start, _)| d >= *start) {
            if d < slot.0 + Duration::weeks(1) {
                slot.1 += a.xp;
            }
        }
    }
    out
}

// ------------------------------------------------------------------ achievements

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Achievement {
    pub id: &'static str,
    pub kanji: &'static str,
    pub title: &'static str,
    pub description: String,
    pub group: &'static str,
    pub current: u32,
    pub target: u32,
    /// When it was unlocked (stored by the caller); `None` while locked.
    pub unlocked: Option<String>,
}

impl Achievement {
    pub fn reached(&self) -> bool {
        self.current >= self.target
    }
}

/// Everything achievements are computed from.
pub struct Facts<'a> {
    pub attempts: &'a [Attempt],
    pub lessons: &'a [LessonInfo],
    pub today: NaiveDate,
    /// Built-in category ids by their role.
    pub user_lessons: u32,
}

pub fn achievements(f: &Facts, unlocked: &HashMap<String, String>) -> Vec<Achievement> {
    let passed = passed(f.attempts);
    let active: BTreeSet<NaiveDate> = days(f.attempts).into_keys().collect();
    let streak = streak(&active, f.today);
    let best_streak = streak.best.max(streak.current);
    let clean = f.attempts.iter().filter(|a| a.passed && a.hints == 0).count() as u32;
    let early = f.attempts.iter().filter(|a| a.passed && a.finished.is_some_and(|t| t.hour() < 9)).count() as u32;
    let by_id: HashMap<&str, &LessonInfo> = f.lessons.iter().map(|l| (l.id.as_str(), l)).collect();
    let quick = f
        .attempts
        .iter()
        .filter(|a| a.passed && by_id.get(a.lesson.as_str()).is_some_and(|l| l.minutes > 0 && a.seconds > 0 && a.seconds < u32::from(l.minutes) * 60))
        .count() as u32;
    let category = |id: &str| {
        let all: Vec<&LessonInfo> = f.lessons.iter().filter(|l| l.category == id && !l.editable).collect();
        (all.iter().filter(|l| passed.contains(&l.id)).count() as u32, all.len().max(1) as u32)
    };
    let (basics, basics_n) = category("basics");
    let (light, light_n) = category("light");
    let core_shadow = f.attempts.iter().any(|a| a.passed && a.hints == 0 && a.lesson == "light.core-shadow") as u32;
    let dan = (rank(total_xp(f.attempts)).index >= 10) as u32;

    let list = [
        ("first", "一", "Первый мазок", "Пройдите первый урок".to_string(), "Мастерство", passed.len().min(1) as u32, 1),
        ("streak7", "火", "Огонь в сердце", "Семь дней практики подряд".into(), "Серии", best_streak, 7),
        ("core-shadow", "影", "Первая тень", "«Собственная тень и полутон» без подсказок".into(), "Самостоятельность", core_shadow, 1),
        ("basics", "基", "Крепкий фундамент", "Вся категория «Основы интерфейса»".into(), "Мастерство", basics, basics_n),
        ("early", "朝", "Ранняя птица", "Пять уроков до 9:00".into(), "Серии", early, 5),
        ("self", "独", "Сам себе сэнсэй", "Десять заданий без подсказок".into(), "Самостоятельность", clean, 10),
        ("light", "光", "Свет мастера", "Вся категория «Свет и тень»".into(), "Мастерство", light, light_n),
        ("quick", "速", "Быстрая рука", "Три урока быстрее их нормы времени".into(), "Скорость", quick, 3),
        ("season", "季", "Сезон кисти", "Тридцать дней практики подряд".into(), "Серии", best_streak, 30),
        ("author", "創", "Свой урок", "Создайте урок в мастерской".into(), "Творчество", f.user_lessons.min(1), 1),
        ("sensei", "師", "Сэнсэй", "Получите 1 дан".into(), "Мастерство", dan, 1),
    ];
    list.into_iter()
        .map(|(id, kanji, title, description, group, current, target)| Achievement {
            id,
            kanji,
            title,
            description,
            group,
            current: current.min(target),
            target,
            unlocked: unlocked.get(id).cloned(),
        })
        .collect()
}

// ------------------------------------------------------------------ greetings

pub fn greeting(hour: u32) -> &'static str {
    match hour {
        5..=11 => "Доброе утро",
        12..=16 => "Добрый день",
        17..=22 => "Добрый вечер",
        _ => "Доброй ночи",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
    }

    fn t(day: u32, hour: u32) -> NaiveDateTime {
        d(day).and_hms_opt(hour, 0, 0).unwrap()
    }

    fn attempt(id: i64, lesson: &str, day: u32, hour: u32, hints: u32, xp: u32) -> Attempt {
        Attempt { id, lesson: lesson.into(), started: t(day, hour), finished: Some(t(day, hour)), passed: true, hints, seconds: 300, xp, similarity: None }
    }

    fn info(id: &str, category: &str, difficulty: u8, skills: &[Skill], requires: &[&str]) -> LessonInfo {
        LessonInfo {
            id: id.into(),
            title: id.into(),
            category: category.into(),
            difficulty,
            minutes: 10,
            skills: skills.to_vec(),
            requires: requires.iter().map(|s| s.to_string()).collect(),
            editable: false,
        }
    }

    #[test]
    fn experience_rewards_difficulty_independence_and_first_passes() {
        assert_eq!(xp_for(1, 1, true), 60);
        assert_eq!(xp_for(1, 0, true), 90);
        assert_eq!(xp_for(5, 0, true), 210);
        assert_eq!(xp_for(5, 2, false), 35);
        assert_eq!(xp_for(0, 1, true), 60, "difficulty is clamped");
    }

    #[test]
    fn ranks_go_from_ten_kyu_to_one_dan() {
        assert_eq!(rank(0).name, "10 кю");
        assert_eq!(rank(0).next, Some(150));
        assert_eq!(rank(149).index, 0);
        assert_eq!(rank(150).name, "9 кю");
        assert_eq!(rank(1700).name, "4 кю");
        assert_eq!(rank(1700).title, "Ученик кисти");
        let top = rank(10_000);
        assert_eq!(top.name, "1 дан");
        assert_eq!(top.title, "Мастер");
        assert_eq!(top.next, None);
        assert!(RANKS.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn streaks_count_back_from_today_or_yesterday() {
        let days: BTreeSet<NaiveDate> = [1, 2, 3, 5, 6, 7, 8, 20, 21].into_iter().map(d).collect();
        let s = streak(&days, d(21));
        assert_eq!(s.current, 2);
        assert_eq!(s.best, 4);
        assert_eq!(s.best_end, Some(d(8)));
        assert_eq!(streak(&days, d(22)).current, 2, "yesterday keeps the streak alive");
        assert_eq!(streak(&days, d(23)).current, 0);
        assert_eq!(streak(&BTreeSet::new(), d(1)), Streak::default());
    }

    #[test]
    fn days_sum_tasks_minutes_and_hint_free_days() {
        let mut a = vec![attempt(1, "a.x", 3, 10, 0, 90), attempt(2, "a.y", 3, 11, 1, 60), attempt(3, "a.z", 4, 9, 0, 90)];
        a.push(Attempt { passed: false, ..attempt(4, "a.w", 4, 12, 0, 0) });
        let map = days(&a);
        assert_eq!(map.len(), 2);
        assert_eq!(map[&d(3)].tasks, 2);
        assert_eq!(map[&d(3)].minutes, 10);
        assert!(!map[&d(3)].no_hints);
        assert!(map[&d(4)].no_hints);
        assert_eq!(map[&d(4)].tasks, 1, "a failed attempt is not a finished task");
    }

    #[test]
    fn skills_grow_with_passed_lessons_and_show_the_period_start() {
        let lessons = vec![
            info("b.one", "b", 1, &[Skill::Tools], &[]),
            info("b.two", "b", 2, &[Skill::Tools, Skill::Light], &["b.one"]),
            info("l.one", "l", 4, &[Skill::Light], &[]),
        ];
        let attempts = vec![attempt(1, "b.one", 1, 10, 1, 60), attempt(2, "b.one", 5, 10, 0, 22), attempt(3, "b.two", 6, 10, 2, 80)];
        let s = skills(&attempts, &lessons, Some(t(4, 0)));
        let tools = &s[0];
        assert_eq!(tools.skill, Skill::Tools);
        // b.one passed clean on the second try (15), b.two with hints (20); possible 15 + 30.
        assert_eq!(tools.points, 35);
        assert!((tools.score - 35.0 / 45.0 * 100.0).abs() < 1e-9);
        assert!((tools.before - 10.0 / 45.0 * 100.0).abs() < 1e-9, "before day 4 only the hinted pass of b.one");
        let light = s.iter().find(|x| x.skill == Skill::Light).unwrap();
        assert_eq!(light.points, 20);
        assert_eq!(s.iter().find(|x| x.skill == Skill::Color).unwrap().score, 0.0);
        assert_eq!(s.len(), 6);
    }

    #[test]
    fn independence_is_the_share_of_hint_free_passes() {
        assert_eq!(independence(&[]), None);
        let a = vec![attempt(1, "a.x", 1, 10, 0, 1), attempt(2, "a.y", 1, 11, 3, 1), attempt(3, "a.z", 1, 12, 0, 1), attempt(4, "a.w", 1, 13, 0, 1)];
        assert_eq!(independence(&a), Some(0.75));
    }

    #[test]
    fn weekly_experience_buckets_by_monday() {
        // 26 October 2026 is a Monday.
        let a = vec![attempt(1, "a.x", 26, 10, 0, 90), attempt(2, "a.y", 25, 10, 0, 60), attempt(3, "a.z", 19, 10, 0, 30), attempt(4, "a.w", 1, 10, 0, 999)];
        let w = weekly_xp(&a, d(28), 2);
        assert_eq!(w, vec![(d(19), 90), (d(26), 90)]);
    }

    #[test]
    fn locked_lessons_need_their_requirements() {
        let l = info("b.two", "b", 1, &[], &["b.one"]);
        assert!(!unlocked(&l, &HashSet::new()));
        assert!(unlocked(&l, &["b.one".to_string()].into()));
    }

    #[test]
    fn achievements_follow_the_facts_and_keep_unlock_times() {
        let lessons = vec![
            info("basics.a", "basics", 1, &[Skill::Tools], &[]),
            info("basics.b", "basics", 1, &[Skill::Tools], &[]),
            info("light.core-shadow", "light", 3, &[Skill::Light], &[]),
        ];
        let attempts = vec![attempt(1, "basics.a", 1, 8, 1, 60), attempt(2, "basics.b", 2, 8, 0, 90), attempt(3, "light.core-shadow", 3, 15, 0, 150)];
        let facts = Facts { attempts: &attempts, lessons: &lessons, today: d(3), user_lessons: 0 };
        let unlocked: HashMap<String, String> = [("first".to_string(), "2026-10-01T08:00:00".to_string())].into();
        let list = achievements(&facts, &unlocked);
        let get = |id: &str| list.iter().find(|a| a.id == id).unwrap();
        assert!(get("first").reached());
        assert_eq!(get("first").unlocked.as_deref(), Some("2026-10-01T08:00:00"));
        assert!(get("basics").reached());
        assert!(get("core-shadow").reached());
        assert_eq!((get("streak7").current, get("streak7").target), (3, 7));
        assert_eq!(get("early").current, 2);
        assert_eq!(get("self").current, 2);
        assert!(!get("author").reached());
        assert!(!get("sensei").reached());
        assert_eq!(get("light").target, 1, "only the light lessons in the library count");
        let ids: HashSet<&str> = list.iter().map(|a| a.id).collect();
        assert_eq!(ids.len(), list.len());
    }

    #[test]
    fn greetings_follow_the_clock() {
        assert_eq!(greeting(7), "Доброе утро");
        assert_eq!(greeting(13), "Добрый день");
        assert_eq!(greeting(20), "Добрый вечер");
        assert_eq!(greeting(2), "Доброй ночи");
    }
}
