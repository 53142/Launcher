use std::{collections::HashSet, env, fs, io::{self, Write}, os::windows::ffi::OsStrExt, path::{Path, PathBuf}};

use walkdir::WalkDir;
use windows_sys::Win32::UI::Shell::ShellExecuteW;

use crate::Program;

fn get_start_menu_file_locations() -> Vec<PathBuf> {
    let mut locations = Vec::new();

    // User-account start menu apps
    if let Some(appdata) = env::var_os("APPDATA") {
        locations.push(PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs"));
    }

    // System-wide start menu apps
    if let Some(programdata) = env::var_os("PROGRAMDATA") {
        locations.push(PathBuf::from(programdata).join(r"Microsoft\Windows\Start Menu\Programs"));
    }

    locations
        .into_iter()
        .filter(|path| path.is_dir()) // Only use dirs actually on computer
        .collect()
}

fn is_program(path: &Path, include_shortcuts: bool) -> bool {
    let Some(extension) = path.extension() else {
        return false;
    };

    let extension = extension.to_string_lossy();

    if include_shortcuts {
        extension.eq_ignore_ascii_case("lnk")
            || extension.eq_ignore_ascii_case("exe")
    } else {
        extension.eq_ignore_ascii_case("exe")
            || extension.eq_ignore_ascii_case("com")
            || extension.eq_ignore_ascii_case("bat")
            || extension.eq_ignore_ascii_case("cmd")
    }
}

// Add paths if they are files and weren't already added, to the programs vector
fn add_program(path: &Path, programs: &mut Vec<Program>, seen: &mut HashSet<String>) {
    if !path.is_file() {
        return;
    }

    let Some(name) = path.file_stem() else {
        return;
    };

    let key = path.to_string_lossy().to_lowercase();

    if seen.insert(key) {
        programs.push(Program { name: name.to_string_lossy().into_owned(), path: path.to_path_buf() });
    }
}

fn get_installed_programs() -> Vec<Program> {
    let mut programs = Vec::new();
    let mut seen = HashSet::new();

    // Search start menu shortcuts
    for location in get_start_menu_file_locations() {
        for entry in WalkDir::new(location)
            // .max_depth(6)
            .into_iter()
            .filter_map(|a| a.ok())
        {
            let path = entry.path();
            if entry.file_type().is_file() && is_program(path, true) {
                add_program(path, &mut programs, &mut seen);
            }
        }
    }

    // Search system PATH
    if let Some(path_var) = env::var_os("PATH") {
        for directory in env::split_paths(&path_var) {
            let Ok(entries) = fs::read_dir(directory) else {
                continue;
            };

            for entry in entries.flatten() {
                let path = entry.path();

                if is_program(&path, false) {
                    add_program(&path, &mut programs, &mut seen);
                }
            }
        }
    }

    programs
}

fn program_match_score(name: &str, query: &str) -> Option<usize> {
    let name = name.to_lowercase();
    let query = query.to_lowercase();

    // Perfect match is best
    if name == query {
        return Some(0);
    }

    // Start is also pretty good
    if name.starts_with(&query) {
        return Some(1);
    }

    // Also if contains
    if name.contains(&query) {
        return Some(2);
    }

    // Allow multiple query words to match separately.
    let words: Vec<&str> = query.split_whitespace().collect();

    if !words.is_empty()
        && words.iter().all(|word| name.contains(word))
    {
        return Some(3);
    }

    None
}

fn open_program(path: &Path) -> Result<(), String> {
    let wide_path: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let operation: Vec<u16> = "open"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            operation.as_ptr(),
            wide_path.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1, // SW_SHOWNORMAL
        )
    };

    // Windows ShellExecute returns a value <= 32 on failure.
    if (result as isize) <= 32 {
        return Err(format!(
            "Windows Shell error code: {}",
            result as isize
        ));
    }

    Ok(())
}

pub fn program_search(input: &str) {
    let input = input.trim();

    // TODO: ADD THIS TO THE OTHER SEARCH OPTIONS???
    if input.is_empty() {
        eprintln!("Please enter a program name.");
        return;
    }

    println!("Searching for programs: {input}");

    // Find installed programs from PATH and start menu folder
    let programs = get_installed_programs();

    // Rank matching programs
    let mut matches: Vec<(usize, Program)> = programs
        .into_iter()
        .filter_map(|program| {
            // Add score to each program, and remove programs that don't match
            program_match_score(&program.name, input)
                .map(|score| (score, program))
        })
        .collect();

    matches.sort_by(|a, b| {
        // Put lower scores first
        a.0.cmp(&b.0)
            // If equal, then sort by name alphabetically
            .then_with(|| {
                a.1.name.to_lowercase()
                .cmp(&b.1.name.to_lowercase())
            })
    });

    // Only show `num_results` results at most
    let num_results = 15;

    if matches.len() > num_results {
        matches.truncate(num_results);
        println!("Found {}+ matching programs:", matches.len());
    } else {
        if matches.is_empty() {
            println!("No matching programs found.");
            return;
        }
        println!("Found {} matching programs:", matches.len());
    }

    // Display all programs
    for (index, (_, program)) in matches.iter().enumerate() {
        println!("[{}] {}", index + 1, program.name);
    }

    println!("\n[0] Cancel");

    // Get user selection
    print!("Select a program to open: ");

    // Immediately print
    let _ = io::stdout().flush();

    let mut selection = String::new();

    if io::stdin().read_line(&mut selection).is_err() {
        eprintln!("Unable to read user input.");
        return;
    }

    let selection: usize = match selection.trim().parse() {
        Ok(number) => number,
        Err(_) => {
            eprintln!("Invalid selection.");
            return;
        }
    };

    if selection == 0 {
        println!("Cancelled.");
        return;
    }

    let Some((_, program)) = matches.get(selection - 1) else {
        eprintln!("Selection out of range.");
        return;
    };

    println!("Opening {}...", program.name);

    if let Err(error) = open_program(&program.path) {
        eprintln!("Failed to open program: {error}");
    }
}