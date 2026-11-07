//! The interface language on the backend side: kept in the database so the first call after a
//! start already speaks it, passed to the lesson core (messages, lesson overlays) and to the
//! editor, and used to re-word journal entries written before the language changed.

use fude_core::control::Call;
use fude_core::i18n::{self, Lang};
use serde_json::json;
use tauri::State;

use crate::db::{Db, JournalEntry};
use crate::editor::Editor;
use crate::error::{AppError, AppResult};
use crate::state::{AppState, lock};

const SETTING: &str = "lang";

/// Applies the saved language at start (Russian when nothing is saved).
pub fn restore(db: &Db) -> AppResult<()> {
    if let Some(lang) = db.setting(SETTING)?.as_deref().and_then(Lang::from_code) {
        i18n::set_lang(lang);
    }
    Ok(())
}

/// Shows PhotoCraft's own menus and panels in FUDE's language.
pub fn apply_to_editor(editor: &mut Editor) {
    let _ = editor.call(&Call::engine("prefs.set", json!({ "path": "interface.language", "value": i18n::lang().code() })));
}

#[tauri::command]
pub fn set_lang(state: State<'_, AppState>, lang: String) -> AppResult<()> {
    let lang = Lang::from_code(&lang).ok_or_else(|| AppError::msg(format!("unknown language “{lang}”")))?;
    lock(&state.db).set_setting(SETTING, lang.code())?;
    i18n::set_lang(lang);
    if let Some(editor) = lock(&state.editor).as_mut() {
        apply_to_editor(editor);
    }
    Ok(())
}

/// Journal wordings in both languages; `{}` is the part that stays (a quoted name, a number).
const JOURNAL: [(&str, &str); 9] = [
    ("Начат урок «{}»", "Started the lesson “{}”"),
    ("Получено достижение «{}»", "Achievement received “{}”"),
    ("Пройден урок «{}»", "Lesson done “{}”"),
    ("Создан урок «{}»", "Lesson created “{}”"),
    ("Импортирован пакет «{}»", "Package imported “{}”"),
    ("+{} XP · без подсказок", "+{} XP · no hints"),
    ("+{} · подсказок: {}", "+{} · hints: {}"),
    ("уроков: {}", "lessons: {}"),
    ("Мастерская", "Workshop"),
];

/// `text` matched against a pattern with `{}` holes: the hole contents, in order.
fn holes<'a>(pattern: &str, text: &'a str) -> Option<Vec<&'a str>> {
    let parts: Vec<&str> = pattern.split("{}").collect();
    let mut rest = text.strip_prefix(parts[0])?;
    let mut out = Vec::new();
    for (i, lit) in parts.iter().enumerate().skip(1) {
        let last = i == parts.len() - 1;
        let at = if last && lit.is_empty() { rest.len() } else { rest.find(lit)? };
        out.push(&rest[..at]);
        rest = &rest[at + lit.len()..];
    }
    rest.is_empty().then_some(out)
}

fn fill(pattern: &str, values: &[&str]) -> String {
    let mut out = String::new();
    for (i, part) in pattern.split("{}").enumerate() {
        if i > 0 {
            out.push_str(values.get(i - 1).copied().unwrap_or_default());
        }
        out.push_str(part);
    }
    out
}

/// A journal line in `lang`, if it was written in the other language with a known wording.
/// English lines saved before English got its own quotes («…» instead of “…”) are recognised too.
pub fn reword(text: &str, lang: Lang) -> String {
    for (ru, en) in JOURNAL {
        let old_en = en.replace('“', "«").replace('”', "»");
        let (from, to) = if lang == Lang::En { ([ru, old_en.as_str()], en) } else { ([en, old_en.as_str()], ru) };
        if let Some(values) = from.iter().find_map(|f| holes(f, text)) {
            return fill(to, &values);
        }
    }
    text.to_string()
}

/// Names the journal quotes, in both languages: lesson titles (the original and its `en.toml`
/// entry) and achievements (titles and descriptions).
pub fn names(state: &AppState) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for c in &state.library().categories {
        for l in &c.lessons {
            let Ok(raw) = fude_core::model::load_lesson(&l.dir) else { continue };
            if let Some(en) = fude_core::locale::Overlay::load(&l.dir, Lang::En).and_then(|o| o.text.get(raw.title.trim()).cloned()) {
                out.push((raw.title, en));
            }
        }
    }
    let facts = crate::progress::Facts { attempts: &[], lessons: &[], today: crate::learn::now().date(), user_lessons: 0 };
    let none = std::collections::HashMap::new();
    let ru = i18n::with_lang(Lang::Ru, || crate::progress::achievements(&facts, &none));
    let en = i18n::with_lang(Lang::En, || crate::progress::achievements(&facts, &none));
    for (r, e) in ru.iter().zip(&en) {
        out.push((r.title.to_string(), e.title.to_string()));
        out.push((r.description.clone(), e.description.clone()));
    }
    out
}

