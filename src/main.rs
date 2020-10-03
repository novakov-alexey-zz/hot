use std::{env, fs, process};
use std::fs::DirEntry;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Error, Result};
use handlebars::Handlebars;
use hocon::{Hocon, HoconLoader};
use serde_json::{Number, Value};

fn read_params(file: &str) -> Result<Value> {
    let hocon = HoconLoader::new().load_file(&file)?.hocon()?;
    hocon_to_json(hocon).ok_or_else(|| {
        Error::msg(format!(
            "Failed to convert config file '{}' to JSON format",
            file
        ))
    })
}

fn hocon_to_json(hocon: Hocon) -> Option<Value> {
    match hocon {
        Hocon::Boolean(b) => Some(Value::Bool(b)),
        Hocon::Integer(i) => Some(Value::Number(Number::from(i))),
        Hocon::Real(f) => Some(Value::Number(
            Number::from_f64(f).unwrap_or_else(|| Number::from(0)),
        )),
        Hocon::String(s) => Some(Value::String(s)),
        Hocon::Array(vec) => Some(Value::Array(
            vec.into_iter()
                .map(hocon_to_json)
                .filter_map(|i| i)
                .collect(),
        )),
        Hocon::Hash(map) => Some(Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, hocon_to_json(v)))
                .filter_map(|(k, v)| v.map(|v| (k, v)))
                .collect(),
        )),
        Hocon::Null => Some(Value::Null),
        Hocon::BadValue(_) => None,
    }
}

fn register_templates(
    file_extension: &'static str,
    templates_dir: &str,
) -> Result<Handlebars<'static>> {
    let mut handlebars = Handlebars::new();
    handlebars.register_templates_directory(file_extension, Path::new(templates_dir))?;
    handlebars.set_strict_mode(true);
    Ok(handlebars)
}

fn render<F: Fn(&String) -> Result<()>>(ctx: TemplateContext, out: F) -> Result<()> {
    let params = read_params(ctx.params_file.as_str())
        .with_context(|| format!("Failed to load parameters from '{}' file", &ctx.params_file))?;
    let extension = &ctx.template_extension;
    let handlebars = register_templates(&*extension, &*ctx.input_path).with_context(|| {
        format!(
            "Failed to register template directory '{}'",
            &ctx.input_path
        )
    })?;

    if Path::is_dir(&ctx.input_path.as_ref()) {
        render_files(&ctx, &params, &handlebars, out)
    } else if Path::is_file(&ctx.input_path.as_ref()) {
        render_file(
            &handlebars,
            &ctx.input_path.as_str(),
            &params,
            &ctx.template_extension,
        )
            .and_then(|o| out(&o))
    } else {
        Err(Error::msg(format!(
            "Cannot read input file/folder '{:?}'",
            ctx.input_path
        )))
    }
}

fn render_files<F: Fn(&String) -> Result<()>>(
    ctx: &TemplateContext,
    params: &Value,
    handlebars: &Handlebars<'static>,
    out: F,
) -> Result<()> {
    let filtered = fs::read_dir(&ctx.input_path)?.filter(|f| {
        f.as_ref()
            .map(|e| included(&e, &ctx.template_extension))
            .unwrap_or(false)
    });
    for file in filtered {
        let f = file?.file_name();
        let file_name = f
            .to_str()
            .ok_or(format!("Failed to read file name: {:?}", f))
            .map_err(Error::msg);
        let _ = render_file(&handlebars, &file_name?, &params, &ctx.template_extension)
            .map(|o| out(&o).and_then(|_| out(&"\n".to_string())))?;
    }
    Ok(())
}

fn render_file(
    handlebars: &Handlebars<'static>,
    file_path: &str,
    params: &Value,
    file_extension: &str,
) -> Result<String> {
    let template = file_path.trim_end_matches(file_extension);
    handlebars
        .render(template, &params)
        .with_context(|| {
            let templates = handlebars.get_templates().keys();
            format!(
                "Failed to render template: {}\nwith params:\n'{}'.\n\nAvailable templates: {:?}",
                template, params, templates
            )
        })
}

fn included(entry: &DirEntry, file_extension: &str) -> bool {
    let p = entry.path();
    p.file_name()
        .and_then(|n| n.to_str())
        .map(|path| path.contains(file_extension))
        .unwrap_or_else(|| false)
}

struct TemplateContext {
    params_file: String,
    input_path: String,
    template_extension: &'static str,
}

const DEFAULT_PARAMS: &str = "params.conf";
const DEFAULT_INPUT_PATH: &str = "./templates/";
const DEFAULT_TEMPLATE_EXTENSION: &str = ".yaml";

fn main() {
    let args: Vec<String> = env::args().collect();
    let default_input_path = &String::from(DEFAULT_INPUT_PATH);
    let input_path = args.get(1).unwrap_or(default_input_path);
    let default_params_file = &DEFAULT_PARAMS.to_string();
    let params_file = args.get(2).unwrap_or(default_params_file);
    let ctx = TemplateContext {
        params_file: params_file.to_string(),
        input_path: input_path.to_string(),
        template_extension: DEFAULT_TEMPLATE_EXTENSION,
    };
    let out = |s: &String| {
        std::io::stdout()
            .write(s.as_bytes())
            .map(|_| ())
            .with_context(|| "Failed to write to std out")
    };
    match render(ctx, out) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("{:?}", e);
            process::exit(1)
        }
    }
}
