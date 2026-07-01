use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// Known RAM address annotations from variables.asm.
/// Maps WRAM offset (low 16 bits) to a human-readable name.
fn known_addresses() -> BTreeMap<usize, &'static str> {
    BTreeMap::from([
        (0x24FA, "kid_x_pos (hi)"),
        (0x24FC, "kid_x_pos (lo)"),
        (0x24FE, "kid_y_pos (hi)"),
        (0x2500, "kid_y_pos (lo)"),
        // ram_layout.txt: FF2506 long Kid X speed, FF250A long Kid Y speed.
        (0x2506, "kid_x_vel (hi)"),
        (0x2508, "kid_x_vel (lo)"),
        (0x250A, "kid_y_vel (hi)"),
        (0x250C, "kid_y_vel (lo)"),
        (0xF81C, "camera_x_pos (hi)"),
        (0xF81E, "camera_x_pos (lo)"),
        (0xF820, "camera_y_pos (hi)"),
        (0xF822, "camera_y_pos (lo)"),
        (0xF84A, "number_objects"),
        (0xF85C, "number_gfx_objects"),
        (0xFA56, "character_movement"),
        (0xFA6D, "currently_transforming"),
        (0xFA73, "just_received_damage"),
        (0xFAC1, "murder_wall_flag"),
        (0xFAC4, "murder_wall_x_pos"),
        (0xFAD0, "game_paused"),
        (0xFBCA, "game_mode"),
        (0xFC12, "rng_offset"),
        (0xFC37, "level_skip_cheat"),
        (0xFC39, "current_player"),
        (0xFC3E, "number_lives"),
        (0xFC40, "number_hitpoints"),
        (0xFC42, "number_diamonds"),
        (0xFC44, "current_level_id"),
        (0xFC46, "current_helmet_id"),
        (0xFC4C, "score"),
        (0xFC5A, "player_1_level_id"),
        (0xFB40, "level_special_effects"),
        (0xFA2B, "invulnerable_frames"),
        (0xF5B8, "diamond_power_active"),
    ])
}

/// Diff two WRAM dump files byte-by-byte and print changed addresses.
pub(crate) fn run_ram_diff(path_a: &Path, path_b: &Path, annotate: bool) -> Result<()> {
    let data_a =
        fs::read(path_a).with_context(|| format!("failed to read {}", path_a.display()))?;
    let data_b =
        fs::read(path_b).with_context(|| format!("failed to read {}", path_b.display()))?;

    print!("{}", render_ram_diff(&data_a, &data_b, annotate)?);
    Ok(())
}

fn render_ram_diff(data_a: &[u8], data_b: &[u8], annotate: bool) -> Result<String> {
    if data_a.len() != data_b.len() {
        bail!(
            "WRAM dumps have different sizes: {} vs {} bytes",
            data_a.len(),
            data_b.len()
        );
    }

    let annotations = if annotate {
        known_addresses()
    } else {
        BTreeMap::new()
    };

    let mut diff_count = 0usize;
    let mut output = String::new();
    writeln!(
        output,
        "{:<10} {:<12} {:<12} annotation",
        "offset", "old", "new"
    )?;
    writeln!(output, "{}", "-".repeat(60))?;

    for (offset, (a, b)) in data_a.iter().zip(data_b.iter()).enumerate() {
        if a != b {
            let annotation = annotations.get(&offset).unwrap_or(&"");
            writeln!(
                output,
                "0x{offset:04X}    0x{a:02X}        0x{b:02X}        {annotation}"
            )?;
            diff_count += 1;
        }
    }

    writeln!(output)?;
    writeln!(
        output,
        "{diff_count} bytes differ out of {} total",
        data_a.len()
    )?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{known_addresses, render_ram_diff, run_ram_diff};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn annotates_kid_velocity_words_at_the_documented_offsets() {
        let addresses = known_addresses();

        assert_eq!(addresses.get(&0x2506), Some(&"kid_x_vel (hi)"));
        assert_eq!(addresses.get(&0x2508), Some(&"kid_x_vel (lo)"));
        assert_eq!(addresses.get(&0x250A), Some(&"kid_y_vel (hi)"));
        assert_eq!(addresses.get(&0x250C), Some(&"kid_y_vel (lo)"));
    }

    #[test]
    fn renders_byte_differences_and_counts_changed_bytes() {
        let output = render_ram_diff(&[0x10, 0x20, 0x30], &[0x10, 0x21, 0x31], false)
            .expect("matching lengths should render");

        assert!(output.contains("offset     old          new          annotation"));
        assert!(output.contains("0x0001    0x20        0x21"));
        assert!(output.contains("0x0002    0x30        0x31"));
        assert!(output.contains("2 bytes differ out of 3 total"));
        assert!(!output.contains("0x0000    0x10        0x10"));
    }

    #[test]
    fn renders_known_annotations_only_when_requested() {
        let output = render_ram_diff(&[0; 0x2507], &[1; 0x2507], true)
            .expect("matching lengths should render");
        let unannotated = render_ram_diff(&[0; 0x2507], &[1; 0x2507], false)
            .expect("matching lengths should render");

        assert!(output.contains("0x2506    0x00        0x01        kid_x_vel (hi)"));
        assert!(!unannotated.contains("kid_x_vel (hi)"));
    }

    #[test]
    fn rejects_wram_dumps_with_different_sizes() {
        let error = render_ram_diff(&[0x10, 0x20], &[0x10], false)
            .expect_err("different lengths should be rejected")
            .to_string();

        assert_eq!(error, "WRAM dumps have different sizes: 2 vs 1 bytes");
    }

    #[test]
    fn run_ram_diff_propagates_size_mismatch_from_input_files() {
        let temp = TempDir::new().expect("tempdir");
        let file_a = temp.path().join("a.wram");
        let file_b = temp.path().join("b.wram");
        fs::write(&file_a, [0x10, 0x20]).expect("write a");
        fs::write(&file_b, [0x10]).expect("write b");

        let error = run_ram_diff(&file_a, &file_b, false)
            .expect_err("different file lengths should be rejected")
            .to_string();

        assert_eq!(error, "WRAM dumps have different sizes: 2 vs 1 bytes");
    }
}
