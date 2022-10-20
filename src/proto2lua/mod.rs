use pb_rs::types::FileDescriptor;
use std::path::{Path, PathBuf};

#[test]
pub fn main(){

	let v:Vec<PathBuf> = vec![];
	match FileDescriptor::read_proto(Path::new("./src/example.proto"), &v) {
		Ok(FileDescriptor) => {
			eprintln!("FileDescriptor = {:#?}", FileDescriptor);
			// println!("{:?}", FileDescriptor);
		}
		Err(_) => {
			println!("-----");
		}
	}
}