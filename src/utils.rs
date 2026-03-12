const VALID_FILE_EXT: &str = include_str!("./valid_file_ext");

/// derive vec or file ext from file in src dir
pub fn get_valid_file_ext() -> Vec<String> {
    VALID_FILE_EXT
        .lines()
        .map(|l| l.trim())
        .map(|l| l.to_owned())
        .collect()
}
