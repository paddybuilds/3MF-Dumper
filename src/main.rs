use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use rayon::prelude::*;
use roxmltree::Document;
use serde::Serialize;
use zip::ZipArchive;

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

#[derive(Debug, Serialize, Default)]
struct ModelSummary {
    unit: Option<String>,
    object_count: usize,
    mesh_object_count: usize,
    components_object_count: usize,
    build_item_count: usize,
    metadata_count: usize,
}

#[derive(Debug)]
struct DecompileReport {
    input: PathBuf,
    output: PathBuf,
    file_count: usize,
    summary: ModelSummary,
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
        } => run_decompile(inputs, out_dir, overwrite, pretty_xml, jobs),
        Commands::Inspect { input } => run_inspect(&input),
    }
}

fn run_decompile(
    inputs: Vec<PathBuf>,
    out_dir: PathBuf,
    overwrite: bool,
    pretty_xml: bool,
    jobs: Option<usize>,
) -> Result<()> {
    if inputs.is_empty() {
        bail!("at least one input file is required");
    }
    if let Some(job_count) = jobs {
        if job_count == 0 {
            bail!("--jobs must be greater than 0");
        }
    }

    let multi_input = inputs.len() > 1;
    if multi_input {
        fs::create_dir_all(&out_dir)
            .with_context(|| format!("failed to create output directory {}", out_dir.display()))?;
    }

    let worker = || {
        inputs
            .par_iter()
            .map(|input| {
                let output = if multi_input {
                    out_dir.join(output_folder_name(input))
                } else {
                    out_dir.clone()
                };
                decompile_file(input, &output, overwrite, pretty_xml)
            })
            .collect::<Vec<Result<DecompileReport>>>()
    };

    let results = if let Some(job_count) = jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(job_count)
            .build()
            .context("failed to create rayon thread pool")?
            .install(worker)
    } else {
        worker()
    };

    let mut failures = 0usize;
    for result in results {
        match result {
            Ok(report) => {
                println!(
                    "[ok] {} -> {} ({} files)",
                    report.input.display(),
                    report.output.display(),
                    report.file_count
                );
                if report.summary.object_count > 0 {
                    println!(
                        "     model: objects={}, mesh_objects={}, component_objects={}, build_items={}, unit={}",
                        report.summary.object_count,
                        report.summary.mesh_object_count,
                        report.summary.components_object_count,
                        report.summary.build_item_count,
                        report.summary.unit.as_deref().unwrap_or("unknown")
                    );
                }
            }
            Err(err) => {
                failures += 1;
                eprintln!("[error] {err:#}");
            }
        }
    }

    if failures > 0 {
        bail!("{failures} file(s) failed during decompilation");
    }

    Ok(())
}

fn run_inspect(input: &Path) -> Result<()> {
    let file =
        File::open(input).with_context(|| format!("failed to open input {}", input.display()))?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader)
        .with_context(|| format!("{} is not a valid zip/3mf archive", input.display()))?;

    let mut compressed_bytes = 0u64;
    let mut uncompressed_bytes = 0u64;
    let mut model_xml: Option<String> = None;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .with_context(|| format!("failed reading archive entry at index {index}"))?;
        compressed_bytes = compressed_bytes.saturating_add(entry.compressed_size());
        uncompressed_bytes = uncompressed_bytes.saturating_add(entry.size());

        if model_xml.is_none() && is_model_file(entry.name()) {
            let mut bytes = Vec::new();
            entry
                .read_to_end(&mut bytes)
                .with_context(|| format!("failed to read model entry {}", entry.name()))?;
            if let Ok(text) = String::from_utf8(bytes) {
                model_xml = Some(text);
            }
        }
    }

    println!("input: {}", input.display());
    println!("entries: {}", archive.len());
    println!("compressed bytes: {}", compressed_bytes);
    println!("uncompressed bytes: {}", uncompressed_bytes);

    if let Some(xml) = model_xml {
        let summary = parse_model_summary(&xml)?;
        println!(
            "model summary: objects={}, mesh_objects={}, component_objects={}, build_items={}, metadata={}, unit={}",
            summary.object_count,
            summary.mesh_object_count,
            summary.components_object_count,
            summary.build_item_count,
            summary.metadata_count,
            summary.unit.as_deref().unwrap_or("unknown")
        );
    } else {
        println!("model summary: no *.model file found");
    }

    Ok(())
}

