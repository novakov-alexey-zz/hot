use std::fs;
use std::fs::DirEntry;
use std::path::Path;

use anyhow::{Context, Error, Result};
use handlebars::Handlebars;
use serde_json::Value;

use crate::hocon::merge_params;
use crate::TemplateContext;

fn register_templates(
    file_extension: &'static str,
    templates_dir: &str,
) -> Result<Handlebars<'static>> {
    let mut handlebars = Handlebars::new();
    handlebars.register_templates_directory(file_extension, Path::new(templates_dir))?;
    handlebars.set_strict_mode(true);
    Ok(handlebars)
}

pub fn render<F: Fn(&String) -> Result<()>>(ctx: TemplateContext, out: F) -> Result<()> {
    let params = merge_params(ctx.params_file.clone()).with_context(|| {
        format!(
            "Failed to load parameters from '{:?}' file",
            &ctx.params_file
        )
    })?;
    let extension = &ctx.template_extension;
    let handlebars = register_templates(&*extension, &*ctx.input_path).with_context(|| {
        format!(
            "Failed to register template directory '{}'",
            &ctx.input_path
        )
    })?;

    if Path::is_dir(&ctx.input_path.as_ref()) {
        if ctx.debug {
            println!("Input path {:?} is a directory", ctx.input_path);
        }
        render_files(&ctx, &params, &handlebars, out)
    } else if Path::is_file(&ctx.input_path.as_ref()) {
        render_file(&handlebars, &ctx.input_path.as_str(), &params, &ctx).and_then(|o| out(&o))
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
    if ctx.debug {
        let keys = handlebars.get_templates().keys();
        if keys.len() > 0 {
            println!("Available templates: {:?}", keys);
        } else {
            println!("There are no template file(s) in '{}'", ctx.input_path);
        }
    }
    let separator = format!("\n{}\n", ctx.separator);
    for (i, file) in filtered.enumerate() {
        let f = file?.file_name();
        if ctx.debug {
            println!("rendering file: {:?}", &f);
        }
        let file_name = f
            .to_str()
            .ok_or(format!("Failed to read file name: {:?}", f))
            .map_err(Error::msg);
        let _ = if i > 0 { out(&separator) } else { Ok(()) }
            .and_then(|_| render_file(&handlebars, &file_name?, &params, &ctx).map(|o| out(&o)))?;
    }
    Ok(())
}

fn render_file(
    handlebars: &Handlebars<'static>,
    file_path: &str,
    params: &Value,
    ctx: &TemplateContext,
) -> Result<String> {
    let template = file_path.trim_end_matches(ctx.template_extension);
    handlebars.render(template, &params).with_context(|| {
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
