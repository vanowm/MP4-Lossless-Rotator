mod config;
use crate::rotate::rotate;
use clipboard_win::{formats, get_clipboard};
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::process::exit;

mod rotate;

const FILENAME_FORMAT_ID: u32 = 15;

fn main() -> Result<(), Box<dyn Error>> {
	println!("Start");

	// Load config file with same name as executable, but .ini extension
	let exe_path = std::env::current_exe().ok();
	let ini_path = exe_path.as_ref().map(|p| {
	    let mut ini = p.to_path_buf();
	    ini.set_extension("ini");
	    ini
	});
	let config = ini_path
	    .as_ref()
	    .and_then(|p| config::Config::from_ini(p));

	// Parse args: [program, [--rotate DEG], file1, file2, ...]
	let args = env::args_os().skip(1);
	let mut rotation: Option<u32> = None;
	let mut filepaths = Vec::new();
	let mut backup: Option<bool> = None;
	for arg in args {
		let arg_str = arg.to_string_lossy();
		#[cfg(debug_assertions)]
			println!("Arg: {}", arg_str);
		if arg_str.starts_with("-") {
			if arg_str == "--backup" || arg_str == "-b" {
				backup = Some(true);
			} else {
				let index = if arg_str.starts_with("-r=") {
					3
				} else if arg_str.starts_with("--rotate=") {
					9
				} else {
					continue;
				};
				let val_str = arg_str[index..].to_string();
				match val_str.parse::<u32>() {
					Ok(0) | Ok(90) | Ok(180) | Ok(270) => rotation = Some(val_str.parse().unwrap()),
					_ => {
						eprintln!("Invalid rotation value: {}. Use 0, 90, 180, or 270.", val_str);
						exit(1);
					}
				}
			}
		} else {
			filepaths.push(arg);
		}
	}

	// Use config file defaults if not set by CLI
	let rotation = rotation.or_else(|| config.as_ref().and_then(|c| c.rotation));
	let backup = backup.or_else(|| config.as_ref().and_then(|c| c.backup)).unwrap_or(false);

	if filepaths.is_empty() {
		// Try clipboard if no files on command line
		filepaths = get_filepaths_from_clipboard().unwrap_or_default();
	}

	if filepaths.is_empty() {
		eprintln!("No file paths given, neither on the command line nor in the clipboard");
		exit(1);
	}

	for filepath in filepaths {
		let orig = PathBuf::from(&filepath);
		println!("Processing file: {}{}", orig.display(), match rotation {
			Some(deg) => format!(" (forced rotation: {}°)", deg),
			None => " (auto rotation)".to_string()
		});
		match rotate(&orig, rotation, backup) {
			Ok(_) => {},
			Err(e) => {
				eprintln!("Error processing {}: {}", orig.display(), e);
				continue;
			}
		}
	}

	println!("Finished.");
	Ok(())
}

fn get_filepaths_from_clipboard() -> Option<Vec<OsString>> {
		if let Ok(b) = get_clipboard(formats::RawData(FILENAME_FORMAT_ID)) {
			if b.len() < 20 {
				eprintln!("Clipboard data too short for CF_HDROP header!");
				return None;
			}
			let file_list_bytes = &b[20..];
			if file_list_bytes.len() % 2 != 0 {
				eprintln!("Warning: File list data length is not a multiple of 2!");
			}
			let u16s: Vec<u16> = file_list_bytes.chunks_exact(2)
				.map(|ch| u16::from_ne_bytes([ch[0], ch[1]]))
				.collect();
			let mut paths = Vec::new();
			let mut start = 0;
			for (i, &c) in u16s.iter().enumerate() {
				if c == 0 {
					if start < i {
						let path = OsString::from_wide(&u16s[start..i]);
						println!("Clipboard file path: {}", PathBuf::from(&path).display());
						paths.push(path);
					}
					start = i + 1;
				}
			}
			println!("Total clipboard file paths parsed: {}", paths.len());
			if !paths.is_empty() {
				Some(paths)
			} else {
				None
			}
	} else {
		None
	}
}
