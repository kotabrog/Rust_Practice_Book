use clap::Parser;
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, SeedableRng};
use std::{
    error::Error,
    path::PathBuf,
    ffi::OsStr,
    fs::{self, File},
    io::{BufRead, BufReader}
};
use regex::{Regex, RegexBuilder};
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;


#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

#[derive(Debug)]
pub struct Fortune {
    source: String,
    text: String,
}

#[derive(Debug, Parser)]
pub struct ConfigArgs {
    #[arg(name = "FILE", help = "Input files or directories")]
    sources: Vec<String>,
    #[arg(short='m', long, help = "Pattern")]
    pattern: Option<String>,
    #[arg(short, long, help = "Case-insensitive pattern matching")]
    insensitive: bool,
    #[arg(short, long, help = "Random seed")]
    seed: Option<u64>,
}

pub fn get_args() -> MyResult<Config> {
    let args = ConfigArgs::parse();
    let pattern = args.pattern
        .map(|val| {
            RegexBuilder::new(&val)
                .case_insensitive(args.insensitive)
                .build()
                .map_err(|_| format!("Invalid --pattern: \"{}\"", val))
        })
        .transpose()?;
    Ok(Config {
        sources: args.sources,
        pattern,
        seed: args.seed,
    })
}

fn find_files(paths: &[String]) -> MyResult<Vec<PathBuf>> {
    let dat = OsStr::new("dat");
    let mut files = vec![];
    for path in paths {
        match fs::metadata(path) {
            Err(err) => return Err(format!("{:?}: {}", path, err).into()),
            Ok(_) => files.extend(
                WalkDir::new(path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.file_type().is_file()
                            && e.path().extension() != Some(dat)
                    })
                    .map(|e| e.path().into()),
            )
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn read_fortunes(paths: &[PathBuf]) -> MyResult<Vec<Fortune>> {
    let mut fortunes = vec![];
    let mut buffer = vec![];
    for path in paths {
        let basename =
            path.file_name().unwrap().to_string_lossy().into_owned();
            let file = File::open(path).map_err(|e| {
                format!("{}: {}", path.to_string_lossy().into_owned(), e)
            })?;

            for line in BufReader::new(file).lines().filter_map(Result::ok) {
                if line == "%" {
                    if !buffer.is_empty() {
                        fortunes.push(Fortune {
                            source: basename.clone(),
                            text: buffer.join("\n"),
                        });
                        buffer.clear();
                    }
                } else {
                    buffer.push(line.to_string());
                }
            }
    }
    Ok(fortunes)
}

fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
    if let Some(val) = seed {
        let mut rng = StdRng::seed_from_u64(val);
       fortunes.choose(&mut rng).map(|f| f.text.to_string())
    } else {
        let mut rng = rand::thread_rng();
        fortunes.choose(&mut rng).map(|f| f.text.to_string())
    }
}

pub fn run(config: Config) -> MyResult<()> {
    let files = find_files(&config.sources)?;
    let fortunes = read_fortunes(&files)?;
    if let Some(pattern) = config.pattern {
        let mut prev_source = None;
        for fortune in fortunes
            .iter()
            .filter(|fortune| pattern.is_match(&fortune.text))
        {
            if prev_source.as_ref().map_or(true, |s| s != &fortune.source) {
                eprintln!("({})\n%", fortune.source);
                prev_source = Some(fortune.source.clone());
            }
        }
    } else {
        println!(
            "{}",
            pick_fortune(&fortunes, config.seed)
                .or_else(|| Some("No fortunes found".to_string()))
                .unwrap()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        find_files, read_fortunes, Fortune, pick_fortune
    };
    use std::path::PathBuf;

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );

        // Fails to find a bad file
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());

        // Finds all the input files, excludes ".dat"
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());

        // Check number and order of files
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));

        // Test for multiple sources, path must be unique and sorted
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }

    #[test]
    fn test_read_fortunes() {
        // Parses all the fortunes without a filter
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());

        if let Ok(fortunes) = res {
            // Correct number and sorting
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\n\
                A. Collared greens."
            );
            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\n\
                A: A bad idea (bad-eye deer)."
            );
        }

        // Filters for matching text
        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }

    #[test]
    fn test_pick_fortune() {
        // Create a slice of fortunes
        let fortunes = &[
            Fortune {
                source: "fortunes".to_string(),
                text: "You cannot achieve the impossible without \
                      attempting the absurd."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Assumption is the mother of all screw-ups."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Neckties strangle clear thinking.".to_string(),
            },
        ];

        // Pick a fortune with a seed
        assert_eq!(
            pick_fortune(fortunes, Some(1)).unwrap(),
            "Neckties strangle clear thinking.".to_string()
        );
    }
}
