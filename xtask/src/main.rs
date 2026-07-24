mod codegen;

use anyhow::Result;
use clap::{
    Parser,
    Subcommand,
};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Codegen {
        #[arg(long)]
        check: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Codegen { check } => codegen::run(check),
    }
}
