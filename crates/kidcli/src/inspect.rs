use super::*;

pub(crate) fn inspect_level(manifest: &AssetManifest, level_spec: &str) -> Result<()> {
    let level_id = parse_level_id(level_spec)?;
    let level = manifest
        .levels
        .iter()
        .find(|level| level.level_id == level_id)
        .ok_or_else(|| anyhow!("level 0x{level_id:02X} not found in asset manifest"))?;
    let bundle = load_level_asset_bundle(manifest, level_id)?;

    println!("level: 0x{:02X}", level_id);
    if let Some(name) = &level.display_name {
        println!("name: {name}");
    }
    let effective_resource_level_id = resource_level_id(manifest, level_id);
    if effective_resource_level_id != level_id {
        println!("resource level: 0x{effective_resource_level_id:02X}");
    }
    if let Some(title_text_id) = level.title_text_id {
        println!("title text id: 0x{:02X}", title_text_id);
    }
    println!("resource paths: {}", level.resource_paths.join(", "));
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
            .map(|segment| segment.enemy_type_slots.as_slice())
            .unwrap_or(enemy_layout.enemy_type_slots.as_slice());
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

pub(crate) fn inspect_respawn_settle(
    manifest: &AssetManifest,
    level_spec: &str,
    steps: usize,
) -> Result<()> {
    let level_id = parse_level_id(level_spec)?;
    let mut runtime = kid_core::NativeRuntime::with_generated_level_layout(manifest.clone());
    runtime.warp_to_level(kid_core::LevelId(level_id));
    let frames = runtime.inspect_respawn_settle(kid_core::LevelId(level_id), steps);

    println!("level: 0x{level_id:02X}");
    if let Some(name) = runtime.level_name(kid_core::LevelId(level_id)) {
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
    let mut runtime = kid_core::NativeRuntime::with_generated_level_layout(manifest.clone());
    runtime.warp_to_level(kid_core::LevelId(level_id));
    let inspection = runtime
        .inspect_spawn_probes(kid_core::LevelId(level_id))
        .ok_or_else(|| anyhow!("level 0x{level_id:02X} has no generated block grid"))?;

    println!("level: 0x{level_id:02X}");
    if let Some(name) = runtime.level_name(kid_core::LevelId(level_id)) {
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

pub(crate) fn print_loaded_asset(label: &str, asset: Option<&kid_data::LoadedAsset>) {
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

pub(crate) fn summarise_enemy_object_types(objects: &[kid_data::EnemyLayoutObject]) -> String {
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

pub(crate) fn parse_level_id(value: &str) -> Result<u8> {
    let trimmed = value.trim();
    let hex = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    u8::from_str_radix(hex, 16)
        .with_context(|| format!("level id must be hex in the range 00-FF, got {value}"))
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
    probe: &kid_md::CpuProbe,
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
    frame_start: Option<usize>,
    frame_end: Option<usize>,
) {
    if let Some(start) = frame_start {
        let end = frame_end.unwrap_or(start);
        for frame in trace
            .iter()
            .filter(|frame| (start..=end).contains(&frame.frame_index))
        {
            let input = script
                .get(frame.frame_index)
                .copied()
                .map(kid_core::FrameInput::from_mask)
                .unwrap_or_default();
            println!(
                "frame {}: input {} mode {:?} camera {:08X},{:08X} kid {:08X},{:08X} vel {:08X},{:08X} ground {:?} invuln {} form_flash {} lives/hp {}/{} active_enemies {} active_platforms {} roster {}",
                frame.frame_index,
                describe_frame_input(input),
                frame.game_mode,
                frame.camera_x_pos,
                frame.camera_y_pos,
                frame.kid_x_pos.unwrap_or_default(),
                frame.kid_y_pos.unwrap_or_default(),
                frame.kid_x_velocity.unwrap_or_default() as u32,
                frame.kid_y_velocity.unwrap_or_default() as u32,
                frame.kid_on_ground,
                frame.invulnerable_frames,
                frame.form_change_flash_frames,
                frame.number_lives,
                frame.number_hitpoints,
                frame.active_enemy_object_count,
                frame.active_platform_object_count,
                describe_active_objects(frame)
            );
        }
    }
}

pub(crate) fn probe_native_scenario(
    manifest: &AssetManifest,
    scenario: Scenario,
    frame_start: Option<usize>,
    frame_end: Option<usize>,
) -> Result<()> {
    let script = scenario.script()?;
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

    print_trace_frames_in_range(&trace, &script, frame_start, frame_end);

    Ok(())
}

pub(crate) fn probe_native_gameplay_scenario(
    manifest: &AssetManifest,
    scenario: GameplayScenario,
    frame_start: Option<usize>,
    frame_end: Option<usize>,
) -> Result<()> {
    let script = scenario.script()?;
    let trace = run_native_script_for_gameplay(manifest, &script, scenario);
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
                "checkpoint {}@{}: {:?} level 0x{:02X} {} camera {:08X},{:08X} kid {:08X},{:08X} lives/hp {}/{} objects {}/{} active_enemies {} active_platforms {} roster {}",
                checkpoint.label,
                checkpoint.frame_index,
                frame.game_mode,
                frame.current_level.0,
                frame.current_level_name.as_deref().unwrap_or(""),
                frame.camera_x_pos,
                frame.camera_y_pos,
                frame.kid_x_pos.unwrap_or_default(),
                frame.kid_y_pos.unwrap_or_default(),
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

    print_trace_frames_in_range(&trace, &script, frame_start, frame_end);

    Ok(())
}

pub(crate) fn describe_frame_input(input: kid_core::FrameInput) -> String {
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
        .map(|object| match object.kind {
            kid_core::NativeObjectKind::Player => {
                format!("player@{:08X},{:08X}", object.x_pos, object.y_pos)
            }
            kid_core::NativeObjectKind::Enemy { object_type } => {
                format!(
                    "enemy_{object_type:02X}@{:08X},{:08X}",
                    object.x_pos, object.y_pos
                )
            }
            kid_core::NativeObjectKind::Platform { kind, subtype } => {
                format!(
                    "platform_{kind:02X}_{subtype:02X}@{:08X},{:08X}",
                    object.x_pos, object.y_pos
                )
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn trace_oracle_enemy_layout(
    repo_root: &Path,
    rom_path: &Path,
    scenario: Scenario,
) -> Result<()> {
    let env = ensure_oracles(repo_root)?;
    let run_dir = repo_root
        .join(".cache/native-oracle-runs")
        .join(scenario.slug());
    let oracle_dir = run_dir.join("oracle-trace");
    if oracle_dir.exists() {
        fs::remove_dir_all(&oracle_dir)
            .with_context(|| format!("failed to remove {}", oracle_dir.display()))?;
    }
    fs::create_dir_all(&oracle_dir)
        .with_context(|| format!("failed to create {}", oracle_dir.display()))?;

    let trace = run_jgenesis_enemy_layout_trace(&env, rom_path, scenario, &oracle_dir)?;
    let trace_path = run_dir.join("enemy-layout-trace.json");
    fs::create_dir_all(&run_dir)
        .with_context(|| format!("failed to create {}", run_dir.display()))?;
    fs::write(&trace_path, serde_json::to_vec_pretty(&trace)?)
        .with_context(|| format!("failed to write {}", trace_path.display()))?;
    print_enemy_layout_trace_summary(&trace_path, &trace);
    Ok(())
}

pub(crate) fn print_enemy_layout_trace_summary(
    path: &Path,
    trace: &OracleEnemyLayoutTraceManifest,
) {
    println!("scenario: {}", trace.scenario);
    println!("trace: {}", path.display());
    for checkpoint in &trace.checkpoints {
        let slot_summary = checkpoint
            .enemy_type_slots
            .iter()
            .map(|slot| format!("0x{slot:04X}"))
            .collect::<Vec<_>>()
            .join(", ");
        let visible_summary = checkpoint
            .visible_enemies
            .iter()
            .map(|enemy| format!("0x{:02X}@{:06X}", enemy.object_type, enemy.layout_ptr))
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "{}@{} camera {:08X},{:08X} header {:06X} object {:06X} axis {} slots [{}] upper 0x{:04X} batch 0x{:02X} count {} visible [{}]",
            checkpoint.label,
            checkpoint.frame_index,
            checkpoint.camera_x_pos,
            checkpoint.camera_y_pos,
            checkpoint.header_ptr,
            checkpoint.object_ptr,
            if checkpoint.axis_uses_y_position {
                "y"
            } else {
                "x"
            },
            slot_summary,
            checkpoint.upper_bound,
            checkpoint.object_batch_flags,
            checkpoint.object_count,
            visible_summary,
        );
    }
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

    let filename = format!("{:02X}.bin", level_id);
    let layout_path = out_dir.join(&filename);
    fs::write(&layout_path, &layout_data)
        .with_context(|| format!("failed to write {}", layout_path.display()))?;

    // Dump the Block List (prize/ghost/teleporter/hidden entries)
    let block_list_path = out_dir.join(format!("{:02X}.blocklist.bin", level_id));
    fs::write(&block_list_path, &block_list_data)
        .with_context(|| format!("failed to write {}", block_list_path.display()))?;

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
    let meta_path = out_dir.join(format!("{:02X}.json", level_id));
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
