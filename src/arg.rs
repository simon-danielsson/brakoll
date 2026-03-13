use std::{io, path::PathBuf};

use crate::IssueStatus;

#[derive(PartialEq, Clone)]
pub struct Arguments {
    // pub no_tui: bool,
    // pub location: String,
    pub help: bool,
    pub summary: bool,
    pub filter_tag: String,
    pub filter_desc: String,
    pub filter_status: Option<IssueStatus>,
    pub opt_dir: PathBuf,
    // pub forecast: i32,
}

pub fn parse() -> io::Result<Arguments> {
    let mut it = std::env::args().skip(1); // skip program name
    let mut filter_tag = String::new();
    let mut filter_status = String::new();
    let mut filter_desc = String::new();
    let mut help = false;
    let mut summary = false;
    let mut opt_dir = PathBuf::new();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-t" => {
                filter_tag =
                    it.next().expect("No tag was given after the \"-t\" flag.");
            }

            "-s" => {
                filter_status = it
                    .next()
                    .expect("No status was given after the \"-s\" flag.");
            }

            "-d" => {
                filter_desc = it
                    .next()
                    .expect("No status was given after the \"-s\" flag.");
            }

            "help" => {
                help = true;
            }

            "summary" => {
                summary = true;
            }

            // "-f" => {
            //     // use next if some and parse to i32, else default
            //     forecast = it
            //         .next()
            //         .as_deref()
            //         .unwrap_or(format!("{}", DEF_FORECAST).as_str())
            //         .parse::<i32>()
            //         .unwrap_or(DEF_FORECAST);
            // }
            other => {
                opt_dir = PathBuf::from(other);
                break;
            }
        }
    }

    let s_lower = filter_status.to_lowercase();
    let status: Option<IssueStatus> = match () {
        _ if s_lower.contains("op") || s_lower.contains("en") => Some(IssueStatus::Open),
        _ if s_lower.contains("pr") || s_lower.contains("og") => {
            Some(IssueStatus::InProgress)
        }
        _ if s_lower.contains("cl") || s_lower.contains("os") => Some(IssueStatus::Closed),
        _ => None,
    };

    let filter_tag = filter_tag.trim();
    let filter_desc = filter_desc.trim();

    Ok(Arguments {
        help,
        summary,
        filter_tag: filter_tag.to_string(),
        filter_desc: filter_desc.to_string(),
        filter_status: status,
        opt_dir,
    })
}
