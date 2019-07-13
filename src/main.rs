#[macro_use]
extern crate derive_more;

use clap::{load_yaml, value_t, App};
use env_logger::Env;
use std::time::Duration;

mod error;
mod executor;
mod interval;
mod pipeline;
mod scheduler;
mod state;

fn main() {
    let cli_yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(cli_yaml).get_matches();

    let log_level = matches.value_of("log").unwrap();
    env_logger::from_env(Env::default().default_filter_or(log_level)).init();

    let refresh_interval = value_t!(matches, "refresh", u32).unwrap();
    let refresh_interval = Duration::from_secs(refresh_interval.into());

    let pipelines_path = matches.value_of("pipelines").unwrap();

    scheduler::run(pipelines_path, refresh_interval);
}
