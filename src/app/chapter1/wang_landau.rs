use std::{num::{NonZeroU32, NonZeroUsize}, time::Duration};
use egui::{Button, CentralPanel, DragValue, Slider};
use crate::misc::*;
use egui_plot::{Legend, Line, Plot, PlotPoints, Points};
use rand::{distributions::Uniform, prelude::Distribution, SeedableRng};
use rand_pcg::Pcg64;
use sampling::{HistU32Fast, Histogram, WangLandau, WangLandauEnergy};
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
    #[derivative(Default(value="NonZeroU32::new(500).unwrap()"))]
    coin_sequence_length: NonZeroU32,
    /// Seed for random number generator
    seed: u64,
    /// Contains Wang landau and true density
    simulation: Option<Simulation>,
    /// Visibility of the side Panel
    side_panel: SidePanelView,
    #[derivative(Default(value="0.00001"))]
    target_log_f: f64,
    /// Log or Linear?
    display: DisplayState,
    #[derivative(Default(value="LineOrPoints::Line"))]
    analytic: LineOrPoints,
    wang_landau: LineOrPoints,
    simple_sample: LineOrPoints,
    slow_motion: Speed,
    #[derivative(Default(value="NonZeroUsize::new(512).unwrap()"))]
    slow_motion_speed: NonZeroUsize
}


