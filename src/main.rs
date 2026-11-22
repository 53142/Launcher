mod queries;

use std::collections::HashSet;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::path::PathBuf;

use regex::Regex;
use rust_search::SearchBuilder;
use walkdir::WalkDir;
use windows_sys::Win32::Storage::FileSystem::GetLogicalDrives;
use windows_sys::Win32::UI::Shell::ShellExecuteW;

// TODO: Maybe make a settings input like '!s' maybe have ability to search through settings?
#[derive(Clone, Copy, Debug, PartialEq)]
enum InputProvider {
    None,
    Wikipedia,
    FileSearch,
    ProgramSearch,
    Web,
    Ai,
    Calculator,
}

impl InputProvider {
    const VARIANTS: [InputProvider; 6] = [
        InputProvider::Wikipedia,
        InputProvider::FileSearch,
        InputProvider::ProgramSearch,
        InputProvider::Web,
        InputProvider::Ai,
        InputProvider::Calculator,
    ];
}

// For storing program name and path
#[derive(Debug, Clone)]
struct Program {
    name: String,
    path: PathBuf,
}

fn main() {
    // first arg is path
    let args: Vec<String> = env::args().skip(1).collect();

    let command = if args.len() == 0 {
        println!("Enter an input: ");
        // read user input etc
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).expect("Unable to read user input!");
        user_input
    } else {
        args.join(" ")
    };

    let (input_provider, input) = input_parser(command.trim());
    // println!("input provider: {:?}; input: {}", input_provider, input);

    match input_provider {
        InputProvider::Web => web_search(&input),
        InputProvider::Wikipedia => wiki_search(&input),
        InputProvider::ProgramSearch => program_search(&input),
        InputProvider::FileSearch => file_search(&input),
        InputProvider::Ai => ai_prompt(&input),
        InputProvider::Calculator => println!("{}", queries::calculator_query::calculator_query(&input).expect("Invalid input")),
        InputProvider::None => { // User didn't specify keyword
            // Try processing query as calculator input first
            if let Some(result) = queries::calculator_query::calculator_query(&input) {
                println!("{}", result);
            } else {
                web_search(&input)
            }
        }
    }
}

// Return common directories to search first
fn get_common_search_locations() -> Vec<PathBuf> {
    let Some(user_profile) = env::var_os("USERPROFILE") else {
        eprintln!("Unable to locate user profile.");
        return Vec::new();
    };

    let user_profile = PathBuf::from(user_profile);

    let directories = [
        "Desktop",
        "Downloads",
        "Documents",
        "Pictures",
        "Videos",
        "Music",
    ];

    directories
        .iter()
        .map(|dir| user_profile.join(dir))
        .filter(|path| path.is_dir())
        .collect()
}

// Return all available drives to search after common search locations to ensure all results are searched
fn get_drive_search_locations() -> Vec<PathBuf> {
    let drive_mask = unsafe {
        GetLogicalDrives()
    };

    if drive_mask == 0 {
        eprintln!("Failed to enumerate drives.");
        return Vec::new();
    }

    let mut locations = Vec::new();

    for bit in 0u32..26 {
        if drive_mask & (1u32 << bit) != 0 {
            let letter = (b'A' + bit as u8) as char;

            locations.push(PathBuf::from(format!("{letter}:\\")));
        }
    }

    locations
}

// Reusable search function
fn search_location(
    location: &Path,
    input: &str,
    depth: Option<usize>,
    include_hidden: bool,
    seen: &mut HashSet<String>,
    total_results: &mut usize,
    max_results: usize,
) -> bool {
    // println!("\nSearching: {}", location.display());
    let _ = io::stdout().flush();

    let mut builder = SearchBuilder::default()
        .location(location)
        .search_input(input)
        .ignore_case();

    // Apply depth limit if specified
    if let Some(depth) = depth {
        builder = builder.depth(depth);
    }

    // Include hidden files and directories when requested
    if include_hidden {
        builder = builder.hidden();
    }

    // Search results arrive through the iterator
    for path in builder.build() {
        // Windows paths are normally case-insensitive
        let key = path.to_lowercase();

        // Skip duplicate results
        if !seen.insert(key) {
            continue;
        }

        *total_results += 1;

        println!("{total_results}. {path}");

        // Flush output to immediately display result
        let _ = io::stdout().flush();

        // Stop when the global result limit is reached
        if *total_results >= max_results {
            return false;
        }
    }

    true
}

