# Тесты

Весь код FUDE сейчас — это редактор в [`photocraft/`](../photocraft/), поэтому тесты — это тесты PhotoCraft плюс тесты, которые мы добавили или изменили для FUDE.

## Последний прогон

26 октября 2026, Windows 11, Rust 1.99.1:

| Набор | Команда | Результат |
|---|---|---|
| движок | `cargo test -p photocraft-engine --lib` | 2182 прошли, 0 упали, 14 пропущены |
| интерфейс | `cargo test -p photocraft-ui-egui --lib` | 3813 прошли, 0 упали, 18 пропущены |
| clippy | `cargo clippy -p photocraft-engine -p photocraft-ui-egui --lib --tests -- -D warnings` | без предупреждений |

Пропущенные тесты (`#[ignore]`) пропускает и сам PhotoCraft: это долгие проверки производительности и тесты, которым нужны внешние корпуса файлов.

## Как запускать

Из папки `photocraft/`:

```bash
# всё, что затрагивает FUDE: темы, настройки, меню, бренд
cargo test -p photocraft-ui-egui --lib theme::
cargo test -p photocraft-ui-egui --lib prefs_ui
cargo test -p photocraft-ui-egui --lib links
cargo test -p photocraft-engine --lib prefs
cargo test -p photocraft-engine --lib shell_commands

# полные наборы
cargo test -p photocraft-engine --lib
cargo test -p photocraft-ui-egui --lib

# перед коммитом
cargo fmt --all
cargo clippy -p photocraft-engine -p photocraft-ui-egui --lib --tests -- -D warnings
```

Полный набор интерфейса идёт около 45 секунд (после сборки), движка — около 15.

## Тесты FUDE

### Темы — `crates/ui-egui/src/theme.rs`

| Тест | Что проверяет |
|---|---|
| `fude_themes_are_the_default_parse_and_hold_text_contrast` | FUDE Тушь — тема по умолчанию; обе темы находятся по имени («FUDE Ink», `fude_ink`, «sumi», «washi»…); построены на раскладке Pro; основной, вторичный и бледный текст держат 4,5:1 на всех поверхностях, включая наведение, нажатие и выделенную строку; статусные цвета — 4,5:1 на спокойных поверхностях и 3:1 на активных; белый текст на индиговом выделении и на красной кнопке — 4,5:1 |
| `fude_tokens_are_the_fude_design_system_colours` | цвета темы совпадают с токенами из Figma (`sumi/*`, `paper/*`, `ink/*`, `ai/*`, `shu/500`) до байта |
| `fude_surfaces_step_in_depth_and_keep_their_tint` | поверхности идут по яркости в правильном порядке (холст → док → панели → наведение → нажатие); тушь отдаёт в синеву, бумага — в тепло, ни одна не нейтрально-серая; скругления 4 / 6 / 8 |
| `applying_a_fude_theme_installs_its_palette_into_egui` | после `apply` стандартные виджеты egui рисуют палитрой FUDE: фон панелей, окон и полей, текст, ссылки, ошибки, предупреждения, выделение, скругление окон, размер шрифта Pro |
| `theme_ids_and_labels_are_unique_and_round_trip` | у всех 16 тем уникальные идентификаторы и названия, каждый идентификатор находится обратно; каждая тема ровно в одном семействе (тёмное или светлое); переключение по кругу проходит через обе темы FUDE |
| `field_selection_is_grey_only_for_the_monochrome_pro_themes` (исходный, продолжает проходить) | серое выделение в полях только у монохромных Pro-тем; у FUDE — индиговое |

### Настройки — `crates/engine/src/prefs/tests.rs`

| Тест | Что проверяет |
|---|---|
| `appearance_defaults_and_legacy_theme_migrate` (обновлён) | новая установка: тёмный режим, FUDE Тушь и FUDE Васи; старый файл с одной темой переносится в свой режим |
| `fude_themes_are_choices_and_a_legacy_fude_theme_selects_its_family` | `fudeInk` — первый в списке тёмных тем, `fudeWashi` — в списке светлых; старый файл с темой FUDE открывается в её режиме |
| `setting_the_legacy_theme_to_a_fude_theme_fixes_its_mode` | `prefs.set interface.theme` с темой FUDE включает её режим и не трогает выбор в другом семействе; светлую тему нельзя записать в тёмный слот и наоборот |

