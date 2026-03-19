use anyhow::{Context, Result, bail};
use std::io::Read;
use std::path::Path;

use crate::scenarios::{DynamicCheckpoint, DynamicScenario};

/// Parsed BK2 (BizHawk) TAS recording.
#[derive(Debug)]
pub struct Bk2Import {
    pub slug: String,
    pub total_frames: usize,
    /// Per-frame button masks using the same encoding as `FrameInput::from_mask`:
    /// bit 0=Up, 1=Down, 2=Left, 3=Right, 4=B, 5=C, 6=A, 7=Start
    pub script: Vec<u8>,
}

/// BizHawk Genesis button names in the order they appear in the LogKey,
/// mapped to our mask bit positions.
const GENESIS_BUTTON_MAP: &[(&str, u8)] = &[
    ("Up", 0),
    ("Down", 1),
    ("Left", 2),
    ("Right", 3),
    ("A", 6),
    ("B", 4),
    ("C", 5),
    ("Start", 7),
];

/// Parse a BK2 file (BizHawk TAS format) into a `Bk2Import`.
///
/// BK2 is a ZIP archive containing:
/// - `Header.txt`: key=value metadata (must have a Genesis/MD system)
/// - `Input Log.txt`: per-frame button state lines
pub fn parse_bk2(path: &Path) -> Result<Bk2Import> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("failed to open BK2 file: {}", path.display()))?;
    parse_bk2_from_reader(file, path.display().to_string())
}

/// Parse from any `Read + Seek` source (allows testing with in-memory ZIPs).
pub fn parse_bk2_from_reader<R: Read + std::io::Seek>(
    reader: R,
    source_label: String,
) -> Result<Bk2Import> {
    let mut archive =
        zip::ZipArchive::new(reader).with_context(|| format!("invalid ZIP: {source_label}"))?;

    // Read and validate header
    let header_text = read_zip_entry(&mut archive, "Header.txt", &source_label)?;
    validate_header(&header_text, &source_label)?;

    // Parse input log
    let input_text = read_zip_entry(&mut archive, "Input Log.txt", &source_label)?;
    let script = parse_input_log(&input_text)?;

    let slug = Path::new(&source_label)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "tas".into())
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_', "_");

    Ok(Bk2Import {
        slug,
        total_frames: script.len(),
        script,
    })
}

fn read_zip_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
    source_label: &str,
) -> Result<String> {
    let mut entry = archive
        .by_name(name)
        .with_context(|| format!("{source_label} missing {name}"))?;
    let mut buf = String::new();
    entry
        .read_to_string(&mut buf)
        .with_context(|| format!("failed to read {name} from {source_label}"))?;
    Ok(buf)
}

/// Verify the BK2 header declares a Genesis/Mega Drive system.
fn validate_header(header: &str, source_label: &str) -> Result<()> {
    let system = header
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(' ')?;
            if key == "Platform" || key == "SystemID" {
                Some(value.trim())
            } else {
                None
            }
        })
        .unwrap_or("");

    let is_genesis = ["GEN", "Genesis", "MD", "MegaDrive", "Mega Drive", "SEG"]
        .iter()
        .any(|s| system.eq_ignore_ascii_case(s));
    if !is_genesis {
        bail!("{source_label}: unsupported platform '{system}' (expected Genesis/MD)");
    }
    Ok(())
}

/// Parse the `Input Log.txt` content into per-frame u8 button masks.
///
/// Format:
/// ```text
/// [Input]
/// LogKey:#Power|Reset|#P1 Up|P1 Down|...|#P2 ...|
/// |..|UDLRABCS......|..............|
/// ```
///
/// The LogKey defines column order within each `|`-delimited section.
/// We parse it dynamically to build a mapping from character position
/// within the P1 section to our mask bit.
fn parse_input_log(input: &str) -> Result<Vec<u8>> {
    let mut lines = input.lines();

    // Skip [Input] header
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if trimmed == "[Input]" {
            break;
        }
    }

    // Parse LogKey to determine P1 button positions
    let logkey_line = lines
        .by_ref()
        .find(|l| l.trim().starts_with("LogKey:"))
        .context("Input Log.txt missing LogKey line")?;
    let button_positions = parse_logkey(logkey_line)?;

    // Parse frame lines
    let mut script = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "[/Input]" {
            continue;
        }
        // Each line: |console|p1buttons|p2buttons|
        let mask = parse_frame_line(trimmed, &button_positions)?;
        script.push(mask);
    }

    Ok(script)
}

