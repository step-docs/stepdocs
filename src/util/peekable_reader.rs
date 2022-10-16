use std::io;
use std::ops::Deref;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, BufReader, ReadBuf};

pub struct PeekableLine<R> {
	inner: BufReader<R>,
	next: Option<String>,
}

// impl<T> Deref for PeekableLine<T> {
// 	type Target = BufReader<T>;
// 
// 	fn deref(&self) -> &Self::Target {
// 		&self.inner
// 	}
// }

impl<T: AsyncRead + Unpin> PeekableLine<T> {
	pub fn new(inner: BufReader<T>) -> Self {
		Self {
			inner,
			next: None,
		}
	}

	pub async fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
		Ok(match self.next.take() {
			None => {
				self.inner.read_line(buf).await?
			}
			Some(val) => {
				buf.push_str(&val);
				val.len()
			}
		})
	}

	pub fn consume_peek(&mut self) {
		self.next = None;
	}

	pub async fn next_line(&mut self, capacity: usize) -> io::Result<String> {
		let mut buf = String::with_capacity(capacity);
		self.read_line(&mut buf).await?;
		Ok(buf)
	}

	pub async fn peek_line(&mut self) -> io::Result<&str> {
		if self.next.is_some() {
			Ok(self.next.as_ref().unwrap())
		} else {
			let mut buf = String::new();
			let len = self.inner.read_line(&mut buf).await?;
			if len == 0 {
				return Ok("");
			}
			self.next = Some(buf);
			Ok(self.next.as_ref().unwrap())
		}
	}
}