use crate::EntryType::*;
use clap::{Parser, ValueEnum};
use regex::Regex;
use walkdir::{WalkDir, DirEntry};
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Eq, PartialEq, Clone)]
enum EntryType {
    Dir,
    File,
    Link,
}

impl std::str::FromStr for EntryType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "d" => Ok(EntryType::Dir),
            "f" => Ok(EntryType::File),
            "l" => Ok(EntryType::Link),
            _ => Err("invalid entry type"),
        }
    }
}

impl ValueEnum for EntryType {
    fn value_variants<'a>() -> &'a [Self] {
        &[Dir, File, Link]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        let value = match self {
            EntryType::Dir => "d",
            EntryType::File => "f",
            EntryType::Link => "l",
        };
        Some(clap::builder::PossibleValue::new(value))
    }
}

#[derive(Parser, Debug)]
pub struct Config {
    #[arg(name = "PATH", default_value = ".")]
    path: Vec<String>,
    #[arg(short, long = "name", num_args(0..))]
    names: Vec<Regex>,
    #[arg(short = 't', long = "type", num_args(0..))]
    entry_types: Vec<EntryType>,
}

pub fn get_args() -> MyResult<Config> {
    Ok(Config::parse())
}

pub fn run(config: Config) -> MyResult<()> {
    let type_filter = |entry: &DirEntry| {
        config.entry_types.is_empty() || config.entry_types.iter().any(
            |entry_type| {
                match entry_type {
                    Dir => entry.file_type().is_dir(),
                    File => entry.file_type().is_file(),
                    Link => entry.file_type().is_symlink(),
                }
            }
        )
    };

    let name_filter = |entry: &DirEntry| {
        config.names.is_empty() || config.names.iter().any(
            |re| re.is_match(
                &entry.file_name().to_string_lossy(),
            )
        )
    };

    for path in config.path {
        let entries = WalkDir::new(path)
            .into_iter()
            .filter_map(|e| match e {
                Ok(entry) => Some(entry),
                Err(e) => {
                    eprintln!("{}", e);
                    None
                },
            })
            .filter(type_filter)
            .filter(name_filter)
            .map(|entry| entry.path().display().to_string())
            .collect::<Vec<_>>();
        println!("{}", entries.join("\n"));
    }
    Ok(())
}
