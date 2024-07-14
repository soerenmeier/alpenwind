use std::fmt::Write;
use std::{env, fs};

use core_build_lib::{read_dir, CORELIB_JS_PATH};

#[cfg(debug_assertions)]
const ASSETS_DIR: &str = "../ui/assets";
#[cfg(not(debug_assertions))]
const ASSETS_DIR: &str = "../ui/dist/assets";

fn main() {
	println!("cargo:rerun-if-changed=../ui/assets");
	println!("cargo:rerun-if-changed=../ui/dist");
	println!("cargo:rerun-if-changed=../../core-lib/js/dist");

	let corelib_path = format!("\"{CORELIB_JS_PATH}\"");

	let out_dir = env::var("OUT_DIR").unwrap();

	let mut s = String::new();
	write!(
		s,
		"\
		use chuchi::Chuchi;\n\
		use chuchi::fs::MemoryFile;\n\n\
	"
	)
	.unwrap();

	let mut i = 0;

	let assets = read_dir(ASSETS_DIR, "/assets/").unwrap();
	for asset in assets {
		let name = format!("ASSET_{i}");
		if asset.uri.ends_with(".js") {
			asset.str_transform_to_memory_file(
				&name,
				|s| {
					let ns = s.replace("\"core-lib\"", &corelib_path);

					*s = ns;
				},
				&mut s,
			);
		} else {
			asset.to_memory_file(&name, &mut s);
		}

		i += 1;
	}

	let core_lib_assets = if cfg!(debug_assertions) {
		vec![]
	} else {
		read_dir("../../core-lib/js/dist", "/assets/core-lib/").unwrap()
	};

	for asset in core_lib_assets {
		let name = format!("ASSET_{i}");
		asset.to_memory_file(&name, &mut s);

		i += 1;
	}

	write!(s, "\npub fn add_routes(fire: &mut Chuchi) {{\n").unwrap();
	for i in 0..i {
		write!(s, "\tfire.add_route(ASSET_{i});\n").unwrap();
	}
	write!(s, "}}\n").unwrap();

	fs::write(format!("{out_dir}/assets_routes.rs"), s).unwrap();
}
