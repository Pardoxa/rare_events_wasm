use derivative::Derivative;
// This is an example
use crate::dark_magic::*;
use egui_plot::{Plot, PlotPoint, PlotPoints, Points};

#[derive(Debug, Derivative)]
#[derivative(Default)]
struct SecondData{
    data: Vec<PlotPoint>,
    x: f64,
    y: f64
}

pub fn chapter_1_switch(any: &mut BoxedAnything, ctx: &egui::Context)
{
    let data: &mut SecondData = any.to_something_or_default_mut();

    egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
        ui.add(
            egui::DragValue::new(&mut data.x).prefix("x: ")
        );
        ui.add(
            egui::DragValue::new(&mut data.y).prefix("y: ")
        );
        if ui.button("Add data to plot").clicked() {
            let point = PlotPoint::new(data.x, data.y);
            data.data.push(point);
        }
    });


    egui::CentralPanel::default().show(ctx, |ui| {
        // The central panel the region left after adding TopPanel's and SidePanel's

        let points = PlotPoints::Owned(data.data.clone());
        let points = Points::new(points).radius(4.0);
        
        Plot::new("my_plot")
            .view_aspect(2.0)
            .show(ui, |plot_ui| plot_ui.points(points));
    });
    
}
