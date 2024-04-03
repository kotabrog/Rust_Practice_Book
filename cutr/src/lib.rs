use Extract::*;
use clap::Parser;
use regex::Regex;
use csv::{ReaderBuilder, StringRecord, WriterBuilder};
use std::{
    error::Error,
    ops::Range,
    num::NonZeroUsize,
    io::{self, BufRead, BufReader},
    fs::File
};

type MyResult<T> = Result<T, Box<dyn Error>>;
type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

#[derive(Parser, Debug)]
pub struct ConfigArgs {
    #[arg(name = "FILE", default_value = "-", help = "Input file(s)")]
    files: Vec<String>,
    #[arg(short, long = "delim", default_value = "\t", help = "Field delimiter")]
    delimiter: String,
    #[arg(short, long, help = "Select fields", conflicts_with_all(&["bytes", "chars"]))]
    fields: Option<String>,
    #[arg(short, long, help = "Select bytes", conflicts_with_all(&["fields", "chars"]))]
    bytes: Option<String>,
    #[arg(short, long, help = "Select characters", conflicts_with_all(&["fields", "bytes"]))]
    chars: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    delimiter: u8,
    extract: Extract,
}

fn parse_index(input: &str) -> Result<usize, String> {
    let value_error = || format!("illegal list value: \"{}\"", input);
    input
        .starts_with('+')
        .then(|| Err(value_error()))
        .unwrap_or_else(|| {
            input
                .parse::<NonZeroUsize>()
                .map(|n| usize::from(n) - 1)
                .map_err(|_| value_error())
        })
}

fn parse_pos(range: &str) -> MyResult<PositionList> {
    let range_re = Regex::new(r"^(\d+)-(\d+)$").unwrap();
    range
        .split(',')
        .into_iter()
        .map(|val| {
            parse_index(val).map(|n| n..n + 1).or_else(|e| {
                range_re.captures(val).ok_or(e).and_then(|captures| {
                    let n1 = parse_index(&captures[1])?;
                    let n2 = parse_index(&captures[2])?;
                    if n1 >= n2 {
                        return Err(format!(
                            "First number in range ({}) \
                            must be lower than second number ({})",
                            n1 + 1,
                            n2 + 1
                        ));
                    }
                    Ok(n1..n2 + 1)
                })
            })
        })
        .collect::<Result<_, _>>()
        .map_err(From::from)
}

pub fn get_args() -> MyResult<Config> {
    let args = ConfigArgs::parse();
    let delimiter = args.delimiter.as_bytes();
    if delimiter.len() != 1 {
        return Err(From::from(format!(
            "--delim \"{}\" must be a single byte",
            args.delimiter
        )));
    }
    let fields = args.fields.as_deref().map(parse_pos).transpose()?;
    let bytes = args.bytes.as_deref().map(parse_pos).transpose()?;
    let chars = args.chars.as_deref().map(parse_pos).transpose()?;
    let extract = if let Some(fields) = fields {
        Fields(fields)
    } else if let Some(bytes) = bytes {
        Bytes(bytes)
    } else if let Some(chars) = chars {
        Chars(chars)
    } else {
        return Err(From::from("Must have --fields, --bytes, or --chars"));
    };
    Ok(Config {
        files: args.files,
        delimiter: delimiter[0],
        extract,
    })
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    let chars: Vec<_> = line.chars().collect();
    char_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| chars.get(i)))
        .collect()
}

fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    let bytes = line.as_bytes();
    let selected: Vec<_> = byte_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| bytes.get(i)).copied())
        .collect();
    String::from_utf8_lossy(&selected).to_string()
}

fn extract_fields(
    record: &StringRecord,
    field_pos: &[Range<usize>],
) -> Vec<String> {
    field_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| record.get(i)))
        .map(String::from)
        .collect()
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in &config.files {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => match &config.extract {
                Fields(field_pos) => {
                    let mut reader = ReaderBuilder::new()
                        .delimiter(config.delimiter)
                        .has_headers(false)
                        .from_reader(file);
                    let mut wtr = WriterBuilder::new()
                        .delimiter(config.delimiter)
                        .from_writer(io::stdout());
                    for result in reader.records() {
                        let record = result?;
                        let fields = extract_fields(&record, field_pos);
                        wtr.write_record(&fields)?;
                    }
                    wtr.flush()?;
                }
                Bytes(byte_pos) => {
                    for line in file.lines() {
                        let line = line?;
                        let bytes = extract_bytes(&line, byte_pos);
                        println!("{}", bytes);
                    }
                }
                Chars(char_pos) => {
                    for line in file.lines() {
                        let line = line?;
                        let chars = extract_chars(&line, char_pos);
                        println!("{}", chars);
                    }
                }
            }
        }
    }
    Ok(())
}
