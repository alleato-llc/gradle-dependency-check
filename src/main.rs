use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};

use gradle_dependency_check::analysis::{
    diff_calculator, duplicate_detector, risk_calculator, scope_validator, table_calculator,
    tree_loader,
};
use gradle_dependency_check::dependency::models::{
    ChangeKind, GradleConfiguration, ReportFormat,
};
use gradle_dependency_check::report::{
    conflict_report, diff_report, dot_export, duplicate_report, scope_validation_report,
    table_report, tree_export,
};
use gradle_dependency_check::runner::gradle_runner::GradleRunner;
use gradle_dependency_check::runner::process_runner::ProcessGradleRunner;

#[derive(Parser)]
#[command(name = "gradle-dependency-check")]
#[command(about = "Analyze Gradle dependency trees")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Output dependency tree in DOT format
    Graph(ProjectArgs),
    /// Report dependency conflicts
    Conflicts(FormatArgs),
    /// List dependencies in flat table format
    Table(TableArgs),
    /// Check for test libraries in production dependency scopes
    Validate(FormatArgs),
    /// Detect duplicate dependencies across or within modules
    Duplicates(FormatArgs),
    /// Compare two dependency trees (each can be a project directory or exported file)
    Diff(DiffArgs),
    /// Export dependency tree as JSON or Gradle text format
    Export(ExportArgs),
}

#[derive(clap::Args)]
struct ProjectArgs {
    /// Path to the Gradle project directory
    project_path: String,
    /// Gradle configuration to analyze
    #[arg(short, long, default_value = "compileClasspath")]
    configuration: String,
    /// Specific module (e.g. :app). Omit for all modules.
    #[arg(short, long)]
    module: Option<String>,
    /// List discovered modules and exit
    #[arg(long)]
    list_modules: bool,
}

#[derive(clap::Args)]
struct FormatArgs {
    /// Path to the Gradle project directory
    project_path: String,
    /// Gradle configuration to analyze
    #[arg(short, long, default_value = "compileClasspath")]
    configuration: String,
    /// Specific module (e.g. :app). Omit for all modules.
    #[arg(short, long)]
    module: Option<String>,
    /// List discovered modules and exit
    #[arg(long)]
    list_modules: bool,
    /// Output format
    #[arg(short, long, default_value = "text")]
    format: Format,
}

#[derive(clap::Args)]
struct TableArgs {
    /// Path to the Gradle project directory
    project_path: String,
    /// Gradle configuration to analyze
    #[arg(short, long, default_value = "compileClasspath")]
    configuration: String,
    /// Specific module (e.g. :app). Omit for all modules.
    #[arg(short, long)]
    module: Option<String>,
    /// List discovered modules and exit
    #[arg(long)]
    list_modules: bool,
    /// Output format
    #[arg(short, long, default_value = "text")]
    format: Format,
    /// Show only dependencies with version conflicts
    #[arg(long)]
    conflicts_only: bool,
}

#[derive(clap::Args)]
struct DiffArgs {
    /// Baseline: path to a Gradle project directory or exported tree file (JSON/text)
    baseline: String,
    /// Current: path to a Gradle project directory or exported tree file (JSON/text)
    current: String,
    /// Gradle configuration (applies to project directory inputs)
    #[arg(short, long, default_value = "compileClasspath")]
    configuration: String,
    /// Specific module (applies to project directory inputs)
    #[arg(short, long)]
    module: Option<String>,
    /// Output format
    #[arg(short, long, default_value = "text")]
    format: Format,
    /// Filter changes: comma-separated list of added,removed,changed,unchanged
    #[arg(long)]
    changes: Option<String>,
}

#[derive(clap::Args)]
struct ExportArgs {
    /// Path to the Gradle project directory
    project_path: String,
    /// Gradle configuration to analyze
    #[arg(short, long, default_value = "compileClasspath")]
    configuration: String,
    /// Specific module (e.g. :app). Omit for all modules.
    #[arg(short, long)]
    module: Option<String>,
    /// List discovered modules and exit
    #[arg(long)]
    list_modules: bool,
    /// Export format
    #[arg(short, long, default_value = "json")]
    format: Format,
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
}

