use std::path::PathBuf;

pub struct GitRepository(PathBuf);

impl GitRepository {
	pub fn new(path: impl Into<PathBuf>) -> Self {
		Self(path.into())
	}
}