### Команды меню — `crates/engine/src/shell_commands.rs`

| Тест | Что проверяет |
|---|---|
| `fude_theme_commands_lead_their_theme_submenus` | команды `window.theme.fudeInk` и `window.theme.fudeWashi` есть, у них правильные названия, они стоят первыми в подменю «Dark Theme» и «Light Theme» и без горячих клавиш |
| `no_command_label_carries_the_artcraft_brand` | ни в одной подписи команды нет ArtCraft или Discord; удалённых пунктов «Справки» нет |

### Окно настроек и переключение — `crates/ui-egui/src/prefs_ui.rs`

| Тест | Что проверяет |
|---|---|
| `fude_theme_menu_commands_select_and_save_their_slot` | новая установка открывается в FUDE Тушь; Window › Theme › FUDE Васи и FUDE Тушь показывают тему, включают её режим и сохраняют её, не трогая другое семейство |
| `theme_cards_offer_fude_first_and_every_option_is_a_saved_theme` | в карточках тем FUDE стоит первой; каждый вариант — сохраняемая тема того же семейства; избранные темы FUDE попадают в меню кнопки оформления |
| `appearance_resolves_both_saved_themes_and_follows_system`, `auto_uses_native_system_theme_when_egui_does_not_report_one`, `darken_and_lighten_interface_…`, `auto_appearance_replaces_a_restored_egui_dark_pin` (обновлены) | режим «как в системе» выбирает FUDE Тушь или FUDE Васи по умолчанию; «Затемнить/осветлить интерфейс» стартует с темы FUDE |
| `auto_appearance_follows_live_changes_without_saving_resolved_palettes` (исходный) | поймал реальную ошибку: начальная тема интерфейса была жёстко ProMedium; исправлено в `state.rs` |

### Бренд и ссылки — `crates/ui-egui/src/links.rs`

| Тест | Что проверяет |
|---|---|
| `help_menu_lists_links_then_separator_then_system_info_and_about` (обновлён) | меню «Справка»: PhotoCraft on GitHub, Сообщить о проблеме, Проверить обновления, разделитель, Сведения о системе, О программе |
| `link_commands_open_their_urls` (обновлён) | ссылки ведут на исходный код PhotoCraft и в Issues FUDE |
| `links_open_through_the_platform_service` (обновлён) | ссылки открываются через системный браузер |
| `no_link_or_help_command_carries_the_artcraft_brand` | ни одна ссылка не ведёт на ArtCraft или Discord; удалённые пункты — неизвестные команды |
| `catalogs_and_repository_carry_no_artcraft_brand` | ни в одном из 21 каталога переводов нет строк ArtCraft или ссылок на Discord ArtCraft; папки с логотипами `docs/brand` нет |
| `link_targets_are_github_projects` | все ссылки — на GitHub, проблемы — в репозиторий FUDE |

### Прочее, что пришлось обновить

| Тест | Почему |
|---|---|
| `canvas::tests::start_screen_buttons_stay_inside_a_narrow_canvas` | кнопки Discord больше нет; тест проверяет ссылку PhotoCraft on GitHub и то, что Discord не появляется |
| `status_bar_tests::appearance_toggle_preserves_pro_layout_and_has_no_fit_button` | светлая тема по умолчанию теперь FUDE Васи |
| тесты докинга панелей (`panel_docking_tests.rs`) | стенд запускается в теме по умолчанию, как само приложение |

## Проверка дизайна

Макет в Figma проверялся по методике impeccable: независимое дизайн-ревью и автоматический замер слоёв (контраст WCAG, текст мельче 12 px, обрезки, наложения, размер кликабельных зон, свечения). После правок контраст и мелкий текст чистые; оставшиеся отметки — намеренные (водяные знаки, бледные знаки закрытых достижений, приглушённый неработающий редактор на экране ошибки).
