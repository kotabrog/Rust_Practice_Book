use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use clap::Parser;

type MyResult<T> = Result<T, Box<dyn Error>>;

/// Rust cat
#[derive(Parser, Debug)]
pub struct Config {
    /// Input files
    #[arg(name = "FILE")]
    files: Vec<String>,

    /// Number the output lines, starting at 1
    #[arg(short, long = "number", conflicts_with = "number_non_blank_lines")]
    number_lines: bool,

    /// Number the non blank output lines, starting at 1
    #[arg(short = 'b', long = "number-nonblank")]
    number_non_blank_lines: bool,
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

pub fn run(mut config: Config) -> MyResult<()> {
    if config.files.len() == 0 {
        config.files.push("-".to_string());
    }
    let mut count = 0;
    for filename in config.files {
        match open(&filename) {
            Err(err) => eprintln!("Failed to open {}: {}", filename, err),
            Ok(file) => {
                for line in file.lines() {
                    let line = line?;
                    if config.number_lines {
                        println!("{:>6}\t{}", count + 1, line);
                        count += 1;
                    } else if config.number_non_blank_lines && !line.trim().is_empty() {
                        println!("{:>6}\t{}", count + 1, line);
                        count += 1;
                    } else {
                        println!("{}", line);
                    }
                }
            }
        }
    }
    Ok(())
}