pub fn wang_landau_gui(
    any: &mut BoxedAnything, 
    ctx: &egui::Context
)
{
    let data: &mut WangLandauConfig = any.to_something_or_default_mut();
    let is_dark_mode = ctx.style()
        .visuals
        .dark_mode;

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
                            Button::new("Hide side panel")
                        ).clicked(){
                            data.side_panel = SidePanelView::Hidden;
                        }

                        ui.horizontal(
                            |ui|
                            {
                                ui.label("Display mode:");
                                ui.radio_value(&mut data.display, DisplayState::Linear, "Linear");
                                ui.radio_value(&mut data.display, DisplayState::Log, "Logarithmic");
                            }
                        );
                        
                        ui.horizontal(
                            |ui|
                            {
                                ui.radio_value(&mut data.slow_motion, Speed::Regular, "Regular Speed");
                                ui.radio_value(&mut data.slow_motion, Speed::SlowMotion, "Slow motion");
                            }
                        );

                        if data.slow_motion.is_slow_motion()
                        {
                            ui.horizontal(
                                |ui|
                                {
                                    ui.label("Max Speed:");
                                    ui.add(
                                        Slider::new(&mut data.slow_motion_speed, NonZeroUsize::new(1).unwrap()..=NonZeroUsize::new(1024).unwrap())
                                    );
                                }
                            );
                        }

                        ui.horizontal(
                            |ui|
                            {
                                let old = data.coin_sequence_length;
                                ui.label("Number of coins");
                                ui.add(
                                    DragValue::new(&mut data.coin_sequence_length)
                                        .range(1..=10000)
                                );
                                if old != data.coin_sequence_length && data.simulation.is_some(){
                                    let sim = Simulation::new(data);
                                    data.simulation = Some(sim);
                                }
                            }
                        );

                        match data.simulation.as_ref(){
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
                                ui.label("target log f");
                                let old_target = data.target_log_f;
                                ui.add(
                                    egui::Slider::new(
                                        &mut data.target_log_f, 
                                        0.000000000001..=0.001
                                    ).logarithmic(true)
                                );
                                if old_target != data.target_log_f 
                                {   
                                    if let Some(sim) = data.simulation.as_mut(){
                                        sim.wl.set_log_f_threshold(data.target_log_f).unwrap();
                                    }
                                }
                                
                            }
                        );
                        if let Some(sim) = data.simulation.as_ref()
                        {
                            ui.label(format!("Current log f: {:e}", sim.wl.log_f()));
                            ui.label(format!("Steps: {:e}", sim.wl.step_counter()));

                            line_or_points_radio_btn(ui, &mut data.analytic, "Analytic:");
                            line_or_points_radio_btn(ui, &mut data.simple_sample, "Simple Sample:");
                            line_or_points_radio_btn(ui, &mut data.wang_landau, "Wang Landau:");
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
                        if ui.button("show side panel").clicked() {
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
                let mut estimate = sim.wl.log_density_base10();

                sampling::norm_log10_sum_to_1(&mut estimate);

                if data.display == DisplayState::Linear{
                    estimate.iter_mut()
                        .for_each(
                            |val|
                            {
                                *val = 10.0_f64.powf(*val);
                            }
                        );
                }

                let wang_landau_estimate = slice_to_line_or_points(
                    &estimate,
                    "Wang Landau",
                    data.wang_landau
                );

                let current_energy_wl_point = match data.slow_motion{
                    Speed::Regular => None,
                    Speed::SlowMotion => {
                        sim.wl
                            .energy()
                            .map(
                                |energy|
                                {
                                    Points::new(PlotPoints::new(vec![[*energy as f64, estimate[*energy as usize]]]))
                                        .radius(13.)
                                        .shape(egui_plot::MarkerShape::Cross)
                                        .name("Current WL walker")
                                        .color(super::parallel_tempering::get_color(3, is_dark_mode))
                                }
                            )
                    }
                };
                

                let true_density = match data.display{
                    DisplayState::Linear => sim.true_density_lin.as_slice(),
                    DisplayState::Log => &sim.true_density_log
                };

                let analytic_results = slice_to_line_or_points(
                    true_density,
                    "Analytic",
                    data.analytic
                );

                let simple_estimate = sim.get_simple_sample_estimate(data.display);
                let simple_plot = slice_to_line_or_points(
                    &simple_estimate,
                    "Simple Sampling",
                    data.simple_sample
                );

                let y_label = get_rich_text_size(
                    data.display.get_y_label(), 
                    15.0
                );
                let x_label = get_rich_text_size(
                    "Number of Heads", 
                    15.0
                );

                let legend = Legend::default()
                    .text_style(egui::TextStyle::Heading);
                

                Plot::new("Wl_plot_HASH")
                    .y_axis_label(y_label)
                    .x_axis_label(x_label)
                    .legend(legend)
                    .show(
                        ui,
                        |plot_ui|
                        {
                            wang_landau_estimate.plot(plot_ui);
                            analytic_results.plot(plot_ui);
                            simple_plot.plot(plot_ui);
                            if let Some(point) = current_energy_wl_point{
                                plot_ui.points(point);
                            }
                        } 
                    );
            }
        );

        sim.sample(data.slow_motion, data.slow_motion_speed);
        

        match data.slow_motion{
            Speed::Regular => {
                ctx.request_repaint();
            },
            Speed::SlowMotion => {
                ctx.request_repaint_after(Duration::new(0, (1. / 60. * 1e9) as u32));
            }
        }

    }
}

pub fn calc_true_log(
    coin_sequence_length: NonZeroU32
) -> Vec<f64>
{
    let binomial = Binomial::new(0.5, coin_sequence_length.get() as u64)
        .unwrap();
    (0..=coin_sequence_length.get() as u64)
        .map(|k| LOG10_E * binomial.ln_pmf(k))
        .collect()
}

