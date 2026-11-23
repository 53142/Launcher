use regex::Regex;

// Open web
pub fn web_search(input: &str) {
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
        if webbrowser::open(&format!("https://www.google.com/search?q={input}")).is_err() {
            panic!("Unable to open web browser!")
        }
    }
}