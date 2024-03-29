use std::io::Write;
use std::process;

use anyhow::Context;
use clap::Parser;

use ht::hbs::render;
use ht::TemplateContext;

#[derive(Parser)]
#[clap(
author,
about = "Command line tool to render 'Handlebars' templates with values from 'HOCON' file.",
version
)]
struct Opts {
    #[clap(
        short,
        long,
        help = "Can take multiple values. Default is <templates>/params.conf",
        multiple = true
    )]
    params: Vec<String>,
    #[clap(
        short,
        long,
        help = "path to a folder with templates or to single template file",
        default_value = "./templates/"
    )]
    templates: String,
    #[clap(
        short,
        long,
        help = "file extension of the template(s)",
        default_value = ".yaml"
    )]
    extension: String,
    #[clap(
        short,
        long,
        help = "text line value to be printed between templates",
        default_value = "---"
    )]
    out_separator: String,
    #[clap(short, help = "Prints debug information")]
    debug: bool,
}

fn to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn main() {
    let opts: Opts = Opts::parse();
    let params = if opts.params.is_empty() {
        vec![format!(
            "{}/params.conf",
            opts.templates.trim_end_matches('/')
        )]
    } else {
        opts.params
    };
    let ctx = TemplateContext {
        params_file: params,
        input_path: opts.templates,
        template_extension: to_static_str(opts.extension.clone()),
        separator: opts.out_separator,
        debug: opts.debug,
    };
    if ctx.debug {
        println!("{:?}", ctx);
    }
    let to_stdout = |s: &String| {
        std::io::stdout()
            .write(s.as_bytes())
            .map(|_| ())
            .with_context(|| "Failed to write to std out")
    };
    match render(ctx, to_stdout) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("{:?}", e);
            process::exit(1)
        }
    }
}
