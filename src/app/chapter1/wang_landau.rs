use std::num::NonZeroU32;
use egui::{Button, DragValue};
use rand::{distributions::Uniform, prelude::Distribution, SeedableRng};
use rand_pcg::Pcg64;
use sampling::HistU32Fast;
use statrs::distribution::{Binomial, Discrete};
use crate::dark_magic::BoxedAnything;
use std::f64::consts::LOG10_E;
use derivative::Derivative;
use sampling::WangLandau1T;

use super::parallel_tempering::SidePanelView;

#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct WangLandauConfig
{
    /// How many coins to consider
    #[derivative(Default(value="NonZeroU32::new(100).unwrap()"))]
    coin_sequence_length: NonZeroU32,
    /// Seed for random number generator
    seed: u64,
    /// Contains Wang landau and true density
    simulation: Option<Simulation>,
    /// Visibility of the side Panel
    side_panel: SidePanelView,
    #[derivative(Default(value="0.0004"))]
    threshold: f64
}

pub fn wang_landau_gui(
    any: &mut BoxedAnything, 
    ctx: &egui::Context
)
{
    let data: &mut WangLandauConfig = any.to_something_or_default_mut();

    match data.side_panel {
        SidePanelView::Default => {
            let screen_width = ctx.screen_rect().width();
            let is_desktop = screen_width > 600.0;
            data.side_panel = if is_desktop{
                SidePanelView::Shown
            } else {
                SidePanelView::Hidden
            };
        },
        SidePanelView::Shown => {
            egui::SidePanel::left("ParallelLeft")
                .show(
                    ctx, 
                    |ui|
                    {
                        if ui.add(
                            Button::new("Hide sidepanel")
                        ).clicked(){
                            data.side_panel = SidePanelView::Hidden;
                        }
                        ui.horizontal(
                            |ui|
                            {
                                
                                ui.label("Number of coins");
                                ui.add(
                                    DragValue::new(&mut data.coin_sequence_length)   
                                );
                            }
                        );

                        match data.simulation{
                            None => {
                                if ui.add(
                                    Button::new("Create Simulation")
                                ).clicked() {
                                    data.simulation = Some(
                                        Simulation::new(
                                            data
                                        )
                                    );
                                }
                                ui.horizontal(
                                    |ui|
                                    {
                                        ui.label("Rng Seed:");
                                        ui.add(
                                            DragValue::new(&mut data.seed)
                                                .speed(1)
                                        );
                                    }
                                );
                            },
                            _ => {
                                if ui.add(
                                    Button::new("Delete Simulation")
                                ).clicked(){
                                    data.simulation = None;
                                }
                            }
                        }
                        ui.horizontal(
                            |ui|
                            {
                                ui.label("threshold");
                                ui.add(
                                    egui::Slider::new(
                                        &mut data.threshold, 
                                        0.00000000001..=0.001
                                    ).logarithmic(true)
                                );
                            }
                        )

                    }
                );
        },
        SidePanelView::Hidden => {
            egui::TopBottomPanel::top("top_panel")
                .show(
                    ctx, 
                    |ui| 
                    {
                        if ui.button("Open Side Panel").clicked() {
                            data.side_panel = SidePanelView::Shown;
                        }
                    }
                );
        }
    }
}

fn calc_true_log(
    coin_sequence_length: NonZeroU32
) -> Vec<f64>
{
    let binomial = Binomial::new(0.5, coin_sequence_length.get() as u64)
        .unwrap();
    (0..coin_sequence_length.get() as u64)
        .map(|k| LOG10_E * binomial.ln_pmf(k))
        .collect()
}

#[derive(Debug)]
pub struct Simulation{
    rng: Pcg64,
    true_density: Vec<f64>,
    simple_sample_hist: HistU32Fast
}

impl Simulation{
    pub fn new(
        data: &WangLandauConfig
    ) -> Self
    {
        let rng = Pcg64::seed_from_u64(data.seed);

        Simulation { 
            true_density: calc_true_log(data.coin_sequence_length),
            simple_sample_hist: HistU32Fast::new_inclusive(0, data.coin_sequence_length.get()).unwrap(),
            rng
        }
    }

    pub fn sample(
        &mut self, 
        rng: &mut Pcg64
    )
    {
        let len = self.simple_sample_hist.right();
        let uniform = Uniform::new_inclusive(0.0, 1.0);
        let mut num_heads = 0;
        uniform.sample_iter(rng)
            .take(len as usize)
            .filter(|&val| val > 0.5)
            .for_each(|_| num_heads += 1);
        self.simple_sample_hist.increment_quiet(num_heads as u32);
    }
}