fn decompile_file(input: &Path, output: &Path, overwrite: bool, pretty_xml: bool) -> Result<DecompileReport> {
    let file =
        File::open(input).with_context(|| format!("failed to open input {}", input.display()))?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader)
        .with_context(|| format!("{} is not a valid zip/3mf archive", input.display()))?;

    if output.exists() {
        if overwrite {
            fs::remove_dir_all(output).with_context(|| {
                format!("failed removing existing output directory {}", output.display())
            })?;
        } else {
            bail!(
                "output directory {} already exists; use --overwrite",
                output.display()
            );
        }
    }
    fs::create_dir_all(output)
        .with_context(|| format!("failed to create output directory {}", output.display()))?;

    let mut file_count = 0usize;
    let mut model_xml: Option<String> = None;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .with_context(|| format!("failed reading archive entry at index {index}"))?;

        let output_path = safe_output_path(output, entry.name())?;
        if entry.is_dir() {
            fs::create_dir_all(&output_path).with_context(|| {
                format!("failed to create directory {}", output_path.display())
            })?;
            continue;
        }

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {}", parent.display()))?;
        }

        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .with_context(|| format!("failed to read entry {}", entry.name()))?;

        let processed_bytes = if pretty_xml && is_xml_file(entry.name()) {
            format_xml(&bytes).unwrap_or(bytes)
        } else {
            bytes
        };

        if model_xml.is_none()
            && is_model_file(entry.name())
            && let Ok(text) = String::from_utf8(processed_bytes.clone())
        {
            model_xml = Some(text);
        }

        fs::write(&output_path, processed_bytes)
            .with_context(|| format!("failed to write {}", output_path.display()))?;
        file_count += 1;
    }

    let summary = match model_xml {
        Some(xml) => parse_model_summary(&xml).unwrap_or_default(),
        None => ModelSummary::default(),
    };

    let summary_path = output.join("_summary.json");
    let summary_json =
        serde_json::to_vec_pretty(&summary).context("failed to serialize summary JSON")?;
    fs::write(&summary_path, summary_json)
        .with_context(|| format!("failed to write {}", summary_path.display()))?;

    Ok(DecompileReport {
        input: input.to_path_buf(),
        output: output.to_path_buf(),
        file_count,
        summary,
    })
}

fn output_folder_name(input: &Path) -> String {
    input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.replace(' ', "_"))
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| "output".to_string())
}

fn safe_output_path(base: &Path, entry_name: &str) -> Result<PathBuf> {
    let mut path = base.to_path_buf();
    for component in Path::new(entry_name).components() {
        match component {
            Component::Normal(name) => path.push(name),
            Component::CurDir => {}
            _ => bail!("archive entry has unsafe path: {entry_name}"),
        }
    }
    Ok(path)
}

fn is_xml_file(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".xml") || lower.ends_with(".model") || lower.ends_with(".rels")
}

fn is_model_file(path: &str) -> bool {
    path.to_ascii_lowercase().ends_with(".model")
}

fn format_xml(bytes: &[u8]) -> Result<Vec<u8>> {
    let element = xmltree::Element::parse(bytes).context("failed to parse XML for formatting")?;
    let mut out = Vec::new();
    let config = xmltree::EmitterConfig::new()
        .perform_indent(true)
        .write_document_declaration(true);
    element
        .write_with_config(&mut out, config)
        .context("failed to format XML")?;
    Ok(out)
}

fn parse_model_summary(xml: &str) -> Result<ModelSummary> {
    let doc = Document::parse(xml).context("failed to parse model XML")?;
    let model_node = doc
        .descendants()
        .find(|node| node.has_tag_name("model"))
        .context("missing <model> node")?;

    let object_nodes: Vec<_> = model_node
        .descendants()
        .filter(|node| node.has_tag_name("object"))
        .collect();

    let mesh_object_count = object_nodes
        .iter()
        .filter(|node| node.children().any(|child| child.has_tag_name("mesh")))
        .count();
    let components_object_count = object_nodes
        .iter()
        .filter(|node| node.children().any(|child| child.has_tag_name("components")))
        .count();

    let build_item_count = model_node
        .descendants()
        .filter(|node| node.has_tag_name("item"))
        .count();
    let metadata_count = model_node
        .descendants()
        .filter(|node| node.has_tag_name("metadata"))
        .count();

    Ok(ModelSummary {
        unit: model_node.attribute("unit").map(ToString::to_string),
        object_count: object_nodes.len(),
        mesh_object_count,
        components_object_count,
        build_item_count,
        metadata_count,
    })
}
