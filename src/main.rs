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

    if b.args.summary {
        b.summary();
        return Ok(());
    }

    b.sort_list();

    // *brakoll - d: implement -t <tag> flag to filter output of list() by tag, p: 50, t: feature, s: closed
    // apply tag filter flag
    let tag_flag = b.args.filter_tag.clone();
    if !tag_flag.is_empty() {
        println!("Filter by tag: {}", tag_flag);
        b.issues =
            b.issues.clone()
                .into_iter()
                .filter(|i| i.tag.contains(&tag_flag))
                .collect();
    }

    // *brakoll - d: implement -s <status> flag to filter output of list() by status, p: 70, t: feature, s: closed
    // apply status filter flag
    let stat_flag = b.args.filter_status.clone();
    if stat_flag != None {
        println!("Filter by status: {}", stat_flag.unwrap());
        b.issues =
            b.issues.clone()
                .into_iter()
                .filter(|i| i.status == stat_flag.unwrap())
                .collect();
    }

    // *brakoll - d: color formatting depending on status (perhaps green=done yellow=prog blue=open), p: 0, t: feature, s: closed
    b.list();
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum IssueStatus {
    Closed,
    Open,
    InProgress,
}

impl IssueStatus {
    /// (header<&str>, ansi_color<&str>)
    fn attr(&self) -> (&str, &str) {
        match self {
            IssueStatus::Closed => ("===", "\x1b[32m"),
            IssueStatus::Open => ("***", "\x1b[34m"),
            IssueStatus::InProgress => ("!!!", "\x1b[31m"),
        }
    }

    fn rank(&self) -> u8 {
        match self {
            IssueStatus::Closed => 0,
            IssueStatus::Open => 1,
            IssueStatus::InProgress => 2,
        }
    }
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

#[derive(Clone, Debug)]
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
            "README".to_string(),
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
            "README".to_string(),
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
        let color_end: String = "\x1b[0m".to_string();
        for (t, count) in status {
            println!(
                "{color_start}{count}\t: {status}{color_end}",
                color_start = t.attr().1,
                status = t
            );
        }
    }

    // *brakoll - d: add sorting logic to list() function, p: 100, t: feature, s: closed
    /// list all issues found
    fn list(&mut self) {
        let len = self.issues.len();
        utils::issues_found_print(len);
        println!("");
        for i in self.issues.iter_mut() {
            let attr = i.status.attr();
            let color_end: String = "\x1b[0m".to_string();

            print!("{}", attr.1);
            println!("{h} {p}: {s} {h}", p = i.prio, s = i.status, h = attr.0,);
            println!("file: {}", i.file);
            println!("line: {l}, tag: {t}", l = i.line, t = i.tag);
            println!("desc: {}", i.desc);
            print!("{}", color_end);
            println!("");
        }
    }

    fn sort_list(&mut self) {
        self.issues.sort_by(|a, b| a.prio.cmp(&b.prio));
        self.issues.sort_by_key(|t| t.status.rank());
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
