use clap::Parser;
use diagnostics::{
    files::FilesCache,
    render::render,
};
use driver::Compiler;
use std::path::PathBuf;
use structure::Program;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "lotus", version, about, long_about = None)]
struct Cli {
    source: PathBuf,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let program = Program::from_path(cli.source, None)?;

    let mut compiler = Compiler::new();
    let root = compiler.ingest_program(program)?;
    info!("Compilation unit: {}", root.name(compiler.db()));

    let files = compiler.files();
    let mut cache = FilesCache::new(compiler.db());

    for file in files {
        let path = file.path(compiler.db()).clone();
        info!("Processing file: {}", path.display());
        let parse = compiler.parse(file);
        let diagnostics = compiler.diagnostics(file);
        if !diagnostics.is_empty() {
            println!("Found {} diagnostics:", diagnostics.len());
            for diagnostic in &diagnostics {
                render(&mut cache, diagnostic);
            }
        }
        let text: String = parse.syntax_node().text().to_string();
        println!("Parsed syntax node text:\n\n{text}");
    }
    Ok(())
}
