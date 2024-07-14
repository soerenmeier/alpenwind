use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub const CORELIB_JS_PATH: &str = "/assets/core-lib/corelib.js?v=4";

#[derive(Debug, Clone)]
pub struct File {
	pub path: PathBuf,
	pub uri: String,
}

impl File {
	// creates `const {name}: MemoryFile = _;`
	pub fn to_memory_file(&self, name: &str, s: &mut String) {
		self.raw_to_memory_file(
			name,
			&format!("include_bytes!({:?})", self.path),
			s,
		);
	}

	pub fn str_transform_to_memory_file<F>(
		&self,
		name: &str,
		f: F,
		s: &mut String,
	) where
		F: FnOnce(&mut String),
	{
		let mut st = fs::read_to_string(&self.path).unwrap();
		f(&mut st);
		self.raw_to_memory_file(name, &format!("{st:?}.as_bytes()"), s);
	}

	/// bytes needs to be for example include_bytes!()
	fn raw_to_memory_file(&self, name: &str, bytes: &str, s: &mut String) {
		write!(
			s,
			"const {name}: MemoryFile = MemoryFile::new(\n\
				\t{:?},\n\
				\t{:?},\n\
				\t{bytes}\n\
			);\n",
			self.uri,
			self.path.file_name().and_then(|a| a.to_str()).unwrap()
		)
		.unwrap();
	}
}

// uri should end with a slash /
pub fn read_dir(dir: impl AsRef<Path>, uri: &str) -> io::Result<Vec<File>> {
	assert!(uri.ends_with('/'));

	let mut list = vec![];
	parse_dirs(dir, uri, &mut list)?;
	Ok(list)
}

// uri should end with a slash /,
fn parse_dirs(
	dir: impl AsRef<Path>,
	uri: &str,
	list: &mut Vec<File>,
) -> io::Result<()> {
	for entry in fs::read_dir(dir.as_ref())? {
		let entry = entry?;
		let entry_name = entry.file_name().into_string().unwrap();
		let path = fs::canonicalize(entry.path()).unwrap();

		if path.is_dir() {
			parse_dirs(path, &format!("{uri}{entry_name}/"), list)?;
			continue;
		}

		list.push(File {
			path,
			uri: format!("{uri}{entry_name}"),
		});
	}

	Ok(())
}
