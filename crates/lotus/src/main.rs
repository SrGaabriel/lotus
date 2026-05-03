use clap::{
    Parser,
    Subcommand,
};
use diagnostics::{
    files::FilesCache,
    render::render,
};
use driver::Compiler;
use elaborator::core::debug::debug_file;
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
}

impl Cli {
    fn file(&self) -> &PathBuf {
        match &self.command {
            Commands::Parse { source } | Commands::Elaborate { source } => source,
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
            }
            Commands::Elaborate { .. } => {
                let elaborated = compiler.dbg_elaborate(file);
                let debug = debug_file(compiler.db(), &elaborated);
                println!("Elaborated file: {debug}");
            }
        }

        let diagnostics = compiler.diagnostics(file);
        if diagnostics.is_empty() {
            println!("No diagnostics found!");
        } else {
            println!("Found {} diagnostics:", diagnostics.len());
            for diagnostic in &diagnostics {
                render(&mut cache, diagnostic);
            }
        }
    }
    Ok(())
}
