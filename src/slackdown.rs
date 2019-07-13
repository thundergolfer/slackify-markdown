// Copyright 2017 Google Inc. All rights reserved.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

// Copyright 2015 Google Inc. All rights reserved.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

//! Slack-flavored markdown (Slackdown) renderer that takes an iterator of events as input.

use std::collections::HashMap;
use std::fmt::{Arguments, Write as FmtWrite};
use std::io::{self, ErrorKind, Write};

use crate::escape::escape_href;

use pulldown_cmark::Event::*;
use pulldown_cmark::{CowStr, Event, Tag};

struct SlackdownWriter<'a, I, W> {
    /// Iterator supplying events.
    iter: I,

    /// Writer to write to.
    writer: W,

    /// Whether or not the last write wrote a newline.
    end_newline: bool,

    /// Used to properly indent sub-lists
    list_indent_lvl: usize,
    /// Used to determine when converting ordered list, and
    /// correctly number each item. 0 means not converting ordered list,
    /// but instead doing unordered.
    curr_ordered_list_item_num: usize,
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
where
    W: Write,
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
where
    W: StrWrite,
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
            list_indent_lvl: 0,
            curr_ordered_list_item_num: 0,
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
        self.write("\n")
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
                    self.writer.write_str(&text)?;
                    self.end_newline = text.ends_with('\n');
                }
                Code(text) => {
                    self.write("`")?;
                    self.write(&text)?;
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
                    self.write(&name)?;
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
                    self.write(" ")
                }
            }
            Tag::Rule => Ok(()),
            Tag::Header(_level) => {
                // Slack doesn't support headers, so just make bold.
                if self.end_newline {
                    self.end_newline = false;
                    self.write("*")
                } else {
                    self.write("\n*")
                }
            }
            Tag::Table(_) | Tag::TableHead | Tag::TableRow | Tag::TableCell => {
                // TODO(Jonathon): Throw error to say this feature is unhandled
                Ok(())
            }
            Tag::BlockQuote => {
                if self.end_newline {
                    self.write(">")
                } else {
                    self.write("\n>")
                }
            }
            Tag::CodeBlock(_info) => {
                if !self.end_newline {
                    self.write_newline()?;
                }
                self.write("```")
            }
            Tag::List(Some(1)) => {
                self.list_indent_lvl += 1;
                self.curr_ordered_list_item_num += 1;
                Ok(())
            }
            Tag::List(Some(start)) => {
                self.curr_ordered_list_item_num = start;
                Ok(())
            }
            Tag::List(None) => {
                self.list_indent_lvl += 1;
                if self.end_newline {
                    self.write("")
                } else {
                    self.write("\n")
                }
            }
            Tag::Item => {
                let tabs = "    ".repeat(self.list_indent_lvl - 1);
                self.write(&tabs).unwrap();
                if self.curr_ordered_list_item_num > 0 {
                    self.write(&format!("{}. ", self.curr_ordered_list_item_num))
                } else {
                    self.write("• ")
                }
            }
            Tag::Emphasis => self.write("_"),
            Tag::Strong => self.write("*"),
            Tag::Strikethrough => self.write("~"),
            Tag::Link(_link_type, _dest, _title) => {
                // Don't write anything. We only want to write the hyperlink's text,
                // not the link itself.
                Ok(())
            }
            Tag::Image(_link_type, dest, title) => {
                self.write("<img src=\"")?;
                escape_href(&mut self.writer, &dest)?;
                self.write("\" alt=\"")?;
                self.raw_text()?;
                if !title.is_empty() {
                    self.write("\" title=\"")?;
                    self.write(&title)?;
                }
                self.write("\" />")
            }
            Tag::FootnoteDefinition(name) => {
                // TODO(Jonathon): Decide how to handle these
                if self.end_newline {
                    self.write("<div class=\"footnote-definition\" id=\"")?;
                } else {
                    self.write("\n<div class=\"footnote-definition\" id=\"")?;
                }
                self.write(&*name)?;
                self.write("\"><sup class=\"footnote-definition-label\">")?;
                let len = self.numbers.len() + 1;
                let number = *self.numbers.entry(name).or_insert(len);
                write!(&mut self.writer, "{}", number)?;
                self.write("</sup>")
            }
            Tag::HtmlBlock => Ok(()),
        }
    }

    fn end_tag(&mut self, tag: Tag) -> io::Result<()> {
        match tag {
            Tag::Paragraph => {
                self.write("")?;
            }
            Tag::Rule => (),
            Tag::Header(_level) => {
                // Slack doesn't support headers
                self.write("*\n")?;
            }
            Tag::Table(_) | Tag::TableHead | Tag::TableRow | Tag::TableCell => {}
            Tag::BlockQuote => {
                self.write("\n")?;
            }
            Tag::CodeBlock(_) => {
                self.write("```\n")?;
            }
            Tag::List(Some(_)) => {
                self.curr_ordered_list_item_num = 0;
            }
            Tag::List(None) => {
                self.list_indent_lvl -= 1;
                self.write("")?;
            }
            Tag::Item => {
                self.write("\n")?;
                if self.curr_ordered_list_item_num > 0 {
                    self.curr_ordered_list_item_num += 1;
                }
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
                self.write("\n")?;
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
                    self.write(&text)?;
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
