use egui::FontDefinitions;

use crate::{dark_magic::BoxedAnything, misc};

use super::{chapter1, ChapterAnchor, GlobalContextMenu, MenuOptions};

pub struct AppState {
    pub menu_options: MenuOptions,
    pub text: String,
    pub anything: BoxedAnything,
}

impl AppState {
    /// Called once before the first frame.
    pub fn new(eframe: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // here I do NOT save the app state

        eframe.egui_ctx.set_fonts(FontDefinitions::default());
        AppState {
            menu_options: MenuOptions::default(),
            text: String::new(),
            anything: BoxedAnything::new(()),
        }
    }
}

impl eframe::App for AppState {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _: &mut dyn eframe::Storage) {
        // I don't save the app state!
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        let old_anchor = self.menu_options.anchor;
        #[cfg(target_arch = "wasm32")]
        if let Some(anchor) = _frame.info().web_info.location.hash.strip_prefix('#') {
            self.menu_options.anchor = match ChapterAnchor::read_str(anchor) {
                Some(a) => a,
                None => ChapterAnchor::Index,
            };
        }
        self.text = format!("{:?}", self.menu_options.anchor);

        super::default_menu(ctx, &mut self.menu_options);

        if old_anchor == self.menu_options.anchor {
            // like this I can now get default values or the stored value,
            // so I can use this to switch between them
            match &self.menu_options.anchor {
                ChapterAnchor::Chapter1(which) => {
                    chapter1::chapter_1_switch(which, &mut self.anything, ctx);
                }
                ChapterAnchor::Chapter2(_) => {
                    let some: &mut u32 = self.anything.to_something_or_default_mut();
                    *some = 10;
                }
                ChapterAnchor::Index => {
                    index_page(ctx, &mut self.menu_options.anchor);
                }
            }
        }

        if old_anchor != self.menu_options.anchor {
            #[cfg(target_arch = "wasm32")]
            if _frame.is_web() {
                ctx.open_url(egui::OpenUrl::same_tab(
                    self.menu_options.anchor.get_string(),
                ))
            }
        }
    }
}

fn index_page(ctx: &egui::Context, anchor: &mut ChapterAnchor) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // The central panel the region left after adding TopPanel's and SidePanel's
        ui.heading("Index");

        ui.label("Hier entsteht eine neue Website mithilfe von Rust und Webassembly. Die website ist bisher in der frühen Testphase. Kommen Sie am besten später wieder!");
        ui.label("A new website is being created here with the help of Rust and Webassembly. The website is currently in the early test phase. It's best to come back later!");

        let menu = GlobalContextMenu::default();
        ui.separator();
        menu.print_links(ui, anchor);

        ui.separator();

        ui.hyperlink_to(
            "Source code",
            "https://github.com/Pardoxa/rare_events_wasm"
        );

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.horizontal(
                |ui| {
                    let crate_version = format!("Website Version {}", misc::VERSION);
                    let git_hash = format!("Git Hash: {}", misc::GIT_HASH);
                    let compile_time = format!("Compile datetime: {}", misc::COMPILE_TIME);
                    ui.label(crate_version);
                    ui.label(git_hash);
                    ui.label(compile_time);
                }
            );
            powered_by_egui_and_eframe(ui);
            egui::warn_if_debug_build(ui);

        });
    });
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
