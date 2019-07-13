extern crate slackify_markdown;

use slackify_markdown::slackdown;

use pulldown_cmark::{Options, Parser};
use std::io::{self, Read};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    /// Path to a file containing markdown. Input taken from stdin if omitted.
    file: Option<PathBuf>,
}

fn get_sdtin() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn slackify(markdown_input: String) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(&markdown_input, options);

    let mut output = String::new();
    slackdown::push_slackdown(&mut output, parser);
    output
}

fn main() {
    let args = Cli::from_args();

    let input = match args.file {
        Some(path) => std::fs::read_to_string(&path),
        None => get_sdtin(),
    };

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
        let input = "## This is a title".to_string();
        let actual = slackify(input);
        let expected = "*This is a title*\n";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_italics() {
        let input = "I want some things to be *italics*".to_string();
        let actual = slackify(input);
        let expected = "I want some things to be _italics_";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bold() {
        let input = "Make this text **bold bold bold** please".to_string();
        let actual = slackify(input);
        let expected = "Make this text *bold bold bold* please";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_inline_code() {
        let input = "redacted redacted redacted `421` situation".to_string();
        let actual = slackify(input);
        let expected = "redacted redacted redacted `421` situation".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_hyperlinks() {
        let input = "This string includes a [https://hyperlink.com.au](https://hyperlink.com.au)"
            .to_string();
        let actual = slackify(input);
        let expected = "This string includes a https://hyperlink.com.au".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_hyperlink_two() {
        let input =
            "The 44th President was [Barack Obama](https://en.wikipedia.org/wiki/Barack_Obama)."
                .to_string();
        let actual = slackify(input);
        let expected = "The 44th President was Barack Obama.";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_quote_formatting() {
        let input = "The following is a quote:
> Education is a system of imposed ignorance. - N. Chomsky
The end."
            .to_string();
        let expected = "The following is a quote:
> Education is a system of imposed ignorance. - N. Chomsky
The end.
";
        let actual = slackify(input);
        assert_eq!(actual, expected);
    }

    // Tests for regression on https://github.com/thundergolfer/slackify-markdown/issues/6
    #[test]
    fn test_quote_chars_formatting() {
        let input = "- Friday was a bit disrupted by \"Permissions pain 😭\", so today I will still be ...".to_string();
        let expected = "• Friday was a bit disrupted by \"Permissions pain 😭\", so today I will still be ...\n".to_string();
        let actual = slackify(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_ordered_lists() {
        let input = "1. This is the first item
2. This is the 2nd item".to_string();
        let expected = "1. This is the first item
2. This is the 2nd item\n".to_string();
        let actual = slackify(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_list_subitem_formatting() {
        let input = "- ⌗ redacted redacted redacted redacted redacted
- 📅 Morning meeting with Tom and Jerry about A Thing
- 📅 Datalake 2.0 Planning meeting
- 📅 D.Eng Observability meeting
- Create JIRA epics for the goals that I own
- Cleaning up in AWS
    - Sub-item 1
    - Sub-item 2
- Got heads-up from redacted redacted redacted redacted redacted redacted
    - redacted redacted redacted redacted redacted redacted redacted errors."
            .to_string();
        let actual = slackify(input);
        let expected = "• ⌗ redacted redacted redacted redacted redacted
• 📅 Morning meeting with Tom and Jerry about A Thing
• 📅 Datalake 2.0 Planning meeting
• 📅 D.Eng Observability meeting
• Create JIRA epics for the goals that I own
• Cleaning up in AWS
    • Sub-item 1
    • Sub-item 2

• Got heads-up from redacted redacted redacted redacted redacted redacted
    • redacted redacted redacted redacted redacted redacted redacted errors.\n\n"
            .to_string();
        assert_eq!(actual, expected);
    }
}
