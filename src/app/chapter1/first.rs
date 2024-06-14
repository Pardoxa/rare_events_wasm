use derivative::Derivative;
// This is an example
use crate::dark_magic::*;
use crate::app::code_editor::Code;

#[derive(Debug, Derivative)]
#[derivative(Default)]
struct FirstData{
    slider: f64,
    #[derivative(Default(value="Code::new(CODE_RUST.to_owned(), CODE_C.to_owned())"))]
    code: Code
}

pub fn chapter_1_switch(any: &mut BoxedAnything, ctx: &egui::Context)
{
    let data: &mut FirstData = any.to_something_or_default_mut();
    let is_dark_mode = ctx.style().visuals.dark_mode;
    egui::CentralPanel::default().show(ctx, |ui| {
        // The central panel the region left after adding TopPanel's and SidePanel's
        ui.add(
            egui::Slider::new(
                &mut data.slider, 
                0.0..=100.0
            ).text("Test value")
        );

        data.code.display(ui, is_dark_mode)
    });
    
}

const CODE_RUST: &str = r#"
ui.add(
    egui::Slider::new(
        &mut data.slider, 
        0.0..=100.0
    ).text("Test value")
);
"#;

const CODE_C: &str = r#"
/******************** gs_block_model() ********************/
/** Generates graph according to stochastic block model. **/
/** Nodes are members of groups. For each pair of nodes **/
/** an edge is addged with probability p1 if they are in **/
/** the same group, and with probability p2, if they are **/
/** in different groups. **/
/** PARAMETERS: (*)= return-paramter **/
/** (*) g: graph **/
/** p1: edge probability (same group) **/
/** p2: edge probability (different groups) **/
/** group: group ID for each node **/
/** RETURNS: **/
/** (nothing) **/
/*********************************************************/
void gs_block_model(gs_graph_t *g, double p1, double p2, int *group)
"#;