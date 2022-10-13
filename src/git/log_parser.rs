use std::future::Future;
use std::io;
use std::io::Error;
use std::pin::Pin;

use tokio::io::BufReader;
use tokio::process::{Child, ChildStdout};
use crate::read_or_none;

use crate::util::iter::AsyncIterator;

pub struct GitLogParser {
	_child: Child,
	inner: BufReader<ChildStdout>,
}

#[derive(Debug)]
pub struct GitLog {
	pub	hash: String,
	pub	author: String,
	pub	message: String,
	pub	date: String,
}

impl AsyncIterator<io::Error> for GitLogParser {
	type Item = GitLog;

	fn next<'a>(&'a mut self) -> Pin<Box<dyn Future<Output=Result<Option<Self::Item>, Error>> + 'a>> {
		Box::pin(self.next_log())
	}
}

impl GitLogParser {
	pub fn new(_child: Child, stdout: ChildStdout) -> Self {
		Self {
			_child,
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