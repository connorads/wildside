use super::*;
use wildside_core::{ActiveProjectileSnapshot, terrain};
use wildside_data::resource_level_record;

pub(crate) fn inspect_level(manifest: &AssetManifest, level_spec: &str) -> Result<()> {
    let level_id = parse_level_id(level_spec)?;
    let level = manifest
        .levels
        .iter()
        .find(|level| level.level_id == level_id)
        .ok_or_else(|| anyhow!("level 0x{level_id:02X} not found in asset manifest"))?;
    let bundle = load_level_asset_bundle(manifest, level_id)?;

    println!("level: 0x{level_id:02X}");
    if let Some(name) = &level.display_name {
        println!("name: {name}");
    }
    let effective_resource_level_id = resource_level_id(manifest, level_id);
    if effective_resource_level_id != level_id {
        println!("resource level: 0x{effective_resource_level_id:02X}");
    }
    if let Some(title_text_id) = level.title_text_id {
        println!("title text id: 0x{title_text_id:02X}");
    }
    let resource_level = resource_level_record(manifest, level_id).unwrap_or(level);
    println!(
        "resource paths: {}",
        resource_level.resource_paths.join(", ")
    );
    if let Some(header) = &bundle.parsed_header {
        println!(
            "header: size {}x{} screens, theme 0x{:02X}, bg theme 0x{:02X}, weather 0x{:X}, murder wall 0x{:X}",
            header.width_screens,
            header.height_screens,
            header.level_theme,
            header.background_theme,
            header.weather_flags,
            header.murder_wall_flags
        );
        println!(
            "player start: {},{}  flag: {},{}",
            header.player_start_x, header.player_start_y, header.flag_x, header.flag_y
        );
    }
    if let Some(bgscroll) = &bundle.parsed_bgscroll {
        let preview = bgscroll
            .layer_speeds
            .iter()
            .take(8)
            .map(|speed| format!("0x{speed:02X}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "bgscroll: {} layers{}{}",
            bgscroll.layer_speeds.len(),
            if preview.is_empty() {
                String::new()
            } else {
                format!(" [{preview}]")
            },
            if bgscroll.has_terminator {
                String::new()
            } else {
                String::from(" (unterminated split blob)")
            }
        );
    }
    if let Some(platform_layout) = &bundle.parsed_platform_layout {
        let preview = platform_layout
            .placements
            .iter()
            .take(3)
            .map(|placement| {
                format!(
                    "({}, {}) t{} s{} {}",
                    placement.x,
                    placement.y,
                    placement.kind,
                    placement.subtype,
                    placement.script.raw
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "platforms: {} entries{}",
            platform_layout.placements.len(),
            if preview.is_empty() {
                String::new()
            } else {
                format!(" [{preview}]")
            }
        );
    }
    if let Some(enemy_layout) = &bundle.parsed_enemy_layout {
        println!(
            "enemy layout: {} objects across {} segment{} ({})",
            enemy_layout.object_count,
            enemy_layout.segments.len().max(1),
            if enemy_layout.segments.len() == 1 {
                ""
            } else {
                "s"
            },
            if enemy_layout.axis_uses_y_position {
                "camera-y keyed"
            } else {
                "camera-x keyed"
            }
        );
        let primary_slots = enemy_layout
            .segments
            .first()
            .map_or(enemy_layout.enemy_type_slots.as_slice(), |segment| {
                segment.enemy_type_slots.as_slice()
            });
        for (slot_index, slot) in primary_slots.iter().enumerate() {
            println!(
                "enemy slot {}: {}",
                slot_index + 1,
                describe_enemy_slot(manifest, *slot)
            );
        }
        if let Some(segment) = enemy_layout.segments.first() {
            println!(
                "enemy segment 1: 0x{:04X}..0x{:04X}, batch 0x{:02X}, {} object{}",
                segment.lower_bound,
                segment.upper_bound,
                segment.object_batch_flags,
                segment.object_count,
                if segment.object_count == 1 { "" } else { "s" }
            );
        } else {
            println!(
                "enemy segment 1: 0x0000..0x{:04X}, batch 0x{:02X}, {} object{}",
                enemy_layout.unknown_word,
                enemy_layout.header_byte,
                enemy_layout.object_count,
                if enemy_layout.object_count == 1 {
                    ""
                } else {
                    "s"
                }
            );
        }
        println!(
            "enemy objects: {}",
            summarise_enemy_object_types(&enemy_layout.objects)
        );
        for line in describe_enemy_layout_objects(manifest, &enemy_layout.objects) {
            println!("{line}");
        }
    }
    let block_grid =
        load_inspect_block_grid(manifest, level_id, effective_resource_level_id, &bundle);
    if let Some(block_list) = load_inspect_block_list(
        manifest,
        level_id,
        effective_resource_level_id,
        &bundle,
        block_grid.as_ref(),
    ) {
        for line in describe_block_list(&block_list) {
            println!("{line}");
        }
    }
    print_loaded_asset("platform", bundle.platform_script.as_ref());
    print_loaded_asset("bgscroll", bundle.bgscroll.as_ref());
    print_loaded_asset("header", bundle.header.as_ref());
    print_loaded_asset("enemy", bundle.enemy.as_ref());
    print_loaded_asset("foreground", bundle.foreground.as_ref());
    print_loaded_asset("block", bundle.block.as_ref());
    print_loaded_asset("background", bundle.background.as_ref());
    Ok(())
}

fn load_inspect_block_grid(
    manifest: &AssetManifest,
    level_id: u8,
    effective_resource_level_id: u8,
    bundle: &wildside_data::LevelAssetBundle,
) -> Option<terrain::BlockGrid> {
    let layout_dir = manifest.output_root.join("level-layout");
    let logical_path = layout_dir.join(format!("{level_id:02X}.bin"));
    let path = if logical_path.is_file() {
        logical_path
    } else {
        let fallback_path = layout_dir.join(format!("{effective_resource_level_id:02X}.bin"));
        if !fallback_path.is_file() {
            return None;
        }
        fallback_path
    };
    let bytes = std::fs::read(path).ok()?;
    let header = bundle.parsed_header.as_ref()?;
    // ASM Load_LevelData loc_15A0: Level_width_blocks = width_screens * 20.
    let width = u16::from(header.width_screens) * 20;
    let height = (bytes.len() / 2 / usize::from(width)).max(1) as u16;
    Some(terrain::BlockGrid::from_level_layout(&bytes, width, height))
}

fn load_inspect_block_list(
    manifest: &AssetManifest,
    level_id: u8,
    effective_resource_level_id: u8,
    bundle: &wildside_data::LevelAssetBundle,
    grid: Option<&terrain::BlockGrid>,
) -> Option<terrain::BlockList> {
    let layout_dir = manifest.output_root.join("level-layout");
    let logical_path = layout_dir.join(format!("{level_id:02X}.blocklist.bin"));
    let path = if logical_path.is_file() {
        Some(logical_path)
    } else {
        let fallback_path =
            layout_dir.join(format!("{effective_resource_level_id:02X}.blocklist.bin"));
        fallback_path.is_file().then_some(fallback_path)
    };

    path.and_then(|path| std::fs::read(path).ok())
        .map(|bytes| {
            let layout_width = bundle
                .parsed_header
                .as_ref()
                .map_or(256, |header| u16::from(header.width_screens) * 20);
            if let Some(grid) = grid {
                terrain::BlockList::from_raw_with_level_layout(&bytes, layout_width, grid)
            } else {
                terrain::BlockList::from_raw(&bytes, layout_width)
            }
        })
        .or_else(|| {
            wildside_core::NativeRuntime::with_generated_level_layout(manifest.clone())
                .block_list_for_level(wildside_core::LevelId(level_id))
        })
}

fn describe_block_list(block_list: &terrain::BlockList) -> Vec<String> {
    let mut lines = vec![format!(
        "block list: {} prize{}, {} ghost group{}, {} teleporter{}, {} hidden{}",
        block_list.prize_blocks.len(),
        if block_list.prize_blocks.len() == 1 {
            ""
        } else {
            "s"
        },
        block_list.ghost_blocks.len(),
        if block_list.ghost_blocks.len() == 1 {
            ""
        } else {
            "s"
        },
        block_list.teleporters.len(),
        if block_list.teleporters.len() == 1 {
            ""
        } else {
            "s"
        },
        block_list.hidden_blocks.len(),
        if block_list.hidden_blocks.len() == 1 {
            ""
        } else {
            "s"
        },
    )];

    for (index, prize) in block_list.prize_blocks.iter().enumerate() {
        lines.push(format!(
            "prize block {index:02}: col {}, row {}, {:?} id 0x{:X}{} layout 0x{:04X}",
            prize.col,
            prize.row,
            prize.content,
            prize.content.to_id(),
            if prize.disguised { " disguised" } else { "" },
            prize.layout_addr,
        ));
    }

    for (index, ghost) in block_list.ghost_blocks.iter().enumerate() {
        lines.push(format!(
            "ghost group {index:02}: timer {:#06X} state {:#06X}, solid {} vanished {}, at ({},{}) count {} {}",
            ghost.timer,
            ghost.state,
            ghost.solid_time,
            ghost.vanished_time,
            ghost.col,
            ghost.row,
            ghost.count,
            if ghost.vertical { "vertical" } else { "horizontal" },
        ));
    }

    for (index, teleporter) in block_list.teleporters.iter().enumerate() {
        lines.push(format!(
            "teleporter {index:02}: col {}, row {}, dest map 0x{:02X}, dest {},{} layout 0x{:04X}",
            teleporter.col,
            teleporter.row,
            teleporter.dest_map_id,
            teleporter.dest_x,
            teleporter.dest_y,
            teleporter.layout_addr,
        ));
    }

    for (index, hidden) in block_list.hidden_blocks.iter().enumerate() {
        lines.push(format!(
            "hidden block {index:02}: col {}, row {}, type 0x{:02X}, setting 0x{:02X}, layout 0x{:04X}",
            hidden.col,
            hidden.row,
            hidden.block_type,
            hidden.block_setting,
            hidden.layout_addr,
        ));
    }

    lines
}

pub(crate) fn inspect_respawn_settle(
    manifest: &AssetManifest,
    level_spec: &str,
    steps: usize,
) -> Result<()> {
    let level_id = parse_level_id(level_spec)?;
    let mut runtime = wildside_core::NativeRuntime::with_generated_level_layout(manifest.clone());
    runtime.warp_to_level(wildside_core::LevelId(level_id));
    let frames = runtime.inspect_respawn_settle(wildside_core::LevelId(level_id), steps);

    println!("level: 0x{level_id:02X}");
    if let Some(name) = runtime.level_name(wildside_core::LevelId(level_id)) {
        println!("name: {name}");
    }
    println!("steps: {}", frames.len().saturating_sub(1));

    for frame in frames {
        println!(
            "step {}: kid {:08X},{:08X} vel {:08X},{:08X} ground {} spikes {} camera {:08X},{:08X} floor {:08X} wide_support {}",
            frame.step_index,
            frame.kid_x_pos,
            frame.kid_y_pos,
            frame.kid_x_velocity as u32,
            frame.kid_y_velocity as u32,
            frame.kid_on_ground,
            frame.kid_on_spikes,
            frame.camera_x_pos,
            frame.camera_y_pos,
            frame.floor_y_pos,
            frame.wide_support_applied,
        );
    }

    Ok(())
}

pub(crate) fn inspect_spawn_probe(manifest: &AssetManifest, level_spec: &str) -> Result<()> {
    let level_id = parse_level_id(level_spec)?;
    let mut runtime = wildside_core::NativeRuntime::with_generated_level_layout(manifest.clone());
    runtime.warp_to_level(wildside_core::LevelId(level_id));
    let inspection = runtime
        .inspect_spawn_probes(wildside_core::LevelId(level_id))
        .ok_or_else(|| anyhow!("level 0x{level_id:02X} has no generated block grid"))?;

    println!("level: 0x{level_id:02X}");
    if let Some(name) = runtime.level_name(wildside_core::LevelId(level_id)) {
        println!("name: {name}");
    }
    println!(
        "spawn: kid {:08X},{:08X}",
        inspection.kid_x_pos, inspection.kid_y_pos
    );
    println!(
        "hitbox probe: ground {} floor {} cell ({}, {}) collision {:?} block {:?} skin 0x{:02X}",
        inspection.hitbox_probe.on_ground,
        inspection.hitbox_probe.floor_y_px,
        inspection.hitbox_probe.cell_x,
        inspection.hitbox_probe.cell_y,
        inspection.hitbox_probe.cell.collision,
        inspection.hitbox_probe.cell.block_type,
        inspection.hitbox_probe.cell.skin,
    );
    println!(
        "span probe: ground {} floor {} cell ({}, {}) collision {:?} block {:?} skin 0x{:02X}",
        inspection.span_probe.on_ground,
        inspection.span_probe.floor_y_px,
        inspection.span_probe.cell_x,
        inspection.span_probe.cell_y,
        inspection.span_probe.cell.collision,
        inspection.span_probe.cell.block_type,
        inspection.span_probe.cell.skin,
    );

    Ok(())
}

pub(crate) fn print_loaded_asset(label: &str, asset: Option<&wildside_data::LoadedAsset>) {
    match asset {
        Some(asset) => println!(
            "{label}: {} ({} bytes)",
            asset.relative_path,
            asset.bytes.len()
        ),
        None => println!("{label}: <none>"),
    }
}

pub(crate) fn describe_enemy_slot(manifest: &AssetManifest, raw_slot: u16) -> String {
    if raw_slot == 0xFFFF {
        return String::from("blank");
    }

    let palette_flags = raw_slot >> 14;
    let slot_id = raw_slot & 0x3FFF;
    let enemy_name = u8::try_from(slot_id)
        .ok()
        .and_then(|enemy_id| find_enemy_definition(manifest, &format!("{enemy_id:02X}")))
        .map(|definition| definition.symbolic_name);

    match enemy_name {
        Some(enemy_name) => format!(
            "0x{raw_slot:04X} -> {enemy_name} (kind 0x{slot_id:02X}, palette {palette_flags})"
        ),
        None => format!("0x{raw_slot:04X} (kind 0x{slot_id:04X}, palette {palette_flags})"),
    }
}

pub(crate) fn summarise_enemy_object_types(objects: &[wildside_data::EnemyLayoutObject]) -> String {
    if objects.is_empty() {
        return String::from("<none>");
    }

    let mut counts = BTreeMap::new();
    for object in objects {
        *counts.entry(object.object_type).or_insert(0usize) += 1;
    }

    counts
        .into_iter()
        .map(|(object_type, count)| format!("0x{object_type:02X}x{count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn describe_enemy_layout_objects(
    manifest: &AssetManifest,
    objects: &[wildside_data::EnemyLayoutObject],
) -> Vec<String> {
    objects
        .iter()
        .enumerate()
        .map(|(index, object)| {
            let enemy_name =
                find_enemy_definition(manifest, &format!("{:02X}", object.object_type))
                    .map_or_else(
                        || String::from("<unknown>"),
                        |definition| definition.symbolic_name,
                    );
            format!(
                "enemy object {:02}: kind 0x{:02X} {enemy_name} at {},{} flags 0x{:02X} hp+1 {}",
                index,
                object.object_type,
                object.x,
                object.y,
                object.flags,
                object.hit_points_plus_one
            )
        })
        .collect()
}

pub(crate) fn parse_level_id(value: &str) -> Result<u8> {
    let trimmed = value.trim();
    let hex = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    u8::from_str_radix(hex, 16)
        .with_context(|| format!("level id must be hex in the range 00-FF, got {value}"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TargetPoint {
    pub(crate) x_px: i64,
    pub(crate) y_px: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TargetCell {
    pub(crate) col: i64,
    pub(crate) row: i64,
}

pub(crate) fn parse_target_point(value: &str) -> Result<TargetPoint> {
    let (x, y) = value
        .split_once(',')
        .ok_or_else(|| anyhow!("target point must be formatted as x,y pixels, got {value}"))?;
    let x_px = x
        .trim()
        .parse::<i64>()
        .with_context(|| format!("target point x must be a signed pixel integer, got {x}"))?;
    let y_px = y
        .trim()
        .parse::<i64>()
        .with_context(|| format!("target point y must be a signed pixel integer, got {y}"))?;
    Ok(TargetPoint { x_px, y_px })
}

pub(crate) fn parse_target_cell(value: &str) -> Result<TargetCell> {
    let (col, row) = value
        .split_once(',')
        .ok_or_else(|| anyhow!("target cell must be formatted as col,row, got {value}"))?;
    let col = col
        .trim()
        .parse::<i64>()
        .with_context(|| format!("target cell col must be a signed integer, got {col}"))?;
    let row = row
        .trim()
        .parse::<i64>()
        .with_context(|| format!("target cell row must be a signed integer, got {row}"))?;
    Ok(TargetCell { col, row })
}

pub(crate) fn inspect_enemy(manifest: &AssetManifest, selector: &str) -> Result<()> {
    let enemy = find_enemy_definition(manifest, selector)
        .ok_or_else(|| anyhow!("enemy {selector} not found in asset manifest"))?;
    println!("enemy id: 0x{:02X}", enemy.id);
    println!("constant: {}", enemy.constant_name);
    println!("name: {}", enemy.symbolic_name);
    match &enemy.source_path {
        Some(path) => println!("source: {path}"),
        None => println!("source: <none>"),
    }
    Ok(())
}

pub(crate) fn print_probe(
    rom_path: &Path,
    probe: &wildside_md::CpuProbe,
    summary_only: bool,
    step_log_limit: usize,
    trace_log_limit: usize,
) {
    println!("rom: {}", rom_path.display());
    println!("reset sp: 0x{:08X}", probe.reset_sp);
    println!("reset pc: 0x{:08X}", probe.reset_pc);
    println!("final sp: 0x{:08X}", probe.final_sp);
    println!("final pc: 0x{:08X}", probe.final_pc);
    println!("synthetic vblanks: {}", probe.synthetic_vblanks);
    println!("v-int counter: {}", probe.v_int_counter);
    println!("v-int done: 0x{:02X}", probe.v_int_done);
    println!("game mode: 0x{:04X}", probe.game_mode);
    println!("demo mode flag: 0x{:02X}", probe.demo_mode_flag);
    println!("current player: 0x{:02X}", probe.current_player);
    println!("number lives: {}", probe.number_lives);
    println!("number hitpoints: {}", probe.number_hitpoints);
    println!("current level id: 0x{:04X}", probe.current_level_id);
    println!("player 1 level id: 0x{:04X}", probe.player_1_level_id);
    println!("current helmet id: 0x{:04X}", probe.current_helmet_id);
    println!("camera x: 0x{:08X}", probe.camera_x_pos);
    println!("camera y: 0x{:08X}", probe.camera_y_pos);
    println!("kid x: 0x{:08X}", probe.kid_x_pos);
    println!("kid y: 0x{:08X}", probe.kid_y_pos);
    println!("number objects: {}", probe.number_objects);
    println!("number gfx objects: {}", probe.number_gfx_objects);
    println!("z80 interrupt mode: {}", probe.z80_interrupt_mode);
    println!("z80 reset count: {}", probe.z80_reset_count);
    println!("vram nonzero bytes: {}", probe.vram_nonzero_bytes);
    println!("cram nonzero bytes: {}", probe.cram_nonzero_bytes);
    println!("vsram nonzero bytes: {}", probe.vsram_nonzero_bytes);
    println!("ym2612 writes: {}", probe.ym2612_write_count);
    println!("ym2612 key-on writes: {}", probe.ym2612_key_on_count);
    println!("psg writes: {}", probe.psg_write_count);
    let ym_register_writes = decode_ym2612_register_writes(&probe.ym2612_writes);
    if !ym_register_writes.is_empty() {
        let key_on_count = ym_register_writes
            .iter()
            .filter(|write| write.register == 0x0028)
            .count();
        println!(
            "ym2612 register writes: {}",
            format_ym2612_register_writes(&ym_register_writes)
        );
        println!("ym2612 key-on writes: {key_on_count}");
        println!(
            "ym2612 replay is audible: {}",
            ym2612_register_writes_are_audible(&ym_register_writes)
        );
    } else if !probe.ym2612_writes.is_empty() {
        println!(
            "ym2612 write samples: {}",
            format_ym2612_writes(&probe.ym2612_writes)
        );
    }
    if !probe.psg_writes.is_empty() {
        println!(
            "psg write samples: {}",
            format_u8_samples(&probe.psg_writes)
        );
    }
    if !probe.banked_68k_reads.is_empty() {
        println!(
            "banked 68k read samples: {}",
            format_banked_68k_accesses(&probe.banked_68k_reads)
        );
    }
    if !probe.banked_68k_writes.is_empty() {
        println!(
            "banked 68k write samples: {}",
            format_banked_68k_accesses(&probe.banked_68k_writes)
        );
    }

    if !summary_only {
        for step in probe.steps.iter().take(step_log_limit) {
            println!(
                "step {:03}: pc 0x{:08X} -> 0x{:08X} ({} cycles)",
                step.index, step.pc_before, step.pc_after, step.cycles
            );
        }
        if probe.steps.len() > step_log_limit {
            println!(
                "step log truncated: showing {} of {} steps",
                step_log_limit,
                probe.steps.len()
            );
        }
    }

    println!("mmio/trace entries: {}", probe.accesses.len());
    if !summary_only {
        for access in probe.accesses.iter().take(trace_log_limit) {
            println!(
                "trace {:?}: 0x{:08X} = 0x{:X}",
                access.kind, access.address, access.value
            );
        }
        if probe.accesses.len() > trace_log_limit {
            println!(
                "trace log truncated: showing {} of {} entries",
                trace_log_limit,
                probe.accesses.len()
            );
        }
    }

    if !probe.vdp_control_writes.is_empty() {
        println!(
            "vdp control writes: {}",
            format_u16_samples(&probe.vdp_control_writes)
        );
    }
    if !probe.vdp_data_writes.is_empty() {
        println!(
            "vdp data writes: {}",
            format_u16_samples(&probe.vdp_data_writes)
        );
    }
}

pub(crate) fn format_u16_samples(values: &[u16]) -> String {
    const EDGE: usize = 8;

    if values.len() <= EDGE * 2 {
        return format!("{values:?}");
    }

    let head = values
        .iter()
        .take(EDGE)
        .map(|value| format!("0x{value:04X}"))
        .collect::<Vec<_>>()
        .join(", ");
    let tail = values
        .iter()
        .skip(values.len() - EDGE)
        .map(|value| format!("0x{value:04X}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{head}, ..., {tail}] ({} total)", values.len())
}

pub(crate) fn format_u8_samples(values: &[u8]) -> String {
    const EDGE: usize = 8;

    if values.len() <= EDGE * 2 {
        return values
            .iter()
            .map(|value| format!("0x{value:02X}"))
            .collect::<Vec<_>>()
            .join(", ");
    }

    let head = values
        .iter()
        .take(EDGE)
        .map(|value| format!("0x{value:02X}"))
        .collect::<Vec<_>>()
        .join(", ");
    let tail = values
        .iter()
        .skip(values.len() - EDGE)
        .map(|value| format!("0x{value:02X}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{head}, ..., {tail}] ({} total)", values.len())
}

pub(crate) fn format_ym2612_writes(values: &[(u16, u8)]) -> String {
    const EDGE: usize = 8;

    let render = |slice: &[(u16, u8)]| {
        slice
            .iter()
            .map(|(port, value)| format!("(0x{port:04X}, 0x{value:02X})"))
            .collect::<Vec<_>>()
            .join(", ")
    };

    if values.len() <= EDGE * 2 {
        return render(values);
    }

    let head = render(&values[..EDGE]);
    let tail = render(&values[values.len() - EDGE..]);
    format!("[{head}, ..., {tail}] ({} total)", values.len())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ym2612RegisterWrite {
    pub(crate) register: u16,
    pub(crate) value: u8,
}

pub(crate) fn decode_ym2612_register_writes(values: &[(u16, u8)]) -> Vec<Ym2612RegisterWrite> {
    let mut latched = [None, None];
    let mut decoded = Vec::new();

    for (port, value) in values {
        match port & 0x0003 {
            0 => latched[0] = Some(u16::from(*value)),
            2 => latched[1] = Some(0x0100 | u16::from(*value)),
            1 => {
                if let Some(register) = latched[0] {
                    decoded.push(Ym2612RegisterWrite {
                        register,
                        value: *value,
                    });
                }
            }
            3 => {
                if let Some(register) = latched[1] {
                    decoded.push(Ym2612RegisterWrite {
                        register,
                        value: *value,
                    });
                }
            }
            _ => {}
        }
    }

    decoded
}

pub(crate) fn format_ym2612_register_writes(values: &[Ym2612RegisterWrite]) -> String {
    const EDGE: usize = 8;

    let render = |slice: &[Ym2612RegisterWrite]| {
        slice
            .iter()
            .map(|write| format!("(0x{:04X} <- 0x{:02X})", write.register, write.value))
            .collect::<Vec<_>>()
            .join(", ")
    };

    if values.len() <= EDGE * 2 {
        return render(values);
    }

    let head = render(&values[..EDGE]);
    let tail = render(&values[values.len() - EDGE..]);
    format!("[{head}, ..., {tail}] ({} total)", values.len())
}

pub(crate) fn format_banked_68k_accesses(values: &[(u32, u8)]) -> String {
    const EDGE: usize = 8;

    let render = |slice: &[(u32, u8)]| {
        slice
            .iter()
            .map(|(address, value)| format!("(0x{address:06X}, 0x{value:02X})"))
            .collect::<Vec<_>>()
            .join(", ")
    };

    if values.len() <= EDGE * 2 {
        return render(values);
    }

    let head = render(&values[..EDGE]);
    let tail = render(&values[values.len() - EDGE..]);
    format!("[{head}, ..., {tail}] ({} total)", values.len())
}

pub(crate) fn ym2612_register_writes_are_audible(values: &[Ym2612RegisterWrite]) -> bool {
    if values.is_empty() {
        return false;
    }

    let mut chip = Ym2612::default();
    for write in values {
        match write.register {
            0x0000..=0x00FF => {
                chip.write_port(0, write.register as u8);
                chip.write_port(1, write.value);
            }
            0x0100..=0x01FF => {
                chip.write_port(2, (write.register & 0x00FF) as u8);
                chip.write_port(3, write.value);
            }
            _ => {}
        }
    }

    let mut left = [0i32; 512];
    let mut right = [0i32; 512];
    chip.generate(&mut left, &mut right);
    left.iter().any(|sample| *sample != left[0]) || right.iter().any(|sample| *sample != right[0])
}

fn print_trace_frames_in_range(
    trace: &[NativeTraceFrame],
    script: &[u8],
    trace_physics: bool,
    debug_fields: bool,
    frame_start: Option<usize>,
    frame_end: Option<usize>,
) {
    if let Some(start) = frame_start {
        let end = frame_end.unwrap_or(start);
        for (trace_index, frame) in trace
            .iter()
            .enumerate()
            .filter(|(index, _)| (start..=end).contains(index))
        {
            let input = script
                .get(trace_index)
                .copied()
                .map(wildside_core::FrameInput::from_mask)
                .unwrap_or_default();
            println!(
                "frame {} (trace {:08X}): input {} mode {:?} boot={:?}@{:?} rng {:04X} time {} camera {:08X},{:08X} kid {:08X},{:08X} vel {:08X},{:08X} ground {:?} slope {} airborne_mapping {:?} invuln {} form_flash {} death_windup {} lives/hp {}/{} diamonds {} hud[diamond={:?}/{:?} life={:?}/{:?}] transform_source={:?}/{:?} floating={:?} shore={:?}/{:?} prize_flash={:?} bounce={:?} tails={:?} debris={:?} sparks={:?} projectiles={:?}/{:?} active_enemies {} active_platforms {} roster {}",
                trace_index,
                frame.frame_index,
                describe_frame_input(input),
                frame.game_mode,
                frame.boot_phase,
                frame.boot_phase_elapsed,
                frame.rng_offset.unwrap_or_default(),
                frame.time_frames_loc_3038a,
                frame.camera_x_pos,
                frame.camera_y_pos,
                frame.kid_x_pos.unwrap_or_default(),
                frame.kid_y_pos.unwrap_or_default(),
                frame.kid_x_velocity.unwrap_or_default() as u32,
                frame.kid_y_velocity.unwrap_or_default() as u32,
                frame.kid_on_ground,
                frame.kid_on_slope,
                frame.airborne_visual_mapping_sub_b270,
                frame.invulnerable_frames,
                frame.form_change_flash_frames,
                frame.death_windup_elapsed,
                frame.number_lives,
                frame.number_hitpoints,
                frame.diamond_count,
                frame.hud_diamond_frame_offset,
                frame.hud_diamond_vram_page_loc_5d4a,
                frame.hud_life_frame_offset,
                frame.hud_life_icon_vram_page_loc_4586,
                frame.transform_source_helmet_id_loc_7ca6,
                frame.transform_source_hitpoints_loc_bdc0,
                frame.floating_items,
                frame.island_shore_vram_frames_loc_1202a,
                frame.island_shore_dma_uploads_loc_1202a,
                frame.prize_block_spawn_flashes,
                frame.block_bounces,
                frame.prize_block_worker_tails_sub_1007a,
                frame.block_destroy_effects,
                frame.kid_projectile_hit_sparks,
                frame.active_projectiles,
                frame.active_projectile_details,
                frame.active_enemy_object_count,
                frame.active_platform_object_count,
                describe_active_objects(frame)
            );
            if debug_fields && !frame.gameplay_debug_fields.is_empty() {
                println!(
                    "  gameplay: {}",
                    format_debug_fields(&frame.gameplay_debug_fields)
                );
            }
            if trace_physics && let Some(physics) = frame.physics_trace {
                println!(
                    "  physics: pre_h {:08X},{:08X} post_h {:08X},{:08X} pre_v {:08X},{:08X} post_v {:08X},{:08X} jump {} jump_y {} ceiling_hit {} ceiling_cell {:?} floor_hit {} floor_y {:?} landing {:?} wall_hit {} wall_x {:?} final_pos {:08X},{:08X} final_vel {:08X},{:08X}",
                    physics.pre_horizontal.0,
                    physics.pre_horizontal.1 as u32,
                    physics.post_horizontal.0,
                    physics.post_horizontal.1 as u32,
                    physics.pre_vertical.0,
                    physics.pre_vertical.1 as u32,
                    physics.post_vertical.0,
                    physics.post_vertical.1 as u32,
                    physics.jump_initiated,
                    physics.jump_y_vel.unwrap_or_default() as u32,
                    physics.ceiling_hit,
                    physics.ceiling_probe_cell,
                    physics.floor_hit,
                    physics.floor_y_px,
                    physics.landing_path,
                    physics.wall_hit,
                    physics.wall_snap_x.map(|x| format!("{x:08X}")),
                    physics.final_pos.0,
                    physics.final_pos.1,
                    physics.final_vel.0 as u32,
                    physics.final_vel.1 as u32,
                );
            }
        }
    }
}

fn format_debug_fields(fields: &std::collections::BTreeMap<String, String>) -> String {
    fields
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn probe_native_scenario(
    manifest: &AssetManifest,
    scenario: Scenario,
    frame_start: Option<usize>,
    frame_end: Option<usize>,
) -> Result<()> {
    let script = scenario.script();
    let trace = run_native_script_for_scenario(manifest, &script, scenario);
    let final_frame = trace
        .last()
        .ok_or_else(|| anyhow!("native trace was empty for scenario {}", scenario.slug()))?;

    println!("scenario: {}", scenario.slug());
    println!("frames: {}", trace.len());
    println!(
        "final mode: {:?} (0x{:04X})",
        final_frame.game_mode, final_frame.game_mode_value
    );
    println!("current level: 0x{:02X}", final_frame.current_level.0);
    if let Some(level_name) = &final_frame.current_level_name {
        println!("current level name: {level_name}");
    }
    if let (Some(player_start_x), Some(player_start_y)) =
        (final_frame.player_start_x, final_frame.player_start_y)
    {
        println!("player start: {player_start_x},{player_start_y}");
    }
    if let (Some(flag_x), Some(flag_y)) = (final_frame.flag_x, final_frame.flag_y) {
        println!("flag: {flag_x},{flag_y}");
    }
    if let Some(level_enemy_object_count) = final_frame.level_enemy_object_count {
        println!("level enemy objects: {level_enemy_object_count}");
    }
    println!(
        "active enemy objects: {}",
        final_frame.active_enemy_object_count
    );
    println!(
        "active platform objects: {}",
        final_frame.active_platform_object_count
    );
    if let (Some(kid_x_pos), Some(kid_y_pos)) = (final_frame.kid_x_pos, final_frame.kid_y_pos) {
        println!("kid pos: {kid_x_pos:08X},{kid_y_pos:08X}");
    }
    println!("active roster: {}", describe_active_objects(final_frame));

    for checkpoint in scenario.checkpoints() {
        if let Some(frame) =
            native_trace_at_checkpoint(&trace, checkpoint.label, checkpoint.frame_index)
        {
            println!(
                "checkpoint {}@{}: {:?} level 0x{:02X} {} camera {:08X},{:08X} objects {}/{} active_enemies {} active_platforms {} roster {}",
                checkpoint.label,
                checkpoint.frame_index,
                frame.game_mode,
                frame.current_level.0,
                frame.current_level_name.as_deref().unwrap_or(""),
                frame.camera_x_pos,
                frame.camera_y_pos,
                frame.number_objects,
                frame.number_gfx_objects,
                frame.active_enemy_object_count,
                frame.active_platform_object_count,
                describe_active_objects(frame)
            );
        }
    }

    print_trace_frames_in_range(&trace, &script, false, true, frame_start, frame_end);

    Ok(())
}

pub(crate) fn probe_native_gameplay_scenario(
    manifest: &AssetManifest,
    scenario: GameplayScenario,
    trace_physics: bool,
    frame_start: Option<usize>,
    frame_end: Option<usize>,
) -> Result<()> {
    let script = scenario.script();
    let trace = if trace_physics {
        run_native_script_for_gameplay_with_trace(manifest, &script, scenario)
    } else {
        run_native_script_for_gameplay(manifest, &script, scenario)
    };
    let final_frame = trace.last().ok_or_else(|| {
        anyhow!(
            "native trace was empty for gameplay scenario {}",
            scenario.slug()
        )
    })?;

    println!("scenario: {}", scenario.slug());
    println!("frames: {}", trace.len());
    println!(
        "final mode: {:?} (0x{:04X})",
        final_frame.game_mode, final_frame.game_mode_value
    );
    println!("current level: 0x{:02X}", final_frame.current_level.0);
    if let Some(level_name) = &final_frame.current_level_name {
        println!("current level name: {level_name}");
    }
    println!(
        "lives/hp: {}/{}",
        final_frame.number_lives, final_frame.number_hitpoints
    );
    println!(
        "active enemy objects: {}",
        final_frame.active_enemy_object_count
    );
    println!(
        "active platform objects: {}",
        final_frame.active_platform_object_count
    );
    if let (Some(kid_x_pos), Some(kid_y_pos)) = (final_frame.kid_x_pos, final_frame.kid_y_pos) {
        println!("kid pos: {kid_x_pos:08X},{kid_y_pos:08X}");
    }
    println!(
        "camera: {:08X},{:08X}",
        final_frame.camera_x_pos, final_frame.camera_y_pos
    );
    println!("active roster: {}", describe_active_objects(final_frame));

    for checkpoint in scenario.checkpoints() {
        if let Some(frame) =
            native_trace_at_checkpoint(&trace, checkpoint.label, checkpoint.frame_index)
        {
            println!(
                "checkpoint {}@{}: {:?} level 0x{:02X} {} rng {:04X} camera {:08X},{:08X} kid {:08X},{:08X} death_windup {} lives/hp {}/{} objects {}/{} active_enemies {} active_platforms {} roster {}",
                checkpoint.label,
                checkpoint.frame_index,
                frame.game_mode,
                frame.current_level.0,
                frame.current_level_name.as_deref().unwrap_or(""),
                frame.rng_offset.unwrap_or_default(),
                frame.camera_x_pos,
                frame.camera_y_pos,
                frame.kid_x_pos.unwrap_or_default(),
                frame.kid_y_pos.unwrap_or_default(),
                frame.death_windup_elapsed,
                frame.number_lives,
                frame.number_hitpoints,
                frame.number_objects,
                frame.number_gfx_objects,
                frame.active_enemy_object_count,
                frame.active_platform_object_count,
                describe_active_objects(frame)
            );
        }
    }

    print_trace_frames_in_range(&trace, &script, trace_physics, true, frame_start, frame_end);

    Ok(())
}

pub(crate) fn probe_dynamic_record_scenario(
    manifest: &AssetManifest,
    scenario: &DynamicScenario,
    scenario_path: Option<&Path>,
    trace_physics: bool,
    debug_fields: bool,
    target_enemy: Option<u8>,
    target_projectile: Option<ActiveProjectileKind>,
    target_projectile_directions: &[BatchTargetProjectileDirection],
    player_motion: bool,
    target_points: &[TargetPoint],
    target_cells: &[TargetCell],
    target_cell_hitboxes: &[TargetCell],
    target_projectile_cells: &[(ActiveProjectileKind, TargetCell)],
    target_projectile_cell_conjunctions: &[(ActiveProjectileKind, TargetCell)],
    target_block_events: &[BatchTargetBlockEvent],
    target_enemy_contact: Option<u8>,
    summary_only: bool,
    frame_start: Option<usize>,
    frame_end: Option<usize>,
) -> Result<()> {
    let trace = run_native_dynamic_scenario(
        manifest.clone(),
        scenario,
        NativeDynamicRunOptions {
            settled_warp: false,
            trace_physics,
        },
    )?;
    let final_frame = trace.last().ok_or_else(|| {
        anyhow!(
            "native trace was empty for dynamic scenario {}",
            scenario.slug
        )
    })?;

    println!("scenario: {}", scenario.slug);
    println!("frames: {}", trace.len());
    println!(
        "final mode: {:?} (0x{:04X})",
        final_frame.game_mode, final_frame.game_mode_value
    );
    println!("current level: 0x{:02X}", final_frame.current_level.0);
    if let Some(level_name) = &final_frame.current_level_name {
        println!("current level name: {level_name}");
    }
    println!(
        "lives/hp: {}/{}",
        final_frame.number_lives, final_frame.number_hitpoints
    );
    println!(
        "active enemy objects: {}",
        final_frame.active_enemy_object_count
    );
    println!(
        "active platform objects: {}",
        final_frame.active_platform_object_count
    );
    if let (Some(kid_x_pos), Some(kid_y_pos)) = (final_frame.kid_x_pos, final_frame.kid_y_pos) {
        println!("kid pos: {kid_x_pos:08X},{kid_y_pos:08X}");
    }
    println!(
        "camera: {:08X},{:08X}",
        final_frame.camera_x_pos, final_frame.camera_y_pos
    );
    println!("active roster: {}", describe_active_objects(final_frame));
    let target_frame_start = frame_start.unwrap_or(0);
    if let Some(object_type) = target_enemy {
        println!("{}", describe_target_enemy_activation(&trace, object_type));
    }
    if let Some(kind) = target_projectile {
        println!("{}", describe_target_projectile_activation(&trace, kind));
    }
    for target in target_projectile_directions {
        println!(
            "{}",
            describe_target_projectile_directional_kid_approach_from_frame(
                &trace,
                target.kind,
                target.direction,
                true,
                target_frame_start,
            )
        );
    }
    if player_motion {
        println!("{}", describe_player_motion(&trace));
    }
    for target_point in target_points {
        println!(
            "{}",
            describe_target_point_approach_from_frame(&trace, *target_point, target_frame_start)
        );
    }
    for target_cell in target_cells {
        println!(
            "{}",
            describe_target_cell_approach_from_frame(&trace, *target_cell, target_frame_start)
        );
    }
    for target_cell in target_cell_hitboxes {
        println!(
            "{}",
            describe_target_cell_hitbox_approach_from_frame(
                &trace,
                *target_cell,
                target_frame_start
            )
        );
    }
    for &(kind, target_cell) in target_projectile_cells {
        println!(
            "{}",
            describe_target_projectile_cell_approach_from_frame(
                &trace,
                kind,
                target_cell,
                target_frame_start
            )
        );
    }
    for &(kind, target_cell) in target_projectile_cell_conjunctions {
        println!(
            "{}",
            describe_target_projectile_cell_conjunction_from_frame(
                &trace,
                kind,
                target_cell,
                target_frame_start
            )
        );
    }
    for target_block_event in target_block_events {
        let event = target_block_event_from_frame(
            &trace,
            target_block_event.kind,
            target_block_event.cell,
            target_frame_start,
        );
        match event {
            Some(event) => println!(
                "block {:?} cell {},{} @{}",
                target_block_event.kind,
                target_block_event.cell.col,
                target_block_event.cell.row,
                event.replay_frame,
            ),
            None => println!(
                "block {:?} cell {},{} <none>",
                target_block_event.kind, target_block_event.cell.col, target_block_event.cell.row,
            ),
        }
    }
    if let Some(object_type) = target_enemy_contact {
        if let Some(contact) =
            target_enemy_contact_from_frame(&trace, object_type, target_frame_start)
        {
            println!("{}", describe_target_enemy_contact(object_type, &contact));
            if let Some(path) = scenario_path {
                println!("contact visual follow-up:");
                for command in target_enemy_contact_visual_follow_up_commands(path, &contact) {
                    println!("{command}");
                }
            }
        } else {
            println!("enemy contact 0x{object_type:02X}: <none>");
        }
    }

    for checkpoint in &scenario.checkpoints {
        if let Some(frame) =
            native_trace_at_checkpoint(&trace, &checkpoint.label, checkpoint.frame_index)
        {
            println!(
                "checkpoint {}@{}: {:?} level 0x{:02X} {} rng {:04X} camera {:08X},{:08X} kid {:08X},{:08X} death_windup {} lives/hp {}/{} objects {}/{} active_enemies {} active_platforms {} roster {}",
                checkpoint.label,
                checkpoint.frame_index,
                frame.game_mode,
                frame.current_level.0,
                frame.current_level_name.as_deref().unwrap_or(""),
                frame.rng_offset.unwrap_or_default(),
                frame.camera_x_pos,
                frame.camera_y_pos,
                frame.kid_x_pos.unwrap_or_default(),
                frame.kid_y_pos.unwrap_or_default(),
                frame.death_windup_elapsed,
                frame.number_lives,
                frame.number_hitpoints,
                frame.number_objects,
                frame.number_gfx_objects,
                frame.active_enemy_object_count,
                frame.active_platform_object_count,
                describe_active_objects(frame)
            );
        }
    }

    if !summary_only {
        print_trace_frames_in_range(
            &trace,
            &scenario.script,
            trace_physics,
            debug_fields,
            frame_start,
            frame_end,
        );
    }

    Ok(())
}

pub(crate) fn describe_frame_input(input: wildside_core::FrameInput) -> String {
    let mut pressed = Vec::new();
    if input.up {
        pressed.push("U");
    }
    if input.down {
        pressed.push("D");
    }
    if input.left {
        pressed.push("L");
    }
    if input.right {
        pressed.push("R");
    }
    if input.a {
        pressed.push("A");
    }
    if input.b {
        pressed.push("B");
    }
    if input.c {
        pressed.push("C");
    }
    if input.start {
        pressed.push("Start");
    }
    if pressed.is_empty() {
        String::from("<none>")
    } else {
        pressed.join("+")
    }
}

pub(crate) fn describe_active_objects(frame: &NativeTraceFrame) -> String {
    if frame.active_objects.is_empty() {
        return String::from("<none>");
    }

    frame
        .active_objects
        .iter()
        .map(|object| {
            let mut summary = match object.kind {
                wildside_core::NativeObjectKind::Player => {
                    format!("player@{:08X},{:08X}", object.x_pos, object.y_pos)
                }
                wildside_core::NativeObjectKind::Enemy { object_type } => {
                    format!(
                        "enemy_{object_type:02X}@{:08X},{:08X}",
                        object.x_pos, object.y_pos
                    )
                }
                wildside_core::NativeObjectKind::Platform { kind, subtype } => {
                    format!(
                        "platform_{kind:02X}_{subtype:02X}@{:08X},{:08X}",
                        object.x_pos, object.y_pos
                    )
                }
            };
            if !object.debug_fields.is_empty() {
                summary.push('{');
                summary.push_str(&format_debug_fields(&object.debug_fields));
                summary.push('}');
            }
            summary
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn describe_target_enemy_details(frame: &NativeTraceFrame, object_type: u8) -> String {
    let details = frame
        .active_enemy_details
        .iter()
        .filter_map(|object| {
            let wildside_core::NativeObjectKind::Enemy {
                object_type: active_type,
            } = object.kind
            else {
                return None;
            };
            (active_type == object_type).then(|| {
                format!(
                    "enemy_{active_type:02X}@{:08X},{:08X}/layout{}",
                    object.x_pos,
                    object.y_pos,
                    object
                        .layout_index
                        .map_or_else(|| String::from("?"), |layout| layout.to_string())
                )
            })
        })
        .collect::<Vec<_>>();
    if details.is_empty() {
        String::from("<none>")
    } else {
        details.join(", ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TargetEnemyActivation {
    first_replay_frame: usize,
    last_replay_frame: usize,
    frame_count: usize,
    sighting_count: usize,
    min_x: u32,
    max_x: u32,
    min_y: u32,
    max_y: u32,
    layout_indices: Vec<usize>,
}

fn target_enemy_activation(
    trace: &[NativeTraceFrame],
    object_type: u8,
) -> Option<TargetEnemyActivation> {
    let mut activation: Option<TargetEnemyActivation> = None;
    let mut previous_active_frame = None;
    let mut layout_indices = std::collections::BTreeSet::new();

    for (replay_frame, frame) in trace.iter().enumerate() {
        if !matches!(frame.game_mode, wildside_core::GameMode::InGame) {
            continue;
        }

        let mut sightings_this_frame = 0usize;
        for object in &frame.active_enemy_details {
            let wildside_core::NativeObjectKind::Enemy {
                object_type: active_type,
            } = object.kind
            else {
                continue;
            };
            if active_type != object_type {
                continue;
            }

            sightings_this_frame = sightings_this_frame.saturating_add(1);
            if let Some(layout_index) = object.layout_index {
                layout_indices.insert(layout_index);
            }

            match &mut activation {
                Some(existing) => {
                    existing.last_replay_frame = replay_frame;
                    existing.sighting_count = existing.sighting_count.saturating_add(1);
                    existing.min_x = existing.min_x.min(object.x_pos);
                    existing.max_x = existing.max_x.max(object.x_pos);
                    existing.min_y = existing.min_y.min(object.y_pos);
                    existing.max_y = existing.max_y.max(object.y_pos);
                }
                None => {
                    activation = Some(TargetEnemyActivation {
                        first_replay_frame: replay_frame,
                        last_replay_frame: replay_frame,
                        frame_count: 0,
                        sighting_count: 1,
                        min_x: object.x_pos,
                        max_x: object.x_pos,
                        min_y: object.y_pos,
                        max_y: object.y_pos,
                        layout_indices: Vec::new(),
                    });
                }
            }
        }

        if sightings_this_frame > 0 && previous_active_frame != Some(replay_frame) {
            if let Some(existing) = &mut activation {
                existing.frame_count = existing.frame_count.saturating_add(1);
            }
            previous_active_frame = Some(replay_frame);
        }
    }

    activation.map(|mut activation| {
        activation.layout_indices = layout_indices.into_iter().collect();
        activation
    })
}

pub(crate) fn describe_target_enemy_activation(
    trace: &[NativeTraceFrame],
    object_type: u8,
) -> String {
    let Some(activation) = target_enemy_activation(trace, object_type) else {
        return format!("target enemy 0x{object_type:02X}: absent");
    };
    let layouts = if activation.layout_indices.is_empty() {
        String::from("<none>")
    } else {
        activation
            .layout_indices
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    };
    format!(
        "target enemy 0x{object_type:02X}: first replay frame {} last replay frame {} active frames {} sightings {} x {:08X}..{:08X} y {:08X}..{:08X} layouts {} first details [{}]",
        activation.first_replay_frame,
        activation.last_replay_frame,
        activation.frame_count,
        activation.sighting_count,
        activation.min_x,
        activation.max_x,
        activation.min_y,
        activation.max_y,
        layouts,
        trace.get(activation.first_replay_frame).map_or_else(
            || String::from("<none>"),
            |frame| describe_target_enemy_details(frame, object_type),
        ),
    )
}

fn describe_target_projectile_details(
    frame: &NativeTraceFrame,
    kind: ActiveProjectileKind,
) -> String {
    let details = frame
        .active_projectile_details
        .iter()
        .filter(|projectile| projectile.kind == kind)
        .map(|projectile| {
            format!(
                "{:?}@{:08X},{:08X}/vel{:08X},{:08X}/remaining{}",
                projectile.kind,
                projectile.x_pos,
                projectile.y_pos,
                projectile.x_velocity,
                projectile.y_velocity,
                projectile.remaining_frames
            )
        })
        .collect::<Vec<_>>();
    if details.is_empty() {
        String::from("<none>")
    } else {
        details.join(", ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TargetProjectileActivation {
    first_replay_frame: usize,
    last_replay_frame: usize,
    frame_count: usize,
    sighting_count: usize,
    min_x: u32,
    max_x: u32,
    min_y: u32,
    max_y: u32,
    min_y_velocity: i32,
    max_y_velocity: i32,
    closest_approach: Option<TargetProjectileClosestApproach>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TargetProjectileClosestApproach {
    pub(crate) replay_frame: usize,
    pub(crate) dx_px: i64,
    pub(crate) dy_px: i64,
    pub(crate) manhattan_px: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TargetProjectileDirectionalApproach {
    pub(crate) replay_frame: usize,
    pub(crate) projectile_x: u32,
    pub(crate) projectile_y: u32,
    pub(crate) projectile_x_velocity: i32,
    pub(crate) projectile_y_velocity: i32,
    pub(crate) dx_px: i64,
    pub(crate) dy_px: i64,
    pub(crate) manhattan_px: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TargetProjectileCellApproach {
    pub(crate) replay_frame: usize,
    pub(crate) projectile_x: u32,
    pub(crate) projectile_y: u32,
    pub(crate) dx_px: i64,
    pub(crate) dy_px: i64,
    pub(crate) manhattan_px: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TargetProjectileCellConjunction {
    pub(crate) replay_frame: usize,
    pub(crate) kid_x: u32,
    pub(crate) kid_y: u32,
    pub(crate) projectile_x: u32,
    pub(crate) projectile_y: u32,
    pub(crate) kid_dx_px: i64,
    pub(crate) kid_dy_px: i64,
    pub(crate) projectile_dx_px: i64,
    pub(crate) projectile_dy_px: i64,
    pub(crate) manhattan_px: i64,
}

fn fixed_delta_px(to: u32, from: u32) -> i64 {
    (i64::from(to) - i64::from(from)) >> 16
}

fn target_projectile_closest_approach(
    replay_frame: usize,
    player_x: Option<u32>,
    player_y: Option<u32>,
    projectile: &ActiveProjectileSnapshot,
) -> Option<TargetProjectileClosestApproach> {
    let (Some(player_x), Some(player_y)) = (player_x, player_y) else {
        return None;
    };
    let dx_px = fixed_delta_px(projectile.x_pos, player_x);
    let dy_px = fixed_delta_px(projectile.y_pos, player_y);
    Some(TargetProjectileClosestApproach {
        replay_frame,
        dx_px,
        dy_px,
        manhattan_px: dx_px.abs().saturating_add(dy_px.abs()),
    })
}

fn projectile_matches_direction(
    projectile: &ActiveProjectileSnapshot,
    direction: TargetProjectileDirection,
) -> bool {
    match direction {
        TargetProjectileDirection::Left => projectile.x_velocity < 0,
        TargetProjectileDirection::Right => projectile.x_velocity > 0,
        TargetProjectileDirection::Up => projectile.y_velocity < 0,
        TargetProjectileDirection::Down => projectile.y_velocity > 0,
        TargetProjectileDirection::Stationary => {
            projectile.x_velocity == 0 && projectile.y_velocity == 0
        }
    }
}

fn target_projectile_directional_closest_approach(
    replay_frame: usize,
    player_x: Option<u32>,
    player_y: Option<u32>,
    projectile: &ActiveProjectileSnapshot,
) -> Option<TargetProjectileDirectionalApproach> {
    let closest = target_projectile_closest_approach(replay_frame, player_x, player_y, projectile)?;
    Some(TargetProjectileDirectionalApproach {
        replay_frame,
        projectile_x: projectile.x_pos,
        projectile_y: projectile.y_pos,
        projectile_x_velocity: projectile.x_velocity,
        projectile_y_velocity: projectile.y_velocity,
        dx_px: closest.dx_px,
        dy_px: closest.dy_px,
        manhattan_px: closest.manhattan_px,
    })
}

pub(crate) fn target_projectile_directional_kid_approach_from_frame(
    trace: &[NativeTraceFrame],
    kind: ActiveProjectileKind,
    direction: TargetProjectileDirection,
    require_healthy_kid: bool,
    frame_start: usize,
) -> Option<TargetProjectileDirectionalApproach> {
    let mut closest: Option<TargetProjectileDirectionalApproach> = None;

    for (replay_frame, frame) in trace.iter().enumerate() {
        if replay_frame < frame_start {
            continue;
        }
        if !matches!(frame.game_mode, wildside_core::GameMode::InGame) {
            continue;
        }
        if require_healthy_kid && (frame.number_hitpoints == 0 || frame.death_windup_elapsed > 0) {
            continue;
        }
        for projectile in &frame.active_projectile_details {
            if projectile.kind != kind || !projectile_matches_direction(projectile, direction) {
                continue;
            }
            let candidate = target_projectile_directional_closest_approach(
                replay_frame,
                frame.kid_x_pos,
                frame.kid_y_pos,
                projectile,
            );
            match (&closest, candidate) {
                (_, None) => {}
                (Some(existing), Some(candidate))
                    if existing.manhattan_px <= candidate.manhattan_px => {}
                (_, Some(candidate)) => closest = Some(candidate),
            }
        }
    }

    closest
}

pub(crate) fn describe_target_projectile_directional_kid_approach_from_frame(
    trace: &[NativeTraceFrame],
    kind: ActiveProjectileKind,
    direction: TargetProjectileDirection,
    require_healthy_kid: bool,
    frame_start: usize,
) -> String {
    target_projectile_directional_kid_approach_from_frame(
        trace,
        kind,
        direction,
        require_healthy_kid,
        frame_start,
    ).map_or_else(|| {
        format!("target projectile {kind:?} direction {direction:?} healthy kid <none>")
    }, |approach| {
        let health = if require_healthy_kid {
            "healthy kid"
        } else {
            "kid"
        };
        format!(
            "target projectile {kind:?} direction {direction:?} {health} dx_px {:+} dy_px {:+} manhattan_px {} @{} projectile {:08X},{:08X} vel {:08X},{:08X}",
            approach.dx_px,
            approach.dy_px,
            approach.manhattan_px,
            approach.replay_frame,
            approach.projectile_x,
            approach.projectile_y,
            approach.projectile_x_velocity,
            approach.projectile_y_velocity,
        )
    })
}

pub(crate) fn target_projectile_kid_approach_from_frame(
    trace: &[NativeTraceFrame],
    kind: ActiveProjectileKind,
    require_healthy_kid: bool,
    frame_start: usize,
) -> Option<TargetProjectileClosestApproach> {
    let mut closest = None;

    for (replay_frame, frame) in trace.iter().enumerate() {
        if replay_frame < frame_start {
            continue;
        }
        if !matches!(frame.game_mode, wildside_core::GameMode::InGame) {
            continue;
        }
        if require_healthy_kid && (frame.number_hitpoints == 0 || frame.death_windup_elapsed > 0) {
            continue;
        }
        for projectile in &frame.active_projectile_details {
            if projectile.kind != kind {
                continue;
            }
            retain_closest_projectile_approach(
                &mut closest,
                target_projectile_closest_approach(
                    replay_frame,
                    frame.kid_x_pos,
                    frame.kid_y_pos,
                    projectile,
                ),
            );
        }
    }

    closest
}

fn retain_closest_projectile_approach(
    closest: &mut Option<TargetProjectileClosestApproach>,
    candidate: Option<TargetProjectileClosestApproach>,
) {
    let Some(candidate) = candidate else {
        return;
    };
    match closest {
        Some(existing) if existing.manhattan_px <= candidate.manhattan_px => {}
        _ => *closest = Some(candidate),
    }
}

fn target_projectile_activation(
    trace: &[NativeTraceFrame],
    kind: ActiveProjectileKind,
) -> Option<TargetProjectileActivation> {
    let mut activation: Option<TargetProjectileActivation> = None;
    let mut previous_active_frame = None;

    for (replay_frame, frame) in trace.iter().enumerate() {
        if !matches!(frame.game_mode, wildside_core::GameMode::InGame) {
            continue;
        }

        let mut sightings_this_frame = 0usize;
        for projectile in &frame.active_projectile_details {
            if projectile.kind != kind {
                continue;
            }

            sightings_this_frame = sightings_this_frame.saturating_add(1);
            match &mut activation {
                Some(existing) => {
                    existing.last_replay_frame = replay_frame;
                    existing.sighting_count = existing.sighting_count.saturating_add(1);
                    existing.min_x = existing.min_x.min(projectile.x_pos);
                    existing.max_x = existing.max_x.max(projectile.x_pos);
                    existing.min_y = existing.min_y.min(projectile.y_pos);
                    existing.max_y = existing.max_y.max(projectile.y_pos);
                    existing.min_y_velocity = existing.min_y_velocity.min(projectile.y_velocity);
                    existing.max_y_velocity = existing.max_y_velocity.max(projectile.y_velocity);
                    retain_closest_projectile_approach(
                        &mut existing.closest_approach,
                        target_projectile_closest_approach(
                            replay_frame,
                            frame.kid_x_pos,
                            frame.kid_y_pos,
                            projectile,
                        ),
                    );
                }
                None => {
                    activation = Some(TargetProjectileActivation {
                        first_replay_frame: replay_frame,
                        last_replay_frame: replay_frame,
                        frame_count: 0,
                        sighting_count: 1,
                        min_x: projectile.x_pos,
                        max_x: projectile.x_pos,
                        min_y: projectile.y_pos,
                        max_y: projectile.y_pos,
                        min_y_velocity: projectile.y_velocity,
                        max_y_velocity: projectile.y_velocity,
                        closest_approach: target_projectile_closest_approach(
                            replay_frame,
                            frame.kid_x_pos,
                            frame.kid_y_pos,
                            projectile,
                        ),
                    });
                }
            }
        }

        if sightings_this_frame > 0 && previous_active_frame != Some(replay_frame) {
            if let Some(existing) = &mut activation {
                existing.frame_count = existing.frame_count.saturating_add(1);
            }
            previous_active_frame = Some(replay_frame);
        }
    }

    activation
}

pub(crate) fn describe_target_projectile_activation(
    trace: &[NativeTraceFrame],
    kind: ActiveProjectileKind,
) -> String {
    let Some(activation) = target_projectile_activation(trace, kind) else {
        return format!("target projectile {kind:?}: absent");
    };
    let closest = activation.closest_approach.map_or_else(
        || String::from("closest kid <none>"),
        |closest| {
            format!(
                "closest kid dx_px {:+} dy_px {:+} manhattan_px {} @{}",
                closest.dx_px, closest.dy_px, closest.manhattan_px, closest.replay_frame
            )
        },
    );
    format!(
        "target projectile {kind:?}: first replay frame {} last replay frame {} active frames {} sightings {} x {:08X}..{:08X} y {:08X}..{:08X} y_vel {:08X}..{:08X} {} first details [{}]",
        activation.first_replay_frame,
        activation.last_replay_frame,
        activation.frame_count,
        activation.sighting_count,
        activation.min_x,
        activation.max_x,
        activation.min_y,
        activation.max_y,
        activation.min_y_velocity,
        activation.max_y_velocity,
        closest,
        trace.get(activation.first_replay_frame).map_or_else(
            || String::from("<none>"),
            |frame| describe_target_projectile_details(frame, kind),
        ),
    )
}

pub(crate) fn target_projectile_cell_approach_from_frame(
    trace: &[NativeTraceFrame],
    kind: ActiveProjectileKind,
    target_cell: TargetCell,
    frame_start: usize,
) -> Option<TargetProjectileCellApproach> {
    let left_px = target_cell.col << 4;
    let right_px = left_px + 15;
    let top_px = target_cell.row << 4;
    let bottom_px = top_px + 15;
    let mut closest: Option<TargetProjectileCellApproach> = None;

    for (replay_frame, frame) in trace.iter().enumerate() {
        if replay_frame < frame_start {
            continue;
        }
        if !matches!(frame.game_mode, wildside_core::GameMode::InGame) {
            continue;
        }
        for projectile in &frame.active_projectile_details {
            if projectile.kind != kind {
                continue;
            }
            let projectile_x_px = i64::from(projectile.x_pos) >> 16;
            let projectile_y_px = i64::from(projectile.y_pos) >> 16;
            let dx_px = axis_distance_to_closed_range(projectile_x_px, left_px, right_px);
            let dy_px = axis_distance_to_closed_range(projectile_y_px, top_px, bottom_px);
            let candidate = TargetProjectileCellApproach {
                replay_frame,
                projectile_x: projectile.x_pos,
                projectile_y: projectile.y_pos,
                dx_px,
                dy_px,
                manhattan_px: dx_px.abs().saturating_add(dy_px.abs()),
            };
            match &closest {
                Some(existing)
                    if existing.manhattan_px < candidate.manhattan_px
                        || (existing.manhattan_px == candidate.manhattan_px
                            && existing.replay_frame <= candidate.replay_frame) => {}
                _ => closest = Some(candidate),
            }
        }
    }

    closest
}

pub(crate) fn describe_target_projectile_cell_approach_from_frame(
    trace: &[NativeTraceFrame],
    kind: ActiveProjectileKind,
    target_cell: TargetCell,
    frame_start: usize,
) -> String {
    target_projectile_cell_approach_from_frame(trace, kind, target_cell, frame_start).map_or_else(|| {
            format!(
                "target projectile {kind:?} cell {},{} <none>",
                target_cell.col, target_cell.row
            )
        }, |approach| {
            format!(
                "target projectile {kind:?} cell {},{} dx_px {:+} dy_px {:+} manhattan_px {} @{} projectile {:08X},{:08X}",
                target_cell.col,
                target_cell.row,
                approach.dx_px,
                approach.dy_px,
                approach.manhattan_px,
                approach.replay_frame,
                approach.projectile_x,
                approach.projectile_y,
            )
        })
}

pub(crate) fn target_projectile_cell_conjunction_from_frame(
    trace: &[NativeTraceFrame],
    kind: ActiveProjectileKind,
    target_cell: TargetCell,
    frame_start: usize,
) -> Option<TargetProjectileCellConjunction> {
    let left_px = target_cell.col << 4;
    let right_px = left_px + 15;
    let top_px = target_cell.row << 4;
    let bottom_px = top_px + 15;
    let mut closest: Option<TargetProjectileCellConjunction> = None;

    for (replay_frame, frame) in trace.iter().enumerate() {
        if replay_frame < frame_start {
            continue;
        }
        if !matches!(frame.game_mode, wildside_core::GameMode::InGame) {
            continue;
        }
        if frame.number_hitpoints == 0 || frame.death_windup_elapsed > 0 {
            continue;
        }
        let (Some(kid_x), Some(kid_y)) = (frame.kid_x_pos, frame.kid_y_pos) else {
            continue;
        };
        let kid_x_px = i64::from(kid_x) >> 16;
        let kid_y_px = i64::from(kid_y) >> 16;
        let half_width = half_width_for_helmet_id_loc_b408(frame.current_helmet_id);
        let height = player_height_for_helmet_id_loc_902a(frame.current_helmet_id);
        let kid_dx_px = range_distance_to_closed_range(
            kid_x_px - half_width,
            kid_x_px + half_width,
            left_px,
            right_px,
        );
        let kid_dy_px =
            range_distance_to_closed_range(kid_y_px - height, kid_y_px, top_px, bottom_px);

        for projectile in &frame.active_projectile_details {
            if projectile.kind != kind {
                continue;
            }
            let projectile_x_px = i64::from(projectile.x_pos) >> 16;
            let projectile_y_px = i64::from(projectile.y_pos) >> 16;
            let projectile_dx_px =
                axis_distance_to_closed_range(projectile_x_px, left_px, right_px);
            let projectile_dy_px =
                axis_distance_to_closed_range(projectile_y_px, top_px, bottom_px);
            let candidate = TargetProjectileCellConjunction {
                replay_frame,
                kid_x,
                kid_y,
                projectile_x: projectile.x_pos,
                projectile_y: projectile.y_pos,
                kid_dx_px,
                kid_dy_px,
                projectile_dx_px,
                projectile_dy_px,
                manhattan_px: kid_dx_px
                    .abs()
                    .saturating_add(kid_dy_px.abs())
                    .saturating_add(projectile_dx_px.abs())
                    .saturating_add(projectile_dy_px.abs()),
            };
            match &closest {
                Some(existing)
                    if existing.manhattan_px < candidate.manhattan_px
                        || (existing.manhattan_px == candidate.manhattan_px
                            && existing.replay_frame <= candidate.replay_frame) => {}
                _ => closest = Some(candidate),
            }
        }
    }

    closest
}

pub(crate) fn describe_target_projectile_cell_conjunction_from_frame(
    trace: &[NativeTraceFrame],
    kind: ActiveProjectileKind,
    target_cell: TargetCell,
    frame_start: usize,
) -> String {
    target_projectile_cell_conjunction_from_frame(trace, kind, target_cell, frame_start).map_or_else(|| {
            format!(
                "target projectile {kind:?} cell conjunction {},{} <none>",
                target_cell.col, target_cell.row
            )
        }, |approach| {
            format!(
                "target projectile {kind:?} cell conjunction {},{} kid_dx_px {:+} kid_dy_px {:+} projectile_dx_px {:+} projectile_dy_px {:+} manhattan_px {} @{} kid {:08X},{:08X} projectile {:08X},{:08X}",
                target_cell.col,
                target_cell.row,
                approach.kid_dx_px,
                approach.kid_dy_px,
                approach.projectile_dx_px,
                approach.projectile_dy_px,
                approach.manhattan_px,
                approach.replay_frame,
                approach.kid_x,
                approach.kid_y,
                approach.projectile_x,
                approach.projectile_y,
            )
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlayerMotionSummary {
    first_replay_frame: usize,
    last_replay_frame: usize,
    frame_count: usize,
    min_x: u32,
    min_x_replay_frame: usize,
    max_x: u32,
    max_x_replay_frame: usize,
    min_y: u32,
    min_y_replay_frame: usize,
    max_y: u32,
    max_y_replay_frame: usize,
    last_x: u32,
    last_y: u32,
    transition_replay_frame: Option<usize>,
}

fn player_motion_summary(trace: &[NativeTraceFrame]) -> Option<PlayerMotionSummary> {
    let mut summary: Option<PlayerMotionSummary> = None;
    let mut transition_replay_frame = None;

    for (replay_frame, frame) in trace.iter().enumerate() {
        if transition_replay_frame.is_none()
            && !matches!(frame.game_mode, wildside_core::GameMode::InGame)
        {
            transition_replay_frame = Some(replay_frame);
        }

        if !matches!(frame.game_mode, wildside_core::GameMode::InGame) {
            continue;
        }
        let (Some(x), Some(y)) = (frame.kid_x_pos, frame.kid_y_pos) else {
            continue;
        };

        match &mut summary {
            Some(existing) => {
                existing.last_replay_frame = replay_frame;
                existing.frame_count = existing.frame_count.saturating_add(1);
                if x < existing.min_x {
                    existing.min_x = x;
                    existing.min_x_replay_frame = replay_frame;
                }
                if x > existing.max_x {
                    existing.max_x = x;
                    existing.max_x_replay_frame = replay_frame;
                }
                if y < existing.min_y {
                    existing.min_y = y;
                    existing.min_y_replay_frame = replay_frame;
                }
                if y > existing.max_y {
                    existing.max_y = y;
                    existing.max_y_replay_frame = replay_frame;
                }
                existing.last_x = x;
                existing.last_y = y;
            }
            None => {
                summary = Some(PlayerMotionSummary {
                    first_replay_frame: replay_frame,
                    last_replay_frame: replay_frame,
                    frame_count: 1,
                    min_x: x,
                    min_x_replay_frame: replay_frame,
                    max_x: x,
                    max_x_replay_frame: replay_frame,
                    min_y: y,
                    min_y_replay_frame: replay_frame,
                    max_y: y,
                    max_y_replay_frame: replay_frame,
                    last_x: x,
                    last_y: y,
                    transition_replay_frame,
                });
            }
        }
    }

    summary.map(|mut summary| {
        summary.transition_replay_frame = transition_replay_frame;
        summary
    })
}

pub(crate) fn describe_player_motion(trace: &[NativeTraceFrame]) -> String {
    let Some(summary) = player_motion_summary(trace) else {
        return String::from("player motion: absent");
    };
    let transition = summary
        .transition_replay_frame
        .map_or_else(|| String::from("<none>"), |frame| frame.to_string());
    format!(
        "player motion: first replay frame {} last replay frame {} live frames {} x {:08X}@{}..{:08X}@{} y {:08X}@{}..{:08X}@{} last live kid {:08X},{:08X} first non-InGame replay frame {}",
        summary.first_replay_frame,
        summary.last_replay_frame,
        summary.frame_count,
        summary.min_x,
        summary.min_x_replay_frame,
        summary.max_x,
        summary.max_x_replay_frame,
        summary.min_y,
        summary.min_y_replay_frame,
        summary.max_y,
        summary.max_y_replay_frame,
        summary.last_x,
        summary.last_y,
        transition,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TargetPointApproach {
    pub(crate) replay_frame: usize,
    pub(crate) kid_x: u32,
    pub(crate) kid_y: u32,
    pub(crate) dx_px: i64,
    pub(crate) dy_px: i64,
    pub(crate) manhattan_px: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TargetCellApproach {
    pub(crate) replay_frame: usize,
    pub(crate) kid_x: u32,
    pub(crate) kid_y: u32,
    pub(crate) dx_px: i64,
    pub(crate) dy_px: i64,
    pub(crate) manhattan_px: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TargetEnemyContactApproach {
    pub(crate) replay_frame: usize,
    pub(crate) score_px: i64,
    pub(crate) result: String,
    pub(crate) gate: String,
    pub(crate) summary: String,
    pub(crate) same_frame_stomp: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetCellProbe {
    Origin,
    PlayerHitbox,
}

pub(crate) fn target_point_approach_from_frame(
    trace: &[NativeTraceFrame],
    target_point: TargetPoint,
    require_healthy_kid: bool,
    frame_start: usize,
) -> Option<TargetPointApproach> {
    let target_x = target_point.x_px << 16;
    let target_y = target_point.y_px << 16;
    let mut closest: Option<TargetPointApproach> = None;

    for (replay_frame, frame) in trace.iter().enumerate() {
        if replay_frame < frame_start {
            continue;
        }
        if !matches!(frame.game_mode, wildside_core::GameMode::InGame) {
            continue;
        }
        if require_healthy_kid && (frame.number_hitpoints == 0 || frame.death_windup_elapsed > 0) {
            continue;
        }
        let (Some(kid_x), Some(kid_y)) = (frame.kid_x_pos, frame.kid_y_pos) else {
            continue;
        };
        let dx_px = (target_x - i64::from(kid_x)) >> 16;
        let dy_px = (target_y - i64::from(kid_y)) >> 16;
        let candidate = TargetPointApproach {
            replay_frame,
            kid_x,
            kid_y,
            dx_px,
            dy_px,
            manhattan_px: dx_px.abs().saturating_add(dy_px.abs()),
        };
        match &closest {
            Some(existing)
                if existing.manhattan_px < candidate.manhattan_px
                    || (existing.manhattan_px == candidate.manhattan_px
                        && existing.replay_frame <= candidate.replay_frame) => {}
            _ => closest = Some(candidate),
        }
    }

    closest
}

fn axis_distance_to_closed_range(value: i64, min: i64, max: i64) -> i64 {
    if value < min {
        min - value
    } else if value > max {
        max - value
    } else {
        0
    }
}

fn range_distance_to_closed_range(left: i64, right: i64, min: i64, max: i64) -> i64 {
    if right < min {
        min - right
    } else if left > max {
        max - left
    } else {
        0
    }
}

fn half_width_for_helmet_id_loc_b408(helmet_id: u16) -> i64 {
    // ASM unk_B408: player collision half-widths indexed by current_helmet_id.
    match helmet_id {
        1 => 10,
        5 => 23,
        9 => 4,
        0 => 5,
        _ => 7,
    }
}

fn player_height_for_helmet_id_loc_902a(helmet_id: u16) -> i64 {
    // ASM loc_902A ceiling/block probes use a shorter upper bound for Juggernaut/Micromax.
    match helmet_id {
        5 | 9 => 15,
        _ => 31,
    }
}

pub(crate) fn target_cell_approach_from_frame(
    trace: &[NativeTraceFrame],
    target_cell: TargetCell,
    require_healthy_kid: bool,
    frame_start: usize,
) -> Option<TargetCellApproach> {
    target_cell_approach_with_probe_from_frame(
        trace,
        target_cell,
        TargetCellProbe::Origin,
        require_healthy_kid,
        frame_start,
    )
}

pub(crate) fn target_cell_hitbox_approach_from_frame(
    trace: &[NativeTraceFrame],
    target_cell: TargetCell,
    require_healthy_kid: bool,
    frame_start: usize,
) -> Option<TargetCellApproach> {
    target_cell_approach_with_probe_from_frame(
        trace,
        target_cell,
        TargetCellProbe::PlayerHitbox,
        require_healthy_kid,
        frame_start,
    )
}

fn target_cell_approach_with_probe_from_frame(
    trace: &[NativeTraceFrame],
    target_cell: TargetCell,
    probe: TargetCellProbe,
    require_healthy_kid: bool,
    frame_start: usize,
) -> Option<TargetCellApproach> {
    let left_px = target_cell.col << 4;
    let right_px = left_px + 15;
    let top_px = target_cell.row << 4;
    let bottom_px = top_px + 15;
    let mut closest: Option<TargetCellApproach> = None;

    for (replay_frame, frame) in trace.iter().enumerate() {
        if replay_frame < frame_start {
            continue;
        }
        if !matches!(frame.game_mode, wildside_core::GameMode::InGame) {
            continue;
        }
        if require_healthy_kid && (frame.number_hitpoints == 0 || frame.death_windup_elapsed > 0) {
            continue;
        }
        let (Some(kid_x), Some(kid_y)) = (frame.kid_x_pos, frame.kid_y_pos) else {
            continue;
        };
        let kid_x_px = i64::from(kid_x) >> 16;
        let kid_y_px = i64::from(kid_y) >> 16;
        let (dx_px, dy_px) = match probe {
            TargetCellProbe::Origin => (
                axis_distance_to_closed_range(kid_x_px, left_px, right_px),
                axis_distance_to_closed_range(kid_y_px, top_px, bottom_px),
            ),
            TargetCellProbe::PlayerHitbox => {
                let half_width = half_width_for_helmet_id_loc_b408(frame.current_helmet_id);
                let height = player_height_for_helmet_id_loc_902a(frame.current_helmet_id);
                (
                    range_distance_to_closed_range(
                        kid_x_px - half_width,
                        kid_x_px + half_width,
                        left_px,
                        right_px,
                    ),
                    range_distance_to_closed_range(kid_y_px - height, kid_y_px, top_px, bottom_px),
                )
            }
        };
        let candidate = TargetCellApproach {
            replay_frame,
            kid_x,
            kid_y,
            dx_px,
            dy_px,
            manhattan_px: dx_px.abs().saturating_add(dy_px.abs()),
        };
        match &closest {
            Some(existing)
                if existing.manhattan_px < candidate.manhattan_px
                    || (existing.manhattan_px == candidate.manhattan_px
                        && existing.replay_frame <= candidate.replay_frame) => {}
            _ => closest = Some(candidate),
        }
    }

    closest
}

pub(crate) fn describe_target_cell_hitbox_approach_from_frame(
    trace: &[NativeTraceFrame],
    target_cell: TargetCell,
    frame_start: usize,
) -> String {
    let Some(approach) =
        target_cell_hitbox_approach_from_frame(trace, target_cell, false, frame_start)
    else {
        return format!(
            "target cell hitbox {},{}: absent",
            target_cell.col, target_cell.row
        );
    };
    let healthy = target_cell_hitbox_approach_from_frame(trace, target_cell, true, frame_start).map_or_else(|| String::from("; closest healthy kid hitbox <none>"), |healthy| {
            format!(
                "; closest healthy kid hitbox dx_px {:+} dy_px {:+} manhattan_px {} @{} kid {:08X},{:08X}",
                healthy.dx_px,
                healthy.dy_px,
                healthy.manhattan_px,
                healthy.replay_frame,
                healthy.kid_x,
                healthy.kid_y,
            )
        });
    format!(
        "target cell hitbox {},{}: closest kid hitbox dx_px {:+} dy_px {:+} manhattan_px {} @{} kid {:08X},{:08X}{}",
        target_cell.col,
        target_cell.row,
        approach.dx_px,
        approach.dy_px,
        approach.manhattan_px,
        approach.replay_frame,
        approach.kid_x,
        approach.kid_y,
        healthy
    )
}

pub(crate) fn describe_target_cell_approach_from_frame(
    trace: &[NativeTraceFrame],
    target_cell: TargetCell,
    frame_start: usize,
) -> String {
    let Some(approach) = target_cell_approach_from_frame(trace, target_cell, false, frame_start)
    else {
        return format!(
            "target cell {},{}: absent",
            target_cell.col, target_cell.row
        );
    };
    let healthy = target_cell_approach_from_frame(trace, target_cell, true, frame_start).map_or_else(|| String::from("; closest healthy kid <none>"), |healthy| {
            format!(
                "; closest healthy kid dx_px {:+} dy_px {:+} manhattan_px {} @{} kid {:08X},{:08X}",
                healthy.dx_px,
                healthy.dy_px,
                healthy.manhattan_px,
                healthy.replay_frame,
                healthy.kid_x,
                healthy.kid_y,
            )
        });
    format!(
        "target cell {},{}: closest kid dx_px {:+} dy_px {:+} manhattan_px {} @{} kid {:08X},{:08X}{}",
        target_cell.col,
        target_cell.row,
        approach.dx_px,
        approach.dy_px,
        approach.manhattan_px,
        approach.replay_frame,
        approach.kid_x,
        approach.kid_y,
        healthy
    )
}

pub(crate) fn describe_target_point_approach_from_frame(
    trace: &[NativeTraceFrame],
    target_point: TargetPoint,
    frame_start: usize,
) -> String {
    let Some(approach) = target_point_approach_from_frame(trace, target_point, false, frame_start)
    else {
        return format!(
            "target point {},{}: absent",
            target_point.x_px, target_point.y_px
        );
    };
    let healthy = target_point_approach_from_frame(trace, target_point, true, frame_start).map_or_else(|| String::from("; closest healthy kid <none>"), |healthy| {
            format!(
                "; closest healthy kid dx_px {:+} dy_px {:+} manhattan_px {} @{} kid {:08X},{:08X}",
                healthy.dx_px,
                healthy.dy_px,
                healthy.manhattan_px,
                healthy.replay_frame,
                healthy.kid_x,
                healthy.kid_y,
            )
        });
    format!(
        "target point {},{}: closest kid dx_px {:+} dy_px {:+} manhattan_px {} @{} kid {:08X},{:08X}{}",
        target_point.x_px,
        target_point.y_px,
        approach.dx_px,
        approach.dy_px,
        approach.manhattan_px,
        approach.replay_frame,
        approach.kid_x,
        approach.kid_y,
        healthy,
    )
}

pub(crate) fn target_enemy_contact_from_frame(
    trace: &[NativeTraceFrame],
    object_type: u8,
    frame_start: usize,
) -> Option<TargetEnemyContactApproach> {
    let label = format!("enemy_{object_type:02X}");
    let same_frame_label = format!("enemy=0x{object_type:02x}@");
    let mut best: Option<TargetEnemyContactApproach> = None;

    for (replay_frame, frame) in trace.iter().enumerate() {
        if replay_frame < frame_start {
            continue;
        }
        if !matches!(frame.game_mode, wildside_core::GameMode::InGame) {
            continue;
        }

        let same_frame_stomp = frame
            .gameplay_debug_fields
            .get("kid_same_frame_stomp_probe")
            .is_some_and(|summary| {
                summary.contains("candidate=1/") && summary.contains(&same_frame_label)
            });

        if let Some(field) = frame.gameplay_debug_fields.get("rock_tank_contact_probe") {
            for summary in field.split("; ").filter(|part| part.contains(&label)) {
                let Some(child_summary) = extract_rock_tank_child_contact_summary(summary) else {
                    continue;
                };
                if !child_summary.contains(&label) {
                    continue;
                }
                let display_summary = format!("rock_tank_contact_probe/{summary}");
                let candidate = target_enemy_contact_candidate(
                    replay_frame,
                    child_summary,
                    display_summary,
                    same_frame_stomp,
                );
                match &best {
                    Some(existing)
                        if enemy_contact_rank(existing) <= enemy_contact_rank(&candidate) => {}
                    _ => best = Some(candidate),
                }
            }
        }

        for key in ["kid_stomp_contact_candidates", "kid_collision_order_probe"] {
            let Some(field) = frame.gameplay_debug_fields.get(key) else {
                continue;
            };
            for summary in field.split("; ").filter(|part| part.contains(&label)) {
                let candidate = target_enemy_contact_candidate(
                    replay_frame,
                    summary,
                    summary.to_string(),
                    same_frame_stomp,
                );
                match &best {
                    Some(existing)
                        if enemy_contact_rank(existing) <= enemy_contact_rank(&candidate) => {}
                    _ => best = Some(candidate),
                }
            }
        }
    }

    best
}

fn target_enemy_contact_candidate(
    replay_frame: usize,
    rank_summary: &str,
    display_summary: String,
    same_frame_stomp: bool,
) -> TargetEnemyContactApproach {
    let score_px = enemy_contact_summary_score_px(rank_summary).unwrap_or(i64::MAX / 2);
    let result = extract_contact_summary_value(rank_summary, "result=")
        .unwrap_or("unknown")
        .to_string();
    let gate = extract_contact_summary_value(rank_summary, "gate=")
        .unwrap_or("unknown")
        .to_string();
    TargetEnemyContactApproach {
        replay_frame,
        score_px,
        result,
        gate,
        summary: display_summary,
        same_frame_stomp,
    }
}

#[cfg(test)]
fn describe_target_enemy_contact_from_frame(
    trace: &[NativeTraceFrame],
    object_type: u8,
    frame_start: usize,
) -> String {
    target_enemy_contact_from_frame(trace, object_type, frame_start).map_or_else(
        || format!("enemy contact 0x{object_type:02X}: <none>"),
        |contact| describe_target_enemy_contact(object_type, &contact),
    )
}

fn describe_target_enemy_contact(object_type: u8, contact: &TargetEnemyContactApproach) -> String {
    format!(
        "enemy contact 0x{object_type:02X}: result {} gate {} score_px {} @{} same_frame_stomp={} {}",
        contact.result,
        contact.gate,
        contact.score_px,
        contact.replay_frame,
        contact.same_frame_stomp,
        contact.summary,
    )
}

fn target_enemy_contact_visual_follow_up_commands(
    scenario_path: &Path,
    contact: &TargetEnemyContactApproach,
) -> Vec<String> {
    let start = contact.replay_frame.saturating_sub(1);
    let mut frames = vec![start, contact.replay_frame, contact.replay_frame + 1];
    frames.dedup();
    let frames = frames
        .into_iter()
        .map(|frame| frame.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let output_root = scenario_path.parent().map_or_else(
        || PathBuf::from(".cache/debug/contact-frame-evidence"),
        |parent| parent.join("contact-frame-evidence"),
    );
    let scenario = scenario_path.display();
    let output = output_root.display();
    vec![
        format!(
            "cargo run -p wildside-cli -- capture-scenario-frames --scenario {scenario} --frames {frames} --output {output}/native-vdp"
        ),
        format!(
            "cargo run -p wildside-cli -- capture-scenario-frames --scenario {scenario} --frames {frames} --scene --output {output}/native-scene"
        ),
        format!(
            "cargo run -p wildside-cli -- capture-scenario-frames --scenario {scenario} --frames {frames} --scene --layer enemies --output {output}/native-enemies"
        ),
        format!(
            "cargo run -p wildside-cli -- capture-oracle-scenario-frames --scenario {scenario} --frames {frames} --output {output}/oracle-vdp"
        ),
    ]
}

fn enemy_contact_rank(contact: &TargetEnemyContactApproach) -> (u8, i64, usize) {
    let result_rank = match contact.result.as_str() {
        "stomp" => 0,
        "damage" => 1,
        "none" => 2,
        _ => 3,
    };
    (result_rank, contact.score_px, contact.replay_frame)
}

fn extract_contact_summary_value<'a>(summary: &'a str, prefix: &str) -> Option<&'a str> {
    let start = summary.find(prefix)? + prefix.len();
    let tail = &summary[start..];
    let end = tail.find('/').unwrap_or(tail.len());
    Some(&tail[..end])
}

fn extract_rock_tank_child_contact_summary(summary: &str) -> Option<&str> {
    let start = summary.find("/child{")? + "/child{".len();
    let tail = &summary[start..];
    let end = tail.find('}')?;
    Some(&tail[..end])
}

fn enemy_contact_summary_score_px(summary: &str) -> Option<i64> {
    let kid = extract_bracketed_i64s(summary, "kid=[")?;
    let object = extract_bracketed_i64s(summary, "obj=[")?;
    if kid.len() != 4 || object.len() != 4 {
        return None;
    }
    let kid_centre_x = i64::midpoint(kid[0], kid[1]);
    let object_centre_x = i64::midpoint(object[0], object[1]);
    let kid_bottom = kid[3];
    let object_top = object[2];
    Some((kid_centre_x - object_centre_x).abs() + (kid_bottom - object_top).abs())
}

fn extract_bracketed_i64s(summary: &str, prefix: &str) -> Option<Vec<i64>> {
    let start = summary.find(prefix)? + prefix.len();
    let tail = &summary[start..];
    let end = tail.find(']')?;
    tail[..end]
        .split(',')
        .map(str::parse::<i64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()
}

pub(crate) fn print_oracle_environment(env: &OracleEnvironment) {
    println!("jgenesis repo: {}", env.jgenesis_dir.display());
    println!("jgenesis helper: {}", env.jgenesis_helper_bin.display());
    println!("blastem repo: {}", env.blastem_dir.display());
    match &env.blastem_binary {
        Some(path) => println!("blastem binary: {}", path.display()),
        None => println!(
            "blastem binary: not built (install libsdl2-dev libglew-dev libgl1-mesa-dev to enable local builds)"
        ),
    }
}

pub(crate) fn trace_audio(repo_root: &Path, rom_bytes: &[u8], scenario: Scenario) -> Result<()> {
    let run_dir = repo_root.join(".cache/oracle-runs").join(scenario.slug());
    let native_dir = run_dir.join("native");
    fs::create_dir_all(&native_dir)
        .with_context(|| format!("failed to create {}", native_dir.display()))?;
    let capture = capture_scenario(rom_bytes, scenario)?;
    write_native_artifacts(&native_dir, &capture)?;
    let trace_path = native_dir.join("audio-trace.json");
    println!("wrote audio trace to {}", trace_path.display());
    println!("z80 reset count: {}", capture.audio_trace.z80_reset_count);
    if let Some(onset) = capture.audio_trace.first_significant_audio_sample {
        println!(
            "first significant audio sample: {} ({:.3}s)",
            onset,
            onset as f32 / AUDIO_SAMPLE_RATE as f32
        );
    } else {
        println!("first significant audio sample: none");
    }
    println!(
        "first YM2612 write frame/sample: {:?}/{:?}",
        capture.audio_trace.first_ym2612_write_frame, capture.audio_trace.first_ym2612_write_sample
    );
    println!(
        "first YM2612 key-on frame/sample: {:?}/{:?}",
        capture.audio_trace.first_ym2612_key_on_frame,
        capture.audio_trace.first_ym2612_key_on_sample
    );
    println!(
        "first PSG write frame/sample: {:?}/{:?}",
        capture.audio_trace.first_psg_write_frame, capture.audio_trace.first_psg_write_sample
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Level layout dump — extracts the collision grid from emulator RAM
// ---------------------------------------------------------------------------

/// RAM addresses for level layout extraction (from variables.asm).
pub(crate) const LEVEL_LAYOUT_ADDR: u32 = 0x00FF_A652;
pub(crate) const LEVEL_LAYOUT_SIZE: usize = 0x41A0;
/// Level_terrain_layout: 1 byte/cell, "30 screens of $118 entries each"
/// (variables.asm:10). The decoded FG stream lands here before the collision
/// map fills Level_Layout; dumped for the #229 native-decoder mapping spike.
pub(crate) const LEVEL_TERRAIN_LAYOUT_ADDR: u32 = 0x00FF_52C2;
pub(crate) const LEVEL_TERRAIN_LAYOUT_SIZE: usize = 0x20D0;
/// Block List RAM: prize/ghost/teleporter/hidden entries (ram_layout.txt).
pub(crate) const BLOCK_LIST_ADDR: u32 = 0x00FF_3B24;
pub(crate) const BLOCK_LIST_SIZE: usize = 0x0898; // FF3B24..FF43BC

pub(crate) fn dump_level_layout(rom_bytes: &[u8], level_id: u8, out_dir: &Path) -> Result<()> {
    let mut runtime = MdRuntime::new(rom_bytes)?;

    // Run through boot + title presses to reach level 0
    let entry_script = gameplay_entry_script()?;
    for (frame_index, buttons) in entry_script.iter().enumerate() {
        runtime.set_controller_1(*buttons);
        if !runtime.run_until_vblank(500_000) {
            bail!("emulator stalled at frame {frame_index}");
        }
    }

    // Wait for level 0 to settle
    for _ in 0..60 {
        runtime.set_controller_1(0);
        runtime.run_until_vblank(500_000);
    }

    // Warp to the target level using the level skip cheat.
    // Set Current_LevelID = target - 1, enable cheat, press Start+A+C.
    // The cheat handler increments LevelID and sets Game_Mode = 8.
    if level_id != 0 {
        runtime.set_level_skip_cheat_enabled(true);
        runtime.set_lives(9);
        runtime.set_hitpoints(9);
        // Write level ID just before the button press frame to minimise
        // time running with a mismatched level ID.
        runtime.set_current_level(u16::from(level_id).wrapping_sub(1));
        // Start=0x80, A=0x40, C=0x20
        runtime.set_controller_1(0xE0);
        runtime.run_until_vblank(500_000);
        runtime.set_controller_1(0);

        // Poll for the level to finish loading (game_mode == 0x0C), then wait
        // for the active layout bytes to stop changing. Some levels populate
        // deeper Level_Layout rows a few frames after the first in-game frame,
        // so sampling only the first cache line can freeze in partially loaded
        // collision data.
        let mut settled = false;
        let mut level_loaded = false;
        let mut stable_frames = 0u8;
        let mut previous_layout = Vec::new();
        let mut previous_block_list = Vec::new();
        for frame in 0..600 {
            if !level_loaded && frame % 30 == 20 {
                runtime.set_controller_1(0x80);
            } else {
                runtime.set_controller_1(0);
            }
            runtime.run_until_vblank(500_000);

            let snapshot = runtime.snapshot();
            if snapshot.current_level_id != u16::from(level_id) {
                continue;
            }
            if snapshot.game_mode == 0x000C {
                level_loaded = true;
            }
            if !level_loaded {
                continue;
            }

            let width_blocks = runtime.read_work_ram_word(0x00FF_F89E);
            let height_blocks = runtime.read_work_ram_word(0x00FF_F8A4);
            let active_layout_bytes = usize::from(width_blocks)
                .saturating_mul(usize::from(height_blocks))
                .saturating_mul(2)
                .min(LEVEL_LAYOUT_SIZE);
            if active_layout_bytes == 0 {
                stable_frames = 0;
                previous_layout.clear();
                previous_block_list.clear();
                continue;
            }

            let layout = runtime.read_work_ram_range(LEVEL_LAYOUT_ADDR, active_layout_bytes);
            let nonzero_words = layout
                .chunks(2)
                .filter(|word| word[0] != 0 || word[1] != 0)
                .count();
            if nonzero_words <= 10 {
                stable_frames = 0;
                previous_layout.clear();
                previous_block_list.clear();
                continue;
            }

            let block_list = runtime.read_work_ram_range(BLOCK_LIST_ADDR, BLOCK_LIST_SIZE);
            if layout == previous_layout && block_list == previous_block_list {
                stable_frames = stable_frames.saturating_add(1);
            } else {
                stable_frames = 0;
                previous_layout = layout;
                previous_block_list = block_list;
            }

            if stable_frames >= 4 {
                settled = true;
                break;
            }
        }

        if !settled {
            let snapshot = runtime.snapshot();
            bail!(
                "level warp to 0x{:02X} did not settle: game_mode=0x{:04X}, level_id=0x{:04X}",
                level_id,
                snapshot.game_mode,
                snapshot.current_level_id,
            );
        }
    }

    // Note: game_mode may have reverted from 0x000C (e.g. player death)
    // by this point. The layout data was captured when game_mode was 0x000C,
    // so it's still valid.

    // variables.asm addresses:
    //   Level_width_blocks  = $FFFFF89E (cells per row, each 16x16 px)
    //   Level_height_blocks = $FFFFF8A4 (rows)
    let width_blocks = runtime.read_work_ram_word(0x00FF_F89E);
    let height_blocks = runtime.read_work_ram_word(0x00FF_F8A4);
    let layout_data = runtime.read_work_ram_range(LEVEL_LAYOUT_ADDR, LEVEL_LAYOUT_SIZE);
    let block_list_data = runtime.read_work_ram_range(BLOCK_LIST_ADDR, BLOCK_LIST_SIZE);

    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;

    let filename = format!("{level_id:02X}.bin");
    let layout_path = out_dir.join(&filename);
    fs::write(&layout_path, &layout_data)
        .with_context(|| format!("failed to write {}", layout_path.display()))?;

    // Dump the Block List (prize/ghost/teleporter/hidden entries)
    let block_list_path = out_dir.join(format!("{level_id:02X}.blocklist.bin"));
    fs::write(&block_list_path, &block_list_data)
        .with_context(|| format!("failed to write {}", block_list_path.display()))?;

    // #229 spike: also dump Level_terrain_layout (1 byte/cell, $118 stride) so
    // the native decoder's FG-stream -> terrain -> Level_Layout mapping can be
    // reverse-engineered against the bundle's decoded foreground.bin.
    let terrain_layout_data =
        runtime.read_work_ram_range(LEVEL_TERRAIN_LAYOUT_ADDR, LEVEL_TERRAIN_LAYOUT_SIZE);
    let terrain_path = out_dir.join(format!("{level_id:02X}.terrain.bin"));
    fs::write(&terrain_path, &terrain_layout_data)
        .with_context(|| format!("failed to write {}", terrain_path.display()))?;

    // Write metadata alongside
    let meta = serde_json::json!({
        "level_id": level_id,
        "width_blocks": width_blocks,
        "height_blocks": height_blocks,
        "width_pixels": u32::from(width_blocks) * 16,
        "height_pixels": u32::from(height_blocks) * 16,
        "data_bytes": usize::from(width_blocks) * usize::from(height_blocks) * 2,
        "dump_bytes": layout_data.len(),
    });
    let meta_path = out_dir.join(format!("{level_id:02X}.json"));
    fs::write(&meta_path, serde_json::to_string_pretty(&meta)?)
        .with_context(|| format!("failed to write {}", meta_path.display()))?;

    println!(
        "dumped Level_Layout for level 0x{:02X}: {}x{} cells ({}x{} px) to {}",
        level_id,
        width_blocks,
        height_blocks,
        u32::from(width_blocks) * 16,
        u32::from(height_blocks) * 16,
        layout_path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use wildside_core::{
        ActiveProjectileKind, ActiveProjectileSnapshot, DragonFlyingDescentFireState, GameMode,
        NativeObjectKind, NativeObjectSnapshot,
    };

    fn cached_generated_manifest() -> Option<AssetManifest> {
        let repo_root = if let Ok(override_root) = std::env::var("KID_REPO_ROOT")
            && !override_root.is_empty()
        {
            PathBuf::from(override_root)
        } else {
            let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            crate_dir
                .parent()
                .and_then(std::path::Path::parent)
                .expect("wildside-cli crate should live under <repo>/crates/wildside-cli")
                .to_path_buf()
        };
        let manifest_path = repo_root
            .join(".cache/generated/28b904000b2863b6760531807760b571f1a5fc1d/asset-manifest.json");
        if !manifest_path.is_file() {
            eprintln!(
                "skipping cache-backed inspect test: generated asset manifest missing at {}",
                manifest_path.display()
            );
            return None;
        }
        Some(
            wildside_data::load_manifest(&manifest_path)
                .expect("load cached generated asset manifest"),
        )
    }

    fn manifest_with_drips_definition() -> AssetManifest {
        AssetManifest {
            generated_bundle_version: 0,
            rom_sha1: String::new(),
            upstream_commit: String::new(),
            source_root: PathBuf::new(),
            output_root: PathBuf::new(),
            map_order: Vec::new(),
            levels: Vec::new(),
            constants: vec![wildside_data::NamedValue {
                name: "objid_Drip".into(),
                value: 0x16,
                comment: None,
            }],
            ram_variables: Vec::new(),
            enemy_sources: Vec::new(),
            theme_assets: Vec::new(),
            scene_assets: Vec::new(),
            animation_assets: Vec::new(),
            sound_assets: Vec::new(),
            ingame_assets: Vec::new(),
            decoded_levels: Vec::new(),
        }
    }

    #[test]
    fn block_list_summary_reports_helmet_prize_entries() {
        let block_list = terrain::BlockList {
            prize_blocks: vec![
                terrain::PrizeBlockEntry {
                    layout_addr: 0xA742,
                    col: 116,
                    row: 10,
                    content: terrain::PrizeContent::Juggernaut,
                    disguised: false,
                },
                terrain::PrizeBlockEntry {
                    layout_addr: 0xC1A0,
                    col: 4,
                    row: 29,
                    content: terrain::PrizeContent::Micromax,
                    disguised: true,
                },
            ],
            ghost_blocks: Vec::new(),
            teleporters: Vec::new(),
            hidden_blocks: vec![terrain::HiddenBlockEntry {
                layout_addr: 0xC1A0,
                col: 4,
                row: 29,
                block_type: 0x09,
                block_setting: 0x0A,
            }],
        };

        assert_eq!(
            describe_block_list(&block_list),
            vec![
                "block list: 2 prizes, 0 ghost groups, 0 teleporters, 1 hidden".to_owned(),
                "prize block 00: col 116, row 10, Juggernaut id 0x6 layout 0xA742".to_owned(),
                "prize block 01: col 4, row 29, Micromax id 0xA disguised layout 0xC1A0".to_owned(),
                "hidden block 00: col 4, row 29, type 0x09, setting 0x0A, layout 0xC1A0".to_owned(),
            ],
            "later-level helmet prize audits need inspect-level to show parsed Block List records directly",
        );
    }

    #[test]
    fn generated_level_01_block_list_reports_teleporter_entry() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let bundle = wildside_data::load_level_asset_bundle(&manifest, 0x01)
            .expect("load level 0x01 bundle");
        let resource_level = resource_level_id(&manifest, 0x01);
        let grid = load_inspect_block_grid(&manifest, 0x01, resource_level, &bundle)
            .expect("level 0x01 generated layout grid");
        let block_list =
            load_inspect_block_list(&manifest, 0x01, resource_level, &bundle, Some(&grid))
                .expect("level 0x01 generated block list");

        assert_eq!(block_list.teleporters.len(), 1);
        let teleporter = &block_list.teleporters[0];
        assert_eq!(teleporter.layout_addr, 0xD48A);
        assert_eq!(teleporter.col, 156);
        assert_eq!(teleporter.row, 36);
        assert_eq!(teleporter.dest_map_id, 0x4B);
        assert_eq!(teleporter.dest_x, 28);
        assert_eq!(teleporter.dest_y, 9);
        assert!(
            describe_block_list(&block_list).iter().any(|line| {
                line == "teleporter 00: col 156, row 36, dest map 0x4B, dest 28,9 layout 0xD48A"
            }),
            "inspect-level should print the teleporter destination, not just the count",
        );
    }

    #[test]
    fn generated_later_level_block_lists_include_helmet_prize_entries() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        for (level_id, expected_content) in [
            (0x05, terrain::PrizeContent::Juggernaut),
            (0x14, terrain::PrizeContent::Cyclone),
            (0x20, terrain::PrizeContent::Skycutter),
        ] {
            let bundle = wildside_data::load_level_asset_bundle(&manifest, level_id)
                .unwrap_or_else(|error| panic!("load level 0x{level_id:02X} bundle: {error}"));
            let block_list = load_inspect_block_list(
                &manifest,
                level_id,
                resource_level_id(&manifest, level_id),
                &bundle,
                None,
            )
            .unwrap_or_else(|| panic!("load level 0x{level_id:02X} block list"));

            assert!(
                block_list
                    .prize_blocks
                    .iter()
                    .any(|entry| entry.content == expected_content),
                "generated level 0x{level_id:02X} block list should expose {expected_content:?} helmet prizes for later-level audits"
            );
        }
    }

    #[test]
    fn enemy_layout_object_listing_includes_exact_kind_name_and_position() {
        let manifest = manifest_with_drips_definition();
        let objects = vec![wildside_data::EnemyLayoutObject {
            object_type: 0x16,
            flags: 0x80,
            hit_points_plus_one: 1,
            x: 456,
            y: 1567,
        }];

        assert_eq!(
            describe_enemy_layout_objects(&manifest, &objects),
            vec!["enemy object 00: kind 0x16 Drip at 456,1567 flags 0x80 hp+1 1"]
        );
    }

    fn target_enemy_frame(
        frame_index: usize,
        object_type: u8,
        layout_index: Option<usize>,
        x_pos: u32,
        y_pos: u32,
    ) -> NativeTraceFrame {
        let mut frame = NativeTraceFrame {
            frame_index,
            game_mode: GameMode::InGame,
            ..NativeTraceFrame::default()
        };
        frame.active_enemy_details.push(NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type },
            x_pos,
            y_pos,
            layout_index,
            x_velocity: None,
            y_velocity: None,
            facing: None,
            on_ground: None,
            hit_points: None,
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            sprite_animation_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        });
        frame
    }

    #[test]
    fn target_enemy_activation_summary_reports_absent_enemy() {
        let trace = vec![target_enemy_frame(
            4,
            0x0A,
            Some(9),
            0x0010_0000,
            0x0020_0000,
        )];

        assert_eq!(
            describe_target_enemy_activation(&trace, 0x16),
            "target enemy 0x16: absent"
        );
    }

    #[test]
    fn target_enemy_activation_summary_uses_authoritative_enemy_details() {
        let mut trace = vec![
            NativeTraceFrame {
                frame_index: 10,
                game_mode: GameMode::InGame,
                ..NativeTraceFrame::default()
            },
            target_enemy_frame(11, 0x16, Some(7), 0x001C_0000, 0x003F_0000),
            target_enemy_frame(12, 0x0A, Some(9), 0x0030_0000, 0x0040_0000),
            target_enemy_frame(13, 0x16, Some(8), 0x0020_0000, 0x0041_0000),
        ];
        trace[3].active_enemy_details.push(NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x16 },
            x_pos: 0x0018_0000,
            y_pos: 0x0040_0000,
            layout_index: Some(7),
            x_velocity: None,
            y_velocity: None,
            facing: None,
            on_ground: None,
            hit_points: None,
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            sprite_animation_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        });

        assert_eq!(
            describe_target_enemy_activation(&trace, 0x16),
            "target enemy 0x16: first replay frame 1 last replay frame 3 active frames 2 sightings 3 x 00180000..00200000 y 003F0000..00410000 layouts 7,8 first details [enemy_16@001C0000,003F0000/layout7]"
        );
    }

    #[test]
    fn target_enemy_activation_summary_ignores_transition_stale_details() {
        let mut transition = target_enemy_frame(21, 0x16, Some(7), 0x001C_0000, 0x003F_0000);
        transition.game_mode = GameMode::LevelTransition;

        assert_eq!(
            describe_target_enemy_activation(&[transition], 0x16),
            "target enemy 0x16: absent"
        );
    }

    fn target_projectile_frame(
        frame_index: usize,
        kind: ActiveProjectileKind,
        x_pos: u32,
        y_pos: u32,
        y_velocity: i32,
        remaining_frames: u16,
    ) -> NativeTraceFrame {
        let mut frame = NativeTraceFrame {
            frame_index,
            game_mode: GameMode::InGame,
            ..NativeTraceFrame::default()
        };
        frame
            .active_projectile_details
            .push(ActiveProjectileSnapshot {
                x_pos,
                y_pos,
                x_velocity: 0,
                y_velocity,
                friendly: false,
                remaining_frames,
                animation_elapsed_frames: 0,
                kind,
                palette_variant: 0,
                ufo_first_hurt_debris_flipped: false,
                boss_death_burst_sprite_frame: wildside_core::BossDeathBurstSpriteFrame::C8460,
                boss_death_burst_flipped: false,
                boss_eye_sprite_mapping_index: 0,
                boss_eye_sprite_order_word_a: 0,
                boss_head_projectile_sprite_frame:
                    wildside_core::BossHeadProjectileSpriteFrame::None,
                dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
                dragon_flying_descent_fire_counter: 0,
            });
        frame
    }

    fn target_projectile_frame_with_velocity(
        frame_index: usize,
        kind: ActiveProjectileKind,
        x_pos: u32,
        y_pos: u32,
        x_velocity: i32,
        y_velocity: i32,
        remaining_frames: u16,
    ) -> NativeTraceFrame {
        let mut frame = target_projectile_frame(
            frame_index,
            kind,
            x_pos,
            y_pos,
            y_velocity,
            remaining_frames,
        );
        if let Some(projectile) = frame.active_projectile_details.last_mut() {
            projectile.x_velocity = x_velocity;
        }
        frame
    }

    #[test]
    fn target_projectile_activation_summary_reports_absent_projectile() {
        let trace = vec![target_projectile_frame(
            4,
            ActiveProjectileKind::DripsSurfaceCrawler,
            0x0010_0000,
            0x0020_0000,
            0x0000_2000,
            9,
        )];

        assert_eq!(
            describe_target_projectile_activation(&trace, ActiveProjectileKind::DripsDrop),
            "target projectile DripsDrop: absent"
        );
    }

    #[test]
    fn target_projectile_activation_summary_uses_authoritative_projectile_details() {
        let mut trace = vec![
            NativeTraceFrame {
                frame_index: 10,
                game_mode: GameMode::InGame,
                ..NativeTraceFrame::default()
            },
            target_projectile_frame(
                11,
                ActiveProjectileKind::DripsDrop,
                0x001C_0000,
                0x003F_0000,
                0x0000_2000,
                0,
            ),
            target_projectile_frame(
                12,
                ActiveProjectileKind::DripsSurfaceCrawler,
                0x0030_0000,
                0x0040_0000,
                0,
                8,
            ),
            target_projectile_frame(
                13,
                ActiveProjectileKind::DripsDrop,
                0x0020_0000,
                0x0041_0000,
                0x0000_4000,
                0,
            ),
        ];
        trace[3]
            .active_projectile_details
            .push(ActiveProjectileSnapshot {
                x_pos: 0x0018_0000,
                y_pos: 0x0040_0000,
                x_velocity: 0,
                y_velocity: 0x0000_3000,
                friendly: false,
                remaining_frames: 0,
                animation_elapsed_frames: 0,
                kind: ActiveProjectileKind::DripsDrop,
                palette_variant: 0,
                ufo_first_hurt_debris_flipped: false,
                boss_death_burst_sprite_frame: wildside_core::BossDeathBurstSpriteFrame::C8460,
                boss_death_burst_flipped: false,
                boss_eye_sprite_mapping_index: 0,
                boss_eye_sprite_order_word_a: 0,
                boss_head_projectile_sprite_frame:
                    wildside_core::BossHeadProjectileSpriteFrame::None,
                dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
                dragon_flying_descent_fire_counter: 0,
            });

        assert_eq!(
            describe_target_projectile_activation(&trace, ActiveProjectileKind::DripsDrop),
            "target projectile DripsDrop: first replay frame 1 last replay frame 3 active frames 2 sightings 3 x 00180000..00200000 y 003F0000..00410000 y_vel 00002000..00004000 closest kid <none> first details [DripsDrop@001C0000,003F0000/vel00000000,00002000/remaining0]"
        );
    }

    #[test]
    fn target_projectile_activation_summary_reports_closest_player_approach() {
        let mut trace = vec![
            target_projectile_frame(
                10,
                ActiveProjectileKind::DripsDrop,
                0x0020_0000,
                0x0040_0000,
                0x0000_2000,
                0,
            ),
            target_projectile_frame(
                11,
                ActiveProjectileKind::DripsDrop,
                0x0024_0000,
                0x0042_0000,
                0x0000_4000,
                0,
            ),
        ];
        trace[0].kid_x_pos = Some(0x0018_0000);
        trace[0].kid_y_pos = Some(0x0048_0000);
        trace[1].kid_x_pos = Some(0x0025_0000);
        trace[1].kid_y_pos = Some(0x0040_0000);

        assert_eq!(
            describe_target_projectile_activation(&trace, ActiveProjectileKind::DripsDrop),
            "target projectile DripsDrop: first replay frame 0 last replay frame 1 active frames 2 sightings 2 x 00200000..00240000 y 00400000..00420000 y_vel 00002000..00004000 closest kid dx_px -1 dy_px +2 manhattan_px 3 @1 first details [DripsDrop@00200000,00400000/vel00000000,00002000/remaining0]"
        );
    }

    #[test]
    fn target_projectile_activation_summary_ignores_transition_stale_details() {
        let mut transition = target_projectile_frame(
            21,
            ActiveProjectileKind::DripsDrop,
            0x001C_0000,
            0x003F_0000,
            0x0000_2000,
            0,
        );
        transition.game_mode = GameMode::LevelTransition;

        assert_eq!(
            describe_target_projectile_activation(&[transition], ActiveProjectileKind::DripsDrop),
            "target projectile DripsDrop: absent"
        );
    }

    #[test]
    fn target_projectile_cell_approach_scores_projectile_against_cell_rectangle() {
        let trace = vec![
            target_projectile_frame(
                10,
                ActiveProjectileKind::DripsDrop,
                0x0248_0000,
                0x0212_0000,
                0x0000_2000,
                0,
            ),
            target_projectile_frame(
                11,
                ActiveProjectileKind::DripsDrop,
                0x0240_0000,
                0x0208_0000,
                0x0000_4000,
                0,
            ),
            target_projectile_frame(
                12,
                ActiveProjectileKind::DripsSurfaceCrawler,
                0x0240_0000,
                0x0208_0000,
                0,
                0,
            ),
        ];

        assert_eq!(
            target_projectile_cell_approach_from_frame(
                &trace,
                ActiveProjectileKind::DripsDrop,
                TargetCell { col: 36, row: 32 },
                0,
            ),
            Some(TargetProjectileCellApproach {
                replay_frame: 1,
                projectile_x: 0x0240_0000,
                projectile_y: 0x0208_0000,
                dx_px: 0,
                dy_px: 0,
                manhattan_px: 0,
            }),
            "issue #260 route scans need to score Drips children against the metal-block cell, not just Kid's hitbox"
        );
        assert_eq!(
            describe_target_projectile_cell_approach_from_frame(
                &trace,
                ActiveProjectileKind::DripsDrop,
                TargetCell { col: 36, row: 32 },
                0,
            ),
            "target projectile DripsDrop cell 36,32 dx_px +0 dy_px +0 manhattan_px 0 @1 projectile 02400000,02080000",
            "issue #260 single-route probes should expose the same projectile/cell scoring as batch probes"
        );
    }

    #[test]
    fn target_projectile_cell_approach_ignores_transition_stale_projectiles() {
        let mut transition = target_projectile_frame(
            21,
            ActiveProjectileKind::DripsDrop,
            0x0240_0000,
            0x0208_0000,
            0x0000_2000,
            0,
        );
        transition.game_mode = GameMode::LevelTransition;

        assert_eq!(
            target_projectile_cell_approach_from_frame(
                &[transition],
                ActiveProjectileKind::DripsDrop,
                TargetCell { col: 36, row: 32 },
                0,
            ),
            None
        );
    }

    #[test]
    fn target_projectile_cell_conjunction_scores_same_frame_kid_projectile_and_cell() {
        let mut far_kid = target_projectile_frame(
            10,
            ActiveProjectileKind::DripsDrop,
            0x0240_0000,
            0x0208_0000,
            0x0000_2000,
            0,
        );
        far_kid.kid_x_pos = Some(0x0200_0000);
        far_kid.kid_y_pos = Some(0x0240_0000);
        far_kid.number_hitpoints = 2;

        let mut close_both = target_projectile_frame(
            11,
            ActiveProjectileKind::DripsDrop,
            0x0248_0000,
            0x0208_0000,
            0x0000_2000,
            0,
        );
        close_both.kid_x_pos = Some(0x0240_0000);
        close_both.kid_y_pos = Some(0x0227_0000);
        close_both.number_hitpoints = 2;
        let trace = vec![far_kid, close_both];

        assert_eq!(
            target_projectile_cell_conjunction_from_frame(
                &trace,
                ActiveProjectileKind::DripsDrop,
                TargetCell { col: 36, row: 32 },
                0,
            ),
            Some(TargetProjectileCellConjunction {
                replay_frame: 1,
                kid_x: 0x0240_0000,
                kid_y: 0x0227_0000,
                projectile_x: 0x0248_0000,
                projectile_y: 0x0208_0000,
                kid_dx_px: 0,
                kid_dy_px: 0,
                projectile_dx_px: 0,
                projectile_dy_px: 0,
                manhattan_px: 0,
            }),
            "issue #260 needs same-frame Kid, Drips child, and metal-block cell scoring"
        );
        assert_eq!(
            describe_target_projectile_cell_conjunction_from_frame(
                &trace,
                ActiveProjectileKind::DripsDrop,
                TargetCell { col: 36, row: 32 },
                0,
            ),
            "target projectile DripsDrop cell conjunction 36,32 kid_dx_px +0 kid_dy_px +0 projectile_dx_px +0 projectile_dy_px +0 manhattan_px 0 @1 kid 02400000,02270000 projectile 02480000,02080000",
            "issue #260 single-route probes should expose same-frame Kid/projectile/cell scoring"
        );
    }

    #[test]
    fn target_projectile_kid_approach_scores_healthy_kid_to_projectile() {
        let mut trace = vec![
            target_projectile_frame(
                10,
                ActiveProjectileKind::DripsDrop,
                0x0248_0000,
                0x0212_0000,
                0x0000_2000,
                0,
            ),
            target_projectile_frame(
                11,
                ActiveProjectileKind::DripsDrop,
                0x0248_0000,
                0x0212_0000,
                0x0000_4000,
                0,
            ),
        ];
        trace[0].kid_x_pos = Some(0x0240_0000);
        trace[0].kid_y_pos = Some(0x0200_0000);
        trace[0].number_hitpoints = 1;
        trace[1].kid_x_pos = Some(0x0249_0000);
        trace[1].kid_y_pos = Some(0x0210_0000);
        trace[1].number_hitpoints = 1;

        assert_eq!(
            target_projectile_kid_approach_from_frame(
                &trace,
                ActiveProjectileKind::DripsDrop,
                true,
                0,
            ),
            Some(TargetProjectileClosestApproach {
                replay_frame: 1,
                dx_px: -1,
                dy_px: 2,
                manhattan_px: 3,
            }),
            "issue #260 batch route scans need to rank Kid/drop proximity directly"
        );
    }

    #[test]
    fn target_projectile_directional_approach_filters_by_velocity() {
        let mut right_shot = target_projectile_frame_with_velocity(
            10,
            ActiveProjectileKind::ShooterBlock,
            0x0248_0000,
            0x0208_0000,
            0x0002_0000,
            0,
            22,
        );
        right_shot.kid_x_pos = Some(0x0248_0000);
        right_shot.kid_y_pos = Some(0x0208_0000);
        right_shot.number_hitpoints = 2;

        let mut left_shot = target_projectile_frame_with_velocity(
            11,
            ActiveProjectileKind::ShooterBlock,
            0x0240_0000,
            0x0208_0000,
            -0x0002_0000,
            0,
            22,
        );
        left_shot.kid_x_pos = Some(0x023E_0000);
        left_shot.kid_y_pos = Some(0x0209_0000);
        left_shot.number_hitpoints = 2;
        let trace = vec![right_shot, left_shot];

        assert_eq!(
            target_projectile_directional_kid_approach_from_frame(
                &trace,
                ActiveProjectileKind::ShooterBlock,
                TargetProjectileDirection::Left,
                true,
                0,
            ),
            Some(TargetProjectileDirectionalApproach {
                replay_frame: 1,
                projectile_x: 0x0240_0000,
                projectile_y: 0x0208_0000,
                projectile_x_velocity: -0x0002_0000,
                projectile_y_velocity: 0,
                dx_px: 2,
                dy_px: -1,
                manhattan_px: 3,
            }),
            "issue #260 needs to rank leftward `$EC2D` shooter shots separately from rightward shots"
        );
        assert_eq!(
            describe_target_projectile_directional_kid_approach_from_frame(
                &trace,
                ActiveProjectileKind::ShooterBlock,
                TargetProjectileDirection::Left,
                true,
                0,
            ),
            "target projectile ShooterBlock direction Left healthy kid dx_px +2 dy_px -1 manhattan_px 3 @1 projectile 02400000,02080000 vel FFFE0000,00000000"
        );
    }

    #[test]
    fn target_projectile_kid_approach_can_require_healthy_kid() {
        let mut trace = vec![target_projectile_frame(
            10,
            ActiveProjectileKind::DripsDrop,
            0x0248_0000,
            0x0212_0000,
            0x0000_2000,
            0,
        )];
        trace[0].kid_x_pos = Some(0x0248_0000);
        trace[0].kid_y_pos = Some(0x0212_0000);
        trace[0].number_hitpoints = 0;
        trace[0].death_windup_elapsed = 1;

        assert_eq!(
            target_projectile_kid_approach_from_frame(
                &trace,
                ActiveProjectileKind::DripsDrop,
                true,
                0,
            ),
            None
        );
        assert_eq!(
            target_projectile_kid_approach_from_frame(
                &trace,
                ActiveProjectileKind::DripsDrop,
                false,
                0,
            )
            .map(|approach| approach.manhattan_px),
            Some(0)
        );
    }

    fn player_frame(
        frame_index: usize,
        game_mode: GameMode,
        x_pos: Option<u32>,
        y_pos: Option<u32>,
    ) -> NativeTraceFrame {
        NativeTraceFrame {
            frame_index,
            game_mode,
            kid_x_pos: x_pos,
            kid_y_pos: y_pos,
            number_hitpoints: 1,
            ..NativeTraceFrame::default()
        }
    }

    #[test]
    fn player_motion_summary_reports_live_bounds_and_transition_frame() {
        let trace = vec![
            player_frame(10, GameMode::InGame, Some(0x0010_0000), Some(0x0040_0000)),
            player_frame(11, GameMode::InGame, Some(0x0018_0000), Some(0x003F_0000)),
            player_frame(12, GameMode::InGame, Some(0x0014_0000), Some(0x0041_0000)),
            player_frame(
                13,
                GameMode::LevelTransition,
                Some(0x0000_0000),
                Some(0x0000_0000),
            ),
        ];

        assert_eq!(
            describe_player_motion(&trace),
            "player motion: first replay frame 0 last replay frame 2 live frames 3 x 00100000@0..00180000@1 y 003F0000@1..00410000@2 last live kid 00140000,00410000 first non-InGame replay frame 3"
        );
    }

    #[test]
    fn player_motion_summary_ignores_transition_reset_position() {
        let trace = vec![
            player_frame(10, GameMode::InGame, Some(0x0010_0000), Some(0x0040_0000)),
            player_frame(
                11,
                GameMode::LevelTransition,
                Some(0x0000_0000),
                Some(0x0000_0000),
            ),
        ];

        assert_eq!(
            describe_player_motion(&trace),
            "player motion: first replay frame 0 last replay frame 0 live frames 1 x 00100000@0..00100000@0 y 00400000@0..00400000@0 last live kid 00100000,00400000 first non-InGame replay frame 1"
        );
    }

    #[test]
    fn player_motion_summary_reports_absent_without_live_player_position() {
        let trace = vec![player_frame(10, GameMode::LevelTransition, None, None)];

        assert_eq!(describe_player_motion(&trace), "player motion: absent");
    }

    #[test]
    fn target_point_approach_reports_closest_live_player_position() {
        let trace = vec![
            player_frame(10, GameMode::InGame, Some(0x0010_0000), Some(0x0040_0000)),
            player_frame(11, GameMode::InGame, Some(0x0018_0000), Some(0x0044_0000)),
            player_frame(12, GameMode::InGame, Some(0x0021_0000), Some(0x003F_0000)),
            player_frame(
                13,
                GameMode::LevelTransition,
                Some(0x0020_0000),
                Some(0x0040_0000),
            ),
        ];

        assert_eq!(
            describe_target_point_approach_from_frame(
                &trace,
                TargetPoint { x_px: 32, y_px: 64 },
                0
            ),
            "target point 32,64: closest kid dx_px -1 dy_px +1 manhattan_px 2 @2 kid 00210000,003F0000; closest healthy kid dx_px -1 dy_px +1 manhattan_px 2 @2 kid 00210000,003F0000"
        );
    }

    #[test]
    fn target_point_approach_reports_closest_healthy_player_position_separately() {
        let mut trace = vec![
            player_frame(10, GameMode::InGame, Some(0x0010_0000), Some(0x0040_0000)),
            player_frame(11, GameMode::InGame, Some(0x0020_0000), Some(0x0040_0000)),
            player_frame(12, GameMode::InGame, Some(0x0024_0000), Some(0x003F_0000)),
        ];
        trace[1].number_hitpoints = 0;
        trace[1].death_windup_elapsed = 3;

        assert_eq!(
            describe_target_point_approach_from_frame(
                &trace,
                TargetPoint { x_px: 32, y_px: 64 },
                0
            ),
            "target point 32,64: closest kid dx_px +0 dy_px +0 manhattan_px 0 @1 kid 00200000,00400000; closest healthy kid dx_px -4 dy_px +1 manhattan_px 5 @2 kid 00240000,003F0000"
        );
    }

    #[test]
    fn target_point_approach_from_frame_ignores_early_matches_without_renumbering() {
        let trace = vec![
            player_frame(10, GameMode::InGame, Some(0x0020_0000), Some(0x0040_0000)),
            player_frame(11, GameMode::InGame, Some(0x0028_0000), Some(0x0044_0000)),
            player_frame(12, GameMode::InGame, Some(0x0024_0000), Some(0x0042_0000)),
        ];

        assert_eq!(
            target_point_approach_from_frame(&trace, TargetPoint { x_px: 32, y_px: 64 }, true, 2),
            Some(TargetPointApproach {
                replay_frame: 2,
                kid_x: 0x0024_0000,
                kid_y: 0x0042_0000,
                dx_px: -4,
                dy_px: -2,
                manhattan_px: 6,
            })
        );
    }

    #[test]
    fn target_cell_parser_accepts_col_row_cells() {
        assert_eq!(
            parse_target_cell("28,32").expect("target cell should parse"),
            TargetCell { col: 28, row: 32 }
        );
        assert!(
            parse_target_cell("28").is_err(),
            "cell probes must specify both column and row"
        );
    }

    #[test]
    fn target_cell_approach_scores_distance_to_cell_rectangle() {
        let trace = vec![
            player_frame(10, GameMode::InGame, Some(0x01B0_0000), Some(0x0200_0000)),
            player_frame(11, GameMode::InGame, Some(0x01C8_0000), Some(0x0208_0000)),
            player_frame(12, GameMode::InGame, Some(0x01BF_0000), Some(0x01FF_0000)),
        ];

        assert_eq!(
            target_cell_approach_from_frame(&trace, TargetCell { col: 28, row: 32 }, true, 0),
            Some(TargetCellApproach {
                replay_frame: 1,
                kid_x: 0x01C8_0000,
                kid_y: 0x0208_0000,
                dx_px: 0,
                dy_px: 0,
                manhattan_px: 0,
            }),
            "cell scoring should treat any Kid position inside the 16x16 target cell as contact"
        );
    }

    #[test]
    fn target_cell_hitbox_approach_scores_player_bounds_against_cell() {
        let trace = vec![player_frame(
            10,
            GameMode::InGame,
            Some(0x0240_0000),
            Some(0x021F_0000),
        )];

        assert_eq!(
            target_cell_hitbox_approach_from_frame(
                &trace,
                TargetCell { col: 36, row: 32 },
                true,
                0
            ),
            Some(TargetCellApproach {
                replay_frame: 0,
                kid_x: 0x0240_0000,
                kid_y: 0x021F_0000,
                dx_px: 0,
                dy_px: 0,
                manhattan_px: 0,
            }),
            "ASM loc_902A/unk_B408 block probes use Kid's upper/body bounds, not his foot origin"
        );
    }

    #[test]
    fn target_cell_approach_reports_closest_healthy_position_separately() {
        let mut trace = vec![
            player_frame(10, GameMode::InGame, Some(0x01C0_0000), Some(0x0200_0000)),
            player_frame(11, GameMode::InGame, Some(0x01C8_0000), Some(0x0208_0000)),
            player_frame(12, GameMode::InGame, Some(0x01E0_0000), Some(0x020F_0000)),
        ];
        trace[1].number_hitpoints = 0;
        trace[1].death_windup_elapsed = 3;

        assert_eq!(
            describe_target_cell_approach_from_frame(&trace, TargetCell { col: 28, row: 32 }, 0),
            "target cell 28,32: closest kid dx_px +0 dy_px +0 manhattan_px 0 @0 kid 01C00000,02000000; closest healthy kid dx_px +0 dy_px +0 manhattan_px 0 @0 kid 01C00000,02000000"
        );
    }

    #[test]
    fn target_cell_approach_from_frame_ignores_early_cell_contact() {
        let trace = vec![
            player_frame(10, GameMode::InGame, Some(0x01C0_0000), Some(0x0200_0000)),
            player_frame(11, GameMode::InGame, Some(0x01B0_0000), Some(0x01F0_0000)),
            player_frame(12, GameMode::InGame, Some(0x01C8_0000), Some(0x01F8_0000)),
        ];

        assert_eq!(
            describe_target_cell_approach_from_frame(&trace, TargetCell { col: 28, row: 32 }, 1),
            "target cell 28,32: closest kid dx_px +0 dy_px +8 manhattan_px 8 @2 kid 01C80000,01F80000; closest healthy kid dx_px +0 dy_px +8 manhattan_px 8 @2 kid 01C80000,01F80000"
        );
    }

    #[test]
    fn target_point_approach_reports_absent_without_live_player_position() {
        let trace = vec![player_frame(
            10,
            GameMode::LevelTransition,
            Some(0x0020_0000),
            Some(0x0040_0000),
        )];

        assert_eq!(
            describe_target_point_approach_from_frame(
                &trace,
                TargetPoint { x_px: 32, y_px: 64 },
                0
            ),
            "target point 32,64: absent"
        );
    }

    #[test]
    fn target_enemy_contact_prefers_stomp_over_closer_miss_loc_3b6c0() {
        let mut trace = vec![
            player_frame(10, GameMode::InGame, Some(0x00C0_0000), Some(0x00D0_0000)),
            player_frame(11, GameMode::InGame, Some(0x00C8_0000), Some(0x00D8_0000)),
        ];
        trace[0].gameplay_debug_fields = std::collections::BTreeMap::from([(
            "kid_stomp_contact_candidates".to_string(),
            "enemy_0A#01 gate=separated/result=none/kid=[190,200,145,176]/obj=[170,195,200,216]/overlap=-1,-24/delta=0x00000000,0x00000000".to_string(),
        )]);
        trace[1].gameplay_debug_fields = std::collections::BTreeMap::from([
            (
                "kid_same_frame_stomp_probe".to_string(),
                "candidate=1/enemy=0x0a@0x00c80000,0x00d80000/pre_y_vel=0x0003fc00/post_y_vel=0xfffa0000/anchor=0x00c80000,0x00d80000".to_string(),
            ),
            (
                "kid_collision_order_probe".to_string(),
                "order=0/target=active_enemy[0]/enemy_0A#01 gate=classify/result=stomp/kid=[198,208,185,216]/obj=[190,215,200,216]/overlap=18,16/delta=0x00000000,0x00000000".to_string(),
            ),
        ]);

        assert_eq!(
            target_enemy_contact_from_frame(&trace, 0x0A, 0),
            Some(TargetEnemyContactApproach {
                replay_frame: 1,
                score_px: 17,
                result: "stomp".to_string(),
                gate: "classify".to_string(),
                summary: "order=0/target=active_enemy[0]/enemy_0A#01 gate=classify/result=stomp/kid=[198,208,185,216]/obj=[190,215,200,216]/overlap=18,16/delta=0x00000000,0x00000000".to_string(),
                same_frame_stomp: true,
            })
        );
    }

    #[test]
    fn target_enemy_contact_uses_rock_tank_child_probe_loc_3b464_loc_3b6c0() {
        let mut trace = vec![player_frame(
            10,
            GameMode::InGame,
            Some(0x00C0_0000),
            Some(0x00D0_0000),
        )];
        trace[0].gameplay_debug_fields = std::collections::BTreeMap::from([(
            "rock_tank_contact_probe".to_string(),
            "active_enemy[0]/family=Tank/segment=Parent/hp=2/fa74=0/fa75=1/cm=3/blocked=0/pending_stomp=1/pending_contact=none/delayed_contact=none/child_latch=1/parent{enemy_0A#01 gate=separated/result=none/kid=[148,158,170,201]/obj=[146,171,208,224]/overlap=10,-7/delta=0x00000000,0x00000000}/child{enemy_0A#01 gate=classify/result=stomp/kid=[148,158,170,201]/obj=[146,171,200,216]/overlap=10,1/delta=0x00000000,0x00000000}".to_string(),
        )]);

        assert_eq!(
            target_enemy_contact_from_frame(&trace, 0x0A, 0),
            Some(TargetEnemyContactApproach {
                replay_frame: 0,
                score_px: 6,
                result: "stomp".to_string(),
                gate: "classify".to_string(),
                summary: "rock_tank_contact_probe/active_enemy[0]/family=Tank/segment=Parent/hp=2/fa74=0/fa75=1/cm=3/blocked=0/pending_stomp=1/pending_contact=none/delayed_contact=none/child_latch=1/parent{enemy_0A#01 gate=separated/result=none/kid=[148,158,170,201]/obj=[146,171,208,224]/overlap=10,-7/delta=0x00000000,0x00000000}/child{enemy_0A#01 gate=classify/result=stomp/kid=[148,158,170,201]/obj=[146,171,200,216]/overlap=10,1/delta=0x00000000,0x00000000}".to_string(),
                same_frame_stomp: false,
            })
        );
    }

    #[test]
    fn target_enemy_contact_same_frame_stomp_matches_requested_enemy_loc_3b464() {
        let mut trace = vec![player_frame(
            10,
            GameMode::InGame,
            Some(0x00C0_0000),
            Some(0x00D0_0000),
        )];
        trace[0].gameplay_debug_fields = std::collections::BTreeMap::from([
            (
                "kid_same_frame_stomp_probe".to_string(),
                "candidate=1/enemy=0x0b@0x00c00000,0x00d00000/pre_y_vel=0x0003fc00/post_y_vel=0xfffa0000".to_string(),
            ),
            (
                "kid_collision_order_probe".to_string(),
                "order=0/target=active_enemy[0]/enemy_0A#01 gate=classify/result=stomp/kid=[198,208,185,216]/obj=[190,215,200,216]/overlap=18,16/delta=0x00000000,0x00000000".to_string(),
            ),
        ]);

        assert!(
            !target_enemy_contact_from_frame(&trace, 0x0A, 0)
                .expect("target enemy contact")
                .same_frame_stomp,
            "same-frame stomp evidence must name the requested enemy, not only any candidate"
        );
    }

    #[test]
    fn target_enemy_contact_ranks_rock_tank_child_candidates_loc_3b464() {
        let mut trace = vec![player_frame(
            10,
            GameMode::InGame,
            Some(0x00C0_0000),
            Some(0x00D0_0000),
        )];
        trace[0].gameplay_debug_fields = std::collections::BTreeMap::from([(
            "rock_tank_contact_probe".to_string(),
            [
                "active_enemy[0]/family=Tank/child{enemy_0A#01 gate=separated/result=none/kid=[148,158,170,201]/obj=[146,171,208,224]/overlap=10,-7/delta=0x00000000,0x00000000}",
                "active_enemy[0]/family=Tank/child{enemy_0A#01 gate=classify/result=stomp/kid=[148,158,170,201]/obj=[146,171,200,216]/overlap=10,1/delta=0x00000000,0x00000000}",
                "active_enemy[0]/family=Tank/child{enemy_0A#01 gate=classify/result=damage/kid=[148,158,170,201]/obj=[146,171,200,216]/overlap=10,1/delta=0x00000000,0x00000000}",
            ]
            .join("; "),
        )]);

        let contact =
            target_enemy_contact_from_frame(&trace, 0x0A, 0).expect("target enemy contact");

        assert_eq!(contact.result, "stomp");
        assert!(
            contact.summary.contains("result=stomp"),
            "the best RockTank child contact should survive earlier and later worse entries"
        );
    }

    #[test]
    fn target_enemy_contact_keeps_better_generic_candidate_loc_3b6c0() {
        let mut trace = vec![
            player_frame(10, GameMode::InGame, Some(0x00C0_0000), Some(0x00D0_0000)),
            player_frame(11, GameMode::InGame, Some(0x00C8_0000), Some(0x00D8_0000)),
        ];
        trace[0].gameplay_debug_fields = std::collections::BTreeMap::from([(
            "kid_collision_order_probe".to_string(),
            "order=0/target=active_enemy[0]/enemy_0A#01 gate=classify/result=stomp/kid=[198,208,185,216]/obj=[190,215,200,216]/overlap=18,16/delta=0x00000000,0x00000000".to_string(),
        )]);
        trace[1].gameplay_debug_fields = std::collections::BTreeMap::from([(
            "kid_collision_order_probe".to_string(),
            "order=0/target=active_enemy[0]/enemy_0A#01 gate=classify/result=none/kid=[198,208,185,216]/obj=[198,208,216,224]/overlap=10,0/delta=0x00000000,0x00000000".to_string(),
        )]);

        let contact =
            target_enemy_contact_from_frame(&trace, 0x0A, 0).expect("target enemy contact");

        assert_eq!(contact.result, "stomp");
        assert_eq!(contact.replay_frame, 0);
    }

    #[test]
    fn target_enemy_contact_ranks_malformed_scores_after_measured_loc_3b6c0() {
        let mut trace = vec![
            player_frame(10, GameMode::InGame, Some(0x00C0_0000), Some(0x00D0_0000)),
            player_frame(11, GameMode::InGame, Some(0x00C8_0000), Some(0x00D8_0000)),
        ];
        trace[0].gameplay_debug_fields = std::collections::BTreeMap::from([(
            "kid_collision_order_probe".to_string(),
            "order=0/target=active_enemy[0]/enemy_0A#01 gate=classify/result=none/overlap=10,0/delta=0x00000000,0x00000000".to_string(),
        )]);
        trace[1].gameplay_debug_fields = std::collections::BTreeMap::from([(
            "kid_collision_order_probe".to_string(),
            "order=0/target=active_enemy[0]/enemy_0A#01 gate=classify/result=none/kid=[198,208,185,216]/obj=[190,215,200,216]/overlap=18,16/delta=0x00000000,0x00000000".to_string(),
        )]);

        let contact =
            target_enemy_contact_from_frame(&trace, 0x0A, 0).expect("target enemy contact");

        assert_eq!(contact.replay_frame, 1);
        assert_eq!(contact.score_px, 17);
    }

    #[test]
    fn target_enemy_contact_visual_follow_up_commands_dump_contact_window_loc_3b464() {
        let contact = TargetEnemyContactApproach {
            replay_frame: 226,
            score_px: 6,
            result: "stomp".to_string(),
            gate: "classify".to_string(),
            summary: "rock_tank_contact_probe/...".to_string(),
            same_frame_stomp: true,
        };

        assert_eq!(
            target_enemy_contact_visual_follow_up_commands(
                Path::new(".cache/recordings/issue271-live-rocktank/scenario.json"),
                &contact,
            ),
            vec![
                "cargo run -p wildside-cli -- capture-scenario-frames --scenario .cache/recordings/issue271-live-rocktank/scenario.json --frames 225,226,227 --output .cache/recordings/issue271-live-rocktank/contact-frame-evidence/native-vdp".to_string(),
                "cargo run -p wildside-cli -- capture-scenario-frames --scenario .cache/recordings/issue271-live-rocktank/scenario.json --frames 225,226,227 --scene --output .cache/recordings/issue271-live-rocktank/contact-frame-evidence/native-scene".to_string(),
                "cargo run -p wildside-cli -- capture-scenario-frames --scenario .cache/recordings/issue271-live-rocktank/scenario.json --frames 225,226,227 --scene --layer enemies --output .cache/recordings/issue271-live-rocktank/contact-frame-evidence/native-enemies".to_string(),
                "cargo run -p wildside-cli -- capture-oracle-scenario-frames --scenario .cache/recordings/issue271-live-rocktank/scenario.json --frames 225,226,227 --output .cache/recordings/issue271-live-rocktank/contact-frame-evidence/oracle-vdp".to_string(),
            ]
        );
    }

    #[test]
    fn target_enemy_contact_visual_follow_up_commands_handle_frame_zero_loc_3b464() {
        let contact = TargetEnemyContactApproach {
            replay_frame: 0,
            score_px: 6,
            result: "damage".to_string(),
            gate: "classify".to_string(),
            summary: "enemy_0A#01 ...".to_string(),
            same_frame_stomp: false,
        };

        assert!(
            target_enemy_contact_visual_follow_up_commands(Path::new("scenario.json"), &contact)
                .first()
                .expect("first command")
                .contains("--frames 0,1"),
            "frame-zero contacts should not duplicate frame 0"
        );
    }

    #[test]
    fn target_enemy_contact_from_frame_filters_early_contacts_loc_3b464() {
        let mut trace = vec![
            player_frame(10, GameMode::InGame, Some(0x00C0_0000), Some(0x00D0_0000)),
            player_frame(11, GameMode::InGame, Some(0x00C8_0000), Some(0x00D8_0000)),
        ];
        for (frame, result) in [(0, "damage"), (1, "none")] {
            trace[frame].gameplay_debug_fields = std::collections::BTreeMap::from([(
                "kid_stomp_contact_candidates".to_string(),
                format!(
                    "enemy_0A#01 gate=classify/result={result}/kid=[198,208,185,216]/obj=[190,215,200,216]/overlap=18,16/delta=0x00000000,0x00000000"
                ),
            )]);
        }

        assert_eq!(
            describe_target_enemy_contact_from_frame(&trace, 0x0A, 1),
            "enemy contact 0x0A: result none gate classify score_px 17 @1 same_frame_stomp=false enemy_0A#01 gate=classify/result=none/kid=[198,208,185,216]/obj=[190,215,200,216]/overlap=18,16/delta=0x00000000,0x00000000"
        );
    }
}
