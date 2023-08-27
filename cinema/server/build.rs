use std::{env, fs};
use std::fmt::Write;

use core_build_lib::{read_dir, CORELIB_JS_PATH};


fn main() {
	println!("cargo:rerun-if-changed=../ui/assets");
	println!("cargo:rerun-if-changed=../ui/dist");

	let corelib_path = format!("\"{CORELIB_JS_PATH}\"");

	let out_dir = env::var("OUT_DIR").unwrap();

	let mut s = String::new();
	write!(s, "\
		use fire::FireBuilder;\n\
		use fire::fs::MemoryFile;\n\n\
	").unwrap();

	let mut i = 0;

	let assets = read_dir("../ui/assets", "/assets/cinema/").unwrap();
	for asset in assets {
		let name = format!("ASSET_{i}");
		asset.to_memory_file(&name, &mut s);

		i += 1;
	}

	#[cfg(not(debug_assertions))]
	{
		let mut js_name = None;
		let mut css_name = None;

		let dist = read_dir("../ui/dist", "/assets/cinema/").unwrap();
		for asset in dist {
			let name = format!("ASSET_{i}");
			if asset.uri.ends_with(".js") {
				asset.str_transform_to_memory_file(&name, |s| {
					let ns = s.replace("\"core-lib\"", &corelib_path);

					*s = ns;
				}, &mut s);
			} else {
				asset.to_memory_file(&name, &mut s);
			}

			// setup js_name, css_name
			if asset.uri.ends_with(".js") {
				js_name = asset.path.file_name().and_then(|n| n.to_str())
					.map(|s| s.to_string());
			} else if asset.uri.ends_with(".css") {
				css_name = asset.path.file_name().and_then(|n| n.to_str())
					.map(|s| s.to_string());
			}

			i += 1;
		}

		if let Some(js) = js_name {
			write!(s, "pub const JS: &str = {js:?};\n").unwrap();
		}

		if let Some(css) = css_name {
			write!(s, "pub const CSS: &str = {css:?};\n").unwrap();
		}
	}

	write!(s, "\npub fn add_routes(fire: &mut FireBuilder) {{\n").unwrap();
	for i in 0..i {
		write!(s, "\tfire.add_route(ASSET_{i});\n").unwrap();
	}
	write!(s, "}}\n").unwrap();

	fs::write(format!("{out_dir}/assets_routes.rs"), s).unwrap();
}