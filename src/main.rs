use pulldown_cmark::{Parser, Options, html};
use std::io::{self, Read};

fn get_sdtin() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn slackify(markdown_input: String) {
    // Set up options and parser. Strikethroughs are not part of the CommonMark standard
    // and we therefore must enable it explicitly.
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&markdown_input, options);

    // Write to String buffer.
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Show the output is what we expected.
    println!("{}", html_output);
}

fn main() {
    println!("Hello, world!");

    let input = get_sdtin();
    match input {
        Err(e) => println!("error receiving input: {:?}", e),
        Ok(str) => slackify(str)
    }
}
