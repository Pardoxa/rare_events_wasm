use derivative::Derivative;
use egui::{Button, DragValue, Label};
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
    temperature_to_add: f64,
    #[derivative(Default(value="100"))]
    num_coins: u32,
    seed: u64,
    rng: Option<Pcg64>
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
        let entry = self.config.choose_mut(rng).unwrap();
        *entry = rng.gen_bool(0.5);
    }
}


impl Temperature{
    pub fn new(temp: f64, length: usize) -> Self
    {
        Temperature{
            temperature: temp,
            config: vec![false; length]
        }
    }

    pub fn count_true(&self) -> usize
    {
        self.config.iter().filter(|&s| *s).count()
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
                            .speed(1)
                            );
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
                
                
                if ui.add(Button::new("add temperature"))
                    .clicked()
                {
                    let to_add = data.temperature_to_add;
                    if !data.contains_temp(to_add){
                        data.temperatures.push(
                            Temperature::new(
                                data.temperature_to_add,
                                data.num_coins as usize
                            )
                        );
                        data.sort_temps();
                    }
                    
                }

        
                if !data.temperatures.is_empty() && ui.add(
                    Button::new("Remove all Temperatures")
                ).clicked()
                {
                    data.temperatures.clear();
                }
                    
            }
        );


    egui::CentralPanel::default().show(ctx, |ui| {
        // The central panel the region left after adding TopPanel's and SidePanel's
        for temp in data.temperatures.iter_mut()
        {
            temp.markov_step(data.rng.as_mut().unwrap());
            let heads = temp.count_true();
            let label = format!("Temp: {} Heads: {}", temp.temperature, heads);
            ui.label(label);
        }
        
    });
    
}