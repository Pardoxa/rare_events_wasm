use std::num::NonZeroU32;
use egui::{Button, DragValue};
use statrs::distribution::{Binomial, Discrete};
use crate::dark_magic::BoxedAnything;
use std::f64::consts::LOG10_E;
use derivative::Derivative;

use super::parallel_tempering::SidePanelView;

#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct WangLandauConfig
{
    #[derivative(Default(value="NonZeroU32::new(100).unwrap()"))]
    coin_sequence_length: NonZeroU32,

    /// Contains Wang landau and true density
    simulation: Option<Simulation>,

    side_panel: SidePanelView
}

#[derive(Debug)]
pub struct Simulation{
    true_density: Vec<f64>
}

impl Simulation{
    pub fn new(num_coins: NonZeroU32) -> Self
    {
        Simulation { 
            true_density: calc_true_log(num_coins) 
        }
    }
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
                                        Simulation::new(data.coin_sequence_length)
                                    )
                                }
                            },
                            _ => {
                                if ui.add(
                                    Button::new("Delete Simulation")
                                ).clicked(){
                                    data.simulation = None;
                                }
                            }
                        }


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


pub fn calc_true_log(
    coin_sequence_length: NonZeroU32
) -> Vec<f64>
{
    let binomial = Binomial::new(0.5, coin_sequence_length.get() as u64).unwrap();
    (0..coin_sequence_length.get())
        .map(|k| LOG10_E*binomial.ln_pmf(k as u64))
        .collect()
}