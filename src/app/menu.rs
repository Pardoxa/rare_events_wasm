use super::chapter_markers::*;
use egui::{Label, Sense};
use strum::IntoEnumIterator;

pub trait MenuAction{
    fn change_chapter_anchor(&self) -> ChapterAnchor
    {
        ChapterAnchor::Chapter2(super::Chapter2::First)
    }
}

impl MenuAction for (){
    fn change_chapter_anchor(&self) -> ChapterAnchor
    {
        ChapterAnchor::Invalid
    }
}

impl MenuAction for Chapter1{
    fn change_chapter_anchor(&self) -> ChapterAnchor {
        ChapterAnchor::Chapter1(*self)
    }
}

impl MenuAction for Chapter2{
    fn change_chapter_anchor(&self) -> ChapterAnchor {
        ChapterAnchor::Chapter2(*self)
    }
}

pub trait GenerateChapterList{
    fn generate_menu_vec() -> Vec<MenuOrSubMenu>;
}

impl<T> GenerateChapterList for T
    where T: IntoEnumIterator + MenuAction + std::fmt::Debug + 'static
{
    fn generate_menu_vec() -> Vec<MenuOrSubMenu> {
        T::iter().map(
            |variant|
            {
                let item = MenuItem{
                    name: format!("{:?}", variant),
                    action: Box::new(variant)
                };
                MenuOrSubMenu::Menu(item)
            }
        ).collect()
    }
}

pub struct GlobalContextMenu{
    pub menu: MenuOrSubMenu
}

impl Default for GlobalContextMenu{
    fn default() -> Self {
        
        let chapter1_list = Chapter1::generate_menu_vec();
        let sub_ch1 = SubMenu{list: chapter1_list,
            sub_menu_name: "Chapter1".to_owned()
        };
        let chapter2_list = Chapter2::generate_menu_vec();
        let sub_ch2 = SubMenu{list: chapter2_list,
            sub_menu_name: "Chapter2".to_owned()
        };

        let global_sub = SubMenu{
            sub_menu_name: "☰".to_string(),
            list: vec![
                MenuOrSubMenu::SubMenu(sub_ch1),
                MenuOrSubMenu::SubMenu(sub_ch2)
            ]
        };
        Self { menu: MenuOrSubMenu::SubMenu(global_sub) }
    }
}

impl GlobalContextMenu{
    fn nested_menu(&self, ui: &mut egui::Ui, anchor: &mut ChapterAnchor)
    {
        self.menu.nested_menu(ui, anchor)
        
    }

    pub fn print_links(&self, ui: &mut egui::Ui, anchor: &mut ChapterAnchor)
    {
        match &self.menu{
            MenuOrSubMenu::SubMenu(sub) => {
                for item in sub.list.iter(){
                    item.nested_links(ui, anchor, 0)
                }
            },
            MenuOrSubMenu::Menu(_) => {
                self.print_links(ui, anchor)
            }
        }
    }
}

pub struct MenuItem{
    pub name: String,
    pub action: Box<dyn MenuAction>
}

pub struct SubMenu{
    sub_menu_name: String,
    list: Vec<MenuOrSubMenu>
}

pub enum MenuOrSubMenu{
    Menu(MenuItem),
    SubMenu(SubMenu)
}

impl MenuOrSubMenu{
    fn nested_menu(&self, ui: &mut egui::Ui, anchor: &mut ChapterAnchor)
    {
        match self{
            MenuOrSubMenu::Menu(item) => {
                if ui.button(&item.name).clicked(){
                    *anchor = item.action.change_chapter_anchor();
                }
            },
            MenuOrSubMenu::SubMenu(sub) => {
                ui.menu_button(
                    &sub.sub_menu_name, 
                    |ui|
                    {
                        for entry in sub.list.iter(){
                            entry.nested_menu(ui, anchor)
                        }
                    }
                );
                
            }
        }
    }

    fn nested_links(&self, ui: &mut egui::Ui, anchor: &mut ChapterAnchor, tabs: u8)
    {
        match self{
            MenuOrSubMenu::Menu(item) => {
                let mut text = String::new();
                for _ in 0..tabs{
                    text.push('\t');
                }
                text.push_str(&item.name);
                let label = Label::new(text)
                    .sense(Sense::click())
                    .selectable(false);
                if ui.add(label).on_hover_cursor(egui::CursorIcon::PointingHand).clicked(){
                    *anchor = item.action.change_chapter_anchor();
                }
            },
            MenuOrSubMenu::SubMenu(sub) => {
                ui.label(&sub.sub_menu_name);
                ui.vertical( 
                    |ui|
                    {
                        for entry in sub.list.iter(){
                            entry.nested_links(ui, anchor, tabs + 1)
                        }
                    }
                );
                
            }
        }
    }
}



pub fn default_menu(ctx: &egui::Context, anchor: &mut ChapterAnchor)
{
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        // The top panel is often a good place for a menu bar:

        egui::menu::bar(ui, |ui| {
            // If I want to do different things in native or web-app
            // let is_web = cfg!(target_arch = "wasm32");
            let default_menu = GlobalContextMenu::default();
            default_menu.nested_menu(ui, anchor);
            

            egui::widgets::global_dark_light_mode_buttons(ui);
        });
    });
}