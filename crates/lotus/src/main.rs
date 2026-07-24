use clap::{
    Parser,
    Subcommand,
};
use diagnostics::{
    Diagnostic,
    files::FilesCache,
    render::render,
};
use driver::Compiler;
use elaborator::core::debug::debug_file as debug_elaborated_file;
use nir::debug::debug_file as debug_nir_file;
use std::path::PathBuf;
use structure::Program;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "lotus", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Parse { source: PathBuf },
    Elaborate { source: PathBuf },
    Lower { source: PathBuf },
}

impl Cli {
    fn file(&self) -> &PathBuf {
        match &self.command {
            Commands::Parse { source }
            | Commands::Elaborate { source }
            | Commands::Lower { source } => source,
        }
    }
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let program = Program::from_path(cli.file().clone(), None)?;

    let mut compiler = Compiler::new();
    let root = compiler.ingest_program(program)?;
    info!("Compilation unit: {}", root.name(compiler.db()));

    let files = compiler.files();
    let mut cache = FilesCache::new(compiler.db());

    for file in files {
        let path = file.path(compiler.db()).clone();
        info!("Processing file: {}", path.display());
        match cli.command {
            Commands::Parse { .. } => {
                let parse = compiler.parse(file);
                let root = parse.syntax_node();
                println!("Syntax tree: {root:#?}");
                let diags = compiler.parsing_diagnostics(file);
                print_diagnostics(&mut cache, &diags);
                return Ok(());
            }
            Commands::Elaborate { .. } => {
                let elaborated = compiler.elaborate(file);
                let debug = debug_elaborated_file(compiler.db(), &elaborated);
                println!("Elaborated file: {debug}");
            }
            Commands::Lower { .. } => {
                let lowered = compiler.lower(file);
                let debug = debug_nir_file(compiler.db(), lowered);
                println!("Lowered file: {debug}");
            }
        }
        let diagnostics = compiler.diagnostics(file);
        print_diagnostics(&mut cache, &diagnostics);
    }
    Ok(())
}

fn print_diagnostics(cache: &mut FilesCache, diagnostics: &Vec<Diagnostic>) {
    if diagnostics.is_empty() {
        println!("No diagnostics found!");
    } else {
        println!("Found {} diagnostics:", diagnostics.len());
        for diagnostic in diagnostics {
            render(cache, diagnostic);
        }
    }
}
