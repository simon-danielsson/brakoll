use dirs::home_dir;

use crate::IssueStatus;

const VALID_FILE_EXT: &str = include_str!("./valid_file_ext");

/// derive vec or file ext from file in src dir
pub fn get_valid_file_ext() -> Vec<String> {
    VALID_FILE_EXT
        .lines()
        .map(|l| l.trim())
        .map(|l| l.to_owned())
        .collect()
}

/// replace home part of path with "~"
pub fn shorten_path(path: String) -> String {
    if let Some(home) = home_dir() {
        let home = home.to_string_lossy();
        if path.starts_with(home.as_ref()) {
            return path.replacen(home.as_ref(), "~", 1);
        }
    }
    path
}

/// helper for list()
pub fn issues_found_print(l: usize) {
    match l {
        0 => {
            println!("No issues were found.");
        }
        1 => {
            println!("1 issue was found.");
        }
        _ => {
            println!("{} issues were found.", l);
        }
    }
}

/// helper for list()
pub fn issue_header_decor(i: &IssueStatus) -> &str {
    match i {
        IssueStatus::Open => return "***",
        IssueStatus::InProgress => return "///",
        IssueStatus::Closed => return "===",
    }
}
