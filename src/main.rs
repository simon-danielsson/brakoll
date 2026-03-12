use dirs::home_dir;
use std::fmt;
use std::fs::read_to_string;
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
const DEF_STAT: IssueStatus = IssueStatus::Todo;

// === program ===

fn main() -> io::Result<()> {
    // let l = read_lines("main.rs");
    // for line in l.iter() {
    //     print!("Line: {}\nIssue: {}\n", line.0, line.1)
    // }
    let mut b = Brakoll::new();
    let files_found = b.walk_children()?;
    b.issues = b.process_issues(files_found);
    b.list();
    Ok(())
}

#[derive(Debug, PartialEq)]
enum IssueStatus {
    Todo,
    Done,
}

impl fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            IssueStatus::Todo => "todo",
            IssueStatus::Done => "done",
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

    fn is_hidden(&mut self, entry: &DirEntry) -> bool {
        entry.file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(false)
    }

    /// returns paths to be searched for issues
    fn walk_children(&mut self) -> io::Result<Vec<String>> {
        let cd = env::current_dir()?;

        let walker = WalkDir::new(cd).into_iter();
        let valid_file_extensions = utils::get_valid_file_ext();
        // println!("{:?}", valid_file_extensions);

        let mut valid_paths_found: Vec<String> = Vec::new();

        for entry in walker.filter_entry(|e| !self.is_hidden(e)) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                // if file has a valid file extension, derive issues from it
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if valid_file_extensions.iter().any(|e| e == &ext) {
                        valid_paths_found.push(path.display().to_string());
                    }
                }
            }
        }
        Ok(valid_paths_found)
    }

    // *bk - d: this program works now, p: 81, t: engine, s: done
    fn process_issues(&mut self, files_found: Vec<String>) -> Vec<Issue> {
        let mut parsed_issues: Vec<Issue> = Vec::new();

        for f in files_found {
            let raw_issues = self.find_issues(&f);

            // example:
            // *brakoll - d: fix formatting issue in debug statement, p: 10, t: debug, s: todo
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
                    if s_as_str.to_lowercase().contains("todo") {
                        s = IssueStatus::Todo;
                    } else {
                        s = IssueStatus::Done;
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

    fn list(&mut self) {
        println!("{} issue(s) were found.", self.issues.len());
        println!("");
        for i in self.issues.iter_mut() {
            if i.status == IssueStatus::Todo {
                println!("*** {p}: {s} ***", p = i.prio, s = i.status);
            } else {
                println!("=== {p}: {s} ===", p = i.prio, s = i.status);
            }
            println!("file: {}", i.file);
            println!("line: {l}, tag: {t}", l = i.line, t = i.tag);
            println!("desc: {}", i.desc);
            println!("");
        }
        println!("{} issue(s) were found.", self.issues.len());
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
