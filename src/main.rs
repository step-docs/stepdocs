#![feature(core_intrinsics)]

use std::intrinsics::size_of;

use crate::git::git_ver;
use crate::util::proc::{OutputMessage, RawOutputMessage};

mod git;
mod util;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	println!("{}", size_of::<Vec<u8>>());
	println!("{}", size_of::<RawOutputMessage>());
	println!("Hello, {:?}!", git_ver().await);
}
