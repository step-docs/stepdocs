use std::ops::Range;

use tokio::io;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::process::{Child, ChildStdout};
use tracing::warn;

use crate::read_or_none;

pub struct GitDiffParser {
	_child: Child,
	inner: BufReader<ChildStdout>,
}

impl GitDiffParser {
	pub fn new(_child: Child, stdout: ChildStdout) -> Self {
		Self {
			_child,
			inner: BufReader::new(stdout),
		}
	}

	pub async fn next_diff(&mut self) -> io::Result<Option<DiffInfo>> {
		let mut command = String::with_capacity(64);
		read_or_none!(self, command);
		let mut new_file = Option::<String>::None;
		let mut index = String::with_capacity(32);
		read_or_none!(self, index);
		if index.starts_with("new") {
			new_file.replace(index);
			index = String::with_capacity(32);
			read_or_none!(self, index);
		}
		let mut buf = String::new();


		read_or_none!(self, buf);
		let source = if buf.starts_with("---") {
			buf[4..].to_string()
		} else {
			warn!("Diff: Wrong target marker expect `---` found `{}`", buf);
			return Ok(None);
		};
		buf.clear();
		read_or_none!(self, buf);
		let target = if buf.starts_with("+++") {
			buf[4..].to_string()
		} else {
			warn!("Diff: Wrong target marker expect `+++` found `{}`", buf);
			return Ok(None);
		};

		buf.clear();
		read_or_none!(self, buf);
		if !buf.starts_with("@@") || !buf.ends_with("@@") {
			// TODO: handle @@ ... @@ <message>
			warn!("Diff: Wrong diff offset expect `@@ N,N N,N @@` found `{}`", buf);
			return Ok(None);
		}
		let mut offset = DiffOffset {
			source_start: 0,
			source_lines: 0,
			target_start: 0,
			target_lines: 0,
		};
		let mut offs = buf.splitn(4, ' ');
		offs.next();// @@
		let src = offs.next();
		let tar = offs.next();

		let mut diffs = String::new();
		if let (Some(source), Some(target)) = (src, tar) {
			let src = source
				.trim_start_matches(['+', '-'])
				.splitn(2, ',')
				.map(|it| it.parse::<i64>().unwrap_or_default().unsigned_abs())
				.collect::<Vec<_>>();
			let tar = target
				.trim_start_matches(['+', '-'])
				.splitn(2, ',')
				.map(|it| it.parse::<i64>().unwrap_or_default().unsigned_abs())
				.collect::<Vec<_>>();
			if src.len() != 2 || tar.len() != 2 {
				warn!("Diff: Wrong offset message `{}`", buf);
				return Ok(None);
			}

			offset.source_start = src[0];
			offset.source_lines = src[1];

			offset.target_start = tar[0];
			offset.target_lines = tar[1];
		} else {
			warn!("Diff: Wrong offset message `{}`", buf);
			return Ok(None);
		}
		
		// TODO: read diff message
		Ok(
			Some(DiffInfo {
				command,
				source,
				target,
				new_file,
				index,
				offset,
			})
		)
	}
}

#[derive(Debug)]
pub struct DiffInfo {
	pub command: String,
	pub source: String,
	pub target: String,
	pub new_file: Option<String>,
	pub index: String,
	pub offset: DiffOffset,
}

#[derive(Debug)]
pub struct DiffOffset {
	pub source_start: u64,
	pub source_lines: u64,
	pub target_start: u64,
	pub target_lines: u64,
}

pub struct Diffs {
	raw_diff: String,
	index: Vec<DiffIndex>,
}

#[derive(Debug, Copy, Clone)]
enum DiffType {
	Add,
	Remove,
	None,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct DiffIndex {
	typ: DiffType,
	start: usize,
	end: usize,
}

impl From<&DiffIndex> for Range<usize> {
	fn from(value: &DiffIndex) -> Self {
		value.start..value.end
	}
}


impl Diffs {
	pub(crate) fn new_with_index(diff: String, index: Vec<DiffIndex>) -> Self {
		Self { raw_diff: diff, index }
	}

	pub fn get_index(&self, index: usize) -> Option<&str> {
		let index: Range<usize> = self.index.get(index)?.into();
		Some(&self.raw_diff[index])
	}
}