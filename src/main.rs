use std::env;
use std::io;

// TODO: Maybe make a settings input like '!s' maybe have ability to search through settings?
#[derive(Clone, Copy)]
enum InputProvider {
    Wikipedia,
    FileSearch,
    Web,
    Ai,
    Calculator,
}

impl InputProvider {
    const VARIANTS: [InputProvider; 5] = [
        InputProvider::Wikipedia,
        InputProvider::FileSearch,
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

    let (input_provider, input) = input_parser(command);
    println!("input: {}", input);
    
    match input_provider {
        InputProvider::Web => web_search(&input),
        InputProvider::Wikipedia => wiki_search(&input),
        InputProvider::FileSearch => file_search(&input),
        InputProvider::Ai => ai_prompt(&input),
        InputProvider::Calculator => calculator_query(&input),
    }
}

fn web_search(input: &str) {
    if webbrowser::open(&format!("https://duckduckgo.com/?q={}", input)).is_err() {
        todo!("handle error")
    }
}

fn wiki_search(input: &str) {
    todo!("")
}

fn file_search(input: &str) {
    todo!("")
}

fn ai_prompt(input: &str) {
    todo!("")
}

fn calculator_query(input: &str) {
    todo!("")
}

fn get_keyword(input_provider: InputProvider) -> Vec<String> {
    match input_provider {
        InputProvider::Wikipedia => vec!["!wiki".to_string()],
        InputProvider::FileSearch => vec!["!file".to_string(), "!f".to_string()],
        InputProvider::Web => vec!["!web".to_string(), "!w".to_string()],
        InputProvider::Ai => vec!["!ai".to_string(), "!a".to_string()],
        InputProvider::Calculator => vec!["!c".to_string()],
    }
}

fn input_parser(mut input: String) -> (InputProvider, String) {
    // First check keywords
    for input_provider in InputProvider::VARIANTS {
        let keywords = &get_keyword(input_provider);
        for keyword in keywords {
            if let Some(pos) = input.find(keyword) {
                // Remove keywords
                input.replace_range(pos..pos+keyword.len(), "");
                return (input_provider, input)
            }
        }
    }
    // TODO: FIGURE OUT SOME WAY TO MAKE MATH INPUTS USE CALCULATOR
    // Maybe use regex?
    (InputProvider::Web, input)
}