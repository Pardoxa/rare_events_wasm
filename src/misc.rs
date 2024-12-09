use egui::{Color32, RichText};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_HASH: &str = env!("GIT_HASH");
pub const COMPILE_TIME: &str = env!("BUILD_TIME_CHRONO");

pub fn get_rich_text_size(text: &str, size: f32) -> RichText
{
    let widget_text: RichText = text
        .into();
    widget_text
        .size(size)
}


#[derive(Debug, Clone, Copy)]
pub struct DarkLightColor
{
    pub dark: Color32,
    pub light: Color32
}

impl DarkLightColor {
    pub const fn get_color(&self, is_dark_mode: bool) -> Color32
    {
        if is_dark_mode{
            self.dark
        } else {
            self.light
        }
    }
}