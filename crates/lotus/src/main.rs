use std::path::PathBuf;

use ast::parse;
use clap::Parser;
use diagnostics::{files::FilesCache, render::render};
use structure::Program;

#[derive(Parser, Debug)]
#[command(name = "lotus", version, about, long_about = None)]
struct Cli {
    source: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let program = Program::from_path(cli.source, None)?;
    let files = program.into_files();
    let mut cache = FilesCache::new(&files);

    for (file_id, file) in files.iter() {
        println!("Processing file: {}", file.path.display());
        let parse = parse(file_id, &file.text);

        let diagnostic_count = parse.diagnostics().len();
        if diagnostic_count > 0 {
            println!("Found {diagnostic_count} diagnostics:");
            for diagnostic in parse.diagnostics() {
                render(&mut cache, diagnostic);
            }
        } else {
            println!("No diagnostics found.");
        }
        let node = parse.syntax_node();
        let text: String = node.text().to_string();
        println!("Parsed syntax node text:\n\n{text}");
    }
    Ok(())
}
