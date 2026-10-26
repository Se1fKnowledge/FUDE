# Редактор: PhotoCraft в стиле FUDE

Папка [`photocraft/`](../photocraft/) — копия [storytold/photocraft](https://github.com/storytold/photocraft) на коммите `c65b5d2`, с изменениями FUDE поверх. PhotoCraft — открытый редактор изображений на Rust (egui + wgpu), повторяющий Photoshop: слои, маски, корректирующие слои, стили, текст, векторы, кисти и PSD. Лицензия — MIT OR Apache-2.0.

История в git разделена: сначала коммит с нетронутым PhotoCraft, затем отдельные коммиты FUDE. Поэтому `git log -- photocraft` и `git diff` от первого коммита показывают ровно то, что изменили мы.

## Что изменено

### 1. Темы FUDE Тушь и FUDE Васи

В PhotoCraft все цвета интерфейса берутся из одного набора токенов — [`photocraft/crates/ui-egui/src/theme.rs`](../photocraft/crates/ui-egui/src/theme.rs). Чужие темы мы не перекрашивали, а добавили две свои и сделали их темами по умолчанию:

| Тема | Идентификатор | Семейство | Основа |
|---|---|---|---|
| FUDE Тушь (`FUDE Ink`) | `fudeInk` | тёмная, по умолчанию | токены `sumi/*` |
| FUDE Васи (`FUDE Washi`) | `fudeWashi` | светлая, по умолчанию | токены `paper/*` |

Обе построены на раскладке **Pro** (как в Photoshop: плоские панели, вкладки), поэтому поменялись только цвета и скругления (4 / 6 / 8 px вместо 3 / 4 / 6). Остальные 14 тем PhotoCraft остались в меню.

Соответствие полей темы токенам FUDE:

| Поле `Tokens` | FUDE Тушь | FUDE Васи |
|---|---|---|
| `canvas` (вокруг документа) | `sumi/bg` `#111216` | `#DED6C6` (самая тёмная бумага) |
| `chrome`, `card` (панели) | `sumi/panel` `#1A1C21` | `paper/base` `#F3EEE4`, `paper/raised` `#FBF8F2` |
| `dock` | `#15161A` | `paper/sunken` `#E9E2D4` |
| `hover` / `pressed` | `sumi/raised` `#23262D` / `#2C3038` | `#ECE6DA` / `#E2DACC` |
| `text` | `sumi/text` `#E8E4DC` | `ink/900` `#16161A` |
| `accent` (выделение) | индиго `#3F6E9F` | `ai/500` `#2F5D8C` |
| `row_selected` | `#29384F` | `ai/100` `#DCE7F1` |
| `primary_bg` (главная кнопка) | `shu/500` `#BC3823` | `shu/500` `#BC3823` |
| `warning` | хурма `#EC944A` | `#844814` |
| `caption_close` (закрыть окно) | `shu/500` | `shu/500` |

Тема подключена везде, где PhotoCraft перечисляет темы:

- настройки движка: `Theme`, `DarkTheme`, `LightTheme` и значения по умолчанию ([`crates/engine/src/prefs.rs`](../photocraft/crates/engine/src/prefs.rs));
- миграция старых настроек и `prefs.set interface.theme`;
- карточки тем в окне настроек ([`crates/ui-egui/src/prefs_ui.rs`](../photocraft/crates/ui-egui/src/prefs_ui.rs));
- команды меню `window.theme.fudeInk` и `window.theme.fudeWashi` ([`crates/engine/src/shell_commands.rs`](../photocraft/crates/engine/src/shell_commands.rs), [`crates/ui-egui/src/menus.rs`](../photocraft/crates/ui-egui/src/menus.rs));
- докинг панелей и канал управления, которые включали раскладку Pro только для `Pro | ProMedium` ([`panel_docking.rs`](../photocraft/crates/ui-egui/src/panel_docking.rs), [`control.rs`](../photocraft/crates/ui-egui/src/control.rs));
- начальная тема интерфейса ([`state.rs`](../photocraft/crates/ui-egui/src/state.rs)) и веб-сборки ([`apps/photocraft-web/src/web.rs`](../photocraft/apps/photocraft-web/src/web.rs)).

Два исправления по ходу стоит знать:

- **Начальная тема интерфейса** была жёстко прописана как ProMedium. При другой теме по умолчанию экран и состояние расходились: сохранённая тема ProMedium могла не примениться при запуске. Теперь начальная тема берётся из `ThemeKind::default()`.
- **Докинг панелей** определял раскладку Pro по списку `Pro | ProMedium`, а отрисовка — по флагу токенов. Для новых тем это ломало перетаскивание вкладок. Темы FUDE добавлены в эти проверки; поведение исходных тем не изменилось.

### 2. Товарные знаки ArtCraft

Условия бренда PhotoCraft требуют, чтобы форки и изменённые версии убрали название, логотипы и ссылки ArtCraft. Сделано:

- удалены логотипы `docs/brand/`;
- из меню «Справка» убраны «Join the ArtCraft Discord…», «ArtCraft Website» и «PhotoCraft Website» (вёл на getartcraft.com); «Сообщить о проблеме» ведёт в [Issues FUDE](https://github.com/Se1fKnowledge/FUDE/issues), «PhotoCraft on GitHub» — на исходный код;
- со стартового экрана и из окна «О программе» убрана кнопка Discord; остались ссылки «PhotoCraft on GitHub · FUDE»;
- из всех 21 каталога переводов удалены неиспользуемые строки ArtCraft и Discord;
- в установщиках Windows (`.wxs`) и Linux (`nfpm.yaml`, `metainfo.xml.in`) адреса заменены на репозиторий FUDE;
- README редактора, пользовательская документация, `ATTRIBUTION.md`, `NOTICE` и книга обновлены.

Сохранено намеренно:

- строка копирайта «ArtCraft Team and the PhotoCraft contributors» в `LICENSE-MIT`, `NOTICE` и README — её требуют лицензии MIT и Apache;
- благодарности и описание происхождения (иконка «нарисована в ArtCraft», список авторов) — это указание авторства, а не использование знака;
- внутренние инструкции разработчиков PhotoCraft (`AGENTS.md`, `docs/user/AGENTS.md`) — они описывают процессы исходного проекта.

Тесты следят, чтобы бренд не вернулся (см. [testing.md](testing.md)).

## Как обновить PhotoCraft из исходного репозитория

Изменения FUDE лежат отдельными коммитами, поэтому их можно перенести на новую версию:

```bash
# посмотреть, что изменили мы
git diff <коммит импорта>..HEAD -- photocraft

# сохранить свои изменения патчем
git diff <коммит импорта>..HEAD -- photocraft > fude.patch

# заменить photocraft/ свежей версией storytold/photocraft (без её .git),
# закоммитить «Импорт PhotoCraft <версия>», затем
git apply --3way fude.patch
```

После обновления — полный прогон тестов (см. [testing.md](testing.md)): тесты тем и бренда сразу покажут, если в новой версии появились места, где темы перечисляются заново или вернулись ссылки ArtCraft.

## Что важно знать про PhotoCraft

- **Правила проекта** описаны в [`photocraft/AGENTS.md`](../photocraft/AGENTS.md): без `unwrap` и `panic` вне тестов, каждое действие — команда, цвета только из `theme::Tokens`, изменения интерфейса проверяются снимком.
- В правилах исходного проекта прямо сказано, что PhotoCraft нельзя оборачивать в Tauri или webview: это нативное приложение. Поэтому в FUDE редактор работает **отдельным процессом**, а приложение уроков управляет им через JSON-канал команд (см. [architecture.md](architecture.md)).
- Снимки интерфейса без окна: `cargo run --release -p photocraft-ui-egui --example snapshot -- --out ui.png`. Скриншоты в [`design/`](../design/) сняты так, с ключом `--safe-gpu`: программный видеоадаптер Windows, на котором рисует снимок, не компилирует один шейдер GPU-композитора, поэтому холст рисуется на процессоре.
