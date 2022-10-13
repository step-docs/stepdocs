use bstr::ByteSlice;

use crate::git::{check_git, GitRepository};
use crate::util::iter::AsyncIterator;
use crate::util::proc::RawOutputMessage;

mod git;
mod util;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt().init();
	let mut diffs = GitRepository::new(".").show("f3ab7b7ef305cfa47f2cb6add43ec98f244950c9")?;
	let res = diffs.next_diff().await?;
	println!("{res:?}");
	//println!("{:?}", log.next_log().await);
	check_git().await;

	Ok(())
}
