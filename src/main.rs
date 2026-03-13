use crossterm::cursor;
use std::collections::HashMap;
use std::io::{Write, stdout};
use std::{env, io};
use std::{fmt, fs};
use walkdir::WalkDir;

use crate::arg::Arguments;
use crate::loading_bar::{LoadingBar, State};

mod arg;
mod help;
mod loading_bar;
mod utils;

const PREFIX: &str = "*brakoll";

// === default values for issues ===
const DEF_DESC: &str = "issue";
const DEF_PRIO: u32 = 0;
const DEF_TAG: &str = "n/a";
const DEF_STAT: IssueStatus = IssueStatus::Open;

// === program ===

fn main() -> io::Result<()> {
    // === get args ===

    let args = arg::parse();
    if args.help {
        help::print();
        return Ok(());
    }
    // *brakoll - d: tree subcommand for more visual feedback on where issues are on the file system, p: 0, t: feature, s: open

    // === init ===
    let mut b = Brakoll::new(args);

    // === search ===
    println!("Searching for issues...");
    let files_found = b.walk_children()?;

    // === process ===
    b.issues = b.process_issues(files_found);

    // === use results ===

    if b.issues.is_empty() {
        utils::issues_found_print(b.issues.len());
        return Ok(());
    };

    // *brakoll - d: implement -t <tag> flag to filter output of summary() by tag, p: 50, t: feature, s: open
    // *brakoll - d: implement -s <status> flag to filter output of summary() by status, p: 70, t: feature, s: open
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

    fn count_search_items(&mut self) -> io::Result<usize> {
        let cd = env::current_dir()?;
        let valid_file_extensions = utils::get_valid_file_ext();

        let blacklist = vec![
            "node_modules".to_string(),
            "target".to_string(),
            ".cargo".to_string(),
            ".git".to_string(),
        ];

        let walker = WalkDir::new(&cd).into_iter();
        let mut count = 0;

        for entry in walker.filter_entry(|e| !utils::should_ignore(e, &blacklist)) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if valid_file_extensions.iter().any(|e| e == ext) {
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }

    /// returns paths to be searched for issues
    fn walk_children(&mut self) -> io::Result<Vec<String>> {
        let items_to_search = self.count_search_items()?;

        // init loading bar
        let sout = stdout();
        let init_cursor_pos = cursor::position()?;
        let mut lb = LoadingBar::new(
            sout,
            items_to_search as i32,
            init_cursor_pos.0,
            init_cursor_pos.1 + 1,
        );
        lb.util_setup()?;

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

        for entry in walker.filter_entry(|e| !utils::should_ignore(e, &blacklist)) {
            lb.controls()?;
            if lb.state == State::Quit {
                break;
            }

            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if valid_file_extensions.iter().any(|e| e == ext) {
                        valid_paths_found.push(path.display().to_string());
                        lb.processed_counter += 1;
                        if lb.processed_counter
                        <= lb.files_to_process as i32
                        {
                            lb.loading_bar()?;
                            lb.sout.flush()?;
                        } else {
                            lb.state = State::Quit;
                        }
                    }
                }
            }
        }

        lb.util_cleanup()?;

        Ok(valid_paths_found)
    }

    fn process_issues(&mut self, files_found: Vec<String>) -> Vec<Issue> {
        let mut parsed_issues = Vec::new();

        for f in files_found {
            let raw_issues = self.find_issues(&f);

            for (line_no, raw_line) in raw_issues {
                let mut d = DEF_DESC;
                let mut t = DEF_TAG;
                let mut s_as_str = "";
                let mut p_as_str = "";

                let string = if let Some(pos) = raw_line.find("d:") {
                    &raw_line[pos..]
                } else {
                    ""
                };

                for pair in string.split(',') {
                    let Some((key, value)) = pair.trim().split_once(':') else {
                        continue;
                    };

                    match key.trim() {
                        "t" => t = value.trim(),
                        "d" => d = value.trim(),
                        "s" => s_as_str = value.trim(),
                        "p" => p_as_str = value.trim(),
                        _ => {}
                    }
                }

                let s_lower = s_as_str.to_lowercase();
                let status = match () {
                    _ if s_lower.contains("op") || s_lower.contains("en") => {
                        IssueStatus::Open
                    }
                    _ if s_lower.contains("pr") || s_lower.contains("og") => {
                        IssueStatus::InProgress
                    }
                    _ if s_lower.contains("cl") || s_lower.contains("os") => {
                        IssueStatus::Closed
                    }
                    _ => DEF_STAT,
                };

                let prio = p_as_str.parse::<u32>().unwrap_or(DEF_PRIO);

                parsed_issues.push(Issue {
                    file: utils::shorten_path(f.clone()),
                    line: line_no,
                    desc: d.to_string(),
                    prio,
                    tag: t.to_string(),
                    status,
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

    // *brakoll - d: add sorting logic to list() function, p: 100, t: feature, s: open
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
    }

    /// returns line beginning with prefix and derives usize(line #) and String (raw issue line)

    fn find_issues(&mut self, filename: &String) -> Vec<(usize, String)> {
        let bytes = fs::read(filename).unwrap();
        let content = String::from_utf8_lossy(&bytes);

        content.lines()
            .enumerate()
            .filter(|(_, line)| line.contains(PREFIX))
            .filter(|(_, line)| line.contains("d:"))
            .map(|(i, line)| (i + 1, line.to_owned()))
            .collect()
    }
}
