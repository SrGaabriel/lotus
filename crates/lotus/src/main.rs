use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "lotus", version, about, long_about = None)]
struct Cli {
    source: String,
}

fn main() {
    let _cli = Cli::parse();
}
