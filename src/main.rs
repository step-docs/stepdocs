use bstr::ByteSlice;

use crate::git::{check_git, GitRepository};
use crate::util::iter::{AsyncIterator, collect};
use crate::util::proc::{OutputMessage, RawOutputMessage, run_process};

mod git;
mod util;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt().init();
	let mut log = GitRepository::new(".").log()?;
	let res = log.filter(|_| true).collect().await?;
	println!("{res:?}");
	//println!("{:?}", log.next_log().await);
	check_git().await;

	Ok(())
}
