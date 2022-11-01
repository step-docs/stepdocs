use std::fmt::{Debug, Display};
use std::future::Future;
use std::io::BufRead;
use std::ops::{Deref, Range};
use std::pin::Pin;

use tokio::io;
use tokio::io::BufReader;
use tokio::process::{Child, ChildStdout};
use tracing::warn;

use crate::read_or_none;
use crate::util::iter::AsyncIterator;
use crate::util::peekable_reader::PeekableLine;
use crate::util::string::StringExt;

pub struct GitDiffParser {
	_child: Child,
	inner: PeekableLine<ChildStdout>,
}

impl GitDiffParser {
	pub fn new(_child: Child, stdout: ChildStdout) -> Self {
		Self {
			_child,
			inner: PeekableLine::new(BufReader::new(stdout)),
		}
	}

	/// Get next patch set from git 
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
		// miminum diff = 7 lines * 32 chars[max=120]
		let mut diff_str = String::with_capacity(8 * 32);
		let mut diff_idx: Vec<(DiffOffset, Vec<PatchIndex>)> = Vec::new();
		let mut patch_idx: Vec<PatchIndex> = Vec::with_capacity(16);

		let inner = &mut self.inner;
		let mut diff_offset: DiffOffset = DiffOffset::default();
		loop {
			let _peek = inner.peek_line().await?;
			let peek = _peek.trim_end_matches(['\n']);
			// end of diff for this file
			if peek.is_empty() || peek.starts_with("diff") { break; }
			if peek.starts_with("@@") {
				if peek.ends_with("@@") {
					if !diff_offset.is_zero() {
						diff_idx.push((diff_offset, patch_idx));
						patch_idx = Vec::with_capacity(16);
					}
					diff_offset = match DiffOffset::parse(peek) {
						Some(it) => { it }
						None => {
							warn!("Illegal offset info {:?}",peek);
							return Ok(None);
						}
					};
					inner.consume_peek();
					continue;
				} else {
					let end_cursor = peek.drop(2).find("@@");
					if let Some(end) = end_cursor {
						if !diff_offset.is_zero() {
							diff_idx.push((diff_offset, patch_idx));
							patch_idx = Vec::with_capacity(16);
						}
						let marker = &peek[..end + 4];
						diff_offset = match DiffOffset::parse(marker) {
							Some(val) => { val }
							None => {
								warn!("Illegal offset info {:?}",peek);
								return Ok(None);
							}
						};
						diff_str.push_str(&peek[end + 4..]);
						inner.consume_peek();
						continue;
					} else {
						warn!("Illegal token {:?}",peek);
						return Ok(None);
					}
				}
			}

			let start = diff_str.len();
			diff_str.push_str(_peek);
			patch_idx.push(PatchIndex {
				typ: match _peek.chars().next() {
					Some(' ') => DiffType::None,
					Some('+') => DiffType::Add,
					Some('-') => DiffType::Remove,
					_ => {
						warn!("Invalid patch {:?}",_peek);
						return Ok(None);
					}
				},
				start,
				end: diff_str.len() - 1,
			});
			inner.consume_peek();
		}
		diff_idx.push((diff_offset, patch_idx));
		Ok(
			Some(DiffInfo {
				command,
				source,
				target,
				new_file,
				index,
				diffs: Patch::new_with_index(diff_str, diff_idx),
			})
		)
	}
}

impl AsyncIterator<io::Error> for GitDiffParser {
	type Item = DiffInfo;

	fn next<'a>(&'a mut self) -> Pin<Box<dyn Future<Output=Result<Option<Self::Item>, io::Error>> + 'a>> {
		Box::pin(self.next_diff())
	}
}

#[derive(Debug)]
pub struct DiffInfo {
	pub command: String,
	pub source: String,
	pub target: String,
	pub new_file: Option<String>,
	pub index: String,
	pub diffs: Patch,
}

#[derive(Debug, Default)]
pub struct DiffOffset {
	pub source_start: u64,
	pub source_lines: u64,
	pub target_start: u64,
	pub target_lines: u64,
}

impl DiffOffset {
	pub fn is_zero(&self) -> bool {
		self.source_start == 0
			&& self.source_lines == 0
			&& self.target_start == 0
			&& self.target_lines == 0
	}
}

impl DiffOffset {
	pub fn parse(token: &str) -> Option<Self> {
		let offset: DiffOffset;
		let mut offs = token.splitn(4, ' ');
		let _ = offs.next();// @@
		let src = offs.next();
		let tar = offs.next();

		if let (Some(source), Some(target)) = (src, tar) {
			let (source_start, source_lines) = Self::parse_section(source)?;
			let (target_start, target_lines) = Self::parse_section(target)?;

			offset = DiffOffset {
				source_start,
				source_lines,
				target_start,
				target_lines,
			}
		} else {
			warn!("Diff: Wrong offset message `{}`", token);
			return None;
		}
		Some(offset)
	}

