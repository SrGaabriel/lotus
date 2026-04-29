# Contribution Guidelines

Feel free to send PRs, open issues, send emails or reach me out in any form. All contributions are welcome!

## Where to start

These are the most relevant crates that you are probably interested in:

* [ast](./crates/ast) converts the `CST` into the `AST` incrementally
* [db](./crates/db) contains the salsa database, useful for incremental compilation and caching
* [diagnostics](./crates/diagnostics) contains the diagnostics system
* [structure](./crates/structure) models the structure of the code, such as projects, modules, packages, etc
* [syntax](./crates/syntax) contains the lexer and parser, which produces the `CST`
* [driver](./crates/driver) orchestrates the compilation process
* [lotus](./crates/lotus) entrypoint for the command line interface