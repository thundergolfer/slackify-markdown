//! Slack-flavored markdown (Slackdown) renderer that takes an iterator of events as input.

use std::collections::HashMap;
use std::io::{self, Write, ErrorKind};
use std::fmt::{Arguments, Write as FmtWrite};

use pulldown_cmark::{Event, CowStr};

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

    pub fn run(mut self) -> io::Result<()> {
        self.write("Hello world");
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