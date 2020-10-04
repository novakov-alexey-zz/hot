#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub params_file: String,
    pub input_path: String,
    pub template_extension: &'static str,
    pub separator: String,
    pub debug: bool,
}

pub mod hbs;
mod hocon;