	fn parse_section(token: &str) -> Option<(u64, u64)> {
		let (line, lines) = token.split_at(token.find(',')?);
		let lines = &lines[1..];
		Some(
			(
				line.trim_start_matches(['-', '+']).parse::<u64>().ok()?,
				lines.trim_start_matches(['-', '+']).parse::<u64>().ok()?
			)
		)
	}
}

#[derive(Debug)]
pub struct Patch {
	raw_diff: String,
	index: Vec<(DiffOffset, Vec<PatchIndex>)>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum DiffType {
	Add,
	Remove,
	None,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct PatchIndex {
	typ: DiffType,
	start: usize,
	end: usize,
}

impl From<&PatchIndex> for Range<usize> {
	fn from(value: &PatchIndex) -> Self {
		value.start..value.end
	}
}

impl Patch {
	pub fn parse(diff: String) -> Result<Self, String> {
		let mut index = Vec::with_capacity(16);
		let mut reader = std::io::BufReader::new(diff.as_bytes()).lines().peekable();
		let mut patch_idx: Vec<PatchIndex> = Vec::with_capacity(16);
		let mut off = 0;
		let mut diff_offset: DiffOffset = DiffOffset::default();
		while let Some(line) = reader.peek() {
			let _peek = line.as_ref().unwrap();
			let peek = _peek.trim_end_matches(['\n']);
			// end of diff for this file
			if peek.is_empty() || peek.starts_with("diff") { break; }
			if peek.starts_with("@@") {
				if peek.ends_with("@@") {
					if !diff_offset.is_zero() {
						index.push((diff_offset, patch_idx));
						patch_idx = Vec::with_capacity(16);
					}
					diff_offset = match DiffOffset::parse(peek) {
						Some(it) => {
							off += _peek.len() + 1;
							it
						}
						None => {
							warn!("Illegal offset info {:?}",peek);
							return Err(diff);
						}
					};
					reader.next();
					continue;
				} else {
					let end_cursor = peek.drop(2).find("@@");
					if let Some(end) = end_cursor {
						if !diff_offset.is_zero() {
							index.push((diff_offset, patch_idx));
							patch_idx = Vec::with_capacity(16);
						}
						let marker = &peek[..end + 4];
						diff_offset = match DiffOffset::parse(marker) {
							Some(val) => { val }
							None => {
								warn!("Illegal offset info {:?}",peek);
								return Err(diff);
							}
						};
						off += peek.len() - (end + 4);
						reader.next();
						continue;
					} else {
						warn!("Illegal token {:?}",peek);
						return Err(diff);
					}
				}
			}

			let start = off;
			off += _peek.len() + 1;
			patch_idx.push(PatchIndex {
				typ: match _peek.chars().next() {
					Some(' ') => DiffType::None,
					Some('+') => DiffType::Add,
					Some('-') => DiffType::Remove,
					_ => {
						warn!("Invalid patch {:?}",_peek);
						return Err(diff);
					}
				},
				start,
				end: off - 1,
			});
			reader.next();
		}
		index.push((diff_offset, patch_idx));
		Ok(Self { raw_diff: diff, index })
	}

	pub(crate) fn new_with_index(diff: String, index: Vec<(DiffOffset, Vec<PatchIndex>)>) -> Self {
		Self { raw_diff: diff, index }
	}

	pub fn patches(&self) -> usize {
		self.index.len()
	}

	pub fn get_patch(&self, index: usize) -> Option<PatchInfo> {
		let (offset, index) = self.index.get(index)?;
		let content_ptr = index.first().map(|it| it.start).unwrap_or_default();
		let content_end = index.last().map(|it| it.end).unwrap_or_default();
		Some(PatchInfo {
			offset,
			index,
			content_ptr,
			contents: &self.raw_diff[content_ptr..=content_end],
		})
	}

	pub fn get_index(&self, patch: usize, index: usize) -> Option<&str> {
		let (_, idx) = self.index.get(patch)?;
		let offset: Range<usize> = idx.get(index)?.into();
		Some(&self.raw_diff[offset])
	}

	pub fn normalize_patch(&mut self, patch: usize) {
		let mut swap_idx = (0, 0);
		let mut swap_line = (0, 0);
		{
			let (_, index) = if let Some(it) = self.index.get(patch) { it } else {
				return;
			};
			if index.len() > 2 {
				let first_remove = index.iter().position(|it| it.typ == DiffType::Remove);
				let last_add = index.iter().rposition(|it| it.typ == DiffType::Add);
				if let (Some(first), Some(last)) = (first_remove, last_add) {
					if let (Some(left), Some(right)) = (self.get_index(patch, first), self.get_index(patch, last)) {
						if left.drop(1) == right.drop(1) {
							println!("SWAP");
							swap_line.0 = first;
							swap_line.1 = last;
							swap_idx.0 = index.get(first).unwrap().start;
							swap_idx.1 = index.get(last).unwrap().start;
						}
					}
				}
			}
		}

		if swap_line.1 != 0 && swap_idx.1 != 0 {
			self.raw_diff.replace_range(swap_idx.0..=swap_idx.0, " ");
			self.raw_diff.replace_range(swap_idx.1..=swap_idx.1, "+");
			if let Some((_, index)) = self.index.get_mut(patch) {
				index.get_mut(swap_line.0).unwrap().typ = DiffType::None;
				index.get_mut(swap_line.1).unwrap().typ = DiffType::Add;
			};
		}
	}
}

#[derive(Debug)]
pub struct PatchInfo<'a> {
	pub offset: &'a DiffOffset,
	index: &'a [PatchIndex],
	/// position of first offset in content
	content_ptr: usize,
	/// information of this patch
	contents: &'a str,
}

impl<'a> PatchInfo<'a> {
	pub fn get_line(&self, line: usize) -> Option<&str> {
		let ptr = self.index.get(line)?;
		let content = self.content_ptr;
		let range: Range<usize> = (ptr.start - content)..(ptr.end - content);
		Some(&self.contents[range])
	}

