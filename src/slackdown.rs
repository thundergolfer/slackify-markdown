//! Slack-flavored markdown (Slackdown) renderer that takes an iterator of events as input.

use std::collections::HashMap;
use std::io::{self, Write, ErrorKind};
use std::fmt::{Arguments, Write as FmtWrite};

use crate::escape::{escape_html, escape_href};

use pulldown_cmark::{Event, CowStr, Tag};
use pulldown_cmark::Event::*;

struct SlackdownWriter<'a, I, W> {
    /// Iterator supplying events.
    iter: I,

    /// Writer to write to.
    writer: W,

    /// Whether or not the last write wrote a newline.
    end_newline: bool,

//    table_state: TableState,
//    table_alignments: Vec<Alignment>,
//    table_cell_index: usize,
    numbers: HashMap<CowStr<'a>, usize>,
}

/// This wrapper exists because we can't have both a blanket implementation
/// for all types implementing `Write` and types of the for `&mut W` where
/// `W: StrWrite`. Since we need the latter a lot, we choose to wrap
/// `Write` types.
struct WriteWrapper<W>(W);

/// Trait that allows writing string slices. This is basically an extension
/// of `std::io::Write` in order to include `String`.
pub(crate) trait StrWrite {
    fn write_str(&mut self, s: &str) -> io::Result<()>;

    fn write_fmt(&mut self, args: Arguments) -> io::Result<()>;
}

impl<W> StrWrite for WriteWrapper<W>
    where W: Write
{
    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.0.write_all(s.as_bytes())
    }

    #[inline]
    fn write_fmt(&mut self, args: Arguments) -> io::Result<()> {
        self.0.write_fmt(args)
    }
}

impl<'w> StrWrite for String {
    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.push_str(s);
        Ok(())
    }

    #[inline]
    fn write_fmt(&mut self, args: Arguments) -> io::Result<()> {
        // FIXME: translate fmt error to io error?
        FmtWrite::write_fmt(self, args).map_err(|_| ErrorKind::Other.into())
    }
}

impl<W> StrWrite for &'_ mut W
    where W: StrWrite
{
    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        (**self).write_str(s)
    }

    #[inline]
    fn write_fmt(&mut self, args: Arguments) -> io::Result<()> {
        (**self).write_fmt(args)
    }
}


