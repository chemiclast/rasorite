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
struct Cli {
    #[arg(short, long, default_value_t = false)]
    /// Plots the analytics series normalized against the benchmark series instead of plotting both the benchmark series and the analytics series
    normalize: bool,

    #[arg(short, long)]
    /// The file to export the graph to. Must be a bitmap image file type
    out_file: PathBuf,

    #[arg(short, long)]
    /// The CSV file exported from Roblox Analytics
    in_file: PathBuf,

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn main() {
    let cli = Cli::parse();

    pretty_env_logger::formatted_builder()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    let analytics = parse_analytics_file(&cli.in_file).unwrap();
    plot_data(analytics, &cli);
    opener::open(cli.out_file).expect("Failed to open output file!");
}
