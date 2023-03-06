use serde_derive::Deserialize;
use std::fs;
use std::process::exit;
use toml;

use crate::cli::Cli;
use crate::snr::Snr;

pub mod cli;
pub mod snr;

#[derive(Deserialize)]
struct Config {
    cli: Cli,
    unix: Cli,
    win: Cli,
    snr: Snr,
    version: String,
}

fn main() {
    let filename = "campfire.toml";

    let contents = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Could not read file `{}`", filename);
            exit(1);
        }
    };

    let data: Config = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(err) => {
            eprintln!("Unable to load data from `{}`", filename);
            eprintln!("Error: {}", err);
            exit(1);
        }
    };

    data.cli.execute();
    data.snr.update();
    data.unix.execute();
    data.win.execute();

}
