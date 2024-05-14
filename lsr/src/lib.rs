mod owner;

use chrono::{DateTime, Local};
use clap::Parser;
use std::{
    error::Error,
    path::PathBuf,
    fs,
    os::unix::fs::MetadataExt,
};
use tabular::{Row, Table};
use users::{get_group_by_gid, get_user_by_uid};
use owner::Owner;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
pub struct Config {
    #[arg(name = "PATH", help = "Files and/or directories", default_value = ".")]
    paths: Vec<String>,
    #[arg(short, long, help = "long listing")]
    long: bool,
    #[arg(short='a', long="all", help = "Show all files")]
    show_hidden: bool,
}

pub fn get_args() -> MyResult<Config> {
    Ok(Config::parse())
}

fn find_files(
    paths: &[String],
    show_hidden: bool,
) -> MyResult<Vec<PathBuf>> {
    let mut results = vec![];
    for name in paths {
        match fs::metadata(name) {
            Err(e) => eprintln!("{}: {}", name, e),
            Ok(meta) => {
                if meta.is_dir() {
                    for entry in fs::read_dir(name)? {
                        let entry = entry?;
                        let path = entry.path();
                        let is_hidden =
                            path.file_name().map_or(false, |file_name| {
                                file_name.to_string_lossy().starts_with('.')
                            });
                        if !is_hidden || show_hidden {
                            results.push(path);
                        }
                    }
                } else {
                    results.push(PathBuf::from(name));
                }
            }
            
        }
    }
    Ok(results)
}

pub fn mk_triple(mode: u32, owner: Owner) -> String {
    let [read, write, execute] = owner.masks();
    format!(
        "{}{}{}",
        if mode & read == 0 { "-" } else { "r" },
        if mode & write == 0 { "-" } else { "w" },
        if mode & execute == 0 { "-" } else { "x" },
    )
}

fn format_mode(mode: u32) -> String {
    format!(
        "{}{}{}",
        mk_triple(mode, Owner::User),
        mk_triple(mode, Owner::Group),
        mk_triple(mode, Owner::Other),
    )
}

fn format_output(paths: &[PathBuf]) -> MyResult<String> {
    let fmt = "{:<}{:<} {:>} {:<} {:<} {:>} {:<} {:<}";
    let mut table = Table::new(fmt);
    for path in paths {
        let metadata = path.metadata()?;

        let uid = metadata.uid();
        let user = get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| uid.to_string());

        let gid = metadata.gid();
        let group = get_group_by_gid(gid)
            .map(|g| g.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| gid.to_string());

        let file_type = if path.is_dir() {"d"} else {"-"};
        let perms = format_mode(metadata.mode());
        let modified: DateTime<Local> = DateTime::from(metadata.modified()?);

        table.add_row(
            Row::new()
                .with_cell(file_type)
                .with_cell(perms)
                .with_cell(metadata.nlink())
                .with_cell(user)
                .with_cell(group)
                .with_cell(metadata.size())
                .with_cell(modified.format("%b %d %y %H:%M"))
                .with_cell(path.display()),
        );
    }
    Ok(format!("{}", table))
}

pub fn run(config: Config) -> MyResult<()> {
    let paths = find_files(&config.paths, config.show_hidden)?;
    if config.long {
        println!("{}", format_output(&paths)?);
    } else {
        for path in paths {
            println!("{}", path.display());
        }
    }
    Ok(())
}
