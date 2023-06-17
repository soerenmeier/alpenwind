use std::{env, fs, io, ops};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use uuid::Uuid;


#[derive(Debug)]
pub struct TempFile {
	path: PathBuf
}

impl TempFile {
	pub fn new(ext: &str) -> io::Result<Self> {
		let mut path = env::temp_dir();
		path.push(Uuid::new_v4().simple().to_string());
		path.set_extension(ext);

		Self::create_file(&path)?;

		Ok(Self { path })
	}

	fn create_file(path: &Path) -> io::Result<()> {
		let mut builder = fs::OpenOptions::new();
		builder.write(true).create_new(true);

		#[cfg(unix)]
		builder.mode(0o600);

		builder.open(path)?;
		Ok(())
	}
}

impl AsRef<Path> for TempFile {
	fn as_ref(&self) -> &Path {
		self.path.as_path()
	}
}

impl ops::Deref for TempFile {
	type Target = PathBuf;
	fn deref(&self) -> &Self::Target {
		&self.path
	}
}

impl Drop for TempFile {
	fn drop(&mut self) {
		if !self.path.exists() {
			return;
		}

		let _ = fs::remove_file(&self);
	}
}