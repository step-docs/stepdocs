use std::future::Future;
use std::io;
use std::io::Error;
use std::pin::Pin;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStdout};

use crate::util::iter::AsyncIterator;

pub struct GitLogParser {
	child: Child,
	inner: BufReader<ChildStdout>,
}

#[derive(Debug)]
pub struct GitLog {
	hash: String,
	author: String,
	message: String,
	date: String,
}

macro_rules! read_or_none {
    ($self:ident, $line:ident) => {
		if 0 == $self.inner.read_line(&mut $line).await? {
			return Ok(None);
		}
		if $line.ends_with(|it:char| it.is_whitespace()) {
	        let _ = $line.pop();
		}
    };
}

impl AsyncIterator<io::Error> for GitLogParser {
	type Item = GitLog;

	fn next<'a>(&'a mut self) -> Pin<Box<dyn Future<Output=Result<Option<Self::Item>, Error>> + Send + 'a>> {
		Box::pin(self.next_log())
	}
}

impl GitLogParser {
	pub fn new(child: Child, stdout: ChildStdout) -> Self {
		Self {
			child,
			inner: BufReader::new(stdout),
		}
	}

	pub async fn next_log(&mut self) -> io::Result<Option<GitLog>> {
		let mut line = String::with_capacity(64);
		while line.trim().is_empty() {
			read_or_none!(self, line);
		}

		let hash = line;

		let mut line = String::with_capacity(64);
		read_or_none!(self, line);
		let author = line;

		let mut line = String::with_capacity(64);
		read_or_none!(self, line);
		let date = line;

		let mut message = String::with_capacity(128);
		let mut line = String::with_capacity(64);
		loop {
			read_or_none!(self, line);
			if line.as_str() == "==END==" {
				break;
			}
			message.push_str(&line);
			line.clear();
		}
		Ok(
			Some(GitLog {
				hash,
				author,
				message,
				date,
			})
		)
	}
}