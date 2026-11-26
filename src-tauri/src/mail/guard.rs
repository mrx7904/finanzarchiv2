//! The third level of read-only (#29, knowledge/decision-mail.md): a stream between the IMAP client
//! and the socket. A command reaches the socket only when its whole line was judged a reading one,
//! so a writing command fails here, whatever the library above would send. Nothing of a line is
//! ever logged, since the line of `LOGIN` carries the app password.

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// The longest command line the guard holds before judging it. A reading command is far shorter.
const LONGEST_LINE: usize = 16 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// The next line starts a command.
    Command,
    /// This many bytes belong to a literal (RFC 9051, 4.3), data rather than a command.
    Literal(usize),
    /// The rest of a command after its literal.
    Continued,
}

/// A stream that lets only reading IMAP commands through.
#[derive(Debug)]
pub struct ReadOnly<S> {
    inner: S,
    /// Bytes of the line being written, not judged yet.
    line: Vec<u8>,
    /// Judged bytes waiting for the socket.
    out: Vec<u8>,
    state: State,
}

impl<S> ReadOnly<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            line: Vec::new(),
            out: Vec::new(),
            state: State::Command,
        }
    }
}

/// Why a command was held back. The text names the command only, never its arguments.
fn refused(why: &str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, why.to_owned())
}

/// Whether a command line only reads: the commands of RFC 9051 that change nothing on the server.
/// `SELECT` is left out since it opens a folder for writing, and `FETCH` passes only with
/// `BODY.PEEK`, since `BODY[…]` and `RFC822` mark a mail as read.
pub fn reads(line: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(line) else {
        return false;
    };
    let text = text.trim_end_matches(['\r', '\n']);
    let Some((_tag, command)) = text.split_once(' ') else {
        return false;
    };
    let (mut name, mut args) = command.split_once(' ').unwrap_or((command, ""));
    let uid = name.eq_ignore_ascii_case("UID");
    if uid {
        (name, args) = args.split_once(' ').unwrap_or((args, ""));
    }
    match name.to_ascii_uppercase().as_str() {
        "CAPABILITY" | "NOOP" | "LOGOUT" | "LOGIN" | "ID" | "LIST" | "STATUS" | "EXAMINE" => !uid,
        "SEARCH" => true,
        "FETCH" => fetch_reads(args),
        _ => false,
    }
}

fn fetch_reads(args: &str) -> bool {
    let args = args.to_ascii_uppercase();
    if args.contains("BODY[") || args.contains("BINARY[") {
        return false;
    }
    args.match_indices("RFC822").all(|(at, word)| {
        let after = &args[at + word.len()..];
        after.starts_with(".SIZE") || after.starts_with(".HEADER")
    })
}

/// The length of a literal a line ends with, `{12}` or `{12+}`.
fn literal_at_end(line: &[u8]) -> Option<usize> {
    let text = std::str::from_utf8(line).ok()?;
    let text = text.trim_end_matches(['\r', '\n']).strip_suffix('}')?;
    let (_, length) = text.rsplit_once('{')?;
    length.trim_end_matches('+').parse().ok()
}

impl<S: AsyncWrite + Unpin> ReadOnly<S> {
    fn take(&mut self, buf: &[u8]) -> io::Result<()> {
        let mut rest = buf;
        while !rest.is_empty() {
            match self.state {
                State::Literal(remaining) => {
                    let taken = remaining.min(rest.len());
                    self.out.extend_from_slice(&rest[..taken]);
                    rest = &rest[taken..];
                    self.state = if taken == remaining {
                        State::Continued
                    } else {
                        State::Literal(remaining - taken)
                    };
                }
                State::Command | State::Continued => {
                    let Some(end) = rest.iter().position(|byte| *byte == b'\n') else {
                        self.line.extend_from_slice(rest);
                        if self.line.len() > LONGEST_LINE {
                            self.line.clear();
                            return Err(refused(
                                "Eine zu lange Befehlszeile wurde zurückgehalten.",
                            ));
                        }
                        return Ok(());
                    };
                    self.line.extend_from_slice(&rest[..=end]);
                    rest = &rest[end + 1..];
                    let line = std::mem::take(&mut self.line);
                    if self.state == State::Command && !reads(&line) {
                        self.state = State::Command;
                        return Err(refused("Ein schreibender Befehl wurde zurückgehalten."));
                    }
                    self.state = match literal_at_end(&line) {
                        Some(length) => State::Literal(length),
                        None => State::Command,
                    };
                    self.out.extend_from_slice(&line);
                }
            }
        }
        Ok(())
    }

    fn poll_drain(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        while !self.out.is_empty() {
            match Pin::new(&mut self.inner).poll_write(cx, &self.out) {
                Poll::Ready(Ok(0)) => return Poll::Ready(Err(io::ErrorKind::WriteZero.into())),
                Poll::Ready(Ok(written)) => {
                    self.out.drain(..written);
                }
                Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
                Poll::Pending => return Poll::Pending,
            }
        }
        Poll::Ready(Ok(()))
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for ReadOnly<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for ReadOnly<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(self.take(buf).map(|()| buf.len()))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.poll_drain(cx) {
            Poll::Ready(Ok(())) => Pin::new(&mut self.inner).poll_flush(cx),
            other => other,
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.poll_drain(cx) {
            Poll::Ready(Ok(())) => Pin::new(&mut self.inner).poll_shutdown(cx),
            other => other,
        }
    }
}
