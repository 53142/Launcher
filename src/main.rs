mod queries;

use std::env;
use std::io;
use std::path::PathBuf;

// TODO: Maybe make a settings input like '!s' maybe have ability to search through settings?
#[derive(Clone, Copy, Debug, PartialEq)]
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

    // Add simply help menu
    if matches!(args.first().map(String::as_str), Some("--help" | "-h")) {
        print_help();
        return;
    }

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

    match input_provider {
        InputProvider::Web => queries::web_query::web_search(&input),
        InputProvider::Wikipedia => queries::wiki_query::wiki_search(&input),
        InputProvider::ProgramSearch => queries::program_query::program_search(&input),
        InputProvider::FileSearch => queries::file_query::file_search(&input),
        InputProvider::Calculator => println!("{}", queries::calculator_query::calculator_query(&input).expect("Invalid input")),
        InputProvider::None => { // User didn't specify keyword
            // Try processing query as calculator input first
            if let Some(result) = queries::calculator_query::calculator_query(&input) {
                println!("{}", result);
            } else { // If can't process as calculator query, default to web search
                queries::web_query::web_search(&input)
            }
        }
    }
}

// Output that displays when user executes help command
fn print_help() {
    println!(
        r#"
Usage:
    launcher.exe [QUERY]
    launcher.exe [KEYWORD] [QUERY]

Options:
    -h, --help          Display this help message

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
"#
    );
}

fn get_keyword(input_provider: InputProvider) -> Vec<String> {
    match input_provider {
        InputProvider::Wikipedia => vec!["!wiki".to_string()],
        InputProvider::ProgramSearch => vec!["!p".to_string()],
        InputProvider::FileSearch => vec!["!file".to_string(), "!f".to_string()],
        InputProvider::Web => vec!["!web".to_string(), "!w".to_string()],
        InputProvider::Calculator => vec!["!c".to_string()],
        InputProvider::None => panic!("Attempted to get keyword for no input provider?")
    }
}

fn input_parser(input: &str) -> (InputProvider, String) {
    // Find and remove leading white space characters
    let split_pos = input
        .find(char::is_whitespace)
        .unwrap_or(input.len());
    let (prefix, rest) = input.split_at(split_pos);

    // First check keywords
    // They allow the user to force the launcher to interpret the query as a specific evaluation
    for input_provider in InputProvider::VARIANTS {
        let keywords = get_keyword(input_provider);
        if keywords.iter().any(|keyword| keyword.as_str() == prefix) {
            return (input_provider, rest.trim().to_string());
        }
    }
    (InputProvider::None, input.to_string())
}