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

pub fn rotate(mut fh: File) -> Result<(), Box<dyn Error>> {
	let top_level_atoms = list_atoms(&mut fh, None)?;

	if top_level_atoms.len() == 0 || top_level_atoms[0].atom_type.as_str() != "ftyp" {
		return Err(Box::from("ftyp box not found"));
	}

	let moov_atom = find_atom(top_level_atoms, "moov")?;

	eprintln!("Found moov box at {}", moov_atom.start);

	let trak_atoms: Vec<_> = list_atoms(&mut fh, Some(moov_atom))?
		.into_iter()
		.filter(|a| a.atom_type.as_str() == "trak")
		.collect();

	let mut video_tracks = Vec::with_capacity(1);

	for trak_atom in trak_atoms {
		eprintln!("Found trak box at {}", trak_atom.start);
		eprint!("Walking trak -> ");
		eprint!("mdia -> ");
		let mdia_atom = find_atom(list_atoms(&mut fh, Some(trak_atom.clone()))?, "mdia")?;
		eprint!("hdlr");
		let hdlr_atom = find_atom(list_atoms(&mut fh, Some(mdia_atom))?, "hdlr")?;
		eprintln!();

		// https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap2/qtff2.html#//apple_ref/doc/uid/TP40000939-CH204-25621
		let component_subtype = read_atom_data(&mut fh, hdlr_atom, 16, 4)?;
		eprintln!("Track type: {}", String::from_utf8_lossy(&component_subtype));

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

	eprintln!("Found video track");

	let tkhd_atom = find_atom(list_atoms(&mut fh, Some(video_track))?, "tkhd")?;

	// https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap2/qtff2.html#//apple_ref/doc/uid/TP40000939-CH204-25550
	let matrix_structure = read_atom_data(&mut fh, tkhd_atom.clone(), 48, 36)?;
	eprint!("Rotation matrix found: ");
	let next_matrix = next_matrix(matrix_structure)?;

	eprintln!("Writing new rotation matrix now");

	fh.seek(SeekFrom::Start(tkhd_atom.start + 48))?;
	fh.write_all(next_matrix)?;

	Ok(())
}

fn next_matrix(current_matrix: Vec<u8>) -> Result<&'static [u8], Box<dyn Error>> {
	Ok(match &*current_matrix {
		RotationMatrix::NO_ROTATION => {
			eprintln!("No rotation => changing to: 90°");
			RotationMatrix::ROTATION_90
		}
		RotationMatrix::ROTATION_90 => {
			eprintln!("90° => changing to: 180°");
			RotationMatrix::ROTATION_180
		}
		RotationMatrix::ROTATION_180 => {
			eprintln!("180° => changing to: 270°");
			RotationMatrix::ROTATION_270
		}
		RotationMatrix::ROTATION_270 => {
			eprintln!("270° => changing to: No rotation");
			RotationMatrix::NO_ROTATION
		}
		_ => return Err(Box::from("Current rotation matrix unknown")),
	})
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
		let atom_size = u32::from_be_bytes(buf[..4].try_into()?);
		let atom_type: ArrayString<[u8; 4]> = ArrayString::from_byte_string(buf[4..].try_into()?)?;
		if atom_size < 8 {
			println!(); // Because we might be in the "Walking ..." part
			return Err(Box::from(format!("Invalid box size {} < 8", atom_size)));
		}
		atoms.push(Atom {
			start: pos,
			size: atom_size as u64,
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
