use dirs::home_dir;

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
