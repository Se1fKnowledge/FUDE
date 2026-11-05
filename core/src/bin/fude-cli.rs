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

const HELP: &str = "fude — уроки FUDE из командной строки

  fude validate <корень>...                    проверить папки уроков
  fude pack <папка категории> <файл.fude> [--title T] [--lesson <папка урока>]...
  fude info <файл.fude>                        что лежит в пакете
  fude import <файл.fude> <свои уроки> [--builtin <корень>] [--replace]
  fude new-category <свои уроки> <название> [--kanji K] [--accent ai|shu|kaki|kin]
  fude new-lesson <папка категории> <название> [--difficulty 1..5]
  fude guide <папка урока>                     команды гайда в JSON (по строке)
  fude assess <папка урока> <inspect.json> [холст.png] [--confirm <шаг>]...";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("fude: {message}");
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
                    let value = iter.next().ok_or_else(|| format!("у --{name} нет значения"))?;
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
        self.positional.get(i).map(String::as_str).ok_or_else(|| format!("не указан {what}\n\n{HELP}"))
    }
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let Some((command, rest)) = args.split_first() else {
        println!("{HELP}");
        return Ok(ExitCode::SUCCESS);
    };
    let a = Args::parse(rest, &["replace"])?;
    let e = |e: fude_core::Error| e.to_string();
    match command.as_str() {
        "help" | "--help" | "-h" => {
            println!("{HELP}");
            Ok(ExitCode::SUCCESS)
        }
        "validate" => {
            if a.positional.is_empty() {
                return Err(format!("не указан корень уроков\n\n{HELP}"));
            }
            let roots: Vec<Root> = a.positional.iter().map(Root::builtin).collect();
            let library = Library::scan(&roots);
            for c in &library.categories {
                println!("{} «{}»: {} уроков", c.category.id, c.category.title, c.lessons.len());
                for l in &c.lessons {
                    println!("  {} «{}», сложность {}, шагов {}", l.lesson.id, l.lesson.title, l.lesson.difficulty, l.lesson.task.steps.len());
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
            println!("Итого: категорий {}, уроков {lessons}, проблем {}.", library.categories.len(), library.problems.len());
            Ok(if library.problems.is_empty() && lessons > 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE })
        }
        "pack" => {
            let category_dir = PathBuf::from(a.at(0, "папка категории")?);
            let out = PathBuf::from(a.at(1, "файл пакета")?);
            let title = a.get("title").map(Into::into).unwrap_or_else(|| fude_core::model::load_category(&category_dir).map(|c| c.title).unwrap_or_default());
            let lessons = a.all("lesson").into_iter().map(PathBuf::from).collect();
            let report = package::export(&out, &title, &[ExportItem { category_dir, lessons }]).map_err(e)?;
            println!("{}: файлов {}, уроков {} ({})", out.display(), report.files, report.lessons.len(), report.lessons.join(", "));
            for s in report.skipped {
                println!("  пропущен {s}");
            }
            Ok(ExitCode::SUCCESS)
        }
        "info" => {
            let file = PathBuf::from(a.at(0, "файл пакета")?);
            let scratch = std::env::temp_dir();
            let info = package::inspect(&file, &scratch, Limits::default()).map_err(e)?;
            println!("«{}» ({})", info.manifest.title, info.manifest.created_by.unwrap_or_default());
            for c in &info.categories {
                println!("  категория {} «{}»", c.id, c.title);
            }
            for l in &info.lessons {
                println!("  урок {} «{}», сложность {}", l.id, l.title, l.difficulty);
            }
            for i in &info.issues {
                println!("  {i}");
            }
            Ok(if info.issues.iter().any(|i| i.is_error()) { ExitCode::FAILURE } else { ExitCode::SUCCESS })
        }
        "import" => {
            let file = PathBuf::from(a.at(0, "файл пакета")?);
            let user = PathBuf::from(a.at(1, "папка своих уроков")?);
            let mut roots: Vec<Root> = a.all("builtin").into_iter().map(Root::builtin).collect();
            roots.push(Root::user(&user));
            let library = Library::scan(&roots);
            let options = ImportOptions { replace: a.switches.contains("replace") };
            let report = package::import(&file, &library, &user, &options, Limits::default()).map_err(e)?;
            println!("Импортирован «{}»", report.title);
            println!("  новые категории: {}", list(&report.categories_added));
            println!("  новые уроки: {}", list(&report.lessons_added));
            println!("  заменены: {}", list(&report.lessons_replaced));
            for w in report.warnings {
                println!("  {w}");
            }
            Ok(ExitCode::SUCCESS)
        }
        "new-category" => {
            let user = PathBuf::from(a.at(0, "папка своих уроков")?);
            let title = a.at(1, "название")?;
            let accent = match a.get("accent").unwrap_or("ai") {
                "ai" => Accent::Ai,
                "shu" => Accent::Shu,
                "kaki" => Accent::Kaki,
                "kin" => Accent::Kin,
                other => return Err(format!("неизвестный цвет «{other}»: ai, shu, kaki или kin")),
            };
            let (dir, category) = authoring::create_category(&user, title, a.get("kanji"), accent, "").map_err(e)?;
            println!("{} — категория «{}»", dir.display(), category.title);
            Ok(ExitCode::SUCCESS)
        }
        "new-lesson" => {
            let dir = PathBuf::from(a.at(0, "папка категории")?);
            let title = a.at(1, "название")?;
            let difficulty = a.get("difficulty").map(str::parse::<u8>).transpose().map_err(|_| "сложность — число от 1 до 5".to_string())?.unwrap_or(1);
            let (dir, lesson) = authoring::create_lesson(&dir, title, difficulty).map_err(e)?;
            println!("{} — урок {}", dir.display(), lesson.id);
            Ok(ExitCode::SUCCESS)
        }
        "guide" => {
            let dir = PathBuf::from(a.at(0, "папка урока")?);
            let lesson = load_lesson(&dir).map_err(e)?;
            let script = lesson.guide.script.as_ref().ok_or("у урока нет сценария гайда")?;
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
                eprintln!("— {} ({} мс)", step.caption, step.pause_ms());
                for call in &step.calls {
                    println!("{}", call.request(id));
                    id += 1;
                }
            }
            eprintln!("Шагов {}, команд {}, пауз на {} с.", script.steps.len(), script.call_count(), script.duration_ms() / 1000);
            Ok(ExitCode::SUCCESS)
        }
        "assess" => {
            let dir = PathBuf::from(a.at(0, "папка урока")?);
            let inspect = PathBuf::from(a.at(1, "ответ ui.inspect (JSON)")?);
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
            println!("{}", if outcome.passed { "Задание выполнено." } else { "Задание не выполнено." });
            Ok(if outcome.passed { ExitCode::SUCCESS } else { ExitCode::FAILURE })
        }
        other => Err(format!("неизвестная команда «{other}»\n\n{HELP}")),
    }
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn list(items: &[String]) -> String {
    if items.is_empty() { "—".into() } else { items.join(", ") }
}
