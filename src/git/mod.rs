use std::process::exit;

use bstr::{ByteSlice, ByteVec};

use crate::util::proc::{RawOutputMessage, run_process};

mod repo;
mod log_parser;

pub use repo::GitRepository;
pub use log_parser::GitLogParser;

pub async fn git_ver() -> Option<String> {
	let output: RawOutputMessage = run_process("git", ["-v"], ".").await.into();
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

pub async fn check_git() {
	if git_ver().await.is_none() {
		eprintln!("Did you have `git` installed?");
		exit(1);
	}
}