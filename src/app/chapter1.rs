use crate::dark_magic::BoxedAnything;

use super::Chapter1;

mod first;
mod second;
mod parallel_tempering;


pub fn chapter_1_switch(which: &Chapter1, any: &mut BoxedAnything, ctx: &egui::Context)
{
    match which{
        Chapter1::First => {
            first::chapter_1_switch(any, ctx)
        },
        Chapter1::Second => {
            second::chapter_1_switch(any, ctx)
        },
        Chapter1::Third => {
            parallel_tempering::parallel_tempering_gui(any, ctx);
        }
    }
}