impl Format {
    fn to_report_format(self) -> ReportFormat {
        match self {
            Format::Text => ReportFormat::Text,
            Format::Json => ReportFormat::Json,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let runner = ProcessGradleRunner;

    match cli.command {
        Commands::Graph(args) => {
            if args.list_modules {
                return print_modules(&runner, &args.project_path);
            }
            let tree = load(&runner, &args.project_path, &args.configuration, args.module.as_deref())?;
            println!("{}", dot_export::export(&tree));
        }
        Commands::Conflicts(args) => {
            if args.list_modules {
                return print_modules(&runner, &args.project_path);
            }
            let mut tree = load(&runner, &args.project_path, &args.configuration, args.module.as_deref())?;
            tree.conflicts = risk_calculator::assess_conflicts(&tree, &runner, &args.project_path);
            println!("{}", conflict_report::report(&tree, args.format.to_report_format()));
        }
        Commands::Table(args) => {
            if args.list_modules {
                return print_modules(&runner, &args.project_path);
            }
            let tree = load(&runner, &args.project_path, &args.configuration, args.module.as_deref())?;
            let mut entries = table_calculator::flat_entries(&tree);
            if args.conflicts_only {
                entries.retain(|e| e.has_conflict);
            }
            println!("{}", table_report::report(&entries, &tree, args.format.to_report_format()));
        }
        Commands::Validate(args) => {
            if args.list_modules {
                return print_modules(&runner, &args.project_path);
            }
            let tree = load(&runner, &args.project_path, &args.configuration, args.module.as_deref())?;
            let results = scope_validator::validate(&tree);
            println!("{}", scope_validation_report::report(&results, &tree, args.format.to_report_format()));
        }
        Commands::Duplicates(args) => {
            if args.list_modules {
                return print_modules(&runner, &args.project_path);
            }
            let tree = load(&runner, &args.project_path, &args.configuration, args.module.as_deref())?;
            let modules = runner.list_projects(&args.project_path).unwrap_or_default();
            let results = duplicate_detector::detect(&tree, &args.project_path, &modules);
            println!("{}", duplicate_report::report(&results, &tree, args.format.to_report_format()));
        }
        Commands::Diff(args) => {
            let baseline_tree = resolve(&runner, &args.baseline, &args.configuration, args.module.as_deref())?;
            let current_tree = resolve(&runner, &args.current, &args.configuration, args.module.as_deref())?;

            let diff_result = diff_calculator::diff(&baseline_tree, &current_tree);
            let mut entries = diff_result.entries.clone();

            if let Some(changes) = &args.changes {
                let allowed: std::collections::HashSet<&str> = changes.split(',').map(|s| s.trim()).collect();
                entries.retain(|e| match e.change_kind {
                    ChangeKind::Added => allowed.contains("added"),
                    ChangeKind::Removed => allowed.contains("removed"),
                    ChangeKind::VersionChanged => allowed.contains("changed"),
                    ChangeKind::Unchanged => allowed.contains("unchanged"),
                });
            } else {
                entries.retain(|e| e.change_kind != ChangeKind::Unchanged);
            }

            println!("{}", diff_report::report(&entries, &diff_result, args.format.to_report_format()));
        }
        Commands::Export(args) => {
            if args.list_modules {
                return print_modules(&runner, &args.project_path);
            }
            let tree = load(&runner, &args.project_path, &args.configuration, args.module.as_deref())?;
            match args.format {
                Format::Json => {
                    let json = tree_export::export_json(&tree)
                        .context("Failed to serialize tree as JSON")?;
                    println!("{}", json);
                }
                Format::Text => {
                    println!("{}", tree_export::export_text(&tree));
                }
            }
        }
    }

    Ok(())
}

fn print_modules(runner: &dyn GradleRunner, project_path: &str) -> Result<()> {
    let modules = runner.list_projects(project_path)
        .context("Failed to list modules")?;
    if modules.is_empty() {
        println!("No submodules found (single-module project).");
    } else {
        for m in &modules {
            println!("{}", m.path);
        }
    }
    Ok(())
}

fn load(
    runner: &dyn GradleRunner,
    project_path: &str,
    configuration: &str,
    module: Option<&str>,
) -> Result<gradle_dependency_check::dependency::models::DependencyTree> {
    let config = GradleConfiguration::from_str(configuration)
        .with_context(|| format!("Unknown configuration: {}", configuration))?;
    tree_loader::load_tree(runner, project_path, config, module)
        .context("Failed to load dependency tree")
}

fn resolve(
    runner: &dyn GradleRunner,
    path: &str,
    configuration: &str,
    module: Option<&str>,
) -> Result<gradle_dependency_check::dependency::models::DependencyTree> {
    let p = Path::new(path);
    if !p.exists() {
        bail!("Path does not exist: {}", path);
    }

    if p.is_dir() {
        load(runner, path, configuration, module)
    } else {
        let data = fs::read(path).context("Failed to read file")?;
        let file_name = p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let config = GradleConfiguration::from_str(configuration)
            .with_context(|| format!("Unknown configuration: {}", configuration))?;
        tree_export::import_tree(&data, &file_name, config)
            .context("Failed to import tree from file")
            .map_err(Into::into)
    }
}
