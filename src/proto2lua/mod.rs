use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::{init_config, MyApp};
use crate::proto2lua::parser::file_descriptor;
use crate::proto2lua::types::FileDescriptor;

mod types;
mod parser;


pub fn main(snake_case: MyApp) {
	let config = init_config(Some(snake_case));
	let client_suffix = "_Client_Handler";
	let ds_suffix = "_DS_Handler";

	if config.dir.is_none() {
		let mut result: Vec<FileDescriptor> = vec![];
		for file in &config.path {
			let path = Path::new(file.as_str());
			if path.is_file() {
				let extension = path.extension().unwrap().to_str().unwrap();
				if extension == "proto" {
					if let Ok(mut f) = File::open(path) {
						let mut buf:&mut Vec<u8> = &mut vec![];
						f.read_to_end(buf).unwrap();
						let desc = file_descriptor(buf).to_full_result();
						if desc.is_ok() {
							eprintln!("desc = {:#?}", desc);
							result.push(desc.unwrap());
						}
					}
				}
			}
		}
		// let mod_dir = Path::new(config.output.as_ref().unwrap().as_str());
		// for file_struct in &result {
		// 	if let Ok(_) = write_lua_file(mod_dir, client_suffix, ds_suffix, file_struct) {}
		// }
		//
		// if config.write_route_config {
		// 	write_lua_route_config_file(mod_dir, &result).expect("write route config failed");
		// }
	} else {
		// if let Some(dir) = &config.dir {
		// 	if let Ok(vec) = parse_dir(Path::new(dir.as_str())) {
		// 		let mod_dir = Path::new(config.output.as_ref().unwrap().as_str());
		// 		for file_struct in &vec {
		// 			if let Ok(_) = write_lua_file(mod_dir, client_suffix, ds_suffix, file_struct) {}
		// 		}
		//
		// 		if config.write_route_config {
		// 			write_lua_route_config_file(mod_dir, &vec).expect("write route config failed");
		// 		}
		// 	}
		// }
	}
}