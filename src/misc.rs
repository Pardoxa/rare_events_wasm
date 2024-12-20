use std::{collections::VecDeque, num::NonZeroUsize};

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


#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone)]
pub struct RingBuffer<T>{
    buffer: VecDeque<T>,
    max_len: NonZeroUsize
}

impl<T> RingBuffer<T>{
    pub fn push(&mut self, t: T){
        if self.buffer.len() == self.max_len.get(){
            self.buffer.pop_front();
        }
        self.buffer.push_back(t);
    }
    
    /// Front to Back Iterator
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &T>
    {
        self.buffer.iter()
    }

    pub fn new(length: NonZeroUsize) -> Self{
        Self { buffer: VecDeque::with_capacity(length.get()), max_len: length }
    }

    /// Fails silently if Ring Buffer is empty
    pub fn repeat_last(&mut self)
    where T: Clone
    {
        if let Some(val) = self.buffer.back(){
            self.push(val.clone());
        }
        
    }

    pub fn reset(&mut self)
    {
        self.buffer.clear();
    }

    #[allow(clippy::len_without_is_empty)]
    #[inline]
    pub fn len(&self) -> usize
    {
        self.buffer.len()
    }
}

pub const HINT: &str = "To increase the size of the texts you can press 'ctrl' + '+'
To decrease the size of the texts you can press 'ctrl' + '-'\n";

pub fn colored_text(text: &str, color: Color32) -> RichText
{
    let txt: RichText = text.into();
    txt.color(color)
}


pub fn ln_to_log10(slice: &mut [f64])
{
    slice.iter_mut()
            .for_each(|val| *val *= std::f64::consts::LOG10_E);
}