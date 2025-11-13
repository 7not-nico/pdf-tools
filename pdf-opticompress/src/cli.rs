use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pdf-opticompress")]
#[command(about = "High-performance PDF optimizer")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Optimize a single PDF file
    Optimize {
        /// Input PDF file
        input: PathBuf,

        /// Output PDF file
        output: PathBuf,

        /// Image quality (0-100)
        #[arg(short, long, default_value = "80")]
        quality: u8,

        /// Optimization preset
        #[arg(short, long, value_enum, default_value = "web")]
        preset: Preset,
    },

    /// Analyze a PDF file and show optimization potential
    Analyze {
        /// Input PDF file
        input: PathBuf,

        /// Show potential savings
        #[arg(long)]
        show_savings: bool,
    },

    /// Batch process multiple PDF files
    Batch {
        /// Input PDF files
        files: Vec<PathBuf>,

        /// Output directory
        #[arg(short, long)]
        output_dir: Option<PathBuf>,

        /// Number of threads to use
        #[arg(short, long, default_value = "4")]
        threads: usize,
    },
}

#[derive(Clone, clap::ValueEnum)]
pub enum Preset {
    /// Web optimization (smaller file size, good quality)
    Web,
    /// Print optimization (high quality, moderate compression)
    Print,
    /// Archive optimization (maximum compression, lossless where possible)
    Archive,
    /// Maximum compression (aggressive optimization)
    Maximum,
}