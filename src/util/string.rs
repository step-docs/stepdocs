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
			let off = self.len() - n;
			&self[..off]
		}
	}
}

/// swap single byte character from left to right
pub fn swap_byte(str: &mut String, left: usize, right: usize) {
	_swap_byte(str, left, right);
}

fn _swap_byte(this: &mut String, left: usize, right: usize) -> Option<()> {
	let mut buf = String::with_capacity(2);
	{
		let i = this.get(left..=left)?;
		let j = this.get(right..=right)?;
		buf.push_str(i);
		buf.push_str(j);
	}
	this.replace_range(right..=right, &buf[..1]);
	this.replace_range(left..=left, &buf[1..]);

	None
}

#[cfg(test)]
mod tests {
	use crate::util::string::{StringExt, swap_byte};

	#[test]
	fn test_swap_str() {
		let mut data = "ABC".to_string();
		swap_byte(&mut data, 0, 2);
		assert_eq!("CBA".to_string(), data);
	}

	#[test]
	fn test_drop_str() {
		let content = "ABC";
		assert_eq!("BC", content.drop(1));
		assert_eq!("AB", content.drop_last(1));
	}
}