#[derive(Debug)]
pub struct Simulation{
    rng: Pcg64,
    true_density_log: Vec<f64>,
    true_density_lin: Vec<f64>,
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
            data.target_log_f, 
            ensemble, 
            wl_rng, 
            1, 
            histogram, 
            data.coin_sequence_length.get() as usize * 10
        ).unwrap();

        // Wl needs to be initialized
        wl.init_greedy_heuristic(energy_fn, None)
            .unwrap();

        let true_density_log = calc_true_log(data.coin_sequence_length);
        let true_density_lin = true_density_log
            .iter()
            .map(|val| 10.0_f64.powf(*val))
            .collect();

        Simulation { 
            true_density_log,
            true_density_lin,
            simple_sample_hist: HistU32Fast::new_inclusive(0, data.coin_sequence_length.get()).unwrap(),
            rng,
            wl
        }
    }

    pub fn sample(
        &mut self,
        slow_motion: Speed,
        slow_motion_speed: NonZeroUsize
    )
    {
        let time = Instant::now();

        match slow_motion{
            Speed::SlowMotion => {
                let cur = self.wl.step_counter();
                let max = cur + slow_motion_speed.get();
                self.wl.wang_landau_while_acc(
                    |ensemble, step, old_energy| {
                        ensemble.update_head_count(step, old_energy)
                    }, 
                    |wl| wl.step_counter() < max && time.elapsed().as_micros() < 100
                );                
            },
            Speed::Regular => {
                self.wl.wang_landau_while_acc(
                    |ensemble, step, old_energy| {
                        ensemble.update_head_count(step, old_energy)
                    }, 
                    |_| time.elapsed().as_micros() < 5000
                );
            }
        };
        
        
        let wl_time = time.elapsed().as_micros();
        let time = Instant::now();
        let uniform = Uniform::new_inclusive(0.0, 1.0);
        let num_coins = self.simple_sample_hist.bin_count() - 1;
        
        while time.elapsed().as_micros() < wl_time {
            for _ in 0..3{
                let mut num_heads = 0;
                uniform.sample_iter(&mut self.rng)
                    .take(num_coins)
                    .filter(|&val| val <= 0.5)
                    .for_each(|_| num_heads += 1);
                self.simple_sample_hist.increment_quiet(num_heads);
            } 
        }
    } 

    fn get_simple_sample_estimate(&self, display: DisplayState) -> Vec<f64>
    {
        let hist = self.simple_sample_hist
            .hist()
            .as_slice();
        let total: usize = hist.iter().sum();
        let factor = (total as f64).recip();
        let mut estimate: Vec<_> = hist.iter()
            .map(
                |&val|
                {
                    val as f64 * factor
                }
            ).collect();
        if display == DisplayState::Log {
            estimate.iter_mut()
                .for_each(
                    |v| 
                    if *v == 0.0 {
                        *v = f64::NAN
                    } else {
                        *v = v.log10()
                    }
                );
        }
        estimate
    }
}

fn energy_fn(seq: &mut CoinFlipSequence::<Pcg64>) -> Option<u32>
{
    Some(seq.head_count())
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum DisplayState{
    #[default]
    Log,
    Linear
}

impl DisplayState
{
    fn get_y_label(&self) -> &str
    {
        match self{
            Self::Linear => {
                "Probability"
            },
            Self::Log => {
                "Log10 of Probability"
            }
        }
    }
}

fn slice_to_line_or_points(
    slice: &[f64],
    name: &str,
    line_or_points: LineOrPoints
) -> LoP
{
    let plot_points = PlotPoints::new(
        slice.iter()
            .enumerate()
            .map(
                |(idx, val)|
                [idx as f64, *val]
            ).collect()
    );

    match line_or_points{
        LineOrPoints::Points => {
            let p = Points::new(plot_points)
                .radius(5.0)
                .name(name);
            LoP::Points(p)
        },
        LineOrPoints::Line => {
            let l = Line::new(plot_points)
                .name(name)
                .width(3.0);
            LoP::Line(l)
        }
    }
    
}

pub enum LoP{
    Line(Line),
    Points(Points)
}

impl LoP{
    pub fn plot(self, plot_ui: &mut egui_plot::PlotUi){
        match self
        {
            Self::Line(line) => {
                plot_ui.line(line);
            },
            Self::Points(p) => {
                plot_ui.points(p);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum LineOrPoints{
    Line,
    #[default]
    Points
}

fn line_or_points_radio_btn(
    ui: &mut egui::Ui,
    current: &mut LineOrPoints,
    name: &str

)
{
    ui.horizontal(
        |ui|
        {
            ui.label(name);
            ui.radio_value(current, LineOrPoints::Line, "Line");
            ui.radio_value(current, LineOrPoints::Points, "Points");
        }
    );
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum Speed{
    Regular,
    #[default]
    SlowMotion
}

impl Speed{
    pub fn is_slow_motion(self) -> bool {
        matches!(self, Speed::SlowMotion)
    }
}