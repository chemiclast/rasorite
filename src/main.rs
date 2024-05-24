use crate::parse::parse_analytics_file;
use crate::plot::plot_data;
use clap::Parser;
use clap_verbosity_flag::WarnLevel;
use log::error;
use std::path::PathBuf;
use std::process::ExitCode;

mod data;
mod parse;
mod plot;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    /// Plots the analytics series normalized against the benchmark series instead of plotting both the benchmark series and the analytics series
    normalize: bool,

    #[arg(short, long)]
    /// The CSV file exported from Roblox Analytics
    in_file: PathBuf,

    /// The file to export the graph to. Must be an image file type, can be either bitmap or vector
    out_file: PathBuf,

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity<WarnLevel>,

    #[arg(short, long)]
    /// Does not try to open the output file after it is created
    silent: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    pretty_env_logger::formatted_builder()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    let analytics = parse_analytics_file(&cli.in_file);

    if let Err(e) = analytics {
        error!("{}", e);
        return ExitCode::FAILURE;
    }

    if let Err(e) = plot_data(analytics.unwrap(), &cli) {
        error!("{}", e);
        return ExitCode::FAILURE;
    };

    if !cli.silent {
        if let Err(e) = opener::open(cli.out_file) {
            error!("{}", e);
            return ExitCode::FAILURE;
        };
    }

    ExitCode::SUCCESS
}
