use bstr::ByteSlice;

use crate::git::{check_git, GitRepository};
use crate::util::iter::{AsyncIterator, collect};
use crate::util::proc::{OutputMessage, RawOutputMessage, run_process};

mod git;
mod util;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt().init();
	let mut diffs = GitRepository::new(".").show("HEAD~1")?;
	let res=diffs.next_diff().await?;
	println!("{res:?}");
	//println!("{:?}", log.next_log().await);
	check_git().await;

	Ok(())
}
