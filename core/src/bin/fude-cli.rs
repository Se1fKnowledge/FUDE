//! `fude`: the lesson core from the command line, for authors and for checking the built-in
//! lessons. Run `fude help` for the commands.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use fude_core::assess::{self, Context, EditorState, Snapshot};
use fude_core::library::{Library, Root};
use fude_core::model::{Accent, load_lesson};
use fude_core::package::{self, ExportItem, ImportOptions, Limits};
use fude_core::{authoring, guide};

const HELP_RU: &str = "fude-cli — уроки FUDE из командной строки

  fude-cli validate <корень>...                    проверить папки уроков
  fude-cli pack <папка категории> <файл.fude> [--title T] [--lesson <папка урока>]...
  fude-cli info <файл.fude>                        что лежит в пакете
  fude-cli import <файл.fude> <свои уроки> [--builtin <корень>] [--replace]
  fude-cli new-category <свои уроки> <название> [--kanji K] [--accent ai|shu|kaki|kin]
  fude-cli new-lesson <папка категории> <название> [--difficulty 1..5]
  fude-cli guide <папка урока>                     команды гайда в JSON (по строке)
  fude-cli assess <папка урока> <inspect.json> [холст.png] [--confirm <шаг>]...";
const HELP_EN: &str = "fude-cli — FUDE lessons from the command line

  fude-cli validate <root>...                      check lesson folders
  fude-cli pack <category folder> <file.fude> [--title T] [--lesson <lesson folder>]...
  fude-cli info <file.fude>                        what is in a package
  fude-cli import <file.fude> <own lessons> [--builtin <root>] [--replace]
  fude-cli new-category <own lessons> <title> [--kanji K] [--accent ai|shu|kaki|kin]
  fude-cli new-lesson <category folder> <title> [--difficulty 1..5]
  fude-cli guide <lesson folder>                   guide commands as JSON (one per line)
  fude-cli assess <lesson folder> <inspect.json> [canvas.png] [--confirm <step>]...

Messages are in Russian; set FUDE_LANG=en for English.";

/// The help text in the current language.
fn help() -> &'static str {
    fude_core::i18n::pick(HELP_RU, HELP_EN)
}

fn main() -> ExitCode {
    if let Some(lang) = std::env::var("FUDE_LANG").ok().as_deref().and_then(fude_core::i18n::Lang::from_code) {
        fude_core::i18n::set_lang(lang);
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("fude-cli: {message}");
            ExitCode::from(2)
        }
    }
}

/// Splits `--flag value` options from positional arguments.
struct Args {
    positional: Vec<String>,
    options: Vec<(String, String)>,
    switches: HashSet<String>,
}

