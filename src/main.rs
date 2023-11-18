use std::{path::Path, time::Duration, sync::mpsc};
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;
use clap::{Parser};
use lightningcss::stylesheet::{StyleSheet, ParserOptions, PrinterOptions};
use std::io::Read;
use std::fs::File;
use serde::Deserialize;
use toml;

const DEFAULT_INPUT: &str = "src";
const DEFAULT_OUTPUT: &str = "public/assets/css";
const DEFAULT_CONFIG_FILE: &str = "lightningcss.config.toml";

#[derive(Parser, Deserialize)]
#[command(author, version, about, long_about = None)]
struct Cli {

    /// Write out only once.
    #[arg(long)]
    build: bool,

    /// Input directory to list css files
    #[arg(short, long, value_name = "DIR", default_value = DEFAULT_INPUT)]
    input: PathBuf,

    /// Output directory to minify css files to
    #[arg(short, long, value_name = "DIR", default_value = DEFAULT_OUTPUT)]
    output: PathBuf,

    /// Config file
    #[arg(short, long, value_name = "FILE", default_value = DEFAULT_CONFIG_FILE)]
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
            if cli.input.to_str().unwrap() == DEFAULT_INPUT {
                cli.input = config.input;
            }
            if cli.output.to_str().unwrap() == DEFAULT_OUTPUT {
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

    fn process_css_files(input_dir: &Path, output_dir: &Path) {
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
          // Create directory if it does not exist
          if !output_dir.exists() {
            fs::create_dir_all(&output_dir).unwrap();
          }                
          fs::write(output_path, minified_css).unwrap();
        }
      }
    }
  
    if cli.build {
      // Build mode
      process_css_files(input_dir, output_dir);
    } else {
      // Normal mode
      env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("debouncer_mini=trace"),
      )
      .init();

      // setup debouncer
      let (tx, rx) = mpsc::channel();

      // No specific tickrate, max debounce time 1 seconds
      let mut debouncer = new_debouncer(Duration::from_secs(1), tx).unwrap();

      // Watch the specified directory and its subdirectories
      debouncer
          .watcher()
          .watch(input_dir, RecursiveMode::Recursive)
          .unwrap();

      // print all events, non returning
      for result in rx {
          match result {
              Ok(events) => events
                  .iter()
                  .for_each(|event| {
                    log::info!("Event {event:?}");
                    process_css_files(input_dir, output_dir); 
                  }),
              Err(error) => log::info!("Error {error:?}"),
          }
      }
    
    }
}