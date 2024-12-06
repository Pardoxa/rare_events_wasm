use std::num::NonZeroU32;
use egui::{Button, CentralPanel, DragValue};
use rand::{distributions::Uniform, prelude::Distribution, SeedableRng};
use rand_pcg::Pcg64;
use sampling::{HistU32Fast, Histogram, WangLandau};
use statrs::distribution::{Binomial, Discrete};
use crate::dark_magic::BoxedAnything;
use super::coin_sequence_wl::*;
use std::f64::consts::LOG10_E;
use derivative::Derivative;
use sampling::WangLandau1T;
use web_time::Instant;

use super::parallel_tempering::SidePanelView;

type ThisWl = WangLandau1T<sampling::HistogramFast<u32>, rand_pcg::Lcg128Xsl64, CoinFlipSequence<rand_pcg::Lcg128Xsl64>, CoinFlipMove, (), u32>;

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

                        match data.simulation{
                            None => {
                                ui.horizontal(
                                    |ui|
                                    {
                                        
                                        ui.label("Number of coins");
                                        ui.add(
                                            DragValue::new(&mut data.coin_sequence_length)   
                                        );
                                    }
                                );

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
                                let old_threshold = data.threshold;
                                ui.add(
                                    egui::Slider::new(
                                        &mut data.threshold, 
                                        0.00000000001..=0.001
                                    ).logarithmic(true)
                                );
                                if old_threshold != data.threshold 
                                {   
                                    if let Some(sim) = data.simulation.as_mut(){
                                        sim.wl.set_log_f_threshold(data.threshold).unwrap();
                                    }
                                }
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

    if let Some(sim) = data.simulation.as_mut(){

        CentralPanel::default().show(
            ctx, 
            |ui|
            {
                let estimate = sim.wl.log_density_base10();
            }
        );

        sim.sample();
        ctx.request_repaint();
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
    simple_sample_hist: HistU32Fast,
    wl: ThisWl
}

impl Simulation{
    pub fn new(
        data: &WangLandauConfig
    ) -> Self
    {
        let mut rng = Pcg64::seed_from_u64(data.seed);
        let wl_rng = Pcg64::from_rng(&mut rng).unwrap();
        let coin_rng = Pcg64::from_rng(&mut rng).unwrap();

        let ensemble = CoinFlipSequence::new(
            data.coin_sequence_length.get() as usize, 
            coin_rng
        );

        let histogram = HistU32Fast::new_inclusive(0, data.coin_sequence_length.get())
            .unwrap();

        let mut wl = WangLandau1T::new(
            data.threshold, 
            ensemble, 
            wl_rng, 
            1, 
            histogram, 
            data.coin_sequence_length.get() as usize * 10
        ).unwrap();

        // Wl needs to be initialized
        wl.init_greedy_heuristic(energy_fn, None)
            .unwrap();

        Simulation { 
            true_density: calc_true_log(data.coin_sequence_length),
            simple_sample_hist: HistU32Fast::new_inclusive(0, data.coin_sequence_length.get()).unwrap(),
            rng,
            wl
        }
    }

    pub fn sample(
        &mut self
    )
    {
        self.wl.wang_landau_step_acc(CoinFlipSequence::update_head_count);
        let time = Instant::now();
        self.wl.wang_landau_while_acc(
            |ensemble, step, old_energy| {
                ensemble.update_head_count(step, old_energy)
            }, 
            |_| {(time.elapsed().as_millis() as f32) < (10.0_f32)}
        );

        let hist = self.simple_sample_hist.hist().as_slice();
        let len = hist.len();
        let simple_samples: usize = hist
            .iter()
            .sum();
        let missing_samples = self.wl.step_counter() - simple_samples;
        let uniform = Uniform::new_inclusive(0.0, 1.0);
        
        for _ in 0..missing_samples{
            let mut num_heads = 0;
            uniform.sample_iter(&mut self.rng)
                .take(len)
                .filter(|&val| val <= 0.5)
                .for_each(|_| num_heads += 1);
            self.simple_sample_hist.increment_quiet(num_heads);
        }

    }
}

fn energy_fn(seq: &mut CoinFlipSequence::<Pcg64>) -> Option<u32>
{
    Some(seq.head_count())
}