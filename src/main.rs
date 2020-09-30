use crate::rotate::rotate;
use clipboard_win::{formats, get_clipboard};
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::process::exit;

mod rotate;

const FILENAME_FORMAT_ID: u32 = 49159;

fn main() -> Result<(), Box<dyn Error>> {
	eprintln!("Start");

	let filepath = None
		.or_else(get_filepath_from_commandline)
		.or_else(get_filepath_from_clipboard);

	if filepath.is_none() {
		eprintln!("No file path given, neither on the command line nor in the clipboard");
		exit(1);
	}

	let filepath = filepath.unwrap();

	eprintln!("Processing file: {}", PathBuf::from(&filepath).display());

	{
		let fh = OpenOptions::new().read(true).write(true).open(filepath)?;
		rotate(fh)?;
		// TODO: Close the file
	}
	eprintln!("Done.");

	Ok(())
}

fn get_filepath_from_commandline() -> Option<OsString> {
	env::args_os().skip(1).next()
}

fn get_filepath_from_clipboard() -> Option<OsString> {
	if let Ok(b) = get_clipboard(formats::RawData(FILENAME_FORMAT_ID)) {
		let b: Vec<_> = b.chunks_exact(2).map(|ch| u16::from_ne_bytes([ch[0], ch[1]])).collect();
		Some(OsString::from_wide(&b[..b.len() - 1]))
	} else {
		None
	}
}
