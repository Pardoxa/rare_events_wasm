use std::num::NonZeroU32;

use derivative::Derivative;
use egui::{Button, DragValue, Label};
use egui_plot::{Plot, PlotBounds, PlotPoints, Points};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_pcg::Pcg64;
use crate::dark_magic::BoxedAnything;

#[derive(Derivative)]
#[derivative(Default)]
pub struct ParallelTemperingData
{
    #[derivative(Default(value="Vec::new()"))]
    temperatures: Vec<Temperature>,
    /// If user clicks on add temperature, this one is added
    #[derivative(Default(value="-0.5"))]
    temperature_to_add: f64,
    #[derivative(Default(value="NonZeroU32::new(100).unwrap()"))]
    num_coins: NonZeroU32,
    seed: u64,
    rng: Option<Pcg64>,
    paused: bool,
    step_once: bool
}

impl ParallelTemperingData{
    pub fn sort_temps(&mut self)
    {
        self.temperatures.sort_unstable_by(
            |a, b| a.temperature.total_cmp(&b.temperature)
        );
    }

    pub fn contains_temp(&self, temp: f64) -> bool
    {
        for val in self.temperatures.iter(){
            if val.temperature == temp {
                return true;
            }
        }
        false
    }
}

pub struct Temperature{
    temperature: f64,
    config: Vec<bool>
}

impl Temperature{
    pub fn markov_step(&mut self, rng: &mut Pcg64)
    {
        let old_energy = self.count_true();
        let entry = self.config.choose_mut(rng).unwrap();
        let old_val = *entry;
        *entry = rng.gen_bool(0.5);
        let new_energy = if old_val == *entry{
            old_energy
        } else if old_val {
            old_energy + 1
        } else {
            old_energy - 1
        };

        let acceptance_prob = ((old_energy - new_energy) as f64 / self.temperature).exp();
        if rng.gen::<f64>() >= acceptance_prob {
            // we reject
            *entry = old_val;
        }
    }
}


impl Temperature{
    pub fn new(temp: f64, length: NonZeroU32, rng: &mut Pcg64) -> Self
    {
        let config = (0..length.get())
            .map(|_| rng.gen_bool(0.5))
            .collect();
        Temperature{
            temperature: temp,
            config
        }
    }

    pub fn count_true(&self) -> isize
    {
        self.config.iter().filter(|&s| *s).count() as isize
    }
}


pub fn parallel_tempering_gui(any: &mut BoxedAnything, ctx: &egui::Context)
{
    let data: &mut ParallelTemperingData = any.to_something_or_default_mut();
    let is_dark_mode = ctx.style().visuals.dark_mode;

    egui::SidePanel::left("ParallelLeft")
        .show(
            ctx, 
            |ui|
            {
                ui.horizontal(
                    |ui|
                    {
                        ui.add(Label::new("Temperature"));
                        ui.add(egui::DragValue::new(&mut data.temperature_to_add)
                                .speed(0.01)
                            ).on_hover_text("Click to type a number. Or drag the value for quick changes.");
                    }
                );

                if data.temperatures.is_empty(){
                    ui.horizontal(
                        |ui|
                        {
                            ui.label("number of Coins");
                            ui.add(
                                egui::DragValue::new(&mut data.num_coins)
                            );
                        }
                    );

                    ui.horizontal(
                        |ui|
                        {
                            ui.label("Seed");
                            ui.add(DragValue::new(&mut data.seed));
                        }
                    );

                    data.rng = Some(
                        Pcg64::seed_from_u64(data.seed)
                    );
                }
                
                let add_btn = ui.add(Button::new("add temperature"));


                if data.temperature_to_add == 0.0 {
                    add_btn.show_tooltip_text("We divide by the temperature in the formula for the acceptance probability. Thus 0 is an invalid temperature.");
                }
                else if add_btn
                    .clicked()
                {
                    let to_add = data.temperature_to_add;
                    if !data.contains_temp(to_add){
                        data.temperatures.push(
                            Temperature::new(
                                data.temperature_to_add,
                                data.num_coins,
                                data.rng.as_mut().unwrap()
                            )
                        );
                        data.sort_temps();
                    }
                    
                }

        
                
                if !data.temperatures.is_empty(){
                    if ui.add(
                        Button::new("Remove all Temperatures")
                    ).clicked()
                    {
                        data.temperatures.clear();
                    }
                    let txt = if data.paused{
                        "continue"
                    } else {
                        "pause"
                    };
                    if ui.add(
                        Button::new(txt)
                    ).clicked() {
                        data.paused = !data.paused;
                    }

                    if data.paused && ui.add(Button::new("step once")).clicked(){
                        data.step_once = true;
                    }
                }
                    
            }
        );


    egui::CentralPanel::default().show(ctx, |ui| {
        // The central panel the region left after adding TopPanel's and SidePanel's

        let mut plot_points = Vec::new();

        for (id, temp) in data.temperatures.iter_mut().enumerate()
        {
            if !data.paused || data.step_once{
                temp.markov_step(data.rng.as_mut().unwrap());
            }
            let heads = temp.count_true();
            let label = format!("Temp: {} Heads: {}", temp.temperature, heads);
            ui.horizontal(
                |ui|
                {
                    ui.label(label);
                    
                }
            );
            plot_points.push([heads as f64, id as f64]);
        }
        let plot_points = PlotPoints::new(plot_points);
        let points = Points::new(plot_points).radius(4.0);

        let plot_bounds = PlotBounds::from_min_max(
            [0.0, 0.0], 
            [data.num_coins.get() as f64, data.temperatures.len() as f64]
        );

        Plot::new("my_plot")
        .x_axis_label("time")
        .y_axis_label("Arbitrary stuff")
        .show(
            ui, 
            |plot_ui|
            {
                plot_ui.points(points);
                plot_ui.set_plot_bounds(plot_bounds);
            }
        );
        data.step_once = false;
    });
    ctx.request_repaint();
}