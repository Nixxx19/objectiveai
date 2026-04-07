use std::fs;
use std::path::Path;

fn check_file(path: &Path, patterns: &[&str]) -> Vec<String> {
    let content = fs::read_to_string(path).unwrap();
    let filename = path.display().to_string();
    let mut violations = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        let line_num = i + 1;
        for pattern in patterns {
            if trimmed.contains(pattern) {
                violations.push(format!("{}:{}: {}", filename, line_num, trimmed));
                break;
            }
        }
    }

    violations
}

fn check_non_main(patterns: &[&str]) -> Vec<String> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let main_rs = src.join("main.rs");

    let mut files = Vec::new();
    collect_rs_files(&src, &mut files);

    let mut all_violations = Vec::new();
    for file in files {
        if file == main_rs {
            continue;
        }
        // Skip test modules — they may contain string literals with print/exit
        if file.file_name().is_some_and(|n| n.to_str().is_some_and(|s| s.ends_with("_tests.rs"))) {
            continue;
        }
        all_violations.extend(check_file(&file, patterns));
    }
    all_violations
}

fn collect_rs_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

#[test]
fn no_println_outside_main() {
    let violations = check_non_main(&["println!", "eprintln!"]);
    assert!(
        violations.is_empty(),
        "Found println!/eprintln! outside of main.rs:\n{}",
        violations.join("\n")
    );
}

#[test]
fn no_exit_outside_main() {
    let violations = check_non_main(&["std::process::exit", "process::exit", "exit("]);
    assert!(
        violations.is_empty(),
        "Found exit outside of main.rs:\n{}",
        violations.join("\n")
    );
}
