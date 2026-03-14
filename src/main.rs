use crossterm::cursor;
use std::collections::HashMap;
use std::io::{Write, stdout};
use std::path::PathBuf;
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

    // *brakoll - d: add "doc" subcommand that saves both the summary and the list output into an md file in the target directory, p: 20, t: feature, s: open
    // *brakoll - d: the status change command adds a newline at the end of every file it adjusts issues inside, p: 10, t: bug, s: open
    // *brakoll - d: add command to copy "tag: desc" of an issue with maybe "brakoll copy <id>" for quicker git commit messages, p: 80, t: feature, s: closed
    // *brakoll - d: implement close/open/prog command with id number to allow user to change status of issues from commandline (implement dynamic id application), p: 20, t: feature, s: closed
    let args = arg::parse()?;
    if args.help {
        help::print();
        return Ok(());
    }
    // *brakoll - d: tree subcommand for more visual feedback on where issues are on the file system, p: 0, t: feature, s: open

    // *brakoll - d: add optional target path that can be added at the end of any command (with some logic to identify if the path is a path and/or it is valid) , p: 90, t: feature, s: closed
    // === init ===
    let mut b = Brakoll::new(args)?;
    // set target dir
    if b.args.opt_dir.exists() {
        b.target_dir = PathBuf::from(b.args.opt_dir.clone());
    }

    // === search ===

    let blacklist = vec![
        "node_modules".to_string(),
        "README".to_string(),
        "target".to_string(),
        ".cargo".to_string(),
        ".git".to_string(),
    ];

    // *brakoll - d: implement -r flag to have the program not search for issues in child directories (i.e non-recursive search), p: 20, t: feature, s: closed
    // *brakoll - d: loadingbar is not rendering properly and looks quite ugly, p: 40, t: bug, s: open
    println!("Searching for issues...");
    let files_found = b.walk_children(&blacklist)?;

    // === process ===
    b.issues = b.process_issues(&files_found);

    // === use results ===

    if b.issues.is_empty() {
        utils::issues_found_print(b.issues.len());
        return Ok(());
    };

    if b.args.summary {
        b.summary();
        return Ok(());
    }

    // change status if appropr. flag was added
    if b.args.change_status.1 != None {
        if b.args.change_status.0 == 0 {
            println!("Invalid ID! Status change has been cancelled.");
            return Ok(());
        }
        match b.change_status_of_issue()? {
            false => {
                println!("Invalid ID! Status change has been cancelled.");
                return Ok(());
            }
            true => {
                println!(
                    "Changed status of issue [{}] to \"{}\"",
                    b.args.change_status.0,
                    b.args.change_status.1.unwrap()
                );
                return Ok(());
            }
        }
    }

    // copy/cp subcommand
    if b.args.copy_id != 0 {
        match b.copy_issue_to_clipboard()? {
            true => {
                println!("Copied issue [{}] to OS clipboard", b.args.copy_id,);
                return Ok(());
            }
            false => {
                println!("Invalid ID! Copy operation has been cancelled.");
                return Ok(());
            }
        };
    }

    // filter & sort
    b.apply_filter_flags_to_issues_list();
    b.sort_list();

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
    fn color(&self) -> &str {
        match self {
            IssueStatus::Closed => "\x1b[32m",
            IssueStatus::Open => "\x1b[34m",
            IssueStatus::InProgress => "\x1b[31m",
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
    id: u32,
}

struct Brakoll {
    target_dir: PathBuf,
    issues: Vec<Issue>,
    args: Arguments,
}

impl Brakoll {
    fn new(args: Arguments) -> io::Result<Self> {
        Ok(Self {
            target_dir: env::current_dir()?,
            issues: Vec::new(),
            args,
        })
    }

    // *brakoll - d: there is a fallback 'n/a' if a tag is omitted during issue creation and this would look weird with the current clipboard formatting, p: 60, t: fix, s: closed
    fn copy_issue_to_clipboard(&mut self) -> io::Result<bool> {
        let mut issues = self
            .issues
            .iter()
            .filter(|i| i.id == self.args.copy_id)
            .take(2);
        let issue = match (issues.next(), issues.next()) {
            (Some(issue), None) => issue, // exactly one
            _ => return Ok(false),        // zero or more than one
        };

        let (_, filename) = issue.file.rsplit_once("/").unwrap();

        let mut tag = issue.tag.clone();

        if tag == "n/a" {
            tag = "issue".to_string();
        }

        let output = format!(
            "{t} ({f}:{l}): {d}",
            t = tag,
            f = filename,
            l = issue.line,
            d = issue.desc
        );
        cli_clipboard::set_contents(output.to_owned()).unwrap();

        Ok(true)
    }

    // *brakoll - d: sometimes the status won't change (i'm not sure if this is due to the bash script or the logic to find the line itself), p: 100, t: fix, s: closed
    /// change status if appropr. flag was added
    fn change_status_of_issue(&mut self) -> io::Result<bool> {
        // find issue in self vector
        let mut issues = self
            .issues
            .iter()
            .filter(|i| i.id == self.args.change_status.0)
            .take(2);
        let issue = match (issues.next(), issues.next()) {
            (Some(issue), None) => issue, // exactly one
            _ => return Ok(false),        // zero or more than one
        };

        let content = fs::read_to_string(&issue.file)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        if issue.line == 0 || issue.line > lines.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Line {} is out of range!", issue.line),
            ));
        }

        let line = &mut lines[issue.line - 1];
        let old_status: String = issue.status.to_string();

        if !line.contains("s:") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Status {} was not found on line {}!",
                    old_status, issue.line
                ),
            ));
        }

        let new_status: String = self.args.change_status.1.unwrap().to_string();

        *line = if let Some(pos) = line.find("s:") {
            format!("{}s: {}", &line[..pos], &new_status)
        } else {
                line.to_string()
            };

        let new_content = lines.join("\n");
        fs::write(&issue.file, new_content)?;

        return Ok(true);
    }

    fn apply_filter_flags_to_issues_list(&mut self) {
        // *brakoll - d: implement -d <word(s)> to filter output of list() by desc, p: 100, t: feature, s: closed
        let desc_flag = self.args.filter_desc.clone();
        if !desc_flag.is_empty() {
            println!("Filter by desc: {}", desc_flag);
            self.issues = self
                .issues
                .clone()
                .into_iter()
                .filter(|i| i.desc.contains(&desc_flag))
                .collect();
        }

        // *brakoll - d: implement -t <tag> flag to filter output of list() by tag, p: 50, t: feature, s: closed
        // apply tag filter flag
        let tag_flag = self.args.filter_tag.clone();
        if !tag_flag.is_empty() {
            println!("Filter by tag: {}", tag_flag);
            self.issues = self
                .issues
                .clone()
                .into_iter()
                .filter(|i| i.tag.contains(&tag_flag))
                .collect();
        }

        // *brakoll - d: implement -s <status> flag to filter output of list() by status, p: 70, t: feature, s: closed
        // apply status filter flag
        let stat_flag = self.args.filter_status.clone();
        if stat_flag != None {
            println!("Filter by status: {}", stat_flag.unwrap());
            self.issues = self
                .issues
                .clone()
                .into_iter()
                .filter(|i| i.status == stat_flag.unwrap())
                .collect();
        }
    }

    // *brakoll - d: refactor blacklist so that only a single one exists for both the count_search_items() function and the other one whatever that was called, p: 40, t: refactor, s: closed
    fn count_search_items(&mut self, blacklist: &Vec<String>) -> io::Result<usize> {
        let valid_file_extensions = utils::get_valid_file_ext();

        // init walker
        let mut walker = WalkDir::new(&self.target_dir);
        if self.args.no_rec {
            walker = walker.max_depth(1);
        }
        let walker = walker.into_iter();

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
    fn walk_children(&mut self, blacklist: &Vec<String>) -> io::Result<Vec<String>> {
        let items_to_search = self.count_search_items(&blacklist)?;

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

        let valid_file_extensions = utils::get_valid_file_ext();

        // init walker
        let mut walker = WalkDir::new(&self.target_dir);
        if self.args.no_rec {
            walker = walker.max_depth(1);
        }
        let walker = walker.into_iter();

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

    fn process_issues(&mut self, files_found: &Vec<String>) -> Vec<Issue> {
        let mut parsed_issues = Vec::new();

        let mut id_counter: u32 = 1;

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

                id_counter += 1;

                parsed_issues.push(Issue {
                    file: f.clone(),
                    line: line_no,
                    desc: d.to_string(),
                    prio,
                    tag: t.to_string(),
                    status,
                    id: id_counter,
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
            println!(
                "{color_start}{count}\t: {status}\x1b[0m",
                color_start = t.color(),
                status = t
            );
        }
    }

    // *brakoll - d: overhaul formatting of issues in list() according to test.txt in root, p: 90, t: feature, s: closed
    // *brakoll - d: add sorting logic to list() function, p: 100, t: feature, s: closed
    /// list all issues found
    fn list(&mut self) {
        let len = self.issues.len();
        utils::issues_found_print(len);
        println!("");
        for i in self.issues.iter_mut() {
            print!("{}", i.status.color());
            if i.status == IssueStatus::Closed {
                println!("[{i}] {s}", i = i.id, s = i.status);
            } else {
                println!("[{i}] {p} - {s}", i = i.id, p = i.prio, s = i.status);
            }
            println!(
                "{f}:{l}",
                f = utils::shorten_path(i.file.clone()),
                l = i.line
            );
            println!("{t}: {d}", t = i.tag, d = i.desc);
            print!("{}", "\x1b[0m");
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