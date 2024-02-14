use clap::Parser;

/// Rust echo
#[derive(Parser, Debug)]
struct Args {
    /// Input text
    #[arg(name = "TEXT", required = true)]
    text: Vec<String>,

    /// Do not print newline
    #[arg(short = 'n', long)]
    omit_newline: bool,
}

fn main() {
    let args = Args::parse();
    let text = args.text.join(" ");
    print!("{}{}", text, if args.omit_newline { "" } else { "\n" });
}