// Open web
fn web_search(input: &str) {
    // Check if input is url
    // This should work for most valid urls
    let url_regex = Regex::new(r"^(https?:\/\/)?(www\.)?.+\..+$").unwrap();
    if url_regex.is_match(input) {
        // If doesn't start with https:// or http://, we need to add it
        // to make sure web browser opens website instead of thinking it is file
        let url = &if input.starts_with("http://") || input.starts_with("https://") {
            input.to_string()
        } else {
            format!("https://{input}")
        };

        if webbrowser::open(url).is_err() {
            panic!("Unable to open web browser!")
        }
    } else {
        // Input not url so default to search with search engine
        if webbrowser::open(&format!("https://duckduckgo.com/?q={input}")).is_err() {
            panic!("Unable to open web browser!")
        }
    }
}

fn wiki_search(input: &str) {
    if webbrowser::open(&format!("https://en.wikipedia.org/w/index.php?search={input}")).is_err() {
        todo!("handle error")
    }
}

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

fn program_search(input: &str) {
    let input = input.trim();

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

    // Only show 10 results at most
    if matches.len() > 10 {
        matches.truncate(10);
        println!("Found {}+ matching programs.", matches.len());
    } else {
        if matches.is_empty() {
            println!("No matching programs found.");
            return;
        }
        println!("Found {} matching programs.", matches.len());
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

// Todo: maybe be able to use regex to search files?
// can prob implement this for the program search as well
fn file_search(input: &str) {
    const MAX_RESULTS: usize = 100;
    const COMMON_SEARCH_DEPTH: usize = 6;

    let input = input.trim();

    if input.is_empty() {
        eprintln!("Please enter a search query.");
        return;
    }

    let start = std::time::Instant::now();

    println!("Searching for: {input}");

    let common_locations = get_common_search_locations();
    let drive_locations = get_drive_search_locations();

    if common_locations.is_empty() && drive_locations.is_empty() {
        eprintln!("No search locations available.");
        return;
    }

    let mut total_results = 0;
    let mut seen = HashSet::new();

    // First search common user dirs with a lower depth
    for location in &common_locations {
        let should_continue = search_location(
            location,
            input,
            Some(COMMON_SEARCH_DEPTH),
            false,
            &mut seen,
            &mut total_results,
            MAX_RESULTS,
        );

        if !should_continue {
            break;
        }
    }

    // Completely search all drives next
    if total_results < MAX_RESULTS {
        for location in &drive_locations {
            let should_continue = search_location(
                location,
                input,
                None, // No depth limit
                true, // Include hidden files
                &mut seen,
                &mut total_results,
                MAX_RESULTS,
            );

            if !should_continue {
                break;
            }
        }
    }

    println!("\nSearch Complete");
    if total_results != 0 {
        println!("Found {total_results} (or more) unique results.");
    } else {
        println!("Found no results.");
    }

    // println!("Elapsed time: {:.2?}", start.elapsed());
}

fn ai_prompt(input: &str) {
    // TODO: IMPLEMENT
}

fn get_keyword(input_provider: InputProvider) -> Vec<String> {
    match input_provider {
        InputProvider::Wikipedia => vec!["!wiki".to_string()],
        InputProvider::ProgramSearch => vec!["!p".to_string()],
        InputProvider::FileSearch => vec!["!file".to_string(), "!f".to_string()],
        InputProvider::Web => vec!["!web".to_string(), "!w".to_string()],
        InputProvider::Ai => vec!["!ai".to_string(), "!a".to_string()],
        InputProvider::Calculator => vec!["!c".to_string()],
        InputProvider::None => panic!("Attempted to get keyword for None InputProvider")
    }
}

fn input_parser(input: &str) -> (InputProvider, String) {
    // Find and remove leading white space characters
    let split_pos = input
        .find(char::is_whitespace)
        .unwrap_or(input.len());
    let (prefix, rest) = input.split_at(split_pos);

    // First check keywords
    for input_provider in InputProvider::VARIANTS {
        let keywords = get_keyword(input_provider);
        if keywords.iter().any(|keyword| keyword.as_str() == prefix) {
            return (input_provider, rest.trim().to_string());
        }
    }
    // TODO: prob try to run all of the searches if not keywords match
    // maybe except if it is math or something
    (InputProvider::None, input.to_string())
}