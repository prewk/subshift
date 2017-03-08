extern crate chrono;
extern crate time;

use chrono::prelude::*;
use time::Duration;
use std::io::BufReader;
use std::io::BufRead;
use std::io::Write;
use std::fs::File;
use std::str::FromStr;

struct Subtitle {
    id: u64,
    from: NaiveTime,
    to: NaiveTime,
    texts: Vec<String>,
}

enum CaptureMode {
    Id,
    Range,
    Texts,
}

fn capture_id(line: String) -> Option<u64> {
    match u64::from_str(&line) {
        Ok(id) => Some(id),
        _ => None,
    }
}

fn capture_ranges(line: String) -> Option<(NaiveTime, NaiveTime)> {
    let mut line_split = line.split(" --> ");

    match (line_split.next(), line_split.next()) {
        (Some(from), Some(to)) => {
            match (
                NaiveTime::parse_from_str(&format!("{}000000", from), "%H:%M:%S,%f"),
                NaiveTime::parse_from_str(&format!("{}000000", to), "%H:%M:%S,%f")
            ) {
                (Ok(from), Ok(to)) => Some((from, to)),
                _ => None,
            }
        },
        _ => None,
    }    
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        writeln!(std::io::stderr(), "Usage: subshift FILE SECONDS").unwrap();
        writeln!(std::io::stderr(), "Example: {} subtitle.srt -20", args[0]).unwrap();

        std::process::exit(1);
    }

    let filename = &args[1];
    let offset_s = match i64::from_str(&args[2]) {
        Ok(s) => s,
        _ => {
            writeln!(std::io::stderr(), "Invalid SECONDS format").unwrap();
            std::process::exit(1);
        },
    };

    let source = File::open(&filename).unwrap();
    let reader = BufReader::new(&source);

    let mut subs: Vec<Subtitle> = vec![];

    let mut next_sub = Subtitle {
        id: 0,
        from: NaiveTime::from_hms_milli(0, 0, 0, 0),
        to: NaiveTime::from_hms_milli(0, 0, 0, 0),
        texts: vec![],
    };

    let mut mode = CaptureMode::Id;

    for wrapped_line in reader.lines() {
        let line = wrapped_line.unwrap();

        match mode {
            CaptureMode::Id => {
                match capture_id(line) {
                    Some(id) => {
                        next_sub.id = id;

                        mode = CaptureMode::Range;
                    },
                    None => {},
                }
            },
            CaptureMode::Range => {
                match capture_ranges(line) {
                    Some((from, to)) => {
                        next_sub.from = from;
                        next_sub.to = to;

                        mode = CaptureMode::Texts;
                    },
                    None => {},
                }
            },
            CaptureMode::Texts => {
                if line.len() > 0 {
                    next_sub.texts.push(line);
                } else {
                    subs.push(Subtitle {
                        id: next_sub.id,
                        from: next_sub.from,
                        to: next_sub.to,
                        texts: next_sub.texts,
                    });

                    next_sub = Subtitle {
                        id: 0,
                        from: NaiveTime::from_hms_milli(0, 0, 0, 0),
                        to: NaiveTime::from_hms_milli(0, 0, 0, 0),
                        texts: vec![],
                    };

                    mode = CaptureMode::Id;
                }
            },
        }
    }

    match mode {
        CaptureMode::Texts => {
            subs.push(Subtitle {
                id: next_sub.id,
                from: next_sub.from,
                to: next_sub.to,
                texts: next_sub.texts,
            });
        },
        _ => {},
    }

    for sub in subs {
        let offset_from = sub.from + Duration::seconds(offset_s);
        let offset_to = sub.to + Duration::seconds(offset_s);

        println!("{}", sub.id);
        println!(
            "{}:{}:{},{} --> {}:{}:{},{}",
            offset_from.hour(),
            offset_from.minute(),
            offset_from.second(),
            offset_from.nanosecond() / 1000000,
            offset_to.hour(),
            offset_to.minute(),
            offset_to.second(),
            offset_to.nanosecond() / 1000000
        );
        for text in sub.texts {
            println!("{}", text);
        }
        println!("");
    }
}
