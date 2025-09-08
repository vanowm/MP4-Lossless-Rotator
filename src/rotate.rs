use arrayvec::ArrayString;
use std::convert::TryInto;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

struct RotationMatrix;
impl RotationMatrix {
	const NO_ROTATION: &'static [u8] = &[
		0, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0, 0, 0,
	];
	const ROTATION_90: &'static [u8] = &[
		0, 0, 0, 0, 0, 0x01, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0,
		0, 0,
	];
	const ROTATION_180: &'static [u8] = &[
		0xFF, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40,
		0, 0, 0,
	];
	const ROTATION_270: &'static [u8] = &[
		0, 0, 0, 0, 0xFF, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0,
		0, 0,
	];
}

const ROTATION_MATRIX_ARRAY: [&'static [u8]; 4] = [
	RotationMatrix::NO_ROTATION,
	RotationMatrix::ROTATION_90,
	RotationMatrix::ROTATION_180,
	RotationMatrix::ROTATION_270
];

use std::path::Path;
pub fn rotate(path: &Path, rotation: Option<u32>, backup: bool) -> Result<(), Box<dyn Error>> {
	let mut fh = File::options().read(true).write(true).open(path)?;
	let top_level_atoms = list_atoms(&mut fh, None)?;

	if top_level_atoms.len() == 0 || top_level_atoms[0].atom_type.as_str() != "ftyp" {
		return Err(Box::from("ftyp box not found"));
	}

	let moov_atom = find_atom(top_level_atoms, "moov")?;

	println!("Found moov box at {}", moov_atom.start);

	let trak_atoms: Vec<_> = list_atoms(&mut fh, Some(moov_atom))?
		.into_iter()
		.filter(|a| a.atom_type.as_str() == "trak")
		.collect();

	let mut video_tracks = Vec::with_capacity(1);

	for trak_atom in trak_atoms {
		println!("Found trak box at {}", trak_atom.start);
		eprint!("Walking trak -> ");
		eprint!("mdia -> ");
		let mdia_atom = find_atom(list_atoms(&mut fh, Some(trak_atom.clone()))?, "mdia")?;
		print!("hdlr");
		let hdlr_atom = find_atom(list_atoms(&mut fh, Some(mdia_atom))?, "hdlr")?;
		println!();

		// https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap2/qtff2.html#//apple_ref/doc/uid/TP40000939-CH204-25621
		let component_subtype = read_atom_data(&mut fh, hdlr_atom, 16, 4)?;
		println!("Track type: {}", String::from_utf8_lossy(&component_subtype));

		if component_subtype == b"vide" {
			video_tracks.push(trak_atom);
		}
	}

	if video_tracks.len() == 0 {
		return Err(Box::from("No video track found"));
	}
	if video_tracks.len() > 1 {
		return Err(Box::from("Multiple video tracks found"));
	}
	let video_track = video_tracks.into_iter().next().unwrap();

	println!("Found video track");

	let tkhd_atom = find_atom(list_atoms(&mut fh, Some(video_track))?, "tkhd")?;

	// https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap2/qtff2.html#//apple_ref/doc/uid/TP40000939-CH204-25550
	let matrix_structure = read_atom_data(&mut fh, tkhd_atom.clone(), 48, 36)?;
	print!("Rotation matrix found: ");
	let index = rotation_matrix_index(&matrix_structure.as_slice())?;
	print!("{}° => ", index * 90);
	let current_matrix = ROTATION_MATRIX_ARRAY[index];
	let next_matrix = match rotation {
		None => ROTATION_MATRIX_ARRAY[(index + 1) % 4],
		Some(rotation) => ROTATION_MATRIX_ARRAY[(rotation / 90) as usize],
	};
	if current_matrix == next_matrix {
		println!("no change needed");
		return Ok(());
	} else {
		println!("changing to: {}", rotation_matrix_to_str(next_matrix)?);
	}
	use std::fs;
	// Backup before writing if requested
	let backup_path = if backup {
		let ext = &path.extension().and_then(std::ffi::OsStr::to_str).unwrap_or("");
		let stem = &path.file_stem().and_then(std::ffi::OsStr::to_str).unwrap_or("");
		let meta = std::fs::metadata(&path)?;
		use chrono::TimeZone;
		let modified_time = meta.modified().ok()
			.and_then(|mtime| mtime.duration_since(std::time::UNIX_EPOCH).ok())
			.map(|d| d.as_secs() as i64)
			.unwrap_or(0);
		let datetime = chrono::Local.timestamp_opt(modified_time, 0).unwrap();
		let datetime_str = datetime.format("%Y%m%d_%H%M%S");
		let backup_name = if ext.is_empty() {
			format!("{}_{}.mp4", stem, datetime_str)
		} else {
			format!("{}_{}.{}", stem, datetime_str, ext)
		};
		Some(&path.with_file_name(backup_name))
	} else {
		None
	};
	if let Some(backup_path) = backup_path {
		println!("Backing up original file to {}", backup_path.display());
		// doing this nonsense so the backup file gets the same modified/created time as the original
		let mut backup_path_temp = backup_path.to_path_buf();
		backup_path_temp.set_extension(
			match backup_path.extension().and_then(|e| e.to_str()) {
				Some(ext) => format!("{}.tmp", ext),
				None => "tmp".to_string(),
			}
		);
		fs::rename(path, &backup_path_temp)?;
		fs::copy(&backup_path_temp, path)?;
		fs::rename(&backup_path_temp, backup_path)?;

		fh = File::options().read(true).write(true).open(path)?;
	}
	print!("Writing new rotation matrix: ");
	if let Err(e) = fh.seek(SeekFrom::Start(tkhd_atom.start + 48)) {
		eprintln!("error seeking to rotation matrix: {}", e);
		return Err(Box::from(e));
	}
	if let Err(e) = fh.write_all(next_matrix) {
		eprintln!("error writing rotation matrix: {}", e);
		return Err(Box::from(e));
	}
	println!("success");
	Ok(())
}

