set windows-shell := ["cmd.exe", "/c"]

ready:
    cargo clippy --all-targets --all-features -- -D warnings
    cargo fmt --all -- --check
    cargo xtask codegen --check
