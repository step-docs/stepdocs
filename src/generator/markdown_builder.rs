/// A helper provided method to create markdown
#[derive(Default)]
pub struct MarkdownBuilder {
	inner: String,
}

struct MarkdownCloseTag<'a>(&'a mut MarkdownBuilder, &'static str);

impl<'a> Drop for MarkdownCloseTag<'a> {
	fn drop(&mut self) {
		self.0.append(self.1);
	}
}

impl MarkdownBuilder {
	/// Reserve more space to speculatively avoid frequent allocations
	pub fn reserve(&mut self, size: usize) -> &mut Self {
		self.inner.reserve(size);
		self
	}

	/// Append text to markdown
	pub fn append(&mut self, text: impl AsRef<str>) -> &mut Self {
		self.inner.push_str(text.as_ref());
		self
	}

	/// Append text with newline to markdown
	#[inline]
	pub fn appendln(&mut self, text: impl AsRef<str>) -> &mut Self {
		let text = text.as_ref();
		self.reserve(text.len() + 1);
		self.append(text)
			.newline()
	}

	/// Append newline to markdown
	#[inline]
	pub fn newline(&mut self) -> &mut Self {
		self.reserve(3)
			.append("  \n")
	}

	/// Append heading to markdown output as `{'#'*$level} `
	pub fn heading(&mut self, level: usize) -> &mut Self {
		self.reserve(level + 1);
		for _ in 0..level {
			self.append("#");
		}
		self.append(" ")
	}

	/// Append link to markdown output as `[$text]($link)`
	pub fn link(&mut self, text: impl AsRef<str>, link: impl AsRef<str>) -> &mut Self {
		let text = text.as_ref();
		let link = link.as_ref();
		self.reserve(4 + text.len() + link.len())
			.append("[")
			.append(text)
			.append("](")
			.append(link)
			.append(")")
	}

	/// Return markdown content and drop a builder
	pub fn build(self) -> String {
		self.inner
	}
}