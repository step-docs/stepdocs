use std::ops::Range;

use tokio::io;
use tokio::io::BufReader;
use tokio::process::{Child, ChildStdout};
use tracing::warn;

use crate::read_or_none;
use crate::util::peekable_reader::PeekableLine;

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

	/// TODO: redesign 
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
		let mut diff_str = String::new();
		//self.inner.read_to_string(&mut diff_str).await?;
		/*loop {
			let len = read_async!(self, buf);
			if len == 0 { break; }
			// collect diff step
			if buf.starts_with("@@") {
				// TODO: handle @@ ... @@ <message>
				if !buf.ends_with("@@") {
					if let Some(off) = buf[2..].find("@@") {
						// message was
						// @@ ... @@ <message>
						println!("{} {}", off, buf);
					} else {
						warn!("Diff: Wrong diff offset expect `@@ N,N N,N @@` found `{}`", buf);
						return Ok(None);
					}
				}
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
			// TODO: uncomment
			break;
		}*/

		// TODO: read diff message
		Ok(
			Some(DiffInfo {
				command,
				source,
				target,
				new_file,
				index,
				diffs: Patch::parse(diff_str),
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
	pub diffs: Patch,
}

#[derive(Debug)]
pub struct DiffOffset {
	pub source_start: u64,
	pub source_lines: u64,
	pub target_start: u64,
	pub target_lines: u64,
}

#[derive(Debug)]
pub struct Patch {
	raw_diff: String,
	index: Vec<(DiffOffset, Vec<DiffIndex>)>,
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

impl Patch {
	pub fn parse(diff: String) -> Self {
		Self { raw_diff: diff, index: vec![] }
	}

	pub(crate) fn new_with_index(diff: String, index: Vec<(DiffOffset, Vec<DiffIndex>)>) -> Self {
		Self { raw_diff: diff, index }
	}

	// pub fn get_index(&self, index: usize) -> Option<&str> {
	// 	let index: Range<usize> = self.index.get(index)?.into();
	// 	Some(&self.raw_diff[index])
	// }
}

#[cfg(feature = "test_data")]
mod test_data {
	const DIFF_SINGLE: &'static str = r#"diff --git a/src/git/log_parser.rs b/src/git/log_parser.rs
index ec5f84e..c5a6ad4 100644
--- a/src/git/log_parser.rs
+++ b/src/git/log_parser.rs
@@ -1,8 +1,13 @@
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
}