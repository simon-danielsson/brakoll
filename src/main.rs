use dirs::home_dir;
use std::fmt;
use std::fs::read_to_string;
use std::path::Path;
use std::{env, io};
use walkdir::{DirEntry, WalkDir};

mod arg;
mod utils;

const PREFIX: &str = "*brakoll";

// === default values for issues ===
const DEF_DESC: &str = "issue";
// default prio if it is both missing or invalid (i.e. -2 or 101)
const DEF_PRIO: u32 = 0;
const DEF_TAG: &str = "n/a";
const DEF_STAT: IssueStatus = IssueStatus::Open;

// === program ===

fn main() -> io::Result<()> {
    let mut b = Brakoll::new();
    let files_found = b.walk_children()?;
    b.issues = b.process_issues(files_found);
    b.list();
    Ok(())
}

#[derive(Debug, PartialEq)]
enum IssueStatus {
    Open,
    InProgress,
    Closed,
}

impl fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            IssueStatus::Open => "open",
            IssueStatus::InProgress => "in progress",
            IssueStatus::Closed => "closed",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug)]
struct Issue {
    file: String,
    line: usize,
    desc: String,
    prio: u32,
    tag: String,
    status: IssueStatus,
}

struct Brakoll {
    issues: Vec<Issue>,
}

impl Brakoll {
    fn new() -> Self {
        Self { issues: Vec::new() }
    }

    fn has_hidden_component(&mut self, path: &Path) -> bool {
        path.components().any(|component| {
            component
                .as_os_str()
                .to_str()
                .map(|s| s.starts_with('.'))
                .unwrap_or(false)
        })
    }

    fn contains_blacklisted_path(&mut self, path: &Path, blacklist: &[String]) -> bool {
        let path_str = path.to_string_lossy();
        blacklist.iter().any(|needle| path_str.contains(needle))
    }

    fn should_ignore(&mut self, entry: &DirEntry, blacklist: &[String]) -> bool {
        let path = entry.path();
        self.has_hidden_component(path) || self.contains_blacklisted_path(path, blacklist)
    }

    /// returns paths to be searched for issues
    fn walk_children(&mut self) -> io::Result<Vec<String>> {
        let cd = env::current_dir()?;
        let valid_file_extensions = utils::get_valid_file_ext();

        let blacklist = vec![
            "node_modules".to_string(),
            "target".to_string(),
            ".cargo".to_string(),
            ".git".to_string(),
        ];

        let walker = WalkDir::new(&cd).into_iter();
        let mut valid_paths_found = Vec::new();

        for entry in walker.filter_entry(|e| !self.should_ignore(e, &blacklist)) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if valid_file_extensions.iter().any(|e| e == ext) {
                        valid_paths_found.push(path.display().to_string());
                    }
                }
            }
        }

        Ok(valid_paths_found)
    }

    fn process_issues(&mut self, files_found: Vec<String>) -> Vec<Issue> {
        let mut parsed_issues: Vec<Issue> = Vec::new();

        for f in files_found {
            let raw_issues = self.find_issues(&f);

            // example:
            // *brakoll - d: fix formatting issue in debug statement, p: 10, t: debug, s: open
            for i in raw_issues {
                let mut d = "";
                let mut t = "";
                let mut s_as_str = "";
                let mut p_as_str = "";

                {
                    let mut string = "";

                    if let Some(pos) = i.1.find("d:") {
                        string = &i.1[pos..];
                    }

                    for pair in string.split(',') {
                        let mut parts = pair.trim().splitn(2, ':');
                        let key = parts.next().unwrap().trim();
                        let value = parts.next().unwrap().trim();

                        match key {
                            "t" => t = value,
                            "d" => d = value,
                            "s" => s_as_str = value,
                            "p" => p_as_str = value,
                            _ => {}
                        }
                    }
                }

                if d.is_empty() {
                    d = DEF_DESC;
                }

                if t.is_empty() {
                    t = DEF_TAG;
                }

                let mut s = DEF_STAT;
                if !s_as_str.is_empty() {
                    _ = s_as_str.to_lowercase();
                    // split up word to account for some misspellings
                    match () {
                        _ if s_as_str.contains("op")
                        | s_as_str.contains("en") =>
                        {
                            s = IssueStatus::Open;
                        }
                        _ if s_as_str.contains("pr")
                        | s_as_str.contains("og") =>
                        {
                            s = IssueStatus::InProgress;
                        }
                        _ if s_as_str.contains("cl")
                        | s_as_str.contains("os") =>
                        {
                            s = IssueStatus::Closed;
                        }
                        _ => {}
                    }
                }

                let mut p = DEF_PRIO;
                if !p_as_str.is_empty() {
                    p = p_as_str.parse::<u32>().unwrap_or_default();
                }

                let f_sh = self.shorten_path(f.clone());

                parsed_issues.push(Issue {
                    file: f_sh,
                    line: i.0,
                    desc: d.to_string(),
                    prio: p,
                    tag: t.to_string(),
                    status: s,
                });
            }
        }

        parsed_issues
    }

    /// helper for list()
    fn issues_found_print(&mut self, len: usize) {
        match len {
            0 => {
                println!("No issues were found.");
            }
            1 => {
                println!("1 issue was found.");
            }
            _ => {
                println!("{} issues were found.", len);
            }
        }
    }

    /// list all issues found
    fn list(&mut self) {
        let len = self.issues.len();
        if self.issues.is_empty() {
            self.issues_found_print(len);
        } else {
            self.issues_found_print(len);
            println!("");
            for i in self.issues.iter_mut() {
                // different header decoration depending on status
                if i.status == IssueStatus::Open {
                    println!("*** {p}: {s} ***", p = i.prio, s = i.status);
                } else if i.status == IssueStatus::InProgress {
                    println!("/// {p}: {s} ///", p = i.prio, s = i.status);
                } else if i.status == IssueStatus::Closed {
                    println!("=== {p}: {s} ===", p = i.prio, s = i.status);
                }
                println!("file: {}", i.file);
                println!("line: {l}, tag: {t}", l = i.line, t = i.tag);
                println!("desc: {}", i.desc);
                println!("");
            }
            self.issues_found_print(len);
        }
    }

    /// returns line beginning with prefix and derives usize(line #) and String (raw issue line)
    fn find_issues(&mut self, filename: &String) -> Vec<(usize, String)> {
        read_to_string(filename)
            .unwrap()
            .lines()
            .enumerate()
            .filter(|(_, line)| line.contains(PREFIX))
            .filter(|(_, line)| line.contains("d:"))
            .map(|(i, line)| (i + 1, line.to_owned()))
            .collect()
    }

    fn shorten_path(&mut self, path: String) -> String {
        if let Some(home) = home_dir() {
            let home = home.to_string_lossy();
            if path.starts_with(home.as_ref()) {
                return path.replacen(home.as_ref(), "~", 1);
            }
        }
        path
    }
}
