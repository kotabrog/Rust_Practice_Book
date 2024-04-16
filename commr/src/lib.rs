use clap::Parser;
use std::{
    error::Error,
    io::{self, BufRead, BufReader},
    fs::File,
    cmp::Ordering::*,
};
use crate::Column::*;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
pub struct ConfigArgs {
    #[arg(name = "FILE1", help = "Input file 1")]
    file1: String,
    #[arg(name = "FILE2", help = "Input file 2")]
    file2: String,
    #[arg(short = '1', help = "Suppress printing of column 1")]
    suppress_col1: bool,
    #[arg(short = '2', help = "Suppress printing of column 2")]
    suppress_col2: bool,
    #[arg(short = '3', help = "Suppress printing of column 3")]
    suppress_col3: bool,
    #[arg(short = 'i', help = "Case-insensitive comparison of lines")]
    insensitive: bool,
    #[arg(short, long = "output-delimiter", default_value = "\t", help = "Output delimiter")]
    delimiter: String,
}

#[derive(Debug)]
pub struct Config {
    file1: String,
    file2: String,
    show_col1: bool,
    show_col2: bool,
    show_col3: bool,
    insensitive: bool,
    delimiter: String,
}

enum Column<'a> {
    Col1(&'a str),
    Col2(&'a str),
    Col3(&'a str),
}

pub fn get_args() -> MyResult<Config> {
    let args = ConfigArgs::parse();
    Ok(Config {
        file1: args.file1,
        file2: args.file2,
        show_col1: !args.suppress_col1,
        show_col2: !args.suppress_col2,
        show_col3: !args.suppress_col3,
        insensitive: args.insensitive,
        delimiter: args.delimiter,
    })
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)
            .map_err(|e| format!("{}: {}", filename, e))?
        ))),
    }
}

pub fn run(config: Config) -> MyResult<()> {
    let print = |col: Column| {
        let mut columns = vec![];
        match col {
            Col1(val) => {
                if config.show_col1 {
                    columns.push(val);
                }
            }
            Col2(val) => {
                if config.show_col2 {
                    if config.show_col1 {
                        columns.push("");
                    }
                    columns.push(val);
                }
            }
            Col3(val) => {
                if config.show_col3 {
                    if config.show_col1 {
                        columns.push("");
                    }
                    if config.show_col2 {
                        columns.push("");
                    }
                    columns.push(val);
                }
            }
        }
        if !columns.is_empty() {
            println!("{}", columns.join(&config.delimiter));
        }
    };

    let file1 = &config.file1;
    let file2 = &config.file2;

    if file1 == "-" && file2 == "-" {
        return Err("Both input files cannot be STDIN (\"-\")".into());
    }

    let case = |line: String| {
        if config.insensitive {
            line.to_lowercase()
        } else {
            line
        }
    };

    let mut lines1 = open(file1)?
        .lines()
        .filter_map(Result::ok)
        .map(case);
    let mut lines2 = open(file2)?
        .lines()
        .filter_map(Result::ok)
        .map(case);
    let mut line1 = lines1.next();
    let mut line2 = lines2.next();

    while line1.is_some() || line2.is_some() {
        match (&line1, &line2) {
            (Some(val1), Some(val2)) => match val1.cmp(val2) {
                Equal => {
                    print(Col3(val1));
                    line1 = lines1.next();
                    line2 = lines2.next();
                }
                Less => {
                    print(Col1(val1));
                    line1 = lines1.next();
                }
                Greater => {
                    print(Col2(val2));
                    line2 = lines2.next();
                }
            }
            (Some(val1), None) => {
                print(Col1(val1));
                line1 = lines1.next();
            }
            (None, Some(val2)) => {
                print(Col2(val2));
                line2 = lines2.next();
            }
            _ => {},
        }
    }

    Ok(())
}
