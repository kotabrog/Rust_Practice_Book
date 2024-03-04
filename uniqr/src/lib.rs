use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

type MyResult<T> = Result<T, Box<dyn Error>>;

/// Rust uniq
#[derive(Parser, Debug)]
pub struct Config {
    /// Input file
    #[arg(name = "IN_FILE", default_value = "-")]
    in_file: String,

    /// Output file
    #[arg(name = "OUT_FILE")]
    out_file: Option<String>,

    /// Show counts
    #[arg(short, long)]
    count: bool,
}

pub fn get_args() -> MyResult<Config> {
    Ok(Config::parse())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

pub fn run(config: Config) -> MyResult<()> {
    let mut file = open(&config.in_file)
        .map_err(|err| format!("{}: {}", config.in_file, err))?;
    let mut line = String::new();
    let mut prev_line = String::new();
    let mut count = 0;
    let mut out_file: Box<dyn Write> = match &config.out_file {
        Some(filename) => Box::new(File::create(filename)?),
        None => Box::new(io::stdout()),
    };

    let mut print = |count: usize, line: &str| -> MyResult<()> {
        if count > 0 {
            if config.count {
                write!(out_file, "{:>4} {}", count, line)?;
            } else {
                write!(out_file, "{}", line)?;
            }
        };
        Ok(())
    };

    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        if line.trim_end() != prev_line.trim_end() {
            print(count, &prev_line)?;
            prev_line = line.clone();
            count = 0;
        }
        count += 1;
        line.clear();
    }
    print(count, &prev_line)?;
    Ok(())
}