/// Mapping from character position within the P1 section to our mask bit.
struct ButtonPositions {
    /// (char_index_in_p1_section, mask_bit)
    mappings: Vec<(usize, u8)>,
}

/// Parse the LogKey line to determine which character positions in the P1
/// section correspond to which buttons.
///
/// Example LogKey:
/// `LogKey:#Power|Reset|#P1 Up|P1 Down|P1 Left|P1 Right|P1 A|P1 B|P1 C|P1 Start|P1 X|P1 Y|P1 Z|P1 Mode|#P2 Up|...`
///
/// Sections are delimited by `#` markers. The P1 section starts at `#P1`
/// and contains all `P1 xxx` entries until the next `#`.
fn parse_logkey(line: &str) -> Result<ButtonPositions> {
    let key_part = line
        .trim()
        .strip_prefix("LogKey:")
        .context("invalid LogKey line")?;

    // Split by | to get all column names
    let columns: Vec<&str> = key_part.split('|').collect();

    // Find the P1 section: entries starting with "P1 " or "#P1 "
    let mut mappings = Vec::new();

    // Track which section we're in by counting `#` markers
    // Sections: #Console | #P1 | #P2
    // We want the P1 section columns
    let mut in_p1 = false;
    let mut p1_col_index = 0;

    for col in &columns {
        let col = col.trim();
        if col.is_empty() {
            continue;
        }

        // A '#' prefix marks the start of a new section
        if let Some(name) = col.strip_prefix('#') {
            if name.starts_with("P1") {
                in_p1 = true;
                // This column itself is a P1 button (e.g. "#P1 Up")
                let button_name = name.strip_prefix("P1 ").unwrap_or(name);
                if let Some(&(_, bit)) = GENESIS_BUTTON_MAP
                    .iter()
                    .find(|(name, _)| *name == button_name)
                {
                    mappings.push((p1_col_index, bit));
                }
                p1_col_index += 1;
                continue;
            } else if in_p1 {
                // Hit a new section marker (e.g. #P2), stop
                break;
            }
            continue;
        }

        if in_p1 {
            let button_name = col.strip_prefix("P1 ").unwrap_or(col);
            if let Some(&(_, bit)) = GENESIS_BUTTON_MAP
                .iter()
                .find(|(name, _)| *name == button_name)
            {
                mappings.push((p1_col_index, bit));
            }
            p1_col_index += 1;
        }
    }

    if mappings.is_empty() {
        bail!("LogKey contains no recognised P1 Genesis buttons");
    }

    Ok(ButtonPositions { mappings })
}

/// Parse a single frame line like `|..|UDLR.BCS......|..............|`
/// into a u8 button mask.
fn parse_frame_line(line: &str, positions: &ButtonPositions) -> Result<u8> {
    // Split by | — sections: ["", console, p1, p2, ""]
    let sections: Vec<&str> = line.split('|').collect();
    // P1 is typically the third section (index 2): |console|p1|p2|
    let p1_section = sections
        .get(2)
        .context("frame line missing P1 section")?;

    let p1_chars: Vec<char> = p1_section.chars().collect();
    let mut mask: u8 = 0;

    for &(char_idx, bit) in &positions.mappings {
        if let Some(&ch) = p1_chars.get(char_idx) {
            if ch != '.' {
                mask |= 1 << bit;
            }
        }
    }

    Ok(mask)
}

