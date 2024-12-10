use core::f64;
use std::{mem::swap, num::{NonZeroU32, NonZeroUsize}};
use num_traits::Signed;
use sampling::{HistU32Fast, Histogram};
use derivative::Derivative;
use egui::{Button, Color32, DragValue, Grid, Label, Rect, RichText, Slider, Widget};
use egui_plot::{AxisHints, Bar, BarChart, Legend, Line, MarkerShape, Plot, PlotBounds, PlotPoints, Points};
use rand::{distributions::Uniform, prelude::Distribution, seq::SliceRandom, Rng, SeedableRng};
use rand_pcg::Pcg64;
use crate::dark_magic::BoxedAnything;
use ordered_float::NotNan;
use crate::misc::*;


const COLORS: [DarkLightColor; 7] = [
    DarkLightColor{dark: Color32::LIGHT_RED, light: Color32::RED},
    DarkLightColor{dark: Color32::LIGHT_BLUE, light: Color32::BLUE},
    DarkLightColor{dark: Color32::ORANGE, light: Color32::ORANGE},
    DarkLightColor{dark: Color32::WHITE, light: Color32::BLACK},
    DarkLightColor{dark: Color32::YELLOW, light: Color32::GOLD},
    DarkLightColor{dark: Color32::LIGHT_GREEN, light: Color32::GREEN},
    DarkLightColor{dark: Color32::LIGHT_YELLOW, light: Color32::KHAKI},
];

const DEFAULT_TEMPERATURES: [f64; 8] = [
    0.1,
    0.01,
    0.005,
    0.0075,
    -0.1,
    -0.01,
    -0.0075,
    -0.005
];

const MONOSPACE_LEN: usize = 15;

const DRAG_SPEED: f64 = 0.01;

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
    #[derivative(Default(value="Pcg64::seed_from_u64(832147)"))]
    rng: Pcg64,
    paused: bool,
    step_once: bool,
    marker_cycle: Option<Box<dyn Iterator<Item=MarkerShape>>>,
    step_counter: u32,
    color_cycle: Option<Box<dyn Iterator<Item=DarkLightColor>>>,
    side_panel: SidePanelView,
    #[derivative(Default(value="Show::Yes"))]
    show_plot: Show,
    show_histogram: Show,
    show_acceptance: Show,
    show_history: Show
}

impl ParallelTemperingData{
    fn add_temperature(&mut self, to_add: f64) -> bool
    {
        if !self.contains_temp(to_add){
            self.temperatures.push(
                Temperature::new(
                    to_add,
                    self.num_coins,
                    &mut self.rng,
                    self.marker_cycle.as_mut()
                        .unwrap()
                        .next()
                        .unwrap(),
                    self.color_cycle
                        .as_mut()
                        .unwrap()
                        .next()
                        .unwrap()
                )
            );
            true
        } else {
            false
        }
    }

    fn remove(&mut self, to_remove: ToRemove)
    {
        match to_remove{
            ToRemove::Nothing => (),
            ToRemove::Top => {
                self.temperatures.pop();
            },
            ToRemove::Bottom => {
                self.temperatures.remove(0);
            }
            ToRemove::Idx(idx) => {
                self.temperatures.remove(idx);
            }
        }
    }

    fn new_length(
        &mut self
    )
    {
        self.temperatures
            .iter_mut()
            .for_each(
                |temp|
                {
                    temp.adjust_length(self.num_coins, &mut self.rng);
                }
            );
    }
}

#[derive(Debug, Default)]
pub enum SidePanelView{
    Shown,
    Hidden,
    #[default]
    Default
}



