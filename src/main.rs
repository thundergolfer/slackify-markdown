extern crate slackify_markdown;

use slackify_markdown::slackdown;

use pulldown_cmark::{Event, Parser, Options, Tag};
use std::io::{self, Read};


fn get_sdtin() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn slackify(markdown_input: String) -> String {
    // Set up options and parser. Strikethroughs are not part of the CommonMark standard
    // and we therefore must enable it explicitly.
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&markdown_input, options);

    let parser = parser.map(|event| match event {
        Event::Text(text) => Event::Text(text.replace("abbr", "abbreviation").into()),
        Event::Start(Tag::Header(_)) => Event::Start(Tag::Strong),
        Event::End(Tag::Header(_)) => Event::End(Tag::Strong),
        _ => {
            println!("{:?}", event);
            return event;
        }
    });

    // Write to String buffer.
    let mut output = String::new();
    slackdown::push_slackdown(&mut output, parser);

    output
}

fn main() {
    println!("Hello, world!");

    let input = get_sdtin();
    match input {
        Err(e) => println!("error receiving input: {:?}", e),
        Ok(str) => {
            let slacked = slackify(str);
            println!("{}", slacked)
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_header_to_bold() {
        let input = "## This is a title".to_owned();
        let actual = slackify(input);
        let expected = "*This is a title*";
        assert_eq!(actual, expected);
    }
}