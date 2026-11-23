mod queries;

use std::env;
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InputProvider {
    None,
    Wikipedia,
    FileSearch,
    ProgramSearch,
    Web,
    Calculator,
}

impl InputProvider {
    const VARIANTS: [InputProvider; 5] = [
        InputProvider::Wikipedia,
        InputProvider::FileSearch,
        InputProvider::ProgramSearch,
        InputProvider::Web,
        InputProvider::Calculator,
    ];

    fn get_keywords(self) -> &'static [&'static str] {
        match self {
            InputProvider::Wikipedia => &["!wiki"],
            InputProvider::FileSearch => &["file", "!f"],
            InputProvider::ProgramSearch => &["!p"],
            InputProvider::Web => &["!w", "!web"],
            InputProvider::Calculator => &["!c"],
            InputProvider::None => &[],
        }
    }
}

fn main() {
    // first arg is path
    let args: Vec<String> = env::args().skip(1).collect();

    // Add simply help menu
    if matches!(args.first().map(String::as_str), Some("--help" | "-h")) {
        print_help();
        return;
    }

    // Handle version
    if matches!(args.first().map(String::as_str), Some("--version" | "-v")) {
        println!("Launcher v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let command = if args.is_empty() {
        println!("Enter an input: ");
        // read user input etc
        let mut user_input = String::new();
        if let Err(error) = io::stdin().read_line(&mut user_input) {
            eprintln!("Failed to read input: {error}");
            return;
        }
        user_input
    } else {
        args.join(" ")
    };

    let command = command.trim();

    if command.is_empty() {
        eprintln!("No input provided.");
        return;
    }

    // Parse input
    let (provider, input) = input_parser(command);

    // Execute query
    execute_query(provider, input);
}

fn execute_query(provider: InputProvider, input: &str) {
    match provider {
        InputProvider::Web => queries::web_query::web_search(input),
        InputProvider::Wikipedia => queries::wiki_query::wiki_search(input),
        InputProvider::ProgramSearch => queries::program_query::program_search(input),
        InputProvider::FileSearch => queries::file_query::file_search(input),
        InputProvider::Calculator => {
            match queries::calculator_query::calculator_query(input) {
                Some(result) => println!("{result}"),
                None => eprintln!("Invalid mathematical expression."),
            }
        }
        InputProvider::None => { // User didn't specify keyword
            // Try processing query as calculator input first
            if let Some(result) = queries::calculator_query::calculator_query(input) {
                println!("{result}");
            } else { // If can't process as calculator query, default to web search
                queries::web_query::web_search(input)
            }
        }
    }
}

// Output that displays when user executes help command
fn print_help() {
    println!(
        r#"
Launcher version: {}

Usage:
    launcher.exe [QUERY]
    launcher.exe [KEYWORD] [QUERY]

Options:
    -h, --help          Display this help message
    -v, --version       Display application version

Keywords:
    !web, !w                    Search in Google or navigate to website (if url specified)
    !wiki                       Search Wikipedia
    !file, !f                   Search for file on device
    !p                          Search for program on device
    !c                          Return the result of a simple expression

Examples:
    launcher.exe google.com     Navigate to "google.com"
    launcher.exe rust           Search "rust" on google
    launcher.exe !web rust      Search "rust" on google
    launcher.exe !wiki Rust     Go to the wikipedia page for "Rust"
    launcher.exe !f rust        Search computer for files that match rust
    launcher.exe !p chrome      Search the computer for programs that match "chrome"
    launcher.exe !c 1+1         Calculate and return the result of the expression
    launcher.exe 5*(2+3)        Calculate and return the result of the expression

If no keyword is specified, the application attempts to evaluate the expression as a calculator input.
If that fails, it searches the input on the web.
"#, env!("CARGO_PKG_VERSION")
    );
}

fn input_parser(input: &str) -> (InputProvider, &str) {
    let input = input.trim();

    // Find and remove leading white space characters
    let split_pos = input
        .find(char::is_whitespace)
        .unwrap_or(input.len());

    let (prefix, rest) = input.split_at(split_pos);

    // First check keywords
    // They allow the user to force the launcher to interpret the query as a specific evaluation
    for input_provider in InputProvider::VARIANTS {
        if input_provider.get_keywords().contains(&prefix) {
            return (input_provider, rest.trim());
        }
    }
    (InputProvider::None, input)
}