use bstr::ByteSlice;

use crate::git::{check_git, GitRepository, Patch};
use crate::util::proc::RawOutputMessage;

mod git;
mod util;
mod generator;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt().init();
	let mut diffs = GitRepository::new(".").show("f3ab7b7ef305cfa47f2cb6add43ec98f244950c9")?;
	let res = diffs.next_diff().await?;
	println!("{res:#?}");

	match res {
		None => {}
		Some(it) => {
			println!("{:?}", it.diffs.get_patch(0));
		}
	}
	
	let  patch = Patch::parse(r#"@@ -1,1 +3,3 @@
-}
+
+ hello
+}"#.to_string());
	println!("{:?}", patch);
	//println!("{:?}", log.next_log().await);
	check_git().await;

	Ok(())
}
