use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;
use clap::{Parser};
use lightningcss::stylesheet::{StyleSheet, ParserOptions, PrinterOptions};
use std::io::Read;
use std::fs::File;
use serde::Deserialize;
use toml;

#[derive(Parser, Deserialize)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input directory to list files
    #[arg(short, long, value_name = "DIR", default_value = "public")]
    input: PathBuf,

    /// Output directory to copy files to
    #[arg(short, long, value_name = "DIR", default_value = "dist")]
    output: PathBuf,

    /// Config file
    #[arg(short, long, value_name = "FILE", default_value = "lightningcss.config.toml")]
    config: Option<PathBuf>,
}

fn main() {
    let mut cli = Cli::parse();

    // If a config file is specified, read the options from the config file
    if let Some(config_path) = cli.config {
        if config_path.exists() {
            let mut config_file = File::open(config_path).unwrap();
            let mut config_string = String::new();
            config_file.read_to_string(&mut config_string).unwrap();
            let config: Cli = toml::from_str(&config_string).unwrap();

            // If the options are not specified in the command line, use the options from the config file
            if cli.input.to_str().unwrap() == "public" {
                cli.input = config.input;
            }
            if cli.output.to_str().unwrap() == "dist" {
                cli.output = config.output;
            }
        }
    }

    let input_dir = cli.input.as_path();
    let output_dir = cli.output.as_path();

    // Create directory if it does not exist
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).unwrap();
    }
    for entry in WalkDir::new(input_dir) {
        let entry = entry.expect("The default input directory 'public' does not exist. \nPlease create the 'public' directory or specify the correct input directory with the config option. \nRefer to '--help' for more information.\n");
        let output_path = output_dir.join(entry.path().strip_prefix(input_dir).unwrap());
        if entry.file_type().is_file() && entry.path().extension().unwrap_or_default() == "css" {
            let css = fs::read_to_string(entry.path()).unwrap();
            let stylesheet = StyleSheet::parse(&css, ParserOptions::default()).unwrap();
            let minified_css = stylesheet.to_css(PrinterOptions {
                minify: true,
                ..PrinterOptions::default()
            }).unwrap().code;
            fs::write(output_path, minified_css).unwrap();
        }
    }
}
