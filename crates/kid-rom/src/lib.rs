use sha1::{Digest, Sha1};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const HEADER_OFFSET: usize = 0x100;
const HEADER_LENGTH: usize = 0x100;
const KNOWN_SHA1: &str = "28b904000b2863b6760531807760b571f1a5fc1d";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedRom {
    pub path: PathBuf,
    pub size: usize,
    pub sha1: String,
    pub identity: RomIdentity,
    pub header: RomHeader,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomIdentity {
    KnownKidChameleon,
    HeaderMatchesKidChameleon,
}

impl fmt::Display for RomIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KnownKidChameleon => f.write_str("known Kid Chameleon ROM"),
            Self::HeaderMatchesKidChameleon => f.write_str("Kid Chameleon-compatible header"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RomHeader {
    pub console_name: String,
    pub domestic_title: String,
    pub overseas_title: String,
    pub serial: String,
    pub regions: String,
}

#[derive(Debug, Error)]
pub enum RomError {
    #[error("failed to read ROM from {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("ROM is too small to contain a Mega Drive header")]
    TooSmall,
    #[error("ROM header does not look like Kid Chameleon")]
    UnexpectedHeader,
}

pub fn default_rom_candidates(repo_root: &Path) -> Vec<PathBuf> {
    [
        repo_root.join("roms/kid.bin"),
        repo_root.join("kid.bin"),
        repo_root.join("kc.md"),
    ]
    .into_iter()
    .collect()
}

pub fn find_default_rom(repo_root: &Path) -> Option<PathBuf> {
    default_rom_candidates(repo_root)
        .into_iter()
        .find(|candidate| candidate.is_file())
}

pub fn verify_rom(path: &Path) -> Result<VerifiedRom, RomError> {
    let bytes = fs::read(path).map_err(|source| RomError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    if bytes.len() < HEADER_OFFSET + HEADER_LENGTH {
        return Err(RomError::TooSmall);
    }

    let header = parse_header(&bytes)?;
    let sha1 = hex_sha1(&bytes);
    let identity = if sha1 == KNOWN_SHA1 {
        RomIdentity::KnownKidChameleon
    } else {
        RomIdentity::HeaderMatchesKidChameleon
    };

    Ok(VerifiedRom {
        path: path.to_path_buf(),
        size: bytes.len(),
        sha1,
        identity,
        header,
    })
}

pub fn known_sha1() -> &'static str {
    KNOWN_SHA1
}

fn parse_header(bytes: &[u8]) -> Result<RomHeader, RomError> {
    let console_name = parse_ascii(bytes, 0x100, 0x10);
    let domestic_title = parse_ascii(bytes, 0x120, 0x30);
    let overseas_title = parse_ascii(bytes, 0x150, 0x30);
    let serial = parse_ascii(bytes, 0x180, 0x0E);
    let regions = parse_ascii(bytes, 0x1F0, 0x10);

    let looks_like_kid = console_name.contains("SEGA MEGA DRIVE")
        && domestic_title.contains("KID CHAMELEON")
        && overseas_title.contains("KID CHAMELEON")
        && serial.contains("MK-1010");

    if !looks_like_kid {
        return Err(RomError::UnexpectedHeader);
    }

    Ok(RomHeader {
        console_name,
        domestic_title,
        overseas_title,
        serial,
        regions,
    })
}

fn parse_ascii(bytes: &[u8], start: usize, len: usize) -> String {
    let end = start + len;
    let slice = &bytes[start..end];
    let value: String = slice
        .iter()
        .copied()
        .filter(|byte| *byte != 0)
        .map(char::from)
        .collect();
    value.trim().to_owned()
}

fn hex_sha1(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_test_rom() -> Vec<u8> {
        let mut rom = vec![0_u8; 0x200];
        write_ascii(&mut rom, 0x100, 0x10, "SEGA MEGA DRIVE");
        write_ascii(&mut rom, 0x120, 0x30, "KID CHAMELEON");
        write_ascii(&mut rom, 0x150, 0x30, "KID CHAMELEON");
        write_ascii(&mut rom, 0x180, 0x0E, "GM MK-1010");
        write_ascii(&mut rom, 0x1F0, 0x10, "JUE");
        rom
    }

    fn write_ascii(rom: &mut [u8], start: usize, len: usize, text: &str) {
        let bytes = text.as_bytes();
        let end = start + len.min(bytes.len());
        rom[start..end].copy_from_slice(&bytes[..end - start]);
    }

    fn temp_test_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("kid-rom-test-{unique}"));
        fs::create_dir_all(&path).expect("test temp dir should be created");
        path
    }

    #[test]
    fn rom_identity_display_names_match_cli_output() {
        assert_eq!(
            RomIdentity::KnownKidChameleon.to_string(),
            "known Kid Chameleon ROM"
        );
        assert_eq!(
            RomIdentity::HeaderMatchesKidChameleon.to_string(),
            "Kid Chameleon-compatible header"
        );
    }

    #[test]
    fn default_rom_candidates_use_repo_relative_search_order() {
        let repo_root = Path::new("/repo");

        assert_eq!(
            default_rom_candidates(repo_root),
            vec![
                PathBuf::from("/repo/roms/kid.bin"),
                PathBuf::from("/repo/kid.bin"),
                PathBuf::from("/repo/kc.md"),
            ],
            "default ROM lookup should keep the documented repo-local search order"
        );
    }

    #[test]
    fn find_default_rom_returns_first_existing_candidate() {
        let repo_root = temp_test_dir();
        fs::create_dir_all(repo_root.join("roms")).expect("roms dir should be created");
        fs::write(repo_root.join("kid.bin"), b"root").expect("root candidate should write");
        fs::write(repo_root.join("kc.md"), b"md").expect("md candidate should write");

        assert_eq!(
            find_default_rom(&repo_root),
            Some(repo_root.join("kid.bin"))
        );

        fs::write(repo_root.join("roms/kid.bin"), b"roms").expect("roms candidate should write");

        assert_eq!(
            find_default_rom(&repo_root),
            Some(repo_root.join("roms/kid.bin")),
            "roms/kid.bin should take priority once it exists"
        );

        fs::remove_dir_all(repo_root).expect("test temp dir should be removed");
    }

    #[test]
    fn verify_rom_rejects_files_smaller_than_the_full_header_window() {
        let repo_root = temp_test_dir();
        let too_small = repo_root.join("too-small.bin");
        fs::write(&too_small, vec![0_u8; HEADER_OFFSET + HEADER_LENGTH - 1])
            .expect("too-small ROM should write");

        assert!(matches!(verify_rom(&too_small), Err(RomError::TooSmall)));

        fs::remove_dir_all(repo_root).expect("test temp dir should be removed");
    }

    #[test]
    fn verify_rom_accepts_exact_header_window_and_reports_header_identity() {
        let repo_root = temp_test_dir();
        let rom_path = repo_root.join("kid-compatible.bin");
        let rom = make_test_rom();
        fs::write(&rom_path, &rom).expect("test ROM should write");

        let verified = verify_rom(&rom_path).expect("compatible test ROM should verify");

        assert_eq!(verified.path, rom_path);
        assert_eq!(verified.size, HEADER_OFFSET + HEADER_LENGTH);
        assert_eq!(verified.sha1, hex_sha1(&rom));
        assert_eq!(verified.identity, RomIdentity::HeaderMatchesKidChameleon);

        fs::remove_dir_all(repo_root).expect("test temp dir should be removed");
    }

    #[test]
    fn known_sha1_exports_the_canonical_rom_hash() {
        assert_eq!(known_sha1(), KNOWN_SHA1);
    }

    #[test]
    fn parses_expected_header() {
        let rom = make_test_rom();
        let header = parse_header(&rom).expect("test ROM should parse");
        assert_eq!(header.console_name, "SEGA MEGA DRIVE");
        assert_eq!(header.domestic_title, "KID CHAMELEON");
        assert_eq!(header.serial, "GM MK-1010");
        assert_eq!(header.regions, "JUE");
    }

    #[test]
    fn rejects_wrong_header() {
        let rom = vec![0_u8; 0x200];
        let error = parse_header(&rom).expect_err("empty ROM should be rejected");
        assert!(matches!(error, RomError::UnexpectedHeader));
    }

    #[test]
    fn rejects_headers_with_any_required_kid_field_missing() {
        let cases = [
            (0x100, 0x10, "SEGA GENESIS"),
            (0x120, 0x30, "OTHER GAME"),
            (0x150, 0x30, "OTHER GAME"),
            (0x180, 0x0E, "GM MK-9999"),
        ];

        for (start, len, replacement) in cases {
            let mut rom = make_test_rom();
            rom[start..start + len].fill(0);
            write_ascii(&mut rom, start, len, replacement);

            assert!(
                matches!(parse_header(&rom), Err(RomError::UnexpectedHeader)),
                "header should reject missing required field at ${start:04X}"
            );
        }
    }

    #[test]
    fn hex_sha1_matches_known_empty_input_digest() {
        assert_eq!(hex_sha1(&[]), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }
}