/// Convert a parsed BK2 import into a `DynamicScenario` with periodic checkpoints.
pub fn bk2_to_scenario(import: &Bk2Import, checkpoint_interval: usize) -> DynamicScenario {
    let frame_count = import.total_frames;

    let mut checkpoints = Vec::new();
    // Always checkpoint frame 0
    checkpoints.push(DynamicCheckpoint {
        label: "frame_0".into(),
        frame_index: 0,
    });

    // Periodic checkpoints
    let mut next = checkpoint_interval;
    while next < frame_count {
        checkpoints.push(DynamicCheckpoint {
            label: format!("frame_{next}"),
            frame_index: next,
        });
        next += checkpoint_interval;
    }

    // Final frame (if not already covered)
    let last = frame_count.saturating_sub(1);
    if last > 0 && checkpoints.last().is_none_or(|cp| cp.frame_index != last) {
        checkpoints.push(DynamicCheckpoint {
            label: format!("frame_{last}"),
            frame_index: last,
        });
    }

    DynamicScenario {
        slug: import.slug.clone(),
        script: import.script.clone(),
        frame_count,
        checkpoints,
        warp_to_level: None, // TAS runs from power-on
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write as IoWrite};

    /// Build a minimal BK2 ZIP in memory for testing.
    fn build_test_bk2(header: &str, input_log: &str) -> Vec<u8> {
        let buf = Vec::new();
        let cursor = Cursor::new(buf);
        let mut zip = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        zip.start_file("Header.txt", options).unwrap();
        zip.write_all(header.as_bytes()).unwrap();

        zip.start_file("Input Log.txt", options).unwrap();
        zip.write_all(input_log.as_bytes()).unwrap();

        let cursor = zip.finish().unwrap();
        cursor.into_inner()
    }

    const GENESIS_HEADER: &str = "Platform GEN\nGameName Kid Chameleon\n";

    const STANDARD_LOGKEY: &str = concat!(
        "LogKey:#Power|Reset|",
        "#P1 Up|P1 Down|P1 Left|P1 Right|P1 A|P1 B|P1 C|P1 Start|",
        "P1 X|P1 Y|P1 Z|P1 Mode|",
        "#P2 Up|P2 Down|P2 Left|P2 Right|P2 A|P2 B|P2 C|P2 Start|",
        "P2 X|P2 Y|P2 Z|P2 Mode|"
    );

    fn standard_input_log(frames: &[&str]) -> String {
        let mut log = String::from("[Input]\n");
        log.push_str(STANDARD_LOGKEY);
        log.push('\n');
        for frame in frames {
            log.push_str(frame);
            log.push('\n');
        }
        log.push_str("[/Input]\n");
        log
    }

    #[test]
    fn parse_empty_input_produces_empty_script() {
        let input_log = standard_input_log(&[]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import =
            parse_bk2_from_reader(Cursor::new(bk2), "test.bk2".into()).expect("parse_bk2");
        assert_eq!(import.total_frames, 0);
        assert!(import.script.is_empty());
    }

    #[test]
    fn parse_idle_frame_produces_zero_mask() {
        let input_log = standard_input_log(&["|..|............|............|"]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import =
            parse_bk2_from_reader(Cursor::new(bk2), "test.bk2".into()).expect("parse_bk2");
        assert_eq!(import.total_frames, 1);
        assert_eq!(import.script[0], 0);
    }

    #[test]
    fn parse_start_button_maps_to_bit_7() {
        // Start is column index 7 in P1 section: U D L R A B C S
        let input_log = standard_input_log(&["|..|.......S....|............|"]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import =
            parse_bk2_from_reader(Cursor::new(bk2), "test.bk2".into()).expect("parse_bk2");
        assert_eq!(import.script[0], 0x80); // bit 7 = Start
    }

    #[test]
    fn parse_right_button_maps_to_bit_3() {
        // Right is column index 3: U D L R
        let input_log = standard_input_log(&["|..|...R........|............|"]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import =
            parse_bk2_from_reader(Cursor::new(bk2), "test.bk2".into()).expect("parse_bk2");
        assert_eq!(import.script[0], 1 << 3); // bit 3 = Right
    }

    #[test]
    fn parse_b_button_maps_to_bit_4() {
        // B is column index 5 in P1 section: U D L R A B C S
        let input_log = standard_input_log(&["|..|.....B......|............|"]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import =
            parse_bk2_from_reader(Cursor::new(bk2), "test.bk2".into()).expect("parse_bk2");
        assert_eq!(import.script[0], 1 << 4); // bit 4 = B (jump)
    }

    #[test]
    fn parse_c_button_maps_to_bit_5() {
        // C is column index 6 in P1 section: U D L R A B C S
        let input_log = standard_input_log(&["|..|......C.....|............|"]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import =
            parse_bk2_from_reader(Cursor::new(bk2), "test.bk2".into()).expect("parse_bk2");
        assert_eq!(import.script[0], 1 << 5); // bit 5 = C (run)
    }

    #[test]
    fn parse_a_button_maps_to_bit_6() {
        // A is column index 4 in P1 section: U D L R A B C S
        let input_log = standard_input_log(&["|..|....A.......|............|"]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import =
            parse_bk2_from_reader(Cursor::new(bk2), "test.bk2".into()).expect("parse_bk2");
        assert_eq!(import.script[0], 1 << 6); // bit 6 = A
    }

    #[test]
    fn parse_multiple_buttons_combined() {
        // Right + B + Start: U D L R A B C S
        let input_log = standard_input_log(&["|..|...R.B.S....|............|"]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import =
            parse_bk2_from_reader(Cursor::new(bk2), "test.bk2".into()).expect("parse_bk2");
        let expected = (1 << 3) | (1 << 4) | (1 << 7); // Right + B + Start
        assert_eq!(import.script[0], expected);
    }

    #[test]
    fn parse_multiple_frames() {
        let input_log = standard_input_log(&[
            "|..|............|............|", // idle
            "|..|.......S....|............|", // Start
            "|..|...R........|............|", // Right
        ]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import =
            parse_bk2_from_reader(Cursor::new(bk2), "test.bk2".into()).expect("parse_bk2");
        assert_eq!(import.total_frames, 3);
        assert_eq!(import.script[0], 0);
        assert_eq!(import.script[1], 0x80);
        assert_eq!(import.script[2], 1 << 3);
    }

    #[test]
    fn parse_dpad_buttons() {
        // Up + Down + Left
        let input_log = standard_input_log(&["|..|UDL.........|............|"]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import =
            parse_bk2_from_reader(Cursor::new(bk2), "test.bk2".into()).expect("parse_bk2");
        let expected = (1 << 0) | (1 << 1) | (1 << 2); // Up + Down + Left
        assert_eq!(import.script[0], expected);
    }

    #[test]
    fn reject_non_genesis_platform() {
        let header = "Platform SNES\nGameName SomeGame\n";
        let input_log = standard_input_log(&[]);
        let bk2 = build_test_bk2(header, &input_log);
        let result = parse_bk2_from_reader(Cursor::new(bk2), "test.bk2".into());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unsupported platform"), "got: {err}");
    }

    #[test]
    fn reject_malformed_zip() {
        let result = parse_bk2_from_reader(Cursor::new(vec![0, 1, 2, 3]), "bad.bk2".into());
        assert!(result.is_err());
    }

    #[test]
    fn slug_sanitised_from_filename() {
        let input_log = standard_input_log(&["|..|............|............|"]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import = parse_bk2_from_reader(
            Cursor::new(bk2),
            "Kid Chameleon (Any%) v2.bk2".into(),
        )
        .expect("parse_bk2");
        assert_eq!(import.slug, "Kid_Chameleon__Any___v2");
    }

    #[test]
    fn bk2_to_scenario_creates_checkpoints_at_interval() {
        let import = Bk2Import {
            slug: "test_tas".into(),
            total_frames: 1000,
            script: vec![0; 1000],
        };
        let scenario = bk2_to_scenario(&import, 300);
        assert_eq!(scenario.slug, "test_tas");
        assert_eq!(scenario.frame_count, 1000);
        assert!(scenario.warp_to_level.is_none());
        let labels: Vec<&str> = scenario.checkpoints.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(
            labels,
            vec!["frame_0", "frame_300", "frame_600", "frame_900", "frame_999"]
        );
    }

    #[test]
    fn bk2_to_scenario_small_script_has_start_and_end() {
        let import = Bk2Import {
            slug: "short".into(),
            total_frames: 50,
            script: vec![0; 50],
        };
        let scenario = bk2_to_scenario(&import, 300);
        let labels: Vec<&str> = scenario.checkpoints.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(labels, vec!["frame_0", "frame_49"]);
    }

    #[test]
    fn round_trip_mask_encoding() {
        // Verify our masks match FrameInput::from_mask expectations
        use kid_core::FrameInput;
        let test_cases: &[(u8, &str)] = &[
            (1 << 0, "up"),
            (1 << 1, "down"),
            (1 << 2, "left"),
            (1 << 3, "right"),
            (1 << 4, "b"),
            (1 << 5, "c"),
            (1 << 6, "a"),
            (1 << 7, "start"),
        ];
        for &(mask, expected_name) in test_cases {
            let input = FrameInput::from_mask(mask);
            let active = match expected_name {
                "up" => input.up,
                "down" => input.down,
                "left" => input.left,
                "right" => input.right,
                "a" => input.a,
                "b" => input.b,
                "c" => input.c,
                "start" => input.start,
                _ => unreachable!(),
            };
            assert!(
                active,
                "mask {mask:#04x} should activate {expected_name}"
            );
        }
    }
}
