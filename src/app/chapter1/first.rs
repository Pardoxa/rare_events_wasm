// This is an example
use crate::dark_magic::*;

#[derive(Debug, Default)]
struct FirstData{
    slider: f64
}

pub fn chapter_1_switch(any: &mut BoxedAnything, ctx: &egui::Context)
{
    let data: &mut FirstData = any.to_something_or_default_mut();

    egui::CentralPanel::default().show(ctx, |ui| {
        // The central panel the region left after adding TopPanel's and SidePanel's
        ui.add(
            egui::Slider::new(
                &mut data.slider, 
                0.0..=100.0
            ).text("Test value")
        );
    });
    
}