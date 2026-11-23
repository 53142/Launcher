use std::{collections::HashSet, env, io::{self, Write}, path::{Path, PathBuf}};

use rust_search::SearchBuilder;
use windows_sys::Win32::Storage::FileSystem::GetLogicalDrives;

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

// Todo: maybe be able to use regex to search files?
// can prob implement this for the program search as well?
pub fn file_search(input: &str) {
    const MAX_RESULTS: usize = 25;
    const COMMON_SEARCH_DEPTH: usize = 6;

    let input = input.trim();

    if input.is_empty() {
        eprintln!("Please enter a search query.");
        return;
    }

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
}