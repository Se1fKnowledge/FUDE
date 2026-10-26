//! Project links: the Help menu, the About dialog, the start screen and the title bar all open
//! these. This FUDE build carries no ArtCraft name, wordmark or links (docs/brand/LICENSE-brand.txt
//! upstream requires forks to remove them) and no link to the upstream PhotoCraft repository: it
//! is FUDE's editor, so every link goes to the FUDE repository. The upstream authors stay
//! credited in the About dialog, NOTICE and the licence files.

use serde_json::{Value, json};

use crate::PhotocraftApp;

/// The FUDE project, which ships this build.
pub const FUDE: &str = "https://github.com/Se1fKnowledge/FUDE";
/// Problems with this build are reported to FUDE, not to the upstream tracker.
pub const ISSUES: &str = "https://github.com/Se1fKnowledge/FUDE/issues";

/// Help-menu link commands: (id, url). Labels live in `menus::UI_COMMANDS`.
pub const COMMANDS: &[(&str, &str)] = &[("help.fude", FUDE), ("help.reportIssue", ISSUES)];

pub fn url_for(id: &str) -> Option<&'static str> {
    COMMANDS.iter().find(|c| c.0 == id).map(|c| c.1)
}

/// Open `url` in the system browser (a new tab on the web) and note it in the status bar. Prefers
/// the platform `open_url` service (reliable on Windows/macOS/Linux); falls back to `ctx.open_url`.
pub fn open(app: &mut PhotocraftApp, ctx: &egui::Context, url: &str) -> Value {
    let opened = app.services.open_url.as_ref().map(|f| f(url).is_ok()).unwrap_or(false);
    if !opened {
        ctx.open_url(egui::OpenUrl::new_tab(url));
    }
    let template = "Opened {url}";
    app.ui.status = crate::i18n::fmt(tl!(template), &[("url", url)]);
    json!({"url": url})
}

