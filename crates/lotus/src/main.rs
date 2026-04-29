use ast::parse;
use clap::Parser;
use diagnostics::files::FileId;

#[derive(Parser, Debug)]
#[command(name = "lotus", version, about, long_about = None)]
struct Cli {
    source: String,
}

fn main() {
    let cli = Cli::parse();
    let contents = std::fs::read_to_string(&cli.source).expect("Failed to read source file");
    let parse = parse(FileId(0), &contents);
    for _error in parse.diagnostics() {}
    let node = parse.syntax_node();
    let text: String = node.text().to_string();
    println!("Parsed text: {text}");
}
