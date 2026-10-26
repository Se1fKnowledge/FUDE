# Сборка и запуск редактора (Windows)

## Что нужно установить

| Что | Зачем | Как |
|---|---|---|
| Visual Studio 2022 Build Tools, компонент «Разработка классических приложений на C++» | компоновщик MSVC для Rust | `winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"` |
| Rust 1.95 или новее (через rustup) | компилятор | `winget install Rustlang.Rustup`, затем `rustup default stable` |
| Git | исходники | `winget install Git.Git` |

Проверка:

```bash
rustc --version
cargo --version
```

## Место на диске

Исходники редактора — около 55 МБ, папка сборки `photocraft/target/` вырастает до 10–15 ГБ (отладочная и релизная сборки вместе). Если на диске с проектом мало места, укажите другую папку сборки:

```bash
set CARGO_TARGET_DIR=C:\photocraft-target   # cmd
export CARGO_TARGET_DIR=/c/photocraft-target  # Git Bash
```

## Запуск

```bash
cd photocraft
cargo run --release -p photocraft
```

Первая сборка занимает несколько минут. Редактор открывается в теме **FUDE Тушь**. Переключение темы: **Window › Theme › Dark Theme / Light Theme** или **Edit › Preferences › Interface**.

Полезные ключи запуска:

| Ключ | Что делает |
|---|---|
| `--safe-gpu` | рисовать холст на процессоре (если видеокарта или драйвер не справляются) |
| `--control <порт>` | открыть JSON-канал управления на порту, например `--control 7878` (так FUDE будет управлять редактором) |

## Ядро уроков

Собственный код FUDE — отдельное рабочее пространство Cargo в корне репозитория (`Cargo.toml`); `photocraft/` в него не входит и собирается своим. Из корня:

```bash
cargo build -p fude-core                         # библиотека и CLI fude
cargo run -p fude-core --bin fude -- help        # команды CLI
cargo run -p fude-core --example lesson_art -- lessons   # перерисовать картинки и гайды встроенных уроков
```

Ядро небольшое и собирается за минуту; сборка лежит в `target/` в корне.

Чтобы попробовать гайд урока в настоящем редакторе, запустите PhotoCraft с каналом управления и корнями файлов:

```bash
cd photocraft
cargo run --release -p photocraft -- --control 7878 --control-token-file ../target/control.token --automation-read-root ../lessons/light/03-cast-shadow --automation-write-root ../target/attempt
```

Подключитесь к `127.0.0.1:7878`. Первой строкой отправьте `{"id":"auth","method":"auth","params":{"token":"<содержимое control.token>"}}`, затем — строки, которые печатает `fude guide lessons/light/03-cast-shadow`. На каждую строку редактор отвечает строкой с тем же `id`.

## Снимок интерфейса без окна

```bash
cd photocraft
cargo run --release -p photocraft-ui-egui --example snapshot -- --out ui.png --size 1440x900 --scale 1 --safe-gpu
```

Тема светлая: добавьте `--script '[["ui.set", {"theme": "fudeWashi"}]]'`. Открыть картинку на холсте: `--open путь/к/картинке.png`.

## Шрифты

PhotoCraft необязательно собирается со шрифтами [craft-fonts](https://github.com/storytold/craft-fonts) (японские и другие шрифты интерфейса): склонируйте их рядом и укажите `CRAFT_FONTS_DIR`. Без них всё работает, интерфейс использует встроенные Inter и JetBrains Mono.

## Если что-то не собирается

- `link.exe not found` — не установлен компонент C++ в Build Tools.
- `rustc 1.xx is not supported` — обновите Rust: `rustup update stable`.
- Не хватает места — задайте `CARGO_TARGET_DIR` на другом диске (см. выше).
