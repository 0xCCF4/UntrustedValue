use crate::output::OutputFormatType;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub targets: Vec<String>,
    #[arg(long)]
    pub graph_dir: PathBuf,
    #[arg(long)]
    pub output_format: OutputFormatType,
    #[arg(long)]
    pub taint_sources: Vec<String>,
    #[arg(long)]
    pub list_taint_sources: bool,
}
