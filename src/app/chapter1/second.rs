use derivative::Derivative;
// This is an example
use crate::dark_magic::*;
use egui_plot::{Plot, PlotPoint, PlotPoints, Points, Line};
use web_time::Instant;
use std::{ops::DerefMut, thread};



#[derive(Default, Clone)]
pub struct InternalData {
    // Nothing, for this simple example we have no internal data
}

#[derive(Derivative)]
#[derivative(Default)]
struct SecondData{
    data: Vec<PlotPoint>,
    start_time: Option<Instant>,
    x: f64,
    y: f64,
    points: Vec<[f64; 2]>,
    thread_helper: ThreadHelper<InternalData, Vec<[f64; 2]>>
}

pub fn chapter_1_switch(any: &mut BoxedAnything, ctx: &egui::Context)
{
    let data: &mut SecondData = any.to_something_or_default_mut();


    if data.start_time.is_none(){
        data.start_time = Some(Instant::now());
    }

    let duration = data.start_time.as_ref().unwrap().elapsed();
    let seconds = duration.as_secs_f64();

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

        // NOTE: Spawning the thread etc is not really needed here, I just wanted to test 
        // that functionality

        if let Some(clone) = data.thread_helper.get_clone(){
            thread::spawn(move || {
                let external: Vec<_> = (0..1000).map(|i| {
                    let x = i as f64 * 0.01 + seconds;
                    [x, x.sin()]
                }).collect();
                if let Some(mut lock) = clone.exposed_data_write_lock(){
                    *(lock.deref_mut()) = ExposedData::Exists(external);
                }
            });
        }

        if let Some(new_points) = data.thread_helper.exposed_data_take(){
            data.points = new_points;
        }
        let point_plots: PlotPoints = data.points.iter()
            .copied()
            .collect();
        let line = Line::new(point_plots);

        //let line = Line::new(sin);
        Plot::new("my_plot")
            .x_axis_label("time")
            .y_axis_label("Arbitrary stuff")
            .show(
                ui, 
                |plot_ui|
                {
                    plot_ui.points(points);
                    plot_ui.line(line)
                }
            );
    });

    ctx.request_repaint();
    
}