impl ParallelTemperingData{
    pub fn sort_temps(&mut self)
    {
        self.temperatures.sort_by_cached_key(
            |a| SortHelper{temp: NotNan::new(a.temperature).unwrap()}
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


#[derive(PartialEq, Eq)]
pub struct SortHelper{
    temp: NotNan<f64>
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for SortHelper{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let o_positive = other.temp.signum() == 1.0;
        let s_positive = self.temp.signum() == 1.0;

        match (s_positive, o_positive) {
            (false, false) => {
                other.temp.partial_cmp(&self.temp)
            },
            (true, true) => {
                other.temp.partial_cmp(&self.temp)
            },
            _ => {
                self.temp.partial_cmp(&other.temp)
            }
        }
    }
}

impl Ord for SortHelper{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, Default)]
pub struct AcceptanceCounter{
    accepted: u64,
    rejected: u64
}

impl AcceptanceCounter{
    pub fn count_acceptance(&mut self)
    {
        self.accepted += 1;
    }
    
    pub fn count_rejected(&mut self)
    {
        self.rejected += 1;
    }

    pub fn acceptance_rate(&self) -> f64
    {
        self.accepted as f64 / (self.accepted + self.rejected) as f64
    }

    pub fn reset(&mut self){
        self.accepted = 0;
        self.rejected = 0;
    }
}

#[derive(Debug)]
pub struct Temperature{
    temperature: f64,
    config: Vec<bool>,
    marker: MarkerShape,
    color: DarkLightColor,
    hist: HistU32Fast,
    acceptance: AcceptanceCounter,
    ring_buffer: RingBuffer<u32>
}

impl Temperature{
    pub fn adjust_length(
        &mut self, 
        length: NonZeroU32,
        rng: &mut Pcg64
    )
    {
        let length_usize = length.get() as usize;
        if length_usize <= self.config.len(){
            self.config.truncate(length_usize);
        } else {
            let missing = length_usize - self.config.len();
            let uniform = Uniform::new_inclusive(0.0, 1.0);
            self.config.extend(
                uniform.sample_iter(rng)
                    .take(missing)
                    .map(|v| v <= 0.5)
            );
        }
        self.hist = HistU32Fast::new_inclusive(0, length.get())
            .unwrap();
        self.acceptance.reset();
        self.ring_buffer.reset();
    }

    pub fn markov_step(&mut self, rng: &mut Pcg64)
    {
        let len = self.config.len();
        let old_heads = self.number_of_heads();
        let entry = self.config.choose_mut(rng).unwrap();
        let old_val = *entry;
        *entry = rng.gen_bool(0.5);
        let mut new_heads = if old_val == *entry{
            old_heads
        } else if old_val {
            old_heads + 1
        } else {
            old_heads - 1
        };

        let acceptance_prob = ((old_heads - new_heads) as f64 / (self.temperature * len as f64)).exp();
        if rng.gen::<f64>() >= acceptance_prob {
            // we reject
            *entry = old_val;
            new_heads = old_heads;
            self.acceptance.count_rejected();
        } else {
            self.acceptance.count_acceptance();
        }
        let new_heads = new_heads as u32;
        self.ring_buffer.push(new_heads);
        self.increment_hist(new_heads);
    }

    pub fn increment_hist(&mut self, val: u32)
    {
        self.hist.increment_quiet(val);
    }

    pub fn new(
        temp: f64, 
        length: NonZeroU32, 
        rng: &mut Pcg64,
        marker: MarkerShape,
        color: DarkLightColor
    ) -> Self
    {
        let config = (0..length.get())
            .map(|_| rng.gen_bool(0.5))
            .collect();
        Temperature{
            temperature: temp,
            config,
            marker,
            hist: HistU32Fast::new_inclusive(0, length.get()).unwrap(),
            color,
            acceptance: AcceptanceCounter::default(),
            ring_buffer: RingBuffer::new(NonZeroUsize::new(2000).unwrap())
        }
    }

    pub fn number_of_heads(&self) -> isize
    {
        self.config
            .iter()
            .filter(|&s| *s)
            .count() as isize
    }

    pub fn heads_rate(&self) -> f64
    {
        self.number_of_heads() as f64 / self.config.len() as f64
    }

    pub fn add_rejected_exchange_to_ringbuffer(
        &mut self
    ){
        self.ring_buffer.repeat_last();
    }
}


pub fn parallel_tempering_gui(any: &mut BoxedAnything, ctx: &egui::Context)
{
    let is_dark_mode = ctx.style()
        .visuals
        .dark_mode;
    let data: &mut ParallelTemperingData = any.to_something_or_default_mut();
    if data.marker_cycle.is_none(){
        let markers: Vec<_> = MarkerShape::all().collect();
        let iter = markers
            .into_iter()
            .cycle();
    
        data.marker_cycle = Some(
            Box::new(iter)
        );
    }

    if data.color_cycle.is_none(){
        let iter = COLORS.iter()
            .cycle()
            .copied();
        data.color_cycle = Some(
            Box::new(iter)
        );
    } 
    
    if matches!(data.side_panel, SidePanelView::Default){
        let screen_width = ctx.screen_rect().width();
        let is_desktop = screen_width > 600.0;
        data.side_panel = if is_desktop{
            SidePanelView::Shown
        } else {
            SidePanelView::Hidden
        };
    }

    match data.side_panel{
        SidePanelView::Shown => {
            egui::SidePanel::left("ParallelLeft")
                .show(
                    ctx, 
                    |ui|
                    {
                        if ui.button("Hide Side Panel")
                            .on_hover_text("Will hide the side panel")
                            .clicked() {
                            data.side_panel = SidePanelView::Hidden;
                        }
                        ui.horizontal(
                            |ui|
                            {
                                ui.add(Label::new("Temperature"));
                                ui.add(egui::DragValue::new(&mut data.temperature_to_add)
                                        .speed(DRAG_SPEED)
                                    ).on_hover_text("Click to type a number. Or drag the value for quick changes.");
                            }
                        );

                        let add_btn = ui.add(Button::new("add temperature"));
                        if data.temperature_to_add == 0.0 {
                            add_btn.show_tooltip_text("We divide by the temperature in the formula for the acceptance probability. Thus 0 is an invalid temperature.");
                        }
                        else if add_btn
                            .clicked()
                        {
                            let to_add = data.temperature_to_add;
                            let added = data.add_temperature(to_add);
                            if added{
                                data.sort_temps();
                                loop{
                                    data.temperature_to_add /= 2.0;
                                    if !data.contains_temp(data.temperature_to_add){
                                        break;
                                    }
                                }
                            }

                        }

                        ui.horizontal(
                            |ui|
                            {
                                ui.label("number of Coins");
                                let old_num = data.num_coins;
                                ui.add(
                                    egui::DragValue::new(&mut data.num_coins)
                                ).on_hover_text("Use this to change the size of the system, i.e., the number of coins. Will reset histograms etc. since the configurations are changed.");
                                if old_num != data.num_coins && !data.temperatures.is_empty(){
                                    data.new_length();
                                }
                            }
                        );
                    
                        if !data.temperatures.is_empty(){

                            ui.label("Which plots to show:");
                            data.show_plot.radio(ui, "Plot");
                            data.show_histogram.radio(ui, "Histogram");
                            data.show_acceptance.radio(ui, "Acceptance Rate");
                            data.show_history.radio(ui, "History");
                            
                        } 

                        if ui.add(Button::new("Add Example Temperatures"))
                                .on_hover_text("Add some example temperatures. If all example temperatures are already present: Nothing happens when clicked.")
                                .clicked()
                            {
                                for tmp in DEFAULT_TEMPERATURES{
                                    let _ = data.add_temperature(tmp);
                                }
                                data.sort_temps();
                            }

                        if !data.temperatures.is_empty() && ui.add(
                                Button::new("Remove all Temperatures")
                            ).clicked()
                        {
                            // cannot be part of he next if statement,
                            // that would result in a bug
                            data.temperatures.clear();
                            data.step_counter = 0;
                        }

                        if !data.temperatures.is_empty(){

                            if ui.add(Button::new("Reset Histograms")).clicked()
                            {
                                data.temperatures.iter_mut()
                                    .for_each(
                                        |t| t.hist.reset()
                                    );
                            }
                            if ui.add(Button::new("Reset Acceptance")).clicked()
                            {
                                data.temperatures.iter_mut()
                                    .for_each(
                                        |t| t.acceptance.reset()
                                    );
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
                        
                            ui.label("Adjust/delete temperatures:");
                        

                            // Adjust top temperature
                            let mut iter = data.temperatures
                                .iter_mut()
                                .rev();
                            let tmp = iter.next().unwrap();

                            fn top<W>(
                                ui: &mut egui::Ui, 
                                widget: W
                            ) -> ToRemove
                            where W: Widget
                            {
                                let mut to_remove = ToRemove::Nothing;
                                ui.horizontal(
                                    |ui|
                                    {
                                        ui.label("Top:");
                                        ui.add(widget);
                                        if ui.button("🗑").clicked(){
                                            to_remove = ToRemove::Top;
                                        }
                                    }
                                );
                                to_remove
                            }
                         

                            if let Some(next_tmp) = iter.next(){
                                let other = next_tmp.temperature;
                                let to_remove = if other.signum() == tmp.temperature.signum(){
                                    let range = if other.is_sign_negative(){
                                        other..=f64::NEG_INFINITY
                                    } else {
                                        f64::EPSILON..=other
                                    };
                                    let widget = DragValue::new(&mut tmp.temperature)
                                        .speed(DRAG_SPEED)
                                        .range(range);
                                    top(ui, widget)
                                } else {
                                    let range = f64::EPSILON..=f64::INFINITY;
                                    let widget = DragValue::new(&mut tmp.temperature)
                                        .speed(DRAG_SPEED)
                                        .range(range);
                                    top(ui, widget)
                                };
                                data.remove(to_remove);
                            }
                        
                        
                            // Adjusting Clamped temperatures. Has been debugged already
                            let current_temperatures: Vec<_> = data.temperatures
                                .iter()
                                .map(|t| t.temperature)
                                .collect();
                        
                            let mut idx = data.temperatures.len() - 1;
                            let windows = current_temperatures.windows(3);
                            let temperature_iter = data.temperatures
                                .iter_mut()
                                .skip(1);
                        
                            let mut to_remove = ToRemove::Nothing;
                            
                            for (window, temperature) in windows.zip(temperature_iter).rev()
                            {
                                idx -= 1;
                                let min = window[0].min(window[2]);
                                let max = window[2].max(window[0]);
                                if max.signum() == min.signum() {
                                    ui.horizontal(
                                        |ui|
                                        {
                                            ui.add(
                                                Slider::new(&mut temperature.temperature, min..=max)
                                            );
                                            if ui.button("🗑").clicked(){
                                                to_remove = ToRemove::Idx(idx);
                                            }
                                        }
                                    );
                                } else{
                                    let range = if window[1].is_sign_negative(){
                                        f64::NEG_INFINITY..=min
                                    } else {
                                        max..=f64::INFINITY
                                    };
                                    // No slider possible because one of the borders is infinite
                                    ui.horizontal(
                                        |ui|
                                        {
                                            ui.add(
                                                DragValue::new(&mut temperature.temperature)
                                                    .range(range)
                                            );
                                            if ui.button("🗑").clicked(){
                                                to_remove = ToRemove::Idx(idx);
                                            }
                                        }
                                    );
                                    
                                }

                            }
                            data.remove(to_remove);
                        
                            // Adjusting bottom temperature
                            let mut iter = data.temperatures.iter_mut();
                            let tmp = iter.next().unwrap();
                            fn bottom<W>(
                                ui: &mut egui::Ui, 
                                widget: W
                            ) -> ToRemove
                            where W: Widget
                            {
                                let mut to_remove = ToRemove::Nothing;
                                ui.horizontal(
                                    |ui|
                                    {
                                        ui.label("Bottom:");
                                        ui.add(widget);
                                        if ui.button("🗑").clicked(){
                                            to_remove = ToRemove::Bottom;
                                        }
                                    }
                                );
                                to_remove
                            }
                        
                            let to_remove = match iter.next(){
                                Some(next_tmp) => {
                                    let other = next_tmp.temperature;
                                    match other.signum() == tmp.temperature.signum(){
                                        true => {
                                            let range = if other.is_sign_negative(){
                                                other..=(-f64::EPSILON)
                                            } else {
                                                other..=f64::INFINITY
                                            };
                                            let widget = DragValue::new(&mut tmp.temperature)
                                                .speed(DRAG_SPEED)
                                                .range(range);
                                            bottom(ui, widget)
                                        },
                                        false => {
                                            let range = -f64::EPSILON..=f64::NEG_INFINITY;
                                            let widget = DragValue::new(&mut tmp.temperature)
                                                .speed(DRAG_SPEED)
                                                .range(range);
                                            bottom(ui, widget)
                                        }
                                    }
                                },
                                None => {
                                    let widget = DragValue::new(&mut tmp.temperature)
                                        .speed(DRAG_SPEED);
                                    bottom(ui, widget)
                                }
                            };
                            data.remove(to_remove);
                        
                        
                        }
                    }
                );
        },
        SidePanelView::Hidden => {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                if ui.button("Open Side Panel").clicked() {
                    data.side_panel = SidePanelView::Shown;
                }
            });
        },
        SidePanelView::Default => unreachable!()
    }

    


    egui::CentralPanel::default().show(ctx, |ui| {
        // The central panel the region left after adding TopPanel's and SidePanel's

        let mut step_performed = false;
        if !data.paused || data.step_once{
            data.temperatures
                .iter_mut()
                .for_each(
                    |temp| temp.markov_step(&mut data.rng)
                );
            // steps were performed if there was at least one config
            step_performed = !data.temperatures.is_empty();
        }


        let mut rect = ui.max_rect();

        let mut amount = 0;
        if data.show_plot.is_show(){
            amount += 1;
        }
        if data.show_acceptance.is_show(){
            amount += 1;
        }
        if data.show_histogram.is_show(){
            amount += 1;
        }
        if data.show_history.is_show(){
            amount += 1;
        }

        let w = rect.width();
        rect.set_width(w/(amount as f32 + 0.1));

        ui.horizontal(
            |ui|
            {
                if data.show_plot.is_show(){
                    ui.vertical(
                        |ui|
                        {
                            ui.label("Plot");
                            show_plot(data, ui, is_dark_mode, rect);
                        }
                    );
                }
                if data.show_histogram.is_show(){
                    ui.vertical(
                        |ui|
                        {
                            ui.label("Histogram");
                            show_hist(data, ui, is_dark_mode, rect);
                        }
                    );
                }
                if data.show_acceptance.is_show(){
                    ui.vertical(
                        |ui|
                        {
                            ui.label("Acceptance Rate");
                            show_acceptance_rate(data, ui, is_dark_mode, rect);
                        }
                    );
                }
                if data.show_history.is_show(){
                    ui.vertical(
                        |ui|
                        {
                            ui.label("History");
                            show_history_plot(data, ui, is_dark_mode, rect);
                        }
                    );
                }
            }
        );


        

        data.step_once = false;
    
        if step_performed{
            data.step_counter += 1;
            if data.step_counter == data.num_coins.get(){
                // try exchanges
                data.step_counter = 0;
                temp_exchanges(
                    &mut data.rng, 
                    &mut data.temperatures
                );
            }
        }
    });  

    ctx.request_repaint();
}

fn show_acceptance_rate(
    data: &ParallelTemperingData,
    ui: &mut egui::Ui,
    is_dark_mode: bool,
    rect: Rect
){
    let mut plot_points = Vec::with_capacity(data.temperatures.len());
    for (id, temp) in data.temperatures.iter().enumerate()
    {
        let acceptance_rate = temp.acceptance.acceptance_rate();
        plot_points.push(([acceptance_rate, id as f64], (temp.marker, temp.color)));
    }

    let all_points = plot_points
        .into_iter()
        .map(
            |(plot_data, plot_config)|
            {
                let plot_points = PlotPoints::new(vec![plot_data]);
                Points::new(plot_points)
                    .radius(10.0)
                    .shape(plot_config.0)
                    .color(plot_config.1.get_color(is_dark_mode))
            }
        );

    let plot_bounds = PlotBounds::from_min_max(
        [0.0, 0.0], 
        [1.0+f64::EPSILON, (data.temperatures.len() - 1).max(1) as f64 + 0.01]
    );

    let y_axis = AxisHints::new_y()
        .label("Temperature")
        .formatter(
            |mark, _|
            {
                if mark.value.fract() < 0.01{
                    let val = mark.value.round() as isize;
                    if val >= 0 {
                        match data.temperatures.get(val as usize){
                            Some(tmp)  => tmp.temperature.to_string(),
                            None => "".to_owned()
                        }
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_owned()
                }
            }
        );

    Plot::new("acc_plot")
        .x_axis_label("Acceptance rate")
        .show_y(false)
        .custom_y_axes(vec![y_axis])
        .width(rect.width())
        .height(rect.height())
        .show(
            ui, 
            |plot_ui|
            {
                for points in all_points{
                    plot_ui.points(points);
                }
                plot_ui.set_plot_bounds(plot_bounds);
            }
        );
}


fn show_plot(
    data: &ParallelTemperingData, 
    ui: &mut egui::Ui,
    is_dark_mode: bool,
    rect: Rect
)
{
    let mut plot_points: Vec<([f64; 2], (MarkerShape, DarkLightColor))> = Vec::with_capacity(data.temperatures.len());
    for (id, temp) in data.temperatures.iter().enumerate()
    {
        let heads_rate = temp.heads_rate();
        plot_points.push(([heads_rate, id as f64], (temp.marker, temp.color)));
    }

    let all_points = plot_points
        .into_iter()
        .map(
            |(plot_data, plot_config)|
            {
                let plot_points = PlotPoints::new(vec![plot_data]);
                Points::new(plot_points)
                    .radius(10.0)
                    .shape(plot_config.0)
                    .color(plot_config.1.get_color(is_dark_mode))
            }
        );

    let plot_bounds = PlotBounds::from_min_max(
        [0.0, -0.33], 
        [1.0+f64::EPSILON, (data.temperatures.len() - 1).max(1) as f64 + 0.33]
    );

    let y_axis = AxisHints::new_y()
        .label("Temperature")
        .formatter(
            |mark, _|
            {
                if mark.value.fract().abs() < 0.01{
                    let val = mark.value.round() as isize;
                    if val >= 0 {
                        match data.temperatures.get(val as usize){
                            Some(tmp)  => {
                                tmp.temperature.to_string()
                            },
                            None => "".to_owned()
                        }
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_owned()
                }
            }
        );

    Plot::new("my_plot")
        .x_axis_label("Heads rate")
        .show_y(false)
        .custom_y_axes(vec![y_axis])
        .width(rect.width())
        .height(rect.height())
        .show(
            ui, 
            |plot_ui|
            {
                for points in all_points{
                    plot_ui.points(points);
                }
                plot_ui.set_plot_bounds(plot_bounds);
            }
        );


}

fn show_history_plot(
    data: &ParallelTemperingData, 
    ui: &mut egui::Ui,
    is_dark_mode: bool,
    rect: Rect
)
{
    let min_height = 0.99 * rect.height() / (data.temperatures.len() as f32);
    Grid::new("HistGrid")
        .min_row_height(min_height)
        .min_col_width(rect.width())
        .show(
        ui, 
        |ui|
        {
            for (id, temp) in data.temperatures.iter().rev().enumerate(){
        
                let line = Line::new(
                    temp
                        .ring_buffer
                        .iter()
                        .enumerate()
                        .map(
                            |(x, &heads)|
                            {
                                [x as f64, heads as f64]
                            }
                        ).collect::<Vec<_>>()
                    ).name(format!("T={}", temp.temperature))
                    .color(temp.color.get_color(is_dark_mode));

                Plot::new(format!("{id}HISTPLOT"))
                    .clamp_grid(true)
                    .legend(Legend::default())
                    .show(
                        ui, 
                        |plot_ui|
                        {
                            plot_ui.line(line);
                        }    
                    );
                ui.end_row();
            }
        }
    );


}

fn show_hist(
    data: &ParallelTemperingData, 
    ui: &mut egui::Ui,
    is_dark_mode: bool,
    rect: Rect
)
{
    let min_height = 0.99 * rect.height() / (data.temperatures.len() as f32);
    Grid::new("HistGrid")
        .min_row_height(min_height)
        .min_col_width(rect.width())
        .show(
        ui, 
        |ui|
        {
            for (id, temp) in data.temperatures.iter().rev().enumerate(){
        
                let chart = BarChart::new(
                    temp.hist
                        .hist()
                        .iter()
                        .enumerate()
                        .map(
                            |(x, hits)|
                            {
                                Bar::new(x as f64, *hits as f64)
                                    .width(1.0)
                            }
                        ).collect()
                ).color(temp.color.get_color(is_dark_mode))
                .name(format!("T={}", temp.temperature));

                Plot::new(format!("{id}HISTPLOT"))
                    .clamp_grid(true)
                    .legend(Legend::default())
                    .show(
                        ui, 
                        |plot_ui|
                        {
                            plot_ui.bar_chart(chart);
                        }    
                    );
                ui.end_row();
            }
        }
    );
}


fn temp_exchanges(rng: &mut Pcg64, temperatures: &mut [Temperature])
{
    if temperatures.len() < 2{
        return;
    }
    let num_pairs = temperatures.len() - 1;

    for _ in 0..num_pairs
    {
        let lower = rng.gen_range(0..num_pairs);
        let mut iter = temperatures
            .iter_mut()
            .skip(lower);
        let a = iter.next().unwrap();
        let b = iter.next().unwrap();
        let exchange_prob = exchange_acceptance_probability(a, b);
        println!(
            "{lower} {exchange_prob}"
        );
        if exchange_prob >= rng.gen()
        {
            exchange_temperatures(a, b);
        } else {
            a.add_rejected_exchange_to_ringbuffer();
            b.add_rejected_exchange_to_ringbuffer();
        }
    }
}

fn exchange_temperatures(
    a: &mut Temperature,
    b: &mut Temperature
){
    swap(
        &mut a.marker, 
        &mut b.marker
    );
    swap(
        &mut a.config,
        &mut b.config 
    );
    swap(
        &mut a.color, 
        &mut b.color
    );

    let ea = a.number_of_heads() as u32;
    let eb = b.number_of_heads() as u32;

    a.hist.increment_quiet(ea);
    a.ring_buffer.push(ea);
    b.hist.increment_quiet(eb);
    b.ring_buffer.push(eb);
}

fn exchange_acceptance_probability(
    a: &Temperature, 
    b: &Temperature
) -> f64
{
    assert!(
        SortHelper{temp: NotNan::new(a.temperature).unwrap()} 
        <=
        SortHelper{temp: NotNan::new(b.temperature).unwrap()},
        "{a:?} {b:?}"
    );
    //assert!(
    //    a.temperature <= b.temperature
    //);
    let ea = a.heads_rate();
    let eb = b.heads_rate();
    1.0_f64.min(
        (-(1.0/a.temperature - 1.0/b.temperature) * (ea - eb))
            .exp()
    )
}


pub enum ToRemove{
    Nothing,
    Top,
    Bottom,
    Idx(usize)
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Show{
    Yes,
    #[default]
    No
}

impl Show{
    pub fn radio(&mut self, ui: &mut egui::Ui, name: &str)
    {
        ui.horizontal(
            |ui|
            {
                label(ui, name, MONOSPACE_LEN);
                ui.radio_value(self, Self::Yes, "Y");
                ui.radio_value(self, Self::No, "N");
            }
        );
    }

    pub fn is_show(&self) -> bool{
        matches!(self, Self::Yes)
    }
}

fn label(ui: &mut egui::Ui, name: &str, len: usize)
{
    let mut this_str = name.to_owned();
    this_str.truncate(len);
    while this_str.len() < len {
        this_str.push(' ');
    }
    let rich: RichText = this_str.into();
    let rich = rich.monospace();
    ui.label(rich);
}