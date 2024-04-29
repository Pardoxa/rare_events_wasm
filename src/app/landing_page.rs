use crate::dark_magic::BoxedAnything;

use super::{ChapterAnchor, GlobalContextMenu};

pub struct AppState {
    pub anchor: ChapterAnchor,
    pub text: String,
    pub anything: BoxedAnything
}


impl AppState {
    /// Called once before the first frame.
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // here I do NOT save the app state

        AppState{
            anchor: ChapterAnchor::default(),
            text: String::new(),
            anything: BoxedAnything::new(())
        }
    }
}

impl eframe::App for AppState {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _: &mut dyn eframe::Storage) {
        // I don't save the app state!
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        let old_anchor = self.anchor;
        #[cfg(target_arch = "wasm32")]
        if let Some(anchor) = frame.info().web_info.location.hash.strip_prefix('#') {
            self.anchor = match ChapterAnchor::read_str(anchor){
                Some(a) => a,
                None => ChapterAnchor::Invalid
            };
        }
        self.text = format!("{:?}", self.anchor);
        
        super::default_menu(ctx, &mut self.anchor);


        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Index");
            let menu = GlobalContextMenu::default();
            ui.separator();
            menu.print_links(ui, &mut self.anchor);

            ui.separator();

            ui.hyperlink_to(
                "Source code",
                "https://github.com/Pardoxa/rare_events_wasm"
            );

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
        if old_anchor != self.anchor{
            #[cfg(target_arch = "wasm32")]
            if frame.is_web()
            {
                ctx.open_url(egui::OpenUrl::same_tab(self.anchor.get_string()))
            }
        } else {
            // like this I can now get default values or the stored value, 
            // so I can use this to switch between them
            match &self.anchor{
                ChapterAnchor::Chapter1(_) => {
                    let some: &mut String = self.anything.to_something_or_default_mut();
                    *some = "bla".to_owned();
                    
                },
                ChapterAnchor::Chapter2(_) => {
                    let some: &mut u32 = self.anything.to_something_or_default_mut();
                    *some = 10;
                    
                },
                _ => {
                    let some: &mut i8 = self.anything.to_something_or_default_mut();
                    *some = -10;
                    
                }
            }
        }
    }
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
