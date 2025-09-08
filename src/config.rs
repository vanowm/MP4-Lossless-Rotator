use std::fs;
use std::path::Path;

pub struct Config {
    pub rotation: Option<u32>,
    pub backup: Option<bool>,
}

impl Config {
    pub fn from_ini<P: AsRef<Path>>(path: P) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        let mut rotation = None;
        let mut backup = None;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with(';') || line.starts_with('#') || line.is_empty() || line.starts_with('[') {
                continue;
            }
            let mut parts = line.splitn(2, '=');
            let key = parts.next()?.trim().to_ascii_lowercase();
            let value = parts.next()?.trim().trim_matches('"');
            match key.as_str() {
                "rotate" => {
                    if let Ok(val) = value.parse::<u32>() {
                        if [0, 90, 180, 270].contains(&val) {
                            rotation = Some(val);
                        }
                    }
                },
                "backup" => {
                    if value.eq_ignore_ascii_case("true") {
                        backup = Some(true);
                    } else if value.eq_ignore_ascii_case("false") {
                        backup = Some(false);
                    }
                },
                _ => {}
            }
        }
        Some(Config { rotation, backup })
    }
}
