pub trait StringExt {
	fn drop(&self, n: usize) -> &str;
	fn drop_last(&self, n: usize) -> &str;
}

impl StringExt for &str {
	fn drop(&self, n: usize) -> &str {
		if self.len() < n {
			&self[..0]
		} else {
			&self[n..]
		}
	}

	fn drop_last(&self, n: usize) -> &str {
		if self.len() < n {
			&self[..0]
		} else {
			let off = self.len() - n - 1;
			&self[..off]
		}
	}
}