fn rotation_matrix_to_str(matrix: &[u8]) -> Result<String, Box<dyn Error>> {
	if let Some(idx) = rotation_matrix_index(matrix).ok() {
		return Ok(format!("{}°", idx * 90));
	}
	else {
		return Err(Box::from("n/a"));
	}
}
fn rotation_matrix_index(matrix: &[u8]) -> Result<usize, Box<dyn Error>> {
	if let Some(idx) = ROTATION_MATRIX_ARRAY.iter().position(|&x| x == matrix) {
		return Ok(idx);
	}
	else {
		return Err(Box::from("n/a"));
	}
}

fn read_atom_data(file: &mut File, atom: Atom, offset: u64, length: u64) -> Result<Vec<u8>, Box<dyn Error>> {
	file.seek(SeekFrom::Start(atom.start + offset))?;
	let mut buf = Vec::with_capacity(length as usize);
	file.take(length).read_to_end(&mut buf)?;
	if buf.len() < length as usize {
		return Err(Box::from("Premature EOF"));
	}
	Ok(buf)
}

fn find_atom(atoms: Vec<Atom>, atom_type: &str) -> Result<Atom, Box<dyn Error>> {
	let mut found = Vec::with_capacity(1);
	found.extend(atoms.into_iter().filter(|a| a.atom_type.as_str() == atom_type));
	if found.len() == 0 {
		return Err(Box::from(format!("No {} box found", atom_type)));
	}
	if found.len() > 1 {
		return Err(Box::from(format!("Multiple {} boxes found", atom_type)));
	}
	Ok(found.into_iter().next().unwrap())
}

fn list_atoms(file: &mut File, in_atom: Option<Atom>) -> Result<Vec<Atom>, Box<dyn Error>> {
	let mut buf = [0; 8];
	let mut atoms = Vec::new();

	let (mut pos, end) = match in_atom {
		None => (0, file.seek(SeekFrom::End(0))?),
		Some(Atom { start, size, .. }) => (start + 8, start + size),
	};

	while pos < end {
		file.seek(SeekFrom::Start(pos))?;
		file.read_exact(&mut buf)?;
		let mut atom_size = u32::from_be_bytes(buf[..4].try_into()?) as u64;
		let atom_type: ArrayString<[u8; 4]> = ArrayString::from_byte_string(buf[4..].try_into()?)?;

		// mdat atoms whose size doesn't fit in the 32-bit length field have their length field set
		// to 1 and start with a 64-bit extended length field.
		if atom_type.as_str() == "mdat" && atom_size == 1 {
			file.read_exact(&mut buf)?;
			atom_size = u64::from_be_bytes(buf);
		}

		if atom_size < 8 {
			println!(); // Because we might be in the "Walking ..." part
			return Err(Box::from(format!("Invalid box size {} < 8", atom_size)));
		}
		atoms.push(Atom {
			start: pos,
			size: atom_size,
			atom_type,
		});
		pos += atom_size as u64;
	}

	Ok(atoms)
}

#[derive(Debug, Clone)]
pub struct Atom {
	start: u64,
	size: u64,
	atom_type: ArrayString<[u8; 4]>,
}