	pub fn is_valid(&self) -> bool {
		let DiffOffset { source_lines, target_lines, .. } = self.offset;
		self.source_lines() as u64 == *source_lines
			&& self.patch_lines() as u64 == *target_lines
	}

	pub fn source_lines(&self) -> usize {
		self.index.iter().fold(0i128, |r, it| match it.typ {
			DiffType::Add => r,
			DiffType::Remove => r + 1,
			DiffType::None => r + 1,
		}) as _
	}

	pub fn output_lines(&self) -> usize {
		self.index.iter().fold(0i128, |r, it| match it.typ {
			DiffType::Add => r + 1,
			DiffType::Remove => r,
			DiffType::None => r,
		}) as _
	}

	pub fn patch_lines(&self) -> usize {
		self.index.len()
	}
}

#[cfg(test)]
mod test_data {
	use crate::git::Patch;

	const SHOULD_NORMALIZE_PATCH: &str = r#"-}
+
+ hello
+}"#;

	const PATCH_SINGLE: &'static str = r#"@@ -1,8 +1,13 @@
+use std::future::Future;
 use std::io;
+use std::io::Error;
+use std::pin::Pin;
 
 use tokio::io::{AsyncBufReadExt, BufReader};
 use tokio::process::{Child, ChildStdout};
 
+use crate::util::iter::AsyncIterator;
+
 pub struct GitLogParser {
        child: Child,
        inner: BufReader<ChildStdout>,
@@ -21,10 +26,20 @@ macro_rules! read_or_none {
                if 0 == $self.inner.read_line(&mut $line).await? {
                        return Ok(None);
                }
-           let _ = $line.pop();
+               if $line.ends_with(|it:char| it.is_whitespace()) {
+               let _ = $line.pop();
+               }
     };
 }
 
+impl AsyncIterator<io::Error> for GitLogParser {
+       type Item = GitLog;"#;

	const DIFF_MULTIPLE: &'static str = r#"diff --git a/src/git/log_parser.rs b/src/git/log_parser.rs
index c5a6ad4..8ea4982 100644
--- a/src/git/log_parser.rs
+++ b/src/git/log_parser.rs
@@ -15,10 +15,10 @@ pub struct GitLogParser {
 
 #[derive(Debug)]
 pub struct GitLog {
-       hash: String,
-       author: String,
-       message: String,
-       date: String,
+       pub     hash: String,
+       pub     author: String,
+       pub     message: String,
+       pub     date: String,
 }
 
 macro_rules! read_or_none {
@@ -35,7 +35,7 @@ macro_rules! read_or_none {
 impl AsyncIterator<io::Error> for GitLogParser {
        type Item = GitLog;
 
-       fn next<'a>(&'a mut self) -> Pin<Box<dyn Future<Output=Result<Option<Self::Item>, Error>> + Send + 'a>> {
+       fn next<'a>(&'a mut self) -> Pin<Box<dyn Future<Output=Result<Option<Self::Item>, Error>> + 'a>> {
                Box::pin(self.next_log())
        }
 }
diff --git a/src/main.rs b/src/main.rs
index 726c255..7e3af3f 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,7 +1,7 @@
 use bstr::ByteSlice;
 
 use crate::git::{check_git, GitRepository};
-use crate::util::iter::collect;
+use crate::util::iter::{AsyncIterator, collect};
 use crate::util::proc::{OutputMessage, RawOutputMessage, run_process};
 
 mod git;
@@ -11,7 +11,7 @@ mod util;
 async fn main() -> anyhow::Result<()> {
        tracing_subscriber::fmt().init();
        let mut log = GitRepository::new(".").log()?;
-       let res = collect(log).await?;
+       let res = log.filter(|_| true).collect().await?;
        println!("{res:?}");
        //println!("{:?}", log.next_log().await);
        check_git().await;
diff --git a/src/util/iter.rs b/src/util/iter.rs
index e84d72c..deb7906 100644
--- a/src/util/iter.rs
+++ b/src/util/iter.rs
@@ -1,14 +1,23 @@
 use std::error::Error;
 use std::future::Future;
+use std::marker::PhantomData;
 use std::pin::Pin;
 
-use thiserror::Error;
-
 // fine for internal use
 #[allow(clippy::type_complexity)]
 pub trait AsyncIterator<E: Error> {
        type Item;
-       fn next<'a>(&'a mut self) -> Pin<Box<dyn Future<Output=Result<Option<Self::Item>, E>> + Send + 'a>>;
+       fn next<'a>(&'a mut self) -> Pin<Box<dyn Future<Output=Result<Option<Self::Item>, E>> + 'a>>;
+       fn collect(self) -> Pin<Box<dyn Future<Output=Result<Vec<Self::Item>, E>>>>
+               where Self: 'static + Sized {
+               Box::pin(async move {
+                       collect(self).await
+               })
+       }
+       fn filter<F: Fn(&Self::Item) -> bool>(self, filter: F) -> AsyncIterFilter<Self::Item, F, E, Self>
+               where Self: 'static + Sized {
+               AsyncIterFilter::new(self, filter)
+       }
 }
 
 /// Attempt to collect async iterator; fail if any iterator return `Err(E)`
