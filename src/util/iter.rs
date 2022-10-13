use std::error::Error;
use std::future::Future;
use std::pin::Pin;

use thiserror::Error;

// fine for internal use
#[allow(clippy::type_complexity)]
pub trait AsyncIterator<E: Error> {
	type Item;
	fn next<'a>(&'a mut self) -> Pin<Box<dyn Future<Output=Result<Option<Self::Item>, E>> + Send + 'a>>;
}

/// Attempt to collect async iterator; fail if any iterator return `Err(E)`
pub async fn collect<E, T, I: AsyncIterator<E, Item=T>>(mut iter: I) -> Result<Vec<T>, E>
	where E: Error {
	let mut vec = Vec::new();
	loop {
		let fut = iter.next();
		let res = fut.await?;
		if res.is_none() {
			break;
		}
		vec.push(res.unwrap());
	}
	Ok(vec)
}