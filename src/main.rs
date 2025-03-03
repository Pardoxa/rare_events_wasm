#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release


// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };
    
    // To add icon:
    //.with_icon(
    //    // NOTE: Adding an icon is optional
    //    eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
    //        .expect("Failed to load icon"),
    //),
    eframe::run_native(
        "LargeDeviationBook",
        native_options,
        Box::new(|cc| Ok(Box::new(rare_events_wasm::app::landing_page::AppState::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::{wasm_bindgen::JsCast, web_sys};



    let web_options = eframe::WebOptions::default();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("the_canvas_id")
        .unwrap();
    let canvas = canvas
        .dyn_into()
        .unwrap();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                canvas, // hardcode it
                web_options,
                Box::new(|cc| Ok(Box::new(rare_events_wasm::app::landing_page::AppState::new(cc)))),
            )
            .await
            .expect("failed to start eframe");
    });
}
