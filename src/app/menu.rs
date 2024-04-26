
pub trait MenuAction{
    fn action(&self){}
}

impl MenuAction for (){
    fn action(&self) {
        
    }
}

pub struct GlobalContextMenu{
    pub menu: Vec<MenuOrSubMenu>
}

impl Default for GlobalContextMenu{
    fn default() -> Self {
        let menu_item = MenuItem{
            name: "Test Btn Without function".to_string(),
            action: Box::new(())
        };
        let menu_item2 = MenuItem{
            name: "Test Btn Without function".to_string(),
            action: Box::new(())
        };
        let menu_item3 = MenuItem{
            name: "Test Btn Without function".to_string(),
            action: Box::new(())
        };
        let menu_item4 = MenuItem{
            name: "Test Btn Without function".to_string(),
            action: Box::new(())
        };
        let menu_item5 = MenuItem{
            name: "Test Btn Without function".to_string(),
            action: Box::new(())
        };
        let menu = MenuOrSubMenu::Menu(menu_item);
        let menu2 = MenuOrSubMenu::Menu(menu_item2);
        let nested_sub = SubMenu{
            sub_menu_name: "Open Sub Menu".to_owned(),
            list: vec![
                MenuOrSubMenu::Menu(menu_item4),
                MenuOrSubMenu::Menu(menu_item5)
            ]
        };
        let sub = SubMenu{list: vec![
                MenuOrSubMenu::Menu(menu_item3), 
                MenuOrSubMenu::SubMenu(nested_sub)
            ],
            sub_menu_name: "Open Sub Menu".to_owned()
        };
        let sub = MenuOrSubMenu::SubMenu(sub);
        Self { menu: vec![menu, menu2, sub] }
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
    fn nested_menu(&self, ui: &mut egui::Ui)
    {
        match self{
            MenuOrSubMenu::Menu(item) => {
                if ui.button(&item.name).clicked(){
                    item.action.action()
                }
            },
            MenuOrSubMenu::SubMenu(sub) => {
                ui.menu_button(
                    &sub.sub_menu_name, 
                    |ui|
                    {
                        for entry in sub.list.iter(){
                            entry.nested_menu(ui)
                        }
                    }
                );
                
            }
        }
    }
}

impl GlobalContextMenu{
    fn nested_menu(&self, ui: &mut egui::Ui)
    {
        for entry in self.menu.iter(){
            entry.nested_menu(ui)
        }
    }
}

pub fn default_menu(ctx: &egui::Context)
{
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        // The top panel is often a good place for a menu bar:

        egui::menu::bar(ui, |ui| {
            // NOTE: no File->Quit on web pages!
            let is_web = cfg!(target_arch = "wasm32");
            if !is_web {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);
            }
            let default_menu = GlobalContextMenu::default();
            default_menu.nested_menu(ui);
            

            egui::widgets::global_dark_light_mode_buttons(ui);
        });
    });
}