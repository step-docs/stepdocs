use std::error::Error;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

// fine for internal use
#[allow(clippy::type_complexity)]
pub trait AsyncIterator<E: Error> {
	type Item;
	fn next<'a>(&'a mut self) -> Pin<Box<dyn Future<Output=Result<Option<Self::Item>, E>> + 'a>>;
	fn collect(self) -> Pin<Box<dyn Future<Output=Result<Vec<Self::Item>, E>>>>
		where Self: 'static + Sized {
		Box::pin(async move {
			collect(self).await
		})
	}
	fn filter<F: Fn(&Self::Item) -> bool>(self, filter: F) -> AsyncIterFilter<Self::Item, F, E, Self>
		where Self: 'static + Sized {
		AsyncIterFilter::new(self, filter)
	}
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

pub struct AsyncIterFilter<T, F, E, I>
	where F: Fn(&T) -> bool,
	      E: Error,
	      I: AsyncIterator<E, Item=T> {
	iter: I,
	cond: F,
	typ: PhantomData<T>,
	err: PhantomData<E>,
}

impl<T, F, E, I> AsyncIterFilter<T, F, E, I>
	where F: Fn(&T) -> bool,
	      E: Error,
	      I: AsyncIterator<E, Item=T> {
	pub fn new(iter: I, cond: F) -> Self {
		Self {
			iter,
			cond,
			typ: Default::default(),
			err: Default::default(),
		}
	}
}

impl<E, T, I, F> AsyncIterator<E> for AsyncIterFilter<T, F, E, I>
	where
		F: Fn(&T) -> bool,
		E: Error,
		I: AsyncIterator<E, Item=T> {
	type Item = T;

	fn next<'a>(&'a mut self) -> Pin<Box<dyn Future<Output=Result<Option<Self::Item>, E>> + 'a>> {
		Box::pin(async {
			loop {
				let buf = I::next(&mut self.iter).await?;
				match buf {
					None => {
						return Ok(None);
					}
					Some(val) => {
						if (self.cond)(&val) {
							return Ok(Some(val));
						}
						drop(val);
					}
				}
			}
		})
	}
}