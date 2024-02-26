use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

type MyResult<T> = Result<T, Box<dyn Error>>;

/// Rust head
#[derive(Parser, Debug)]
pub struct Config {
    /// Input files
    #[arg(name = "FILE", default_value = "-")]
    files: Vec<String>,

    /// Number of lines to display
    #[arg(short = 'n', long, default_value = "10")]
    lines: usize,

    /// Number the output bytes, default is None
    #[arg(short = 'c', long, conflicts_with = "lines")]
    bytes: Option<usize>,
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
    let num_files = config.files.len();

    for (file_num, filename) in config.files.iter().enumerate() {
        match open(&filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(mut file) => {
                if num_files > 1 {
                    println!(
                        "{}==> {} <==",
                        if file_num > 0 { "\n" } else { "" },
                        filename
                    );
                }
                if let Some(num_bytes) = config.bytes {
                    let mut hundle = file.take(num_bytes as u64);
                    let mut buffer = vec![0; num_bytes];
                    let bytes_read = hundle.read(&mut buffer)?;
                    print!(
                        "{}",
                        String::from_utf8_lossy(&buffer[..bytes_read])
                    )
                } else {
                    let mut line = String::new();
                    for _ in 0..config.lines {
                        let bytes = file.read_line(&mut line)?;
                        if bytes == 0 {
                            break;
                        }
                        print!("{}", line);
                        line.clear();
                    }
                }
            },
        }
    }
    Ok(())
}
