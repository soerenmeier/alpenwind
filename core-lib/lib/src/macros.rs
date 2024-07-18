#[macro_export]
macro_rules! migration_files {
	($($file:expr),* $(,)?) => {
		&[
			$(
				(
					$file,
					include_str!(concat!("./migrations/", $file, ".sql")),
				)
			),*
		]
	};
}
