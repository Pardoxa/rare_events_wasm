use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use strum_macros::EnumString;
use std::marker;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChapterAnchor{
    Chapter1(Chapter1Marks),
    Chapter2(Chapter2Marks),
    Invalid
}

impl ChapterAnchor{
    pub fn get_string(&self) -> String
    {
        let s = match self{
            Self::Invalid => "Invalid".to_owned(),
            Self::Chapter1(mark) => ChapterReading::get_string(mark),
            Self::Chapter2(mark) => ChapterReading::get_string(mark)
        };
        format!("#{s}")
    }
}

impl Default for ChapterAnchor{
    fn default() -> Self {
        Self::Chapter1(Chapter1Marks::First)
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumString)]
pub enum Chapter1Marks{
    #[strum(ascii_case_insensitive)]
    First,
    #[strum(ascii_case_insensitive)]
    Second
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumString)]
pub enum Chapter2Marks{
    #[strum(ascii_case_insensitive)]
    First
}

pub trait ChapterReading {
    fn get_string(&self) -> String;
    fn read_str(s: &str) -> Option<Self>
        where Self: marker::Sized;
}

impl<T> ChapterReading for T 
    where T: IntoEnumIterator + std::fmt::Debug + std::str::FromStr 
{
    fn get_string(&self) -> String{
        let name = std::any::type_name::<Self>();
        let last = name.split("::").last().unwrap();
        let digits = r"\d+";
        let re = regex::Regex::new(digits).unwrap();
        let number = re.find(last)
            .map(|m| &last[m.start()..m.end()]);
        match number{
            Some(num) => {
                let name = format!("{self:?}");
                format!("chapter{num}-{}", name.to_ascii_lowercase())
            },
            None => "Invalid".to_owned()
        }
    }
    fn read_str(s: &str) -> Option<Self>{
        Self::from_str(s).ok()
    }
}




impl ChapterAnchor{
    pub fn read_str(url_infos: &str) -> Option<Self>
    {
        let mut iter = url_infos.split('-');
        let (chapter_str, jump_str) = match (iter.next(), iter.next()){
            (Some(ch_s), Some(jp_s)) => {
                (ch_s, jp_s)
            },
            _ => {
                return None;
            }
        };
        match chapter_str.to_ascii_lowercase().as_str(){
            "chapter1" | "chapter01" => {
                Chapter1Marks::read_str(jump_str)
                    .map(ChapterAnchor::Chapter1)
            },
            "chapter2" | "chapter02" => {
                Chapter2Marks::read_str(jump_str)
                    .map(ChapterAnchor::Chapter2)
            },
            _ => None
        }
    }
}



