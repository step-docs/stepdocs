use std::io;
use std::ops::Deref;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, BufReader, ReadBuf};

pub struct PeekableLine<R> {
	inner: BufReader<R>,
	buffer: String,
	has_next: bool,
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
			has_next: false,
			buffer: String::with_capacity(128),
		}
	}

	pub async fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
		Ok(if !self.has_next {
			self.inner.read_line(buf).await?
		} else {
			buf.push_str(&self.buffer);
			self.buffer.len()
		})
	}

	pub fn consume_peek(&mut self) {
		self.has_next = false;
	}

	pub async fn next_line(&mut self, capacity: usize) -> io::Result<String> {
		let mut buf = String::with_capacity(capacity);
		self.read_line(&mut buf).await?;
		Ok(buf)
	}

	pub async fn peek_line(&mut self) -> io::Result<&str> {
		if self.has_next {
			Ok(&self.buffer)
		} else {
			self.buffer.clear();
			let len = self.inner.read_line(&mut self.buffer).await?;
			if len == 0 {
				return Ok("");
			}
			Ok(&self.buffer)
		}
	}
}