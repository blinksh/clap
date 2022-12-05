use crate::util::color::ColorChoice;

use std::{
    fmt::{self, Display, Formatter},
    io::{self, Write},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Stream {
    Stdout,
    Stderr,
}

#[derive(Clone, Debug)]
pub(crate) struct Colorizer {
    stream: Stream,
    #[allow(unused)]
    color_when: ColorChoice,
    pieces: Vec<(String, Style)>,
}

impl Colorizer {
    #[inline(never)]
    pub(crate) fn new(stream: Stream, color_when: ColorChoice) -> Self {
        Colorizer {
            stream,
            color_when,
            pieces: vec![],
        }
    }

    #[inline(never)]
    pub(crate) fn good(&mut self, msg: impl Into<String>) {
        self.pieces.push((msg.into(), Style::Good));
    }

    #[inline(never)]
    pub(crate) fn warning(&mut self, msg: impl Into<String>) {
        self.pieces.push((msg.into(), Style::Warning));
    }

    #[inline(never)]
    pub(crate) fn error(&mut self, msg: impl Into<String>) {
        self.pieces.push((msg.into(), Style::Error));
    }

    #[inline(never)]
    #[allow(dead_code)]
    pub(crate) fn hint(&mut self, msg: impl Into<String>) {
        self.pieces.push((msg.into(), Style::Hint));
    }

    #[inline(never)]
    pub(crate) fn none(&mut self, msg: impl Into<String>) {
        self.pieces.push((msg.into(), Style::Default));
    }
}

/// Printing methods.
impl Colorizer {
    #[cfg(all(feature = "color", not(feature = "ios_system")))]
    pub(crate) fn print(&self) -> io::Result<()> {
        use termcolor::{BufferWriter, ColorChoice as DepColorChoice, ColorSpec, WriteColor};

        let color_when = match self.color_when {
            ColorChoice::Always => DepColorChoice::Always,
            ColorChoice::Auto if is_a_tty(self.stream) => DepColorChoice::Auto,
            _ => DepColorChoice::Never,
        };

        let writer = match self.stream {
            Stream::Stderr => BufferWriter::stderr(color_when),
            Stream::Stdout => BufferWriter::stdout(color_when),
        };

        let mut buffer = writer.buffer();

        for piece in &self.pieces {
            let mut color = ColorSpec::new();
            match piece.1 {
                Style::Good => {
                    color.set_fg(Some(termcolor::Color::Green));
                }
                Style::Warning => {
                    color.set_fg(Some(termcolor::Color::Yellow));
                }
                Style::Error => {
                    color.set_fg(Some(termcolor::Color::Red));
                    color.set_bold(true);
                }
                Style::Hint => {
                    color.set_dimmed(true);
                }
                Style::Default => {}
            }

            buffer.set_color(&color)?;
            buffer.write_all(piece.0.as_bytes())?;
            buffer.reset()?;
        }

        writer.print(&buffer)
    }

    #[cfg(all(feature = "color", feature = "ios_system"))]
    pub(crate) fn print(&self) -> io::Result<()> {
        use termcolor::{
            BufferedStandardStream, ColorChoice as DepColorChoice, ColorSpec, WriteColor,
        };
        #[link(name = "ios_system", kind = "framework")]
        extern "C" {
            fn ios_stdout() -> *mut libc::FILE;
            fn ios_stderr() -> *mut libc::FILE;
        }

        let color_when = match self.color_when {
            ColorChoice::Always => DepColorChoice::Always,
            ColorChoice::Auto if is_a_tty(self.stream) => DepColorChoice::AlwaysAnsi,
            _ => DepColorChoice::Never,
        };

        let fd = match self.stream {
            Stream::Stderr => unsafe { libc::fileno(ios_stdout()) },
            Stream::Stdout => unsafe { libc::fileno(ios_stderr()) },
        };

        let mut writer = BufferedStandardStream::file(fd, color_when);

        for piece in &self.pieces {
            let mut color = ColorSpec::new();
            match piece.1 {
                Style::Good => {
                    color.set_fg(Some(termcolor::Color::Green));
                }
                Style::Warning => {
                    color.set_fg(Some(termcolor::Color::Yellow));
                }
                Style::Error => {
                    color.set_fg(Some(termcolor::Color::Red));
                    color.set_bold(true);
                }
                Style::Hint => {
                    color.set_dimmed(true);
                }
                Style::Default => {}
            }

            writer.set_color(&color)?;
            writer.write_all(piece.0.as_bytes())?;
            writer.reset()?;
        }

        // writer.flash();
        Ok(())
    }

    #[cfg(all(not(feature = "color"), not(feature = "ios_system")))]
    pub(crate) fn print(&self) -> io::Result<()> {
        // [e]println can't be used here because it panics
        // if something went wrong. We don't want that.
        match self.stream {
            Stream::Stdout => {
                let stdout = std::io::stdout();
                let mut stdout = stdout.lock();
                write!(stdout, "{}", self)
            }
            Stream::Stderr => {
                let stderr = std::io::stderr();
                let mut stderr = stderr.lock();
                write!(stderr, "{}", self)
            }
        }
    }
    #[cfg(all(not(feature = "color"), feature = "ios_system"))]
    pub(crate) fn print(&self) -> io::Result<()> {
        use std::fs::File;
        use std::os::unix::io::FromRawFd;
        #[link(name = "ios_system", kind = "framework")]
        extern "C" {
            fn ios_stdout() -> *mut libc::FILE;
            fn ios_stderr() -> *mut libc::FILE;
        }
        // [e]println can't be used here because it panics
        // if something went wrong. We don't want that.
        let fd = match self.stream {
            Stream::Stdout => unsafe { libc::fileno(ios_stdout()) },
            Stream::Stderr => unsafe { libc::fileno(ios_stderr()) },
        };
        if fd > 0 {
            unsafe {
                let mut file = File::from_raw_fd(fd);
                write!(file, "{}", self)
            }
        } else {
            Ok(())
        }
    }
}

/// Color-unaware printing. Never uses coloring.
impl Display for Colorizer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for piece in &self.pieces {
            Display::fmt(&piece.0, f)?;
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Style {
    Good,
    Warning,
    Error,
    Hint,
    Default,
}

impl Default for Style {
    fn default() -> Self {
        Self::Default
    }
}

#[cfg(all(feature = "color", not(feature = "ios_system")))]
fn is_a_tty(stream: Stream) -> bool {
    let stream = match stream {
        Stream::Stdout => atty::Stream::Stdout,
        Stream::Stderr => atty::Stream::Stderr,
    };

    atty::is(stream)
}

#[cfg(all(feature = "color", feature = "ios_system"))]
fn is_a_tty(stream: Stream) -> bool {
    #[link(name = "ios_system", kind = "framework")]
    extern "C" {
        fn ios_stdout() -> *mut libc::FILE;
        fn ios_stderr() -> *mut libc::FILE;
        fn ios_isatty(fd: i32) -> i32;
    }
    let fd = match stream {
        Stream::Stdout => unsafe { libc::fileno(ios_stdout()) },
        Stream::Stderr => unsafe { libc::fileno(ios_stderr()) },
    };

    if fd == 0 {
        false
    } else {
        unsafe { ios_isatty(fd) == 1 }
    }
}
