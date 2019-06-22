extern crate slackify_markdown;

use slackify_markdown::slackdown;

use pulldown_cmark::{Parser, Options};
use std::io::{self, Read};


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
        let input = "## This is a title".to_string();
        let actual = slackify(input);
        let expected = "*This is a title*\n";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_italics() {
        let input = "I want some things to be *italics*".to_string();
        let actual = slackify(input);
        let expected = "I want some things to be _italics_\n";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bold() {
        let input = "Make this text **bold bold bold** please".to_string();
        let actual = slackify(input);
        let expected = "Make this text *bold bold bold* please\n";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_inline_code() {
        let input = "redacted redacted redacted `421` situation".to_string();
        let actual = slackify(input);
        let expected = "redacted redacted redacted `421` situation\n".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_hyperlinks() {
        let input = "This string includes a [https://hyperlink.com.au](https://hyperlink.com.au)".to_string();
        let actual = slackify(input);
        let expected = "This string includes a https://hyperlink.com.au\n".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_hyperlink_two() {
        let input = "The 44th President was [Barack Obama](https://en.wikipedia.org/wiki/Barack_Obama).".to_string();
        let actual = slackify(input);
        let expected = "The 44th President was Barack Obama.\n";
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
    - redacted redacted redacted redacted redacted redacted redacted errors.".to_string();
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
    • redacted redacted redacted redacted redacted redacted redacted errors.\n\n".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_tables() {
        let input = "The following is a table:\
| Syntax      | Description |
| ----------- | ----------- |
| Header      | Title       |
| Paragraph   | Text        |".to_string();
        let actual = slackify(input);
        let expected = "Abc";
        assert_eq!(actual, expected);
    }
}