impl Args {
    fn parse(args: &[String], switches: &[&str]) -> Result<Args, String> {
        let mut out = Args { positional: vec![], options: vec![], switches: HashSet::new() };
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            if let Some(name) = arg.strip_prefix("--") {
                if switches.contains(&name) {
                    out.switches.insert(name.into());
                } else {
                    let value = iter.next().ok_or_else(|| fude_core::tr!("у --{name} нет значения", "--{name} has no value"))?;
                    out.options.push((name.into(), value.clone()));
                }
            } else {
                out.positional.push(arg.clone());
            }
        }
        Ok(out)
    }

    fn get(&self, name: &str) -> Option<&str> {
        self.options.iter().rev().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
    }

    fn all(&self, name: &str) -> Vec<&str> {
        self.options.iter().filter(|(k, _)| k == name).map(|(_, v)| v.as_str()).collect()
    }

    fn at(&self, i: usize, what: &str) -> Result<&str, String> {
        self.positional.get(i).map(String::as_str).ok_or_else(|| fude_core::tr!("не указан {what}\n\n{}", "{what} is missing\n\n{}", help()))
    }
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let Some((command, rest)) = args.split_first() else {
        println!("{}", help());
        return Ok(ExitCode::SUCCESS);
    };
    let a = Args::parse(rest, &["replace"])?;
    let e = |e: fude_core::Error| e.to_string();
    match command.as_str() {
        "help" | "--help" | "-h" => {
            println!("{}", help());
            Ok(ExitCode::SUCCESS)
        }
        "validate" => {
            if a.positional.is_empty() {
                return Err(fude_core::tr!("не указан корень уроков\n\n{}", "the lessons root is missing\n\n{}", help()));
            }
            let roots: Vec<Root> = a.positional.iter().map(Root::builtin).collect();
            let library = Library::scan(&roots);
            for c in &library.categories {
                println!("{}", fude_core::tr!("{} «{}»: {} уроков", "{} “{}”: {} lessons", c.category.id, c.category.title, c.lessons.len()));
                for l in &c.lessons {
                    println!(
                        "{}",
                        fude_core::tr!(
                            "  {} «{}», сложность {}, шагов {}",
                            "  {} “{}”, difficulty {}, {} steps",
                            l.lesson.id,
                            l.lesson.title,
                            l.lesson.difficulty,
                            l.lesson.task.steps.len()
                        )
                    );
                    for w in &l.warnings {
                        println!("    {w}");
                    }
                }
            }
            for p in &library.problems {
                println!("{}: {}", p.path.display(), p.message);
                for i in &p.issues {
                    println!("  {i}");
                }
            }
            let lessons = library.lessons().count();
            println!(
                "{}",
                fude_core::tr!(
                    "Итого: категорий {}, уроков {lessons}, проблем {}.",
                    "Total: {} categories, {lessons} lessons, {} problems.",
                    library.categories.len(),
                    library.problems.len()
                )
            );
            Ok(if library.problems.is_empty() && lessons > 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE })
        }
        "pack" => {
            let category_dir = PathBuf::from(a.at(0, fude_core::i18n::pick("папка категории", "category folder"))?);
            let out = PathBuf::from(a.at(1, fude_core::i18n::pick("файл пакета", "package file"))?);
            let title = a.get("title").map(Into::into).unwrap_or_else(|| fude_core::model::load_category(&category_dir).map(|c| c.title).unwrap_or_default());
            let lessons = a.all("lesson").into_iter().map(PathBuf::from).collect();
            let report = package::export(&out, &title, &[ExportItem { category_dir, lessons }]).map_err(e)?;
            println!(
                "{}",
                fude_core::tr!(
                    "{}: файлов {}, уроков {} ({})",
                    "{}: {} files, {} lessons ({})",
                    out.display(),
                    report.files,
                    report.lessons.len(),
                    report.lessons.join(", ")
                )
            );
            for s in report.skipped {
                println!("{}", fude_core::tr!("  пропущен {s}", "  skipped {s}"));
            }
            Ok(ExitCode::SUCCESS)
        }
        "info" => {
            let file = PathBuf::from(a.at(0, fude_core::i18n::pick("файл пакета", "package file"))?);
            let scratch = std::env::temp_dir();
            let info = package::inspect(&file, &scratch, Limits::default()).map_err(e)?;
            println!("{} ({})", fude_core::i18n::quote(&info.manifest.title), info.manifest.created_by.unwrap_or_default());
            for c in &info.categories {
                println!("{}", fude_core::tr!("  категория {} «{}»", "  category {} “{}”", c.id, c.title));
            }
            for l in &info.lessons {
                println!("{}", fude_core::tr!("  урок {} «{}», сложность {}", "  lesson {} “{}”, difficulty {}", l.id, l.title, l.difficulty));
            }
            for i in &info.issues {
                println!("  {i}");
            }
            Ok(if info.issues.iter().any(|i| i.is_error()) { ExitCode::FAILURE } else { ExitCode::SUCCESS })
        }
        "import" => {
            let file = PathBuf::from(a.at(0, fude_core::i18n::pick("файл пакета", "package file"))?);
            let user = PathBuf::from(a.at(1, fude_core::i18n::pick("папка своих уроков", "own lessons folder"))?);
            let mut roots: Vec<Root> = a.all("builtin").into_iter().map(Root::builtin).collect();
            roots.push(Root::user(&user));
            let library = Library::scan(&roots);
            let options = ImportOptions { replace: a.switches.contains("replace") };
            let report = package::import(&file, &library, &user, &options, Limits::default()).map_err(e)?;
            println!("{}", fude_core::tr!("Импортирован «{}»", "Imported “{}”", report.title));
            println!("{}", fude_core::tr!("  новые категории: {}", "  new categories: {}", list(&report.categories_added)));
            println!("{}", fude_core::tr!("  новые уроки: {}", "  new lessons: {}", list(&report.lessons_added)));
            println!("{}", fude_core::tr!("  заменены: {}", "  replaced: {}", list(&report.lessons_replaced)));
            for w in report.warnings {
                println!("  {w}");
            }
            Ok(ExitCode::SUCCESS)
        }
        "new-category" => {
            let user = PathBuf::from(a.at(0, fude_core::i18n::pick("папка своих уроков", "own lessons folder"))?);
            let title = a.at(1, fude_core::i18n::pick("название", "title"))?;
            let accent = match a.get("accent").unwrap_or("ai") {
                "ai" => Accent::Ai,
                "shu" => Accent::Shu,
                "kaki" => Accent::Kaki,
                "kin" => Accent::Kin,
                other => return Err(fude_core::tr!("неизвестный цвет «{other}»: ai, shu, kaki или kin", "unknown colour “{other}”: ai, shu, kaki or kin")),
            };
            let (dir, category) = authoring::create_category(&user, title, a.get("kanji"), accent, "").map_err(e)?;
            println!("{}", fude_core::tr!("{} — категория «{}»", "{} — category “{}”", dir.display(), category.title));
            Ok(ExitCode::SUCCESS)
        }
        "new-lesson" => {
            let dir = PathBuf::from(a.at(0, fude_core::i18n::pick("папка категории", "category folder"))?);
            let title = a.at(1, fude_core::i18n::pick("название", "title"))?;
            let difficulty = a
                .get("difficulty")
                .map(str::parse::<u8>)
                .transpose()
                .map_err(|_| fude_core::tr!("сложность — число от 1 до 5", "the difficulty is a number from 1 to 5"))?
                .unwrap_or(1);
            let (dir, lesson) = authoring::create_lesson(&dir, title, difficulty).map_err(e)?;
            println!("{}", fude_core::tr!("{} — урок {}", "{} — lesson {}", dir.display(), lesson.id));
            Ok(ExitCode::SUCCESS)
        }
        "guide" => {
            let dir = PathBuf::from(a.at(0, fude_core::i18n::pick("папка урока", "lesson folder"))?);
            let lesson = load_lesson(&dir).map_err(e)?;
            let script = lesson.guide.script.as_ref().ok_or(fude_core::i18n::pick("у урока нет сценария гайда", "the lesson has no guide script"))?;
            let script = guide::load(&dir.join(script)).map_err(e)?;
            let issues = guide::validate(&script, Some(&dir));
            if issues.iter().any(|i| i.is_error()) {
                return Err(issues.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n"));
            }
            let mut id = 1;
            for call in script.setup() {
                println!("{}", call.request(id));
                id += 1;
            }
            for step in &script.steps {
                eprintln!("{}", fude_core::tr!("— {} ({} мс)", "— {} ({} ms)", step.caption, step.pause_ms()));
                for call in &step.calls {
                    println!("{}", call.request(id));
                    id += 1;
                }
            }
            eprintln!(
                "{}",
                fude_core::tr!(
                    "Шагов {}, команд {}, пауз на {} с.",
                    "{} steps, {} commands, {} s of pauses.",
                    script.steps.len(),
                    script.call_count(),
                    script.duration_ms() / 1000
                )
            );
            Ok(ExitCode::SUCCESS)
        }
        "assess" => {
            let dir = PathBuf::from(a.at(0, fude_core::i18n::pick("папка урока", "lesson folder"))?);
            let inspect = PathBuf::from(a.at(1, fude_core::i18n::pick("ответ ui.inspect (JSON)", "ui.inspect reply (JSON)"))?);
            let lesson = load_lesson(&dir).map_err(e)?;
            let context = Context::for_lesson(&lesson, &dir).map_err(e)?;
            let reply: serde_json::Value = read_json(&inspect)?;
            let canvas = a.positional.get(2).map(|p| assess::load_image(Path::new(p))).transpose().map_err(e)?;
            let snapshot = Snapshot { state: EditorState::from_inspect(&reply), canvas };
            let confirmed: HashSet<String> = a.all("confirm").into_iter().map(Into::into).collect();
            let outcome = assess::task(&lesson.task, &context, &snapshot, &confirmed);
            for (step, s) in lesson.task.steps.iter().zip(&outcome.steps) {
                println!("{} {}", if s.passed { "✓" } else { "·" }, step.text);
                for c in &s.checks {
                    println!("    {} {}: {}", if c.passed { "✓" } else { "✗" }, c.kind, c.message);
                }
            }
            println!(
                "{}",
                if outcome.passed {
                    fude_core::i18n::pick("Задание выполнено.", "Task done.")
                } else {
                    fude_core::i18n::pick("Задание не выполнено.", "Task not done.")
                }
            );
            Ok(if outcome.passed { ExitCode::SUCCESS } else { ExitCode::FAILURE })
        }
        other => Err(fude_core::tr!("неизвестная команда «{other}»\n\n{}", "unknown command “{other}”\n\n{}", help())),
    }
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn list(items: &[String]) -> String {
    if items.is_empty() { "—".into() } else { items.join(", ") }
}
