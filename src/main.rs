use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use three_mf_dumper::{DecompileOptions, run_decompile, run_inspect};

#[derive(Parser, Debug)]
#[command(name = "3mf-dumper")]
#[command(about = "Decompile .3mf files into readable folder structures")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Decompile {
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
        #[arg(short, long, default_value = "decompiled")]
        out_dir: PathBuf,
        #[arg(long)]
        overwrite: bool,
        #[arg(long)]
        pretty_xml: bool,
        #[arg(long)]
        jobs: Option<usize>,
    },
    Inspect {
        input: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Decompile {
            inputs,
            out_dir,
            overwrite,
            pretty_xml,
            jobs,
        } => run_decompile(DecompileOptions {
            inputs,
            out_dir,
            overwrite,
            pretty_xml,
            jobs,
        }),
        Commands::Inspect { input } => run_inspect(&input),
    }
}
