use std::io;
use std::path::PathBuf;

use crate::git::log_parser::GitLogParser;
use crate::RawOutputMessage;
use crate::util::proc::{run_process, spawn};

pub struct GitRepository(PathBuf);

impl GitRepository {
	pub fn new(path: impl Into<PathBuf>) -> Self {
		Self(path.into())
	}

	pub async fn init(&self) -> RawOutputMessage {
		run_process("git", ["init"], &self.0).await.into()
	}

	pub fn log(&self) -> io::Result<GitLogParser> {
		let mut child = spawn("git", ["log", "--all", "--pretty=format:%H%n%aN <%aE>%n%ad%n%f%n==END=="], &self.0)?;
		let stdout = child.stdout.take().unwrap();
		Ok(GitLogParser::new(child, stdout))
	}
}