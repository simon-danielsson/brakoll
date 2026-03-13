use std::collections::HashMap;
use std::fmt;
use std::fs::read_to_string;
use std::path::Path;
use std::{env, io};
use walkdir::{DirEntry, WalkDir};

use crate::arg::Arguments;

mod arg;
mod help;
mod utils;

const PREFIX: &str = "*brakoll";

// === default values for issues ===
const DEF_DESC: &str = "issue";
const DEF_PRIO: u32 = 0;
const DEF_TAG: &str = "n/a";
const DEF_STAT: IssueStatus = IssueStatus::Open;

// === program ===

fn main() -> io::Result<()> {
    // get args
    let args = arg::parse();
    if args.help {
        help::print();
        return Ok(());
    }
    // *brakoll - d: test, p: 0, t: example, s: in progress

    // init app
    let mut b = Brakoll::new(args);

    // get issues
    let files_found = b.walk_children()?;
    b.issues = b.process_issues(files_found);

    // display issues
    if b.issues.is_empty() {
        utils::issues_found_print(b.issues.len());
        return Ok(());
    };

    if b.args.summary {
        b.summary();
        return Ok(());
    }

    b.list();
    Ok(())
}

#[derive(Debug, Eq, Hash, PartialEq)]
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
    args: Arguments,
}

impl Brakoll {
    fn new(args: Arguments) -> Self {
        Self {
            issues: Vec::new(),
            args,
        }
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

                let f_sh = utils::shorten_path(f.clone());

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

    /// total tags and status count
    fn summary(&mut self) {
        let len = self.issues.len();
        utils::issues_found_print(len);
        println!("");

        let mut tag_counts: HashMap<&str, usize> = HashMap::new();
        let mut status_counts: HashMap<&IssueStatus, usize> = HashMap::new();

        for t in &self.issues {
            *tag_counts.entry(&t.tag).or_insert(0) += 1;
            *status_counts.entry(&t.status).or_insert(0) += 1;
        }

        // sort

        let mut tags: Vec<_> = tag_counts.into_iter().collect();
        tags.sort_by_key(|(_, c)| std::cmp::Reverse(*c));

        let mut status: Vec<_> = status_counts.into_iter().collect();
        status.sort_by_key(|(_, c)| std::cmp::Reverse(*c));

        // print

        println!("tags");
        println!("-------");
        for (tag, count) in tags {
            println!("{count}\t: {tag}");
        }

        println!("\nstatus");
        println!("-------");
        for (t, count) in status {
            println!("{count}\t: {}", t);
        }
    }

    /// list all issues found
    fn list(&mut self) {
        let len = self.issues.len();
        utils::issues_found_print(len);
        println!("");
        for i in self.issues.iter_mut() {
            println!(
                "{h} {p}: {s} {h}",
                p = i.prio,
                s = i.status,
                h = utils::issue_header_decor(&i.status)
            );
            println!("file: {}", i.file);
            println!("line: {l}, tag: {t}", l = i.line, t = i.tag);
            println!("desc: {}", i.desc);
            println!("");
        }
        utils::issues_found_print(len);
    }
    // *brakoll - d: tetet, p: 10, t: example, s: closed

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
}