impl<'a, I, W> SlackdownWriter<'a, I, W>
    where
        I: Iterator<Item = Event<'a>>,
        W: StrWrite,
{
    fn new(iter: I, writer: W) -> Self {
        Self {
            iter,
            writer,
            end_newline: true,
            numbers: HashMap::new(),
        }
    }

    /// Writes a buffer, and tracks whether or not a newline was written.
    #[inline]
    fn write(&mut self, s: &str) -> io::Result<()> {
        self.writer.write_str(s)?;

        if !s.is_empty() {
            self.end_newline = s.ends_with('\n');
        }
        Ok(())
    }

    /// Writes a new line.
    fn write_newline(&mut self) -> io::Result<()> {
        self.end_newline = true;
        self.writer.write_str("\n")
    }

    pub fn run(mut self) -> io::Result<()> {
        while let Some(event) = self.iter.next() {
            match event {
                Event::Start(tag) => {
                    self.start_tag(tag)?;
                }
                Event::End(tag) => {
                    self.end_tag(tag)?;
                }
                Text(text) => {
                    escape_html(&mut self.writer, &text)?;
                    self.end_newline = text.ends_with('\n');
                }
                Code(text) => {
                    self.write("`")?;
                    escape_html(&mut self.writer, &text)?;
                    self.write("`")?;
                }
                Html(html) | InlineHtml(html) => {
                    self.write(&html)?;
                }
                SoftBreak => {
                    self.write_newline()?;
                }
                HardBreak => {
                    self.write("<br />\n")?;
                }
                FootnoteReference(name) => {
                    let len = self.numbers.len() + 1;
                    self.write("<sup class=\"footnote-reference\"><a href=\"#")?;
                    escape_html(&mut self.writer, &name)?;
                    self.write("\">")?;
                    let number = *self.numbers.entry(name).or_insert(len);
                    write!(&mut self.writer, "{}", number)?;
                    self.write("</a></sup>")?;
                }
                TaskListMarker(true) => {
                    self.write("<input disabled=\"\" type=\"checkbox\" checked=\"\"/>\n")?;
                }
                TaskListMarker(false) => {
                    self.write("<input disabled=\"\" type=\"checkbox\"/>\n")?;
                }
            }
        }
        Ok(())
    }

    /// Writes the start of an HTML tag.
    fn start_tag(&mut self, tag: Tag<'a>) -> io::Result<()> {
        match tag {
            Tag::Paragraph => {
                if self.end_newline {
                    self.write("")
                } else {
                    self.write("\n")
                }
            }
            Tag::Rule => {
                Ok(())
            }
            Tag::Header(_level) => {
                // Slack doesn't support headers, so just make bold.
                if self.end_newline {
                    self.end_newline = false;
                    self.write("*")
                } else {
                    self.write("\n*")
                }
            }
            Tag::Table(_alignments) => {
                // TODO(Jonathon): Throw error to say this feature is unhandled
//                self.table_alignments = alignments;
//                self.write("<table>")
                Ok(())
            }
            Tag::TableHead => {
//                self.table_state = TableState::Head;
//                self.table_cell_index = 0;
//                self.write("<thead><tr>")
                Ok(())
            }
            Tag::TableRow => {
//                self.table_cell_index = 0;
//                self.write("<tr>")
                Ok(())
            }
            Tag::TableCell => {
//                match self.table_state {
//                    TableState::Head => {
//                        self.write("<th")?;
//                    }
//                    TableState::Body => {
//                        self.write("<td")?;
//                    }
//                }
//                match self.table_alignments.get(self.table_cell_index) {
//                    Some(&Alignment::Left) => {
//                        self.write(" align=\"left\">")
//                    }
//                    Some(&Alignment::Center) => {
//                        self.write(" align=\"center\">")
//                    }
//                    Some(&Alignment::Right) => {
//                        self.write(" align=\"right\">")
//                    }
//                    _ => self.write(">"),
//                }
                Ok(())
            }
            Tag::BlockQuote => {
                if self.end_newline {
                    self.write("<blockquote>\n")
                } else {
                    self.write("\n<blockquote>\n")
                }
            }
            Tag::CodeBlock(_info) => {
                if !self.end_newline {
                    self.write_newline()?;
                }
                self.write("```")
            }
            Tag::List(Some(1)) => {
                if self.end_newline {
                    self.write("<ol>\n")
                } else {
                    self.write("\n<ol>\n")
                }
            }
            Tag::List(Some(start)) => {
                if self.end_newline {
                    self.write("<ol start=\"")?;
                } else {
                    self.write("\n<ol start=\"")?;
                }
                write!(&mut self.writer, "{}", start)?;
                self.write("\">\n")
            }
            Tag::List(None) => {
                if self.end_newline {
                    self.write("\n")
                } else {
                    self.write("\n\n")
                }
            }
            Tag::Item => {
                if self.end_newline {
                    self.write("• ")
                } else {
                    self.write("\n• ")
                }
            }
            Tag::Emphasis => self.write("_"),
            Tag::Strong => self.write("*"),
            Tag::Strikethrough => self.write("~"),
            Tag::Link(_link_type, dest, _title) => {
                escape_href(&mut self.writer, &dest)?;
                Ok(())
            }
            Tag::Image(_link_type, dest, title) => {
                self.write("<img src=\"")?;
                escape_href(&mut self.writer, &dest)?;
                self.write("\" alt=\"")?;
                self.raw_text()?;
                if !title.is_empty() {
                    self.write("\" title=\"")?;
                    escape_html(&mut self.writer, &title)?;
                }
                self.write("\" />")
            }
            Tag::FootnoteDefinition(name) => {
                if self.end_newline {
                    self.write("<div class=\"footnote-definition\" id=\"")?;
                } else {
                    self.write("\n<div class=\"footnote-definition\" id=\"")?;
                }
                escape_html(&mut self.writer, &*name)?;
                self.write("\"><sup class=\"footnote-definition-label\">")?;
                let len = self.numbers.len() + 1;
                let number = *self.numbers.entry(name).or_insert(len);
                write!(&mut self.writer, "{}", number)?;
                self.write("</sup>")
            }
            Tag::HtmlBlock => Ok(())
        }
    }

    fn end_tag(&mut self, tag: Tag) -> io::Result<()> {
        match tag {
            Tag::Paragraph => {
                self.write("\n")?;
            }
            Tag::Rule => (),
            Tag::Header(_level) => {
                // Slack doesn't support headers
                self.write("*\n")?;
            }
            Tag::Table(_) => {
                // TODO(Jonaton): Raise error to say this is not supported
//                self.write("</tbody></table>\n")?;
            }
            Tag::TableHead => {
//                self.write("</tr></thead><tbody>\n")?;
//                self.table_state = TableState::Body;
            }
            Tag::TableRow => {
//                self.write("</tr>\n")?;
            }
            Tag::TableCell => {
//                match self.table_state {
//                    TableState::Head => {
//                        self.write("</th>")?;
//                    }
//                    TableState::Body => {
//                        self.write("</td>")?;
//                    }
//                }
//                self.table_cell_index += 1;
            }
            Tag::BlockQuote => {
                self.write("\n")?;
            }
            Tag::CodeBlock(_) => {
                self.write("```\n")?;
            }
            Tag::List(Some(_)) => {
                self.write("</ol>\n")?;
            }
            Tag::List(None) => {
                self.write("\n")?;
            }
            Tag::Item => {
                self.write("\n")?;
            }
            Tag::Emphasis => {
                self.write("_")?;
            }
            Tag::Strong => {
                self.write("*")?;
            }
            Tag::Strikethrough => {
                self.write("~")?;
            }
            Tag::Link(_, _, _) => {}
            Tag::Image(_, _, _) => (), // shouldn't happen, handled in start
            Tag::FootnoteDefinition(_) => {
                self.write("</div>\n")?;
            }
            Tag::HtmlBlock => {}
        }
        Ok(())
    }

    // run raw text, consuming end tag
    fn raw_text(&mut self) -> io::Result<()> {
        let mut nest = 0;
        while let Some(event) = self.iter.next() {
            match event {
                Event::Start(_) => nest += 1,
                Event::End(_) => {
                    if nest == 0 {
                        break;
                    }
                    nest -= 1;
                }
                Html(_) => (),
                InlineHtml(text) | Code(text) | Text(text) => {
                    escape_html(&mut self.writer, &text)?;
                    self.end_newline = text.ends_with('\n');
                }
                SoftBreak | HardBreak => {
                    self.write(" ")?;
                }
                FootnoteReference(name) => {
                    let len = self.numbers.len() + 1;
                    let number = *self.numbers.entry(name).or_insert(len);
                    write!(&mut self.writer, "[{}]", number)?;
                }
                TaskListMarker(true) => self.write("[x]")?,
                TaskListMarker(false) => self.write("[ ]")?,
            }
        }
        Ok(())
    }
}

pub fn push_slackdown<'a, I>(s: &mut String, iter: I)
    where
        I: Iterator<Item = Event<'a>>,
{
    SlackdownWriter::new(iter, s).run().unwrap();
}

pub fn write_slackdown<'a, I, W>(writer: W, iter: I) -> io::Result<()>
    where
        I: Iterator<Item = Event<'a>>,
        W: Write,
{
    SlackdownWriter::new(iter, WriteWrapper(writer)).run()
}