@@ -24,4 +33,55 @@ pub async fn collect<E, T, I: AsyncIterator<E, Item=T>>(mut iter: I) -> Result<V
                vec.push(res.unwrap());
        }
        Ok(vec)
+}
+
+pub struct AsyncIterFilter<T, F, E, I>
+       where F: Fn(&T) -> bool,
+             E: Error,
+             I: AsyncIterator<E, Item=T> {
+       iter: I,
+       cond: F,
+       typ: PhantomData<T>,
+       err: PhantomData<E>,
+}
+
+impl<T, F, E, I> AsyncIterFilter<T, F, E, I>
+       where F: Fn(&T) -> bool,
+             E: Error,
+             I: AsyncIterator<E, Item=T> {
+       pub fn new(iter: I, cond: F) -> Self {
+               Self {
+                       iter,
+                       cond,
+                       typ: Default::default(),
+                       err: Default::default(),
+               }
+       }
+}
+
+impl<E, T, I, F> AsyncIterator<E> for AsyncIterFilter<T, F, E, I>
+       where
+               F: Fn(&T) -> bool,
+               E: Error,
+               I: AsyncIterator<E, Item=T> {
+       type Item = T;
+
+       fn next<'a>(&'a mut self) -> Pin<Box<dyn Future<Output=Result<Option<Self::Item>, E>> + 'a>> {
+               Box::pin(async {
+                       loop {
+                               let buf = I::next(&mut self.iter).await?;
+                               match buf {
+                                       None => {
+                                               return Ok(None);
+                                       }
+                                       Some(val) => {
+                                               if (self.cond)(&val) {
+                                                       return Ok(Some(val));
+                                               }
+                                               drop(val);
+                                       }
+                               }
+                       }
+               })
+       }
 }
\ No newline at end of file
"#;

	#[test]
	fn test_patch() {
		let patch = Patch::parse(PATCH_SINGLE.to_string());
		assert!(patch.is_ok());
		let patch = patch.unwrap();
		assert_eq!(patch.patches(), 2);
		let inner = patch.get_patch(0);
		assert!(inner.is_some());
		let inner = inner.unwrap();
		assert!(inner.is_valid());
		println!("{:?}", inner);
		println!("{:#?}", patch);
	}
}