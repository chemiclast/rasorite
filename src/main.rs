use crate::parse::parse_analytics_file;
use crate::plot::plot_data;
use clap::{Parser, Subcommand};
use serde::de::Error;
use std::path::PathBuf;

mod benches;
mod data;
mod parse;
mod plot;

#[derive(Parser)]
#[command(version, about, name = "rasorite", long_about = None)]
struct Cli {}

#[derive(Subcommand)]
enum Commands {}

fn main() {
    let analytics = parse_analytics_file(PathBuf::from(
        r"C:\Users\Callie\RustroverProjects\rasorite\Wealdland Foods- DailyActiveUsers, , 1684627200000 to 1716163200000.csv",
    ))
    .unwrap();
    plot_data(analytics, true);
}
