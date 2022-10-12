use std::ffi::OsStr;
use std::fmt::{Debug, Display, Formatter, Write};
use std::process::Stdio;
use std::string::FromUtf8Error;

use tokio::io;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

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

pub async fn run_process(cmd: impl AsRef<OsStr>, args: impl IntoIterator<Item=impl AsRef<OsStr>>) -> io::Result<RawOutputMessage> {
	let mut child = Command::new(cmd)
		.args(args)
		.stdin(Stdio::null())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;
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
		Ok(RawOutputMessage::Error(buffer))
	}
}