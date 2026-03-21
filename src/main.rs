mod des;
mod reconstruct;

use anyhow::{Context, Result};
use clap::ArgGroup;
use clap::{Parser, Subcommand, ValueHint};
use des::SourceMapModel;
use reconstruct::ProjectReconstructor;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(
    name = "sunmap",
    version,
    about = "Reconstruct JavaScript projects from sourcemaps.\nAuthor: Angelo DeLuca",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Reconstruct sources from a sourcemap file
    Rebuild(RebuildArgs),
    // Inspect a sourcemap
    // Inspect(InspectArgs),
}

#[derive(Debug, Parser)]
#[command(group(
    ArgGroup::new("input")
        .required(true)
        .args(["sourcemap", "sourcemap_dir"])
))]
pub struct RebuildArgs {
    /// Path to a single source map file
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub sourcemap: Option<PathBuf>,

    /// Directory to search for source map files (e.g. *.map)
    #[arg(long, value_hint = ValueHint::DirPath)]
    pub sourcemap_dir: Option<PathBuf>,

    #[arg(long)]
    pub recursive: bool,

    /// Output directory for reconstructed sources
    #[arg(short, long, value_hint = ValueHint::DirPath)]
    pub out_dir: PathBuf,

    /// Overwrite existing files
    #[arg(long, default_value_t = false)]
    pub overwrite: bool,
}

#[derive(Debug, Parser)]
pub struct InspectArgs {
    /// Path to the source map file
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub sourcemap: PathBuf,

    /// Print parsed sourcemap as JSON
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Rebuild(args) => {
            println!("rebuild from {:?}", args.sourcemap);
            println!("output to {:?}", args.out_dir);
            println!("overwrite: {}", args.overwrite);

            handle_rebuild(args)
        } // Commands::Inspect(args) => {
          //     println!("inspect {:?}", args.sourcemap);
          //     println!("json: {}", args.json);
          //     Ok(())
          // }
    }
}

fn handle_rebuild(args: RebuildArgs) -> Result<()> {
    let mut sourcemap_paths = Vec::new();

    if let Some(file) = args.sourcemap {
        sourcemap_paths.push(file);
    }

    if let Some(dir) = args.sourcemap_dir {
        collect_sourcemaps_in_dir(&dir, &mut sourcemap_paths, args.recursive)?;
    }

    sourcemap_paths.sort();
    sourcemap_paths.dedup();

    if sourcemap_paths.is_empty() {
        anyhow::bail!("No sourcemap files found.");
    }

    let mut source_maps = Vec::with_capacity(sourcemap_paths.len());

    for path in &sourcemap_paths {
        let text = fs::read_to_string(path)
            .with_context(|| format!("Failed to read sourcemap file: {}", path.display()))?;
        let model: SourceMapModel = serde_json::from_str(&text)
            .with_context(|| format!("Invalid sourcemap JSON in: {}", path.display()))?;
        source_maps.push(model);
    }

    let refs: Vec<&SourceMapModel> = source_maps.iter().collect();
    let reconstructor = ProjectReconstructor::new(&refs);
    reconstructor.extract_to(&args.out_dir);
    let (fcount, data) = reconstructor.dump_size();

    println!(
        "Processed {} sourcemap file(s) into {}. Dumped {fcount} file(s) and {data} bytes.",
        sourcemap_paths.len(),
        args.out_dir.display()
    );

    Ok(())
}

fn collect_sourcemaps_in_dir(dir: &Path, out: &mut Vec<PathBuf>, recursive: bool) -> Result<()> {
    for entry in
        fs::read_dir(dir).with_context(|| format!("Failed to read directory: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() && recursive {
            collect_sourcemaps_in_dir(&path, out, true)?;
            continue;
        }

        if path.display().to_string().ends_with(".js.map") {
            out.push(path);
        }
    }

    Ok(())
}
