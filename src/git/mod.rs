mod repo;

use std::path::PathBuf;

use bstr::{ByteSlice, ByteVec};

use crate::util::proc::{RawOutputMessage, run_process};


pub async fn git_ver() -> Option<String> {
	let output: RawOutputMessage = run_process("git", ["-v"]).await.into();
	match output {
		RawOutputMessage::Success(ver) => {
			let mut ver = ver.splitn_str(2, "version ");
			ver.next();
			let ver_no = ver.next().unwrap();
			Some(String::from_utf8_lossy(&ver_no[..ver_no.len() - 1]).to_string())
		}
		RawOutputMessage::Error(_) => {
			None
		}
	}
}