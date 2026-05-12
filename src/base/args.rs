use clap::{Parser, crate_description};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = crate_description!())]
pub struct Args {
    /// Path to the configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: PathBuf,
}
