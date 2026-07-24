pub mod ast;

use anyhow::{
    Context,
    Result,
    bail,
};
use std::path::PathBuf;

pub fn run(check: bool) -> Result<()> {
    let workspace = workspace_root()?;
    let grammar_path = workspace.join("crates/ast/lotus.ungram");
    let target_path = workspace.join("crates/ast/src/generated.rs");

    let grammar_text = std::fs::read_to_string(&grammar_path)
        .with_context(|| format!("reading {}", grammar_path.display()))?
        .replace("\r\n", "\n");
    let grammar: ungrammar::Grammar = grammar_text.parse()?;

    let ast_src = ast::lower(&grammar)?;
    let generated = ast::emit::emit(&ast_src)?;

    if check {
        let on_disk = std::fs::read_to_string(&target_path).unwrap_or_default();
        if on_disk != generated {
            bail!(
                "{} is out of date. Run `cargo xtask codegen`.",
                target_path.display()
            );
        }
    } else {
        std::fs::write(&target_path, generated)?;
    }
    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    let dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    Ok(dir.parent().context("xtask has no parent")?.to_path_buf())
}
