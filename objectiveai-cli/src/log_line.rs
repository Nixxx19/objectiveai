const PREFIX: &str = "Logs ID: ";

/// Prints the log ID line for the given path.
pub fn print_log_id(path: &str) {
    println!("{PREFIX}{path}");
}

/// Returns true if the line is a log ID line.
pub fn is_log_id_line(line: &str) -> bool {
    line.trim().starts_with(PREFIX)
}
