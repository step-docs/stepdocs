#![feature(core_intrinsics)]

use crate::git::{check_git, GitRepository};
use crate::util::proc::RawOutputMessage;

mod git;
mod util;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt().init();
	let mut log = GitRepository::new(".").log()?;
	println!("{:?}", log.next().await);

	check_git().await;

	Ok(())
}
