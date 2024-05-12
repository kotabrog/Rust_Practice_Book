use ansi_term::Style;
use chrono::{Datelike, Local, NaiveDate};
use clap::Parser;
use itertools::izip;
use std::error::Error;

const LINE_WIDTH: usize = 22;

const MONTH_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December"
];

#[derive(Debug)]
pub struct Config {
    month: Option<u32>,
    year: i32,
    today: NaiveDate,
}

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
pub struct ConfigArgs {
    #[arg(name = "YEAR", help = "Year (1-9999)")]
    year: Option<i32>,
    #[arg(short, help = "Month name or number (1-12)")]
    month: Option<String>,
    #[arg(short = 'y', long = "year", help = "Show whole current year")]
    year_flag: bool,
}

fn parse_year(year: i32) -> MyResult<i32> {
    if (1..=9999).contains(&year) {
        Ok(year)
    } else {
        Err(format!("year \"{}\" not in the range 1 through 9999", year).into())
    }
}

fn parse_month(month: String) -> MyResult<u32> {
    match month.parse::<u32>() {
        Ok(month) => {
            if (1..=12).contains(&month) {
                Ok(month)
            } else {
                Err(format!("month \"{}\" not in the range 1 through 12", month).into())
            }
        }
        _ => {
            let lower = month.to_lowercase();
            let matches: Vec<_> = MONTH_NAMES.iter().enumerate().filter_map(
                |(i, name)| {
                    if name.to_lowercase().starts_with(&lower) {
                        Some(i as u32 + 1)
                    } else {
                        None
                    }
                }
            ).collect();
            if matches.len() == 1 {
                Ok(matches[0])
            } else {
                Err(format!("invalid month name \"{}\"", month).into())
            }
        }
    }
}

pub fn get_args() -> MyResult<Config> {
    let args = ConfigArgs::parse();
    let mut month = args.month.map(parse_month).transpose()?;
    let mut year = args.year.map(parse_year).transpose()?;
    let today = Local::now().naive_local();
    if args.year_flag {
        month = None;
        year = Some(today.year());
    } else if month.is_none() && year.is_none() {
        month = Some(today.month());
        year = Some(today.year());
    }
    Ok(Config {
        month,
        year: year.unwrap_or_else(|| today.year()),
        today: today.date(),
    })
}

fn last_day_in_month(year: i32, month: u32) -> NaiveDate {
    let (y, m) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(y, m, 1).unwrap().pred_opt().unwrap()
}

fn format_month(
    year: i32,
    month: u32,
    print_year: bool,
    today: NaiveDate,
) -> Vec<String> {
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let mut days: Vec<String> = (1..first.weekday().number_from_sunday())
        .map(|_| "  ".to_string())
        .collect();
    let is_today = |day: u32| {
        day == today.day() && month == today.month() as u32 && year == today.year()
    };
    let last = last_day_in_month(year, month);
    days.extend((first.day()..=last.day()).map(|num| {
        let fmt = format!("{:>2}", num);
        if is_today(num) {
            Style::new().reverse().paint(fmt).to_string()
        } else {
            fmt
        }
    }));
    let month_name = MONTH_NAMES[month as usize - 1];
    let mut lines = Vec::with_capacity(8);
    lines.push(format!(
        "{:^20}  ",
        if print_year {
            format!("{} {}", month_name, year)
        } else {
            month_name.to_string()
        }
    ));

    lines.push("Su Mo Tu We Th Fr Sa  ".to_string());

    for week in days.chunks(7) {
        lines.push(format!(
            "{:width$}  ",
            week.join(" "),
            width = LINE_WIDTH - 2
        ));
    }

    while lines.len() < 8 {
        lines.push(" ".repeat(LINE_WIDTH));
    }

    lines
}

pub fn run(config: Config) -> MyResult<()> {
    match config.month {
        Some(month) => {
            let lines = format_month(config.year, month, true, config.today);
            println!("{}", lines.join("\n"));
        }
        None => {
            println!("{:^32}", config.year);
            let months: Vec<_> = (1..=12)
                .map(|month| format_month(config.year, month, false, config.today))
                .collect();
            for (i, chunk) in months.chunks(3).enumerate() {
                if let [m1, m2, m3] = chunk {
                    for lines in izip!(m1, m2, m3) {
                        println!("{}{}{}", lines.0, lines.1, lines.2);
                    }
                    if i < 3 {
                        println!();
                    }
                }
            }
        }
    }
    Ok(())
}
