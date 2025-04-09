use egui::{OutputCommand, Ui};
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use lazy_static::lazy_static;

#[derive(Clone, Copy, Debug, Default)]
pub enum DisplayLanguage
{
    C,
    #[default]
    Rust
}

lazy_static!{
    static ref C_SYNTAX: Syntax ={
        let c_types = [
            "int",
            "void",
            "double",
            "char",
            "short",
            "long",
            "unsigned",
            "signed",
            "float",
            "union"
        ];

        let keywords = [
            "if",
            "else",
            "struct",
            "break",
            "continue",
            "for",
            "while",
            "do",
            "auto",
            "static",
            "volatile",
            "goto",
            "default",
            "typedef",
            "switch",
            "enum",
            "case",
            "const"
        ];

        let special = [
            "return"
        ];

        Syntax::new("C")
            .with_comment("//")
            .with_comment_multiline(["/*", "*/"])
            .with_types(c_types)
            .with_keywords(keywords)
            .with_case_sensitive(true)
            .with_special(special)
    };
}

impl DisplayLanguage{
    pub fn get_syntax(self) -> Syntax
    {
        match self{
            Self::C => {
                (*C_SYNTAX).clone()
            },
            Self::Rust => {
                Syntax::rust()
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Code{
    pub current_display_language: DisplayLanguage,
    pub c_code: Option<String>,
    pub rust_code: Option<String>,
    pub show_code: bool
}

impl Code{
    pub fn new_rust(
        rust_code: String
    ) -> Self 
    {
        Self{
            current_display_language: DisplayLanguage::Rust,
            c_code: None,
            rust_code: Some(rust_code),
            show_code: true
        }
    }

    pub fn new_c(
        c_code: String
    ) -> Self 
    {
        Self{
            current_display_language: DisplayLanguage::C,
            c_code: Some(c_code),
            rust_code: None,
            show_code: true
        }
    }

    pub fn new(
        rust_code: String,
        c_code: String
    ) -> Self 
    {
        Self{
            current_display_language: DisplayLanguage::default(),
            rust_code: Some(rust_code),
            c_code: Some(c_code),
            show_code: true
        }
    }

    pub fn display(&mut self, ui: &mut Ui, is_dark_mode: bool)
    {

        let both_avail = self.c_code.is_some() && self.rust_code.is_some();
        let code = match self.current_display_language{
            DisplayLanguage::C => {
                if self.c_code.is_none(){
                    self.c_code = Some("No Code Available".to_owned());
                }
                self.c_code.as_mut().unwrap()
            },
            DisplayLanguage::Rust => {
                if self.rust_code.is_none(){
                    self.rust_code = Some("No Code Available".to_owned());
                }
                self.rust_code.as_mut().unwrap()
            }
        };

        ui.horizontal(
            |ui|
            {
                let txt = if self.show_code{
                    "Hide Code"
                } else {
                    "Show Code"
                };
                if ui.button(txt).clicked(){
                    self.show_code = !self.show_code;
                }
                if self.show_code && ui.button("Copy Code 📋").on_hover_text("Click to copy code").clicked() {
                    ui.output_mut(
                        |o| {
                            let command = OutputCommand::CopyText(code.clone());
                            o.commands.push(command);
                        }
                    )
                    
                }

                if both_avail && self.show_code{
                    match self.current_display_language{
                        DisplayLanguage::C => {
                            if ui.button("Switch to Rust").clicked(){
                                self.current_display_language = DisplayLanguage::Rust;
                            }
                        },
                        DisplayLanguage::Rust => {
                            if ui.button("Switch to C").clicked(){
                                self.current_display_language = DisplayLanguage::C;
                            }
                        }
                    }
                }
            }
        );

        if self.show_code{
            let theme = if is_dark_mode{
                ColorTheme::AYU_DARK
            } else {
                ColorTheme::AYU
            };
    
            CodeEditor::default()
                .id_source("code editor")
                .with_rows(2)
                .with_fontsize(14.0)
                .with_theme(theme)
                .with_syntax(self.current_display_language.get_syntax())
                .with_numlines(true)
                .auto_shrink(true)
                .show(ui, code);
        }

    }
}