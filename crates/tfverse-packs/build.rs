//! Embeds the official packs from `packs/` so they work offline.

use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
	let packs = Path::new(&std::env::var("CARGO_MANIFEST_DIR")?).join("../../packs");
	println!("cargo::rerun-if-changed={}", packs.display());

	let mut names: Vec<PathBuf> = fs::read_dir(&packs)?
		.map(|entry| entry.map(|entry| entry.path()))
		.collect::<Result<_, _>>()?;
	names.retain(|path| path.join("pack.toml").is_file());
	names.sort();

	let mut code = String::from("pub(crate) static OFFICIAL: &[(&str, Files)] = &[\n");
	for dir in names {
		let name = dir
			.file_name()
			.and_then(|name| name.to_str())
			.ok_or("pack folder is not UTF-8")?;
		let mut files = Vec::new();
		collect(&dir, &mut files)?;
		files.sort();
		writeln!(code, "\t({name:?}, &[")?;
		for file in files {
			let relative = file
				.strip_prefix(&dir)?
				.to_str()
				.ok_or("pack file is not UTF-8")?
				.replace('\\', "/");
			let absolute = file.canonicalize()?.to_string_lossy().into_owned();
			writeln!(code, "\t\t({relative:?}, include_bytes!({absolute:?})),")?;
		}
		code.push_str("\t]),\n");
	}
	code.push_str("];\n");
	fs::write(Path::new(&std::env::var("OUT_DIR")?).join("official.rs"), code)?;
	Ok(())
}

fn collect(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
	for entry in fs::read_dir(dir)? {
		let entry = entry?;
		let kind = entry.file_type()?;
		if kind.is_dir() {
			collect(&entry.path(), files)?;
		} else if kind.is_file() && entry.file_name() != ".DS_Store" {
			println!("cargo::rerun-if-changed={}", entry.path().display());
			files.push(entry.path());
		}
	}
	Ok(())
}
