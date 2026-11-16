mod queries;

use std::env;
use std::io;
use std::path::PathBuf;

use regex::Regex;
use rust_search::SearchBuilder;
use rust_search::similarity_sort;
use windows_sys::Win32::Storage::FileSystem::GetLogicalDrives;

// TODO: Maybe make a settings input like '!s' maybe have ability to search through settings?
#[derive(Clone, Copy, Debug)]
enum InputProvider {
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
    println!("input provider: {:?}. input: {}", input_provider, input);

    match input_provider {
        InputProvider::Web => web_search(&input),
        InputProvider::Wikipedia => wiki_search(&input),
        InputProvider::ProgramSearch => program_search(&input),
        InputProvider::FileSearch => file_search(&input),
        InputProvider::Ai => ai_prompt(&input),
        InputProvider::Calculator => queries::calculator_query::calculator_query(&input),
    }
}

fn get_search_locations() -> Vec<PathBuf> {
    // Get bitmask of available drives
    // Each bit cooresponds to drive letter
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
            // Add device letter with :\ thingy
            locations.push(PathBuf::from(format!("{letter}:\\")));
        }
    }

    locations
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

fn program_search(input: &str) {
    println!("")
}

// Todo: maybe be able to use regex to search files?
// can prob implement this for the program search as well
fn file_search(input: &str) {
    println!("Searching for: {input}");
    let search_locations = get_search_locations();

    // Debug print search locations
    for location in &search_locations {
        println!("Location: {}", location.display());
    }

    // Configure search
    let extra_locations: Vec<PathBuf> = search_locations.iter().skip(1).cloned().collect();

    let mut builder = SearchBuilder::default()
        .location(&search_locations[0])
        .search_input(input)
        .ignore_case();

    if !extra_locations.is_empty() {
        builder = builder.more_locations(extra_locations);
    }

    // Collect results
    let mut results: Vec<String> = builder
        .build()
        .collect();

    // Sort by similarity
    similarity_sort(&mut results, input);

    println!("Found {} results", results.len());

    // Only show first 10 results
    for path in &results[..10] {
        println!("{path}");
    }
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
    (InputProvider::Web, input.to_string())
}