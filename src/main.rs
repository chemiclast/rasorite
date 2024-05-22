mod benches;
mod data;
mod parse;
mod plot;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, name, long_about = None)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {}

fn main() {
    println!("Hello, world!");
}
