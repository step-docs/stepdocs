use std::ffi::OsStr;
use std::fmt::{Debug};
use std::path::Path;
use std::process::Stdio;

use bstr::BStr;
use tokio::io;
use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};
use tracing::warn;

pub enum RawOutputMessage {
	Success(Vec<u8>),
	Error(Vec<u8>),
}

macro_rules! vec_to_string {
    ($vec:ident) => {
	    match String::from_utf8($vec) {
			Ok(s) => s,
			Err(v) => String::from_utf8_lossy(v.as_bytes()).to_string()
		}
    };
}

impl RawOutputMessage {
	pub fn into(self) -> OutputMessage {
		match self {
			RawOutputMessage::Success(msg) => {
				OutputMessage::Success(vec_to_string!(msg))
			}
			RawOutputMessage::Error(msg) => {
				OutputMessage::Success(vec_to_string!(msg))
			}
		}
	}
}

impl From<io::Result<RawOutputMessage>> for RawOutputMessage {
	fn from(value: io::Result<RawOutputMessage>) -> Self {
		match value {
			Ok(ok) => ok,
			Err(err) => {
				RawOutputMessage::Error(err.to_string().into_bytes())
			}
		}
	}
}

#[derive(Debug)]
pub enum OutputMessage {
	Success(String),
	Error(String),
}

pub fn spawn(cmd: impl AsRef<OsStr>, args: impl IntoIterator<Item=impl AsRef<OsStr>>, cwd: impl AsRef<Path>) -> io::Result<Child> {
	Command::new(cmd)
		.kill_on_drop(true)
		.args(args)
		.current_dir(cwd)
		.stdin(Stdio::null())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
}

pub async fn run_process(cmd: impl AsRef<OsStr>, args: impl IntoIterator<Item=impl AsRef<OsStr>>, cwd: impl AsRef<Path>) -> io::Result<RawOutputMessage> {
	let mut child = spawn(cmd, args, cwd)?;
	let status = child.wait().await?;

	if status.success() {
		let mut buffer = Vec::new();
		let mut out = child.stdout.take().unwrap();
		out.read_to_end(&mut buffer).await?;
		Ok(RawOutputMessage::Success(buffer))
	} else {
		let mut buffer = Vec::new();
		let mut out = child.stderr.take().unwrap();
		out.read_to_end(&mut buffer).await?;
		warn!("Child error: {}", BStr::new(&buffer));
		Ok(RawOutputMessage::Error(buffer))
	}
}