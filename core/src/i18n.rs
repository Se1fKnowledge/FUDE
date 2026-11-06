//! The language of messages: Russian (the source language) or English.
//!
//! The app sets the language once per change with [`set_lang`]; every message the core makes
//! — check results for the student, validation errors for authors, CLI output — goes through
//! [`tr!`](crate::tr), which picks the Russian or the English wording. A test can switch the
//! language for its own thread with [`with_lang`] without disturbing tests running alongside.

use std::cell::Cell;
use std::sync::atomic::{AtomicU8, Ordering};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[default]
    Ru,
    En,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::Ru => "ru",
            Lang::En => "en",
        }
    }

    /// `ru`, `en` (any case, `en-US` too); anything else is not a language FUDE speaks.
    pub fn from_code(code: &str) -> Option<Lang> {
        let primary = code.split(['-', '_']).next().unwrap_or_default().to_ascii_lowercase();
        match primary.as_str() {
            "ru" => Some(Lang::Ru),
            "en" => Some(Lang::En),
            _ => None,
        }
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

thread_local! {
    static OVERRIDE: Cell<Option<Lang>> = const { Cell::new(None) };
}

/// The language messages are made in: this thread's override, else the app-wide choice.
pub fn lang() -> Lang {
    OVERRIDE.with(Cell::get).unwrap_or(if CURRENT.load(Ordering::Relaxed) == 1 { Lang::En } else { Lang::Ru })
}

/// The app-wide language.
pub fn set_lang(lang: Lang) {
    CURRENT.store(u8::from(lang == Lang::En), Ordering::Relaxed);
}

/// Runs `f` with this thread speaking `lang`.
pub fn with_lang<T>(lang: Lang, f: impl FnOnce() -> T) -> T {
    let before = OVERRIDE.with(|o| o.replace(Some(lang)));
    let out = f();
    OVERRIDE.with(|o| o.set(before));
    out
}

/// One of two texts by the current language.
pub fn pick<T>(ru: T, en: T) -> T {
    match lang() {
        Lang::Ru => ru,
        Lang::En => en,
    }
}

/// A message in the current language: `tr!("урок «{id}» не найден", "lesson «{id}» not found")`.
/// Both texts are format strings and may use the variables in scope or extra arguments.
#[macro_export]
macro_rules! tr {
    ($ru:literal, $en:literal $(,)?) => {
        match $crate::i18n::lang() {
            $crate::i18n::Lang::Ru => format!($ru),
            $crate::i18n::Lang::En => format!($en),
        }
    };
    ($ru:literal, $en:literal, $($arg:tt)+) => {
        match $crate::i18n::lang() {
            $crate::i18n::Lang::Ru => format!($ru, $($arg)+),
            $crate::i18n::Lang::En => format!($en, $($arg)+),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_by_language_and_overrides_per_thread() {
        let id = "light.sphere";
        assert_eq!(with_lang(Lang::Ru, || tr!("урок «{id}» не найден", "lesson «{id}» not found")), "урок «light.sphere» не найден");
        assert_eq!(with_lang(Lang::En, || tr!("урок «{id}» не найден", "lesson «{id}» not found")), "lesson «light.sphere» not found");
        assert_eq!(with_lang(Lang::En, || tr!("{} из {}", "{} of {}", 2, 5)), "2 of 5");
        assert_eq!(with_lang(Lang::En, || with_lang(Lang::Ru, lang)), Lang::Ru);
    }

    #[test]
    fn reads_language_codes() {
        assert_eq!(Lang::from_code("en-US"), Some(Lang::En));
        assert_eq!(Lang::from_code("RU"), Some(Lang::Ru));
        assert_eq!(Lang::from_code("de"), None);
        assert_eq!(Lang::En.code(), "en");
    }
}