/// "FUDE on GitHub · Report an Issue" as links, centred. Clicks route through [`open`] (the platform
/// browser service) rather than `ui.hyperlink_to`, which uses the unreliable `ctx.open_url`.
pub fn link_row(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = crate::theme::Tokens::get(ui.ctx());
    let links = [(tl!("FUDE on GitHub"), FUDE), (tl!("Report an Issue"), ISSUES)];
    let font = egui::FontId::proportional(12.5);
    let sep = "  ·  ";
    let width: f32 = links.iter().map(|(l, _)| ui.painter().layout_no_wrap((*l).into(), font.clone(), t.text).size().x).sum::<f32>()
        + ui.painter().layout_no_wrap(sep.into(), font.clone(), t.text).size().x;
    let mut clicked: Option<&str> = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
        for (i, (label, url)) in links.iter().enumerate() {
            if i > 0 {
                ui.label(egui::RichText::new(sep).font(font.clone()).color(t.text_faint));
            }
            if ui.link(egui::RichText::new(*label).font(font.clone()).color(t.accent)).on_hover_text(*url).clicked() {
                clicked = Some(url);
            }
        }
    });
    if let Some(url) = clicked {
        open(app, ui.ctx(), url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_open_through_the_platform_service() {
        // Issue #14: links (Help menu, start-page links) must open in the browser.
        // They route through the `open_url` service rather than the unreliable `ctx.open_url`.
        use std::sync::{Arc, Mutex};
        let opened: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let rec = opened.clone();
        let services = crate::Services {
            open_url: Some(Box::new(move |u: &str| {
                rec.lock().unwrap().push(u.to_string());
                Ok(())
            })),
            ..Default::default()
        };
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), services);
        let ctx = egui::Context::default();
        // Every Help-menu link id reaches the service with its URL.
        for (id, url) in COMMANDS {
            crate::menus::invoke(&mut app, &ctx, id, json!({})).unwrap();
            assert_eq!(opened.lock().unwrap().last().map(String::as_str), Some(*url), "{id}");
        }
        // The direct open() helper (start-page and About links) also uses it.
        open(&mut app, &ctx, FUDE);
        assert_eq!(opened.lock().unwrap().last().map(String::as_str), Some(FUDE));
    }

    #[test]
    fn help_menu_lists_links_then_separator_then_system_info_and_about() {
        let app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        let help: Vec<String> = crate::menus::menu_items(&app).into_iter().filter(|i| i.path == ["Help"]).map(|i| i.id).collect();
        assert_eq!(help, ["help.fude", "help.reportIssue", "---", "help.systemInfo", "help.about"]);
        for (id, _) in COMMANDS {
            assert!(crate::menus::is_live(id) && crate::menus::is_enabled(&app, id), "{id}");
        }
    }

    #[test]
    fn link_commands_open_their_urls() {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        let ctx = egui::Context::default();
        for (id, url) in [("help.fude", "https://github.com/Se1fKnowledge/FUDE"), ("help.reportIssue", "https://github.com/Se1fKnowledge/FUDE/issues")] {
            let r = crate::menus::invoke(&mut app, &ctx, id, serde_json::json!({})).unwrap();
            assert_eq!(r["url"], url);
            assert_eq!(app.ui.status, format!("Opened {url}"));
        }
    }

    /// FUDE ships without the ArtCraft name, wordmark or links: no link points at an ArtCraft
    /// property and the removed Help items stay unknown commands.
    #[test]
    fn no_link_or_help_command_carries_the_artcraft_brand() {
        for (id, url) in COMMANDS.iter().chain([("start", FUDE)].iter()) {
            let lower = url.to_ascii_lowercase();
            assert!(!lower.contains("artcraft") && !lower.contains("discord"), "{id}: {url}");
        }
        for removed in ["help.discord", "help.artcraftWebsite", "help.website", "help.github", "help.checkForUpdates"] {
            assert_eq!(url_for(removed), None, "{removed}");
            assert!(!crate::menus::is_live(removed), "{removed} is still a live command");
        }
    }

    /// No translation catalog still carries an ArtCraft string or a link to the ArtCraft Discord
    /// (comments in the file headers don't count), and the upstream brand assets are gone.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn catalogs_and_repository_carry_no_artcraft_brand() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let dir = root.join("src/i18n");
        let mut checked = 0;
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) != Some("tsv") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            for (n, line) in text.lines().enumerate() {
                if line.starts_with('#') {
                    continue;
                }
                let lower = line.to_lowercase();
                assert!(!lower.contains("artcraft") && !lower.contains("discord.gg"), "{}:{}: {line}", path.display(), n + 1);
            }
            checked += 1;
        }
        assert!(checked >= 20, "found {checked} catalogs");
        assert!(!root.join("../../docs/brand").exists(), "the ArtCraft brand assets must stay removed");
    }

    /// Every link the editor can open goes to the FUDE repository: none to upstream PhotoCraft.
    #[test]
    fn every_link_goes_to_the_fude_repository() {
        for (id, url) in COMMANDS.iter().chain([("fude", FUDE), ("issues", ISSUES)].iter()) {
            assert!(url.starts_with("https://github.com/Se1fKnowledge/FUDE"), "{id}: {url}");
            assert!(!url.to_ascii_lowercase().contains("storytold"), "{id}: {url}");
        }
    }

    /// No source file of the editor's interface, engine or app carries a link to the upstream
    /// PhotoCraft repository or its release API (comments citing upstream issues are history).
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn no_runtime_link_to_the_upstream_repository() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut stack = vec![root.join("crates/ui-egui/src"), root.join("crates/engine/src"), root.join("apps/photocraft/src")];
        let mut checked = 0;
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                    continue;
                }
                let text = std::fs::read_to_string(&path).unwrap();
                for (n, line) in text.lines().enumerate() {
                    let code = line.trim_start();
                    if code.starts_with("//") {
                        continue;
                    }
                    for banned in [concat!("github.com/", "storytold"), concat!("api.", "github.com")] {
                        assert!(!code.contains(banned), "{}:{}: {line}", path.display(), n + 1);
                    }
                }
                checked += 1;
            }
        }
        assert!(checked > 100, "checked {checked} files");
    }
}
