pub fn wiki_search(input: &str) {
    if webbrowser::open(&format!("https://en.wikipedia.org/w/index.php?search={input}")).is_err() {
        todo!("Unable to open Wikipedia in the web browser!")
    }
}