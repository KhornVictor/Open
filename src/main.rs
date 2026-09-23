mod app;
mod cli;
mod config;
mod launcher;
mod ui;

use std::env;
use config::Config;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::load();
    let action = cli::parse_args(&args);
    cli::handle_cli(action, config);
}