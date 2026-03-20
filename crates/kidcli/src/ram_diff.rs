use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;
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
        (0x2502, "kid_x_vel (hi)"),
        (0x2504, "kid_x_vel (lo)"),
        (0x2506, "kid_y_vel (hi)"),
        (0x2508, "kid_y_vel (lo)"),
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
    let data_a = fs::read(path_a)
        .with_context(|| format!("failed to read {}", path_a.display()))?;
    let data_b = fs::read(path_b)
        .with_context(|| format!("failed to read {}", path_b.display()))?;

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
    println!("{:<10} {:<12} {:<12} {}", "offset", "old", "new", "annotation");
    println!("{}", "-".repeat(60));

    for (offset, (a, b)) in data_a.iter().zip(data_b.iter()).enumerate() {
        if a != b {
            let annotation = annotations.get(&offset).unwrap_or(&"");
            println!(
                "0x{offset:04X}    0x{a:02X}        0x{b:02X}        {annotation}"
            );
            diff_count += 1;
        }
    }

    println!("\n{diff_count} bytes differ out of {} total", data_a.len());
    Ok(())
}