/// `text` with quoted names («…» or “…”) and whole known names put into `lang`. The quotes
/// themselves stay as they are: they belong to the wording around the name.
fn rename(text: &str, lang: Lang, names: &[(String, String)]) -> String {
    let swap = |s: &str| -> Option<String> {
        names.iter().find_map(|(ru, en)| match lang {
            Lang::En if s == ru => Some(en.clone()),
            Lang::Ru if s == en => Some(ru.clone()),
            _ => None,
        })
    };
    if let Some(whole) = swap(text) {
        return whole;
    }
    let mut out = String::new();
    let mut rest = text;
    while let Some(open) = rest.find(['«', '“']) {
        let quote = rest[open..].chars().next().unwrap_or('«');
        let end = if quote == '«' { '»' } else { '”' };
        let after = &rest[open + quote.len_utf8()..];
        let Some(close) = after.find(end) else { break };
        out.push_str(&rest[..open]);
        let name = &after[..close];
        out.push(quote);
        out.push_str(&swap(name).unwrap_or_else(|| name.to_string()));
        out.push(end);
        rest = &after[close + end.len_utf8()..];
    }
    out.push_str(rest);
    out
}

/// Journal entries worded in the current language.
pub fn journal(entries: Vec<JournalEntry>, names: &[(String, String)]) -> Vec<JournalEntry> {
    let lang = i18n::lang();
    entries
        .into_iter()
        .map(|e| JournalEntry { title: rename(&reword(&e.title, lang), lang, names), detail: rename(&reword(&e.detail, lang), lang, names), ..e })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewords_known_journal_lines_both_ways_and_keeps_the_rest() {
        assert_eq!(reword("Пройден урок «Холст и масштаб»", Lang::En), "Lesson done “Холст и масштаб”");
        assert_eq!(reword("Lesson done “Canvas”", Lang::Ru), "Пройден урок «Canvas»");
        assert_eq!(reword("Lesson done «Canvas»", Lang::Ru), "Пройден урок «Canvas»", "saved with the old quotes");
        assert_eq!(reword("Lesson done «Canvas»", Lang::En), "Lesson done “Canvas”", "old quotes brought up to date");
        assert_eq!(reword("+60 XP · без подсказок", Lang::En), "+60 XP · no hints");
        assert_eq!(reword("+60 XP · подсказок: 2", Lang::En), "+60 XP · hints: 2");
        assert_eq!(reword("уроков: 3", Lang::En), "lessons: 3");
        assert_eq!(reword("Мастерская", Lang::En), "Workshop");
        assert_eq!(reword("Пройден урок «Холст»", Lang::Ru), "Пройден урок «Холст»", "already Russian");
        assert_eq!(reword("Что-то своё", Lang::En), "Что-то своё");
    }

    #[test]
    fn puts_quoted_and_whole_names_into_the_language() {
        let names =
            vec![("Холст и масштаб".to_string(), "Canvas and zoom".to_string()), ("Пройдите первый урок".to_string(), "Complete the first lesson".to_string())];
        assert_eq!(rename("Lesson done “Холст и масштаб”", Lang::En, &names), "Lesson done “Canvas and zoom”");
        assert_eq!(rename("Lesson done «Холст и масштаб»", Lang::En, &names), "Lesson done «Canvas and zoom»");
        assert_eq!(rename("Пройден урок «Canvas and zoom»", Lang::Ru, &names), "Пройден урок «Холст и масштаб»");
        assert_eq!(rename("Пройдите первый урок", Lang::En, &names), "Complete the first lesson");
        assert_eq!(rename("«Незнакомое» имя", Lang::En, &names), "«Незнакомое» имя");
    }

    #[test]
    fn keeps_the_language_in_the_database() {
        let db = Db::in_memory(crate::learn::now()).unwrap();
        assert_eq!(db.setting(SETTING).unwrap(), None);
        db.set_setting(SETTING, "en").unwrap();
        db.set_setting(SETTING, "ru").unwrap();
        assert_eq!(db.setting(SETTING).unwrap().as_deref(), Some("ru"));
    }
}
