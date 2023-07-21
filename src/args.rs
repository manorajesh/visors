use clap::Parser;
use clap::ValueHint;

#[derive(Parser, Debug, Clone)]
#[command(
    author,
    version,
    about = "Visualize Rust's AST",
    long_about = "Generate a graphviz DOT file from Rust source code's AST"
)]
pub struct Args {
    /// Path to file
    #[clap(value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub file: String,
}
