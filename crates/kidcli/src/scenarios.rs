use anyhow::Result;
use clap::ValueEnum;
use kid_core::{NativeTraceFrame, PlayerForm, ScriptRunner};
use kid_data::AssetManifest;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Scenario {
    BootTitleIdle10s,
    BootTitleStartLevel1,
    BootTitleStartLevel1MoveRight5s,
    Level1EnemyWindow,
    WalkRight3s,
    JumpFromStandstill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum GameplayScenario {
    PauseResumeLevel1,
    DeathRespawnLevel1,
    DeathRespawnStartResumeLevel1,
    ProgressTwoTransitions,
    SoakLevel10m,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameCheckpoint {
    pub label: &'static str,
    pub frame_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameplayCheckpoint {
    pub label: &'static str,
    pub frame_index: usize,
}

const BOOT_TITLE_IDLE_10S_CHECKPOINTS: [FrameCheckpoint; 2] = [
    FrameCheckpoint {
        label: "title",
        frame_index: 360,
    },
    FrameCheckpoint {
        label: "title_tail",
        frame_index: 599,
    },
];

const BOOT_TITLE_START_LEVEL_1_CHECKPOINTS: [FrameCheckpoint; 3] = [
    FrameCheckpoint {
        label: "title",
        frame_index: 360,
    },
    FrameCheckpoint {
        label: "level_entry",
        frame_index: 520,
    },
    FrameCheckpoint {
        label: "level_settle",
        frame_index: 719,
    },
];

const WALK_RIGHT_3S_CHECKPOINTS: [FrameCheckpoint; 4] = [
    FrameCheckpoint {
        label: "level_settle",
        frame_index: 719,
    },
    FrameCheckpoint {
        label: "walk_mid",
        frame_index: 810,
    },
    FrameCheckpoint {
        label: "walk_turn",
        frame_index: 849,
    },
    FrameCheckpoint {
        label: "walk_end",
        frame_index: 899,
    },
];

const LEVEL_1_ENEMY_WINDOW_CHECKPOINTS: [FrameCheckpoint; 4] = [
    FrameCheckpoint {
        label: "level_settle",
        frame_index: 719,
    },
    FrameCheckpoint {
        label: "enemy_window_open",
        frame_index: 730,
    },
    FrameCheckpoint {
        label: "enemy_first_hit",
        frame_index: 786,
    },
    FrameCheckpoint {
        label: "enemy_window_tail",
        frame_index: 820,
    },
];

const JUMP_FROM_STANDSTILL_CHECKPOINTS: [FrameCheckpoint; 3] = [
    FrameCheckpoint {
        label: "level_settle",
        frame_index: 719,
    },
    FrameCheckpoint {
        label: "jump_apex",
        frame_index: 730,
    },
    FrameCheckpoint {
        label: "jump_land",
        frame_index: 780,
    },
];

const BOOT_TITLE_START_LEVEL_1_MOVE_RIGHT_5S_CHECKPOINTS: [FrameCheckpoint; 7] = [
    FrameCheckpoint {
        label: "title",
        frame_index: 360,
    },
    FrameCheckpoint {
        label: "level_entry",
        frame_index: 520,
    },
    FrameCheckpoint {
        label: "move_mid",
        frame_index: 840,
    },
    FrameCheckpoint {
        label: "move_turn",
        frame_index: 849,
    },
    FrameCheckpoint {
        label: "move_post_turn",
        frame_index: 899,
    },
    FrameCheckpoint {
        label: "move_tail",
        frame_index: 959,
    },
    FrameCheckpoint {
        label: "move_end",
        frame_index: 1_019,
    },
];

const PAUSE_RESUME_LEVEL_1_CHECKPOINTS: [GameplayCheckpoint; 4] = [
    GameplayCheckpoint {
        label: "level_settle",
        frame_index: 719,
    },
    GameplayCheckpoint {
        label: "pre_pause",
        frame_index: 839,
    },
    GameplayCheckpoint {
        label: "paused_tail",
        frame_index: 919,
    },
    GameplayCheckpoint {
        label: "resumed_tail",
        frame_index: 1_119,
    },
];

const DEATH_RESPAWN_LEVEL_1_CHECKPOINTS: [GameplayCheckpoint; 10] = [
    GameplayCheckpoint {
        label: "level_settle",
        frame_index: 719,
    },
    GameplayCheckpoint {
        label: "pre_death",
        frame_index: 1_680,
    },
    GameplayCheckpoint {
        label: "death_tail",
        frame_index: 1_762,
    },
    GameplayCheckpoint {
        label: "respawn_tail",
        frame_index: 1_919,
    },
    GameplayCheckpoint {
        label: "respawn_bootstrap_tail",
        frame_index: 9_000,
    },
    GameplayCheckpoint {
        label: "respawn_bootstrap_tail_9500",
        frame_index: 9_500,
    },
    GameplayCheckpoint {
        label: "respawn_bootstrap_tail_10500",
        frame_index: 10_500,
    },
    GameplayCheckpoint {
        label: "respawn_bootstrap_tail_12200",
        frame_index: 12_200,
    },
    GameplayCheckpoint {
        label: "respawn_bootstrap_tail_16000",
        frame_index: 16_000,
    },
    GameplayCheckpoint {
        label: "respawn_bootstrap_tail_22000",
        frame_index: 22_000,
    },
];

const DEATH_RESPAWN_START_RESUME_LEVEL_1_CHECKPOINTS: [GameplayCheckpoint; 5] = [
    GameplayCheckpoint {
        label: "level_settle",
        frame_index: 719,
    },
    GameplayCheckpoint {
        label: "pre_death",
        frame_index: 1_254,
    },
    GameplayCheckpoint {
        label: "respawn_hidden",
        frame_index: 1_919,
    },
    GameplayCheckpoint {
        label: "start_release",
        frame_index: 1_922,
    },
    GameplayCheckpoint {
        label: "resumed_tail",
        frame_index: 1_950,
    },
];

const PROGRESS_TWO_TRANSITIONS_CHECKPOINTS: [GameplayCheckpoint; 6] = [
    GameplayCheckpoint {
        label: "level_settle",
        frame_index: 719,
    },
    GameplayCheckpoint {
        label: "opening_hit",
        frame_index: 786,
    },
    GameplayCheckpoint {
        label: "progress_mid",
        frame_index: 1_999,
    },
    GameplayCheckpoint {
        label: "transition_tail",
        frame_index: 2_479,
    },
    GameplayCheckpoint {
        label: "level_2_settle",
        frame_index: 2_701,
    },
    GameplayCheckpoint {
        label: "level_2_live_tail",
        frame_index: 3_300,
    },
];

const SOAK_LEVEL_10M_CHECKPOINTS: [GameplayCheckpoint; 4] = [
    GameplayCheckpoint {
        label: "level_settle",
        frame_index: 719,
    },
    GameplayCheckpoint {
        label: "soak_2m30s",
        frame_index: 9_719,
    },
    GameplayCheckpoint {
        label: "soak_5m",
        frame_index: 18_719,
    },
    GameplayCheckpoint {
        label: "soak_end",
        frame_index: 36_719,
    },
];

/// A scenario defined at runtime — generated programmatically rather than
/// hard-coded as an enum variant. Carries its own script, frame count, and
/// checkpoints so both the native and oracle sides can execute it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicScenario {
    pub slug: String,
    /// Per-frame button mask (index = frame number).
    pub script: Vec<u8>,
    pub frame_count: usize,
    pub checkpoints: Vec<DynamicCheckpoint>,
    /// Optional probe-only starting position applied after a settled level
    /// warp. This lets native/oracle recordings reduce long routes to the
    /// specific gameplay window under investigation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_position: Option<DynamicStartPosition>,
    /// Optional probe-only camera position applied after a settled level warp.
    /// ASM `Camera_X_pos`/`Camera_Y_pos` live at `$FFFFF81C`/`$FFFFF820`
    /// (variables.asm); visual probes need these patched with Kid so the
    /// target gameplay window is actually on screen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub camera_position: Option<DynamicStartPosition>,
    /// Optional probe-only helmet/form id applied after a settled level warp.
    /// ASM `Current_Helmet_ID` lives at `$FFFFFC46` and HP at `$FFFFFC40`
    /// (variables.asm); form-specific probes need both patched together.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playtest_form: Option<u16>,
    /// Optional probe-only diamond count applied after a settled level warp.
    /// ASM `Number_Diamonds` lives at `$FFFFFC42`; `DiamondPower_Check`
    /// (`kid.asm:44357`) compares/subtracts it before dispatching the weapon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_diamond_count: Option<u16>,
    /// Target level to warp to before running the script.
    /// Both native and oracle boot to level 0 first, then advance
    /// to this level using their own mechanism (native: direct skip,
    /// oracle: memory write + cheat flag). The script runs after warping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warp_to_level: Option<u8>,
    /// When true, native/oracle warped scenarios wait until the target level is
    /// live and unpaused before scenario frame 0. Focused object probes need
    /// this so checkpoint 0 samples gameplay, not the title-card shell.
    #[serde(default, skip_serializing_if = "is_false")]
    pub settle_oracle_warp: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicCheckpoint {
    pub label: String,
    pub frame_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicStartPosition {
    pub x: u16,
    pub y: u16,
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde skip_serializing_if requires a reference predicate"
)]
fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct NativeDynamicRunOptions {
    pub(crate) settled_warp: bool,
    pub(crate) trace_physics: bool,
}

pub(crate) fn run_native_dynamic_scenario(
    manifest: AssetManifest,
    scenario: &DynamicScenario,
    options: NativeDynamicRunOptions,
) -> Result<Vec<NativeTraceFrame>> {
    let mut runner = ScriptRunner::new(manifest);
    if let Some(level) = scenario.warp_to_level {
        runner = runner.warp_to_level(level);
    }
    if options.settled_warp {
        runner = runner.settle_warp();
    } else if scenario.settle_oracle_warp {
        runner = runner.settle_recording_warp();
    }
    if options.trace_physics {
        runner = runner.trace_physics();
    }
    if let Some(position) = scenario.start_position {
        runner = runner.probe_start_position(position.x, position.y);
    }
    if let Some(position) = scenario.camera_position {
        runner = runner.probe_camera_position(position.x, position.y);
    }
    if let Some(helmet_id) = scenario.playtest_form {
        let form = PlayerForm::from_helmet_id(helmet_id)
            .ok_or_else(|| anyhow::anyhow!("invalid playtest_form helmet id {helmet_id}"))?;
        runner = runner.playtest_form(form);
    }
    if let Some(count) = scenario.initial_diamond_count {
        runner = runner.initial_diamond_count(count);
    }

    Ok(runner.run(&scenario.script, scenario.frame_count))
}

/// Idle on a level for `idle_frames` (default 300, ~5 s at 60 fps).
///
/// Uses `warp_to_level` to reach the target level. Both native and oracle
/// boot to level 0 in their own pre-run phase, then advance to `level_id`.
/// The script itself is pure idle — no button presses — so the comparison
/// phase is identical on both sides.
pub fn idle_level_scenario(level_id: u8, idle_frames: Option<usize>) -> DynamicScenario {
    let idle = idle_frames.unwrap_or(300);
    let settle_frames: usize = 60;
    let total_frames = settle_frames + idle;

    let checkpoints = vec![
        DynamicCheckpoint {
            label: "level_settle".into(),
            frame_index: settle_frames,
        },
        DynamicCheckpoint {
            label: "idle_tail".into(),
            frame_index: total_frames.saturating_sub(1),
        },
    ];

    DynamicScenario {
        slug: format!("idle_level_{level_id:02x}"),
        script: vec![0u8; total_frames.saturating_add(1)],
        frame_count: total_frames,
        checkpoints,
        start_position: None,
        camera_position: None,
        playtest_form: None,
        initial_diamond_count: None,
        warp_to_level: Some(level_id),
        settle_oracle_warp: false,
    }
}

/// Idle on a level with visual key-frame checkpoints for full-frame sweeps.
///
/// The state tier keeps the compact settle/tail checkpoints. Full-frame visual
/// diagnosis needs early and mid-window probes so renderer drift is visible
/// before it accumulates into the final idle frame.
pub fn idle_level_full_frame_scenario(level_id: u8) -> DynamicScenario {
    let mut scenario = idle_level_scenario(level_id, None);
    scenario.slug = format!("idle_level_full_frames_{level_id:02x}");
    scenario.settle_oracle_warp = true;
    scenario.checkpoints = vec![
        DynamicCheckpoint {
            label: "frame_000".into(),
            frame_index: 0,
        },
        DynamicCheckpoint {
            label: "frame_030".into(),
            frame_index: 30,
        },
        DynamicCheckpoint {
            label: "level_settle".into(),
            frame_index: 60,
        },
        DynamicCheckpoint {
            label: "frame_120".into(),
            frame_index: 120,
        },
        DynamicCheckpoint {
            label: "frame_300".into(),
            frame_index: 300,
        },
        DynamicCheckpoint {
            label: "idle_tail".into(),
            frame_index: 359,
        },
    ];
    scenario
}

/// Walk right for `walk_frames` with periodic jumps. Useful for verifying
/// movement, enemy encounters, and basic traversal on any level.
fn walk_right_scenario_for(
    slug: String,
    level_id: u8,
    walk_frames: usize,
    playtest_form: Option<PlayerForm>,
) -> DynamicScenario {
    let settle_frames: usize = 60;
    let total_frames = settle_frames + walk_frames;
    let walk_start = settle_frames;
    let walk_end = total_frames.saturating_sub(1);

    let mut script = vec![0u8; total_frames.saturating_add(1)];
    // Hold right throughout the walk phase
    apply_range(&mut script, walk_start, walk_end, 1 << 3);
    // Periodic jumps every 60 frames (1 second), 4-frame press
    add_periodic_jump(&mut script, walk_start + 30, walk_end, 60, 4);

    let checkpoints = vec![
        DynamicCheckpoint {
            label: "level_settle".into(),
            frame_index: settle_frames,
        },
        DynamicCheckpoint {
            label: "walk_quarter".into(),
            frame_index: settle_frames + walk_frames / 4,
        },
        DynamicCheckpoint {
            label: "walk_half".into(),
            frame_index: settle_frames + walk_frames / 2,
        },
        DynamicCheckpoint {
            label: "walk_end".into(),
            frame_index: walk_end,
        },
    ];

    DynamicScenario {
        slug,
        script,
        frame_count: total_frames,
        checkpoints,
        start_position: None,
        camera_position: None,
        playtest_form: playtest_form.map(PlayerForm::helmet_id),
        initial_diamond_count: None,
        warp_to_level: Some(level_id),
        settle_oracle_warp: false,
    }
}

pub fn walk_right_scenario(level_id: u8, walk_frames: usize) -> DynamicScenario {
    walk_right_scenario_for(
        format!("walk_right_{level_id:02x}"),
        level_id,
        walk_frames,
        None,
    )
}

/// Natural Level 00 approach for #211 form animation capture. Unlike the
/// direct mid-slope probe, this route lets ASM loc_9D22/loc_9F18 enter the
/// `$4000` slope path through ordinary rightward movement from the level spawn.
pub fn walk_right_red_stealth_scenario(level_id: u8, walk_frames: usize) -> DynamicScenario {
    walk_right_scenario_for(
        format!("walk_right_red_stealth_{level_id:02x}"),
        level_id,
        walk_frames,
        Some(PlayerForm::RedStealth),
    )
}

pub fn walk_right_iron_knight_scenario(level_id: u8, walk_frames: usize) -> DynamicScenario {
    walk_right_scenario_for(
        format!("walk_right_iron_knight_{level_id:02x}"),
        level_id,
        walk_frames,
        Some(PlayerForm::IronKnight),
    )
}

pub fn walk_right_juggernaut_scenario(level_id: u8, walk_frames: usize) -> DynamicScenario {
    walk_right_scenario_for(
        format!("walk_right_juggernaut_{level_id:02x}"),
        level_id,
        walk_frames,
        Some(PlayerForm::Juggernaut),
    )
}

/// Focused 2026-05-30 playtest repro for the first Blue Lake Woods 1
/// up-right slope.
///
/// This starts directly on the ROM `$4000` SlopeUp cell around layout
/// column 47 / row 14 and holds Right without jump noise. It is intentionally
/// probe-shaped, not a broad traversal route: state checkpoints include frame
/// 0 so native/oracle can prove whether the patched live object and camera
/// begin from the same gameplay window before slope animation/facing diverges.
fn level_00_slope_right_probe_scenario_for(
    slug: &str,
    playtest_form: Option<PlayerForm>,
) -> DynamicScenario {
    const FRAME_COUNT: usize = 120;
    const RIGHT: u8 = 1 << 3;

    let mut script = vec![0u8; FRAME_COUNT.saturating_add(1)];
    apply_range(&mut script, 0, FRAME_COUNT.saturating_sub(1), RIGHT);

    DynamicScenario {
        slug: slug.into(),
        script,
        frame_count: FRAME_COUNT,
        checkpoints: vec![
            DynamicCheckpoint {
                label: "slope_probe_start".into(),
                frame_index: 0,
            },
            DynamicCheckpoint {
                label: "slope_first_step".into(),
                frame_index: 1,
            },
            DynamicCheckpoint {
                label: "slope_anim_window".into(),
                frame_index: 16,
            },
            DynamicCheckpoint {
                label: "slope_mid".into(),
                frame_index: 60,
            },
            DynamicCheckpoint {
                label: "slope_tail".into(),
                frame_index: FRAME_COUNT.saturating_sub(1),
            },
        ],
        start_position: Some(DynamicStartPosition { x: 760, y: 231 }),
        camera_position: Some(DynamicStartPosition { x: 608, y: 32 }),
        playtest_form: playtest_form.map(PlayerForm::helmet_id),
        initial_diamond_count: None,
        warp_to_level: Some(0x00),
        settle_oracle_warp: true,
    }
}

pub fn level_00_slope_right_probe_scenario() -> DynamicScenario {
    level_00_slope_right_probe_scenario_for("level_00_slope_right_probe", None)
}

/// ASM GfxObjects_Collision_UpSlope (`$4000` cells) feeds Kid movement through
/// loc_9D22/loc_9F18; these form variants keep that exact window but seed the
/// playtest helmet IDs so animation latches can be compared per form.
pub fn level_00_slope_right_probe_red_stealth_scenario() -> DynamicScenario {
    level_00_slope_right_probe_scenario_for(
        "level_00_slope_right_probe_red_stealth",
        Some(PlayerForm::RedStealth),
    )
}

pub fn level_00_slope_right_probe_iron_knight_scenario() -> DynamicScenario {
    level_00_slope_right_probe_scenario_for(
        "level_00_slope_right_probe_iron_knight",
        Some(PlayerForm::IronKnight),
    )
}

pub fn level_00_slope_right_probe_juggernaut_scenario() -> DynamicScenario {
    level_00_slope_right_probe_scenario_for(
        "level_00_slope_right_probe_juggernaut",
        Some(PlayerForm::Juggernaut),
    )
}

/// Walk right and jump to headbutt prize blocks on level 0x00.
/// The prize blocks in Blue Lake Woods 1 are above the starting position;
/// the script walks right briefly then jumps under them.
pub fn prize_block_scenario(level_id: u8) -> DynamicScenario {
    let settle_frames: usize = 60;
    // Walk right 60 frames → jump → land → walk right 60 frames → jump again
    let total_frames: usize = settle_frames + 300;

    let mut script = vec![0u8; total_frames.saturating_add(1)];

    // Phase 1: walk right towards prize blocks (frames 60-119)
    apply_range(&mut script, settle_frames, settle_frames + 59, 1 << 3);
    // Phase 2: jump under prize block (frames 120-124, hold B for 5 frames)
    apply_range(
        &mut script,
        settle_frames + 60,
        settle_frames + 64,
        (1 << 3) | (1 << 4),
    );
    // Phase 3: keep walking right (frames 125-179)
    apply_range(&mut script, settle_frames + 65, settle_frames + 179, 1 << 3);
    // Phase 4: second jump under next block (frames 180-184)
    apply_range(
        &mut script,
        settle_frames + 180,
        settle_frames + 184,
        (1 << 3) | (1 << 4),
    );
    // Phase 5: coast to end (frames 185-359)
    apply_range(
        &mut script,
        settle_frames + 185,
        total_frames.saturating_sub(1),
        1 << 3,
    );

    let checkpoints = vec![
        DynamicCheckpoint {
            label: "level_settle".into(),
            frame_index: settle_frames,
        },
        DynamicCheckpoint {
            label: "pre_first_jump".into(),
            frame_index: settle_frames + 59,
        },
        DynamicCheckpoint {
            label: "post_first_jump".into(),
            frame_index: settle_frames + 90,
        },
        DynamicCheckpoint {
            label: "pre_second_jump".into(),
            frame_index: settle_frames + 179,
        },
        DynamicCheckpoint {
            label: "post_second_jump".into(),
            frame_index: settle_frames + 210,
        },
        DynamicCheckpoint {
            label: "scenario_end".into(),
            frame_index: total_frames.saturating_sub(1),
        },
    ];

    DynamicScenario {
        slug: format!("prize_block_{level_id:02x}"),
        script,
        frame_count: total_frames,
        checkpoints,
        start_position: None,
        camera_position: None,
        playtest_form: None,
        initial_diamond_count: None,
        warp_to_level: Some(level_id),
        settle_oracle_warp: false,
    }
}

/// Focused #216 prize/block audio audit for Blue Lake Woods 1.
///
/// The generic `prize_block_scenario(0x00)` walks through an enemy contact
/// before the block-hit cue, polluting `audio-events.txt` with damage SFX.
/// This route starts just under the first usable prize cluster, performs a
/// vertical jump, then idles so the recording captures `sfx_Block_hit` and the
/// later `sfx_Diamond_prize` without a damage event.
pub fn issue_216_prize_audio_audit_scenario() -> DynamicScenario {
    const FRAME_COUNT: usize = 180;
    const JUMP: u8 = 1 << 4;

    let mut script = vec![0u8; FRAME_COUNT.saturating_add(1)];
    apply_range(&mut script, 20, 24, JUMP);

    DynamicScenario {
        slug: "issue_216_prize_audio_audit_00".into(),
        script,
        frame_count: FRAME_COUNT,
        checkpoints: vec![
            DynamicCheckpoint {
                label: "route_start".into(),
                frame_index: 0,
            },
            DynamicCheckpoint {
                label: "jump_input".into(),
                frame_index: 20,
            },
            DynamicCheckpoint {
                label: "block_hit_window".into(),
                frame_index: 31,
            },
            DynamicCheckpoint {
                label: "diamond_pickup_window".into(),
                frame_index: 132,
            },
            DynamicCheckpoint {
                label: "route_tail".into(),
                frame_index: FRAME_COUNT.saturating_sub(1),
            },
        ],
        start_position: Some(DynamicStartPosition { x: 230, y: 271 }),
        camera_position: Some(DynamicStartPosition { x: 16, y: 136 }),
        playtest_form: None,
        initial_diamond_count: None,
        warp_to_level: Some(0x00),
        settle_oracle_warp: true,
    }
}

/// Idle on a level so a known enemy id has time to enter its initialised state,
/// then sample the enemy roster so native-vs-oracle parity can detect missing
/// or misclassified spawns.
///
/// Used to convert "is enemy X actually working?" audit questions into a
/// regression-grade scenario: the slug embeds the enemy id, the warp targets
/// a level that hosts it, and the checkpoints bracket the spawn window so a
/// later sweep run can read off the enemy roster at each label.
///
/// The script is pure idle (no input). For bosses like HeadyMetal (Plethora 1,
/// level 0x33) the player spawns inside the arena; oracle and native both pull
/// the boss into the active object pool during the spawn-animation window
/// (~120 frames per `Enemy20_HeadyMetal_Init` / `loc_3761E`). A 600-frame
/// idle covers the spawn, the early phase transitions, and the first hit
/// window without input noise.
pub fn enemy_spawn_audit_scenario(enemy_id: u8, level_id: u8) -> DynamicScenario {
    let settle_frames: usize = 60;
    let idle_frames: usize = 600;
    let total_frames = settle_frames + idle_frames;
    let mid = settle_frames + idle_frames / 2;
    let tail = total_frames.saturating_sub(1);

    let checkpoints = vec![
        DynamicCheckpoint {
            label: "level_settle".into(),
            frame_index: settle_frames,
        },
        DynamicCheckpoint {
            label: "enemy_spawn_window".into(),
            frame_index: settle_frames + 180,
        },
        DynamicCheckpoint {
            label: "enemy_visible_mid".into(),
            frame_index: mid,
        },
        DynamicCheckpoint {
            label: "enemy_visible_tail".into(),
            frame_index: tail,
        },
    ];

    DynamicScenario {
        slug: format!("enemy_spawn_audit_{enemy_id:02x}_level_{level_id:02x}"),
        script: vec![0u8; total_frames.saturating_add(1)],
        frame_count: total_frames,
        checkpoints,
        start_position: None,
        camera_position: None,
        playtest_form: None,
        initial_diamond_count: None,
        warp_to_level: Some(level_id),
        settle_oracle_warp: false,
    }
}

/// Idle inside an active lava-weather level so the ROM's special-effect object
/// workers are visible to native-vs-oracle comparison.
///
/// ASM `Init_SpecialEffect_Lava -> loc_14C5E/loc_14C6A` allocates lava geyser
/// objects from the per-level position table, then `Obj_Lava_Geyser` runs the
/// script loop starting at `loc_147BA`. Keeping this as a named scenario makes
/// #205's weather-object harness reproducible without checking in `.cache/`
/// recordings.
pub fn lava_weather_audit_scenario(level_id: u8) -> DynamicScenario {
    let frame_count: usize = 180;

    DynamicScenario {
        slug: format!("lava_weather_audit_{level_id:02x}"),
        script: vec![0u8; frame_count.saturating_add(1)],
        frame_count,
        checkpoints: vec![
            DynamicCheckpoint {
                label: "weather_object_start".into(),
                frame_index: 0,
            },
            DynamicCheckpoint {
                label: "weather_object_window_30".into(),
                frame_index: 30,
            },
            DynamicCheckpoint {
                label: "weather_object_window_60".into(),
                frame_index: 60,
            },
            DynamicCheckpoint {
                label: "weather_object_window_120".into(),
                frame_index: 120,
            },
            DynamicCheckpoint {
                label: "weather_object_tail".into(),
                frame_index: frame_count.saturating_sub(1),
            },
        ],
        start_position: None,
        camera_position: None,
        playtest_form: None,
        initial_diamond_count: None,
        warp_to_level: Some(level_id),
        settle_oracle_warp: true,
    }
}

fn diamond_power_activation_audit_scenario(
    level_id: u8,
    slug_name: &str,
    playtest_form: u16,
    initial_diamond_count: u16,
) -> DynamicScenario {
    const FRAME_COUNT: usize = 120;
    const START_A: u8 = (1 << 7) | (1 << 6);

    let mut script = vec![0u8; FRAME_COUNT.saturating_add(1)];
    apply_range(&mut script, 0, 2, START_A);

    let mut checkpoints = vec![
        DynamicCheckpoint {
            label: "diamond_power_input".into(),
            frame_index: 0,
        },
        DynamicCheckpoint {
            label: "diamond_power_activation".into(),
            frame_index: 1,
        },
        DynamicCheckpoint {
            label: "diamond_power_slot_window".into(),
            frame_index: 8,
        },
        DynamicCheckpoint {
            label: "diamond_power_mid".into(),
            frame_index: 32,
        },
    ];

    // ASM `DiamondPower_Check` dispatches the power object quickly, but #206
    // failures can surface much later after the settled warp starts moving.
    // Keep a dense tail window so recording verification identifies the first
    // causal frame instead of only reporting the final-frame divergence.
    for frame_index in (40..FRAME_COUNT.saturating_sub(1)).step_by(8) {
        checkpoints.push(DynamicCheckpoint {
            label: format!("diamond_power_tail_probe_{frame_index:03}"),
            frame_index,
        });
    }
    for frame_index in 57..64 {
        checkpoints.push(DynamicCheckpoint {
            label: format!("diamond_power_transition_probe_{frame_index:03}"),
            frame_index,
        });
    }
    checkpoints.sort_by_key(|checkpoint| checkpoint.frame_index);
    checkpoints.push(DynamicCheckpoint {
        label: "diamond_power_tail".into(),
        frame_index: FRAME_COUNT.saturating_sub(1),
    });

    DynamicScenario {
        slug: format!("{slug_name}_{level_id:02x}"),
        script,
        frame_count: FRAME_COUNT,
        checkpoints,
        start_position: None,
        camera_position: None,
        playtest_form: Some(playtest_form),
        initial_diamond_count: Some(initial_diamond_count),
        warp_to_level: Some(level_id),
        settle_oracle_warp: true,
    }
}

/// Focused #206 Diamond Power audit for The Kid's Circle of Doom.
///
/// ASM `DiamondPower_Check` spends from `Number_Diamonds` and dispatches id 0
/// to `DiamondPower_Init_CircleOfDoom` / `loc_3F7D0`. The probe seeds exactly
/// 20 diamonds, holds Start+A across the first input window, and samples the
/// early object lifecycle where native currently uses live Circle slots.
pub fn diamond_power_circle_audit_scenario(level_id: u8) -> DynamicScenario {
    diamond_power_activation_audit_scenario(level_id, "diamond_power_circle_audit", 0, 20)
}

/// Focused #206 Diamond Power audit for Berzerker's Wall of Death.
///
/// ASM `DiamondPower_Check` spends 20 diamonds, then spends the tier-2 30
/// diamonds and dispatches Berzerker to `DiamondPower_Init_WallOfDeath`
/// / `loc_3FFE4`. The probe seeds exactly 50 diamonds so the tier branch is
/// deliberate and does not depend on ignored `.cache` state.
pub fn diamond_power_wall_audit_scenario(level_id: u8) -> DynamicScenario {
    diamond_power_activation_audit_scenario(level_id, "diamond_power_wall_audit", 7, 50)
}

/// Focused #206 Diamond Power audit for Micromax's MiniSnake.
///
/// ASM `DiamondPower_Check` dispatches Micromax tier 1 to
/// `DiamondPower_Init_MiniSnake loc_3F8EE`, which then converges on the shared
/// `DiamondPower_Init_Snake loc_3F91C` / `DiamondPower_Main_Snake loc_3F978`
/// slot path. The probe seeds exactly 20 diamonds so it stays on MiniSnake
/// rather than tier-2 DeathSnake.
pub fn diamond_power_snake_audit_scenario(level_id: u8) -> DynamicScenario {
    diamond_power_activation_audit_scenario(level_id, "diamond_power_snake_audit", 9, 20)
}

/// Focused #206 Diamond Power audit for Skycutter's Invulnerability.
///
/// ASM `DiamondPower_Check` dispatches Skycutter tier 1 to
/// `DiamondPower_Init_Invulnerability loc_3FC78`, which seeds the shared
/// `DiamondPower_Main_Invulnerability_And_SamuraiHaze loc_3FD28` DP-object
/// slot path. The probe seeds exactly 20 diamonds so it stays on tier 1.
pub fn diamond_power_invulnerability_audit_scenario(level_id: u8) -> DynamicScenario {
    diamond_power_activation_audit_scenario(level_id, "diamond_power_invulnerability_audit", 1, 20)
}

/// Focused #206 Diamond Power audit for Red Stealth's Samurai Haze.
///
/// ASM `DiamondPower_Check` dispatches Red Stealth tier 1 to
/// `DiamondPower_Init_SamuraiHaze loc_3FC82`, which then reuses the
/// invulnerability/haze display worker at `loc_3FD28`. The probe seeds exactly
/// 20 diamonds so it stays on tier 1 rather than tier-2 Shuriken.
pub fn diamond_power_samurai_haze_audit_scenario(level_id: u8) -> DynamicScenario {
    diamond_power_activation_audit_scenario(level_id, "diamond_power_samurai_haze_audit", 3, 20)
}

/// Focused #214 Highwater Pass shooter/barrier live-route audit.
///
/// ASM `loc_9564 -> loc_EAE4 -> loc_EC76 -> loc_F02A` is the live Berzerker
/// route from the Level 02 spawn: holding Right builds the head-forward hitbox,
/// touches the stacked `$EA1F` shooter cells, emits typed shooter-block shots,
/// and should erase the first `$E203` ice barrier rows. This scenario keeps the
/// route as a reproducible recording source instead of relying on ignored
/// `.cache` playtest bundles.
pub fn highwater_shooter_barrier_audit_scenario() -> DynamicScenario {
    const FRAME_COUNT: usize = 150;
    const RIGHT: u8 = 1 << 3;

    let mut script = vec![0u8; FRAME_COUNT.saturating_add(1)];
    apply_range(&mut script, 0, FRAME_COUNT.saturating_sub(1), RIGHT);

    DynamicScenario {
        slug: "highwater_shooter_barrier_audit_02".into(),
        script,
        frame_count: FRAME_COUNT,
        checkpoints: vec![
            DynamicCheckpoint {
                label: "route_start".into(),
                frame_index: 0,
            },
            DynamicCheckpoint {
                label: "charge_building".into(),
                frame_index: 30,
            },
            DynamicCheckpoint {
                label: "shooter_contact_window".into(),
                frame_index: 60,
            },
            DynamicCheckpoint {
                label: "projectile_barrier_window".into(),
                frame_index: 90,
            },
            DynamicCheckpoint {
                label: "first_shooter_objects_visible_loc_ec76".into(),
                frame_index: 92,
            },
            DynamicCheckpoint {
                label: "shooter_cascade_window_loc_f04a".into(),
                frame_index: 104,
            },
            DynamicCheckpoint {
                label: "barrier_clear_check".into(),
                frame_index: 120,
            },
            DynamicCheckpoint {
                label: "route_tail".into(),
                frame_index: FRAME_COUNT.saturating_sub(1),
            },
        ],
        start_position: None,
        camera_position: None,
        playtest_form: Some(7),
        initial_diamond_count: None,
        warp_to_level: Some(0x02),
        settle_oracle_warp: true,
    }
}

/// Focused #215 Highwater Pass water/foreground priority audit.
///
/// ASM `Load_BackgroundMappings_Layered loc_12910` builds Mountain Plane B
/// with the `$7E5` follow-up art, then `Transfer_ScrollDataToVRAM loc_300BA`
/// latches the camera scroll used by the visible waterfall/foreground holes.
/// This reproduces the live Berzerker route from the Level 02 spawn far enough
/// to break the early smash-column stopper and sample the early water hole and
/// waterfall column without relying on ignored cache recordings.
pub fn highwater_water_priority_audit_scenario() -> DynamicScenario {
    const FRAME_COUNT: usize = 1_860;
    const RIGHT: u8 = 1 << 3;

    let mut script = vec![0u8; FRAME_COUNT.saturating_add(1)];
    apply_range(&mut script, 0, FRAME_COUNT.saturating_sub(1), RIGHT);

    DynamicScenario {
        slug: "highwater_water_priority_audit_02".into(),
        script,
        frame_count: FRAME_COUNT,
        checkpoints: vec![
            DynamicCheckpoint {
                label: "level_settle".into(),
                frame_index: 60,
            },
            DynamicCheckpoint {
                label: "water_hole_entry_loc_12910".into(),
                frame_index: 510,
            },
            DynamicCheckpoint {
                label: "waterfall_overlap_loc_300ba".into(),
                frame_index: 960,
            },
            DynamicCheckpoint {
                label: "waterfall_palette_tail_loc_1223c".into(),
                frame_index: FRAME_COUNT.saturating_sub(1),
            },
        ],
        start_position: None,
        camera_position: None,
        playtest_form: Some(7),
        initial_diamond_count: None,
        warp_to_level: Some(0x02),
        settle_oracle_warp: true,
    }
}

/// Focused #238 Highwater Pass level-complete transition audit.
///
/// Latest live playtest reported instant death after finishing Highwater. A
/// direct Level 0x03/0x04 settled warp does not exercise ASM `loc_D7F4`'s
/// normal flagpole-to-title-card hand-off, so this probe starts at the real
/// Highwater Pass 1 flag, waits for the score-board transition, then presses
/// Start on the title-card shell to exercise ASM `loc_1AE2E`.
pub fn highwater_flag_transition_audit_scenario() -> DynamicScenario {
    const FRAME_COUNT: usize = 2_400;
    const TITLE_CARD_START_FRAME: usize = 1_100;
    let mut script = vec![0u8; FRAME_COUNT.saturating_add(1)];
    script[TITLE_CARD_START_FRAME] = 1 << 7;

    DynamicScenario {
        slug: "highwater_flag_transition_audit_02".into(),
        script,
        frame_count: FRAME_COUNT,
        checkpoints: vec![
            DynamicCheckpoint {
                label: "flag_touch".into(),
                frame_index: 0,
            },
            DynamicCheckpoint {
                label: "scoreboard_window".into(),
                frame_index: 600,
            },
            DynamicCheckpoint {
                label: "transition_window".into(),
                frame_index: 1_200,
            },
            DynamicCheckpoint {
                label: "next_level_entry".into(),
                frame_index: 1_800,
            },
            DynamicCheckpoint {
                label: "next_level_survival_tail".into(),
                frame_index: FRAME_COUNT.saturating_sub(1),
            },
        ],
        start_position: Some(DynamicStartPosition { x: 912, y: 144 }),
        camera_position: Some(DynamicStartPosition { x: 752, y: 0 }),
        playtest_form: None,
        initial_diamond_count: None,
        warp_to_level: Some(0x02),
        settle_oracle_warp: true,
    }
}

/// Focused #267 Highwater Pass floor-contact flag audit.
///
/// Unlike `highwater_flag_transition_audit_scenario`, this starts Kid just left
/// of the real Highwater Pass 1 flag and walks right into it. The route keeps
/// the same ASM `loc_D7F4` title-card hand-off coverage while proving the flag
/// can trigger from floor-level side contact instead of an initial overlap.
pub fn highwater_flag_floor_contact_audit_scenario() -> DynamicScenario {
    const FRAME_COUNT: usize = 2_400;
    const APPROACH_END_FRAME: usize = 89;
    const TITLE_CARD_START_FRAME: usize = 1_100;
    const RIGHT: u8 = 1 << 3;
    const START: u8 = 1 << 7;

    let mut script = vec![0u8; FRAME_COUNT.saturating_add(1)];
    apply_range(&mut script, 0, APPROACH_END_FRAME, RIGHT);
    script[TITLE_CARD_START_FRAME] = START;

    DynamicScenario {
        slug: "highwater_flag_floor_contact_audit_02".into(),
        script,
        frame_count: FRAME_COUNT,
        checkpoints: vec![
            DynamicCheckpoint {
                label: "floor_flag_approach".into(),
                frame_index: 0,
            },
            DynamicCheckpoint {
                label: "floor_flag_contact_window_loc_31d7a".into(),
                frame_index: 45,
            },
            DynamicCheckpoint {
                label: "scoreboard_window_loc_d7f4".into(),
                frame_index: 600,
            },
            DynamicCheckpoint {
                label: "next_level_entry_loc_1ae2e".into(),
                frame_index: 1_800,
            },
            DynamicCheckpoint {
                label: "next_level_survival_tail".into(),
                frame_index: FRAME_COUNT.saturating_sub(1),
            },
        ],
        start_position: Some(DynamicStartPosition { x: 888, y: 144 }),
        camera_position: Some(DynamicStartPosition { x: 728, y: 0 }),
        playtest_form: None,
        initial_diamond_count: None,
        warp_to_level: Some(0x02),
        settle_oracle_warp: true,
    }
}

/// Focused Highwater Pass 2 Juggernaut level-complete transition audit.
///
/// Latest live playtest reported an instant death after finishing Level 0x03
/// as the Rhino/Juggernaut form. This starts at Highwater Pass 2's real flag,
/// carries Juggernaut through ASM `loc_D7F4`'s flagpole-to-title-card path, then
/// presses Start in ASM `loc_1AE2E`'s title-card shell to enter Level 0x04.
pub fn highwater_pass2_juggernaut_flag_transition_audit_scenario() -> DynamicScenario {
    const FRAME_COUNT: usize = 2_400;
    const TITLE_CARD_START_FRAME: usize = 1_100;
    let mut script = vec![0u8; FRAME_COUNT.saturating_add(1)];
    script[TITLE_CARD_START_FRAME] = 1 << 7;

    DynamicScenario {
        slug: "highwater_pass2_juggernaut_flag_transition_audit_03".into(),
        script,
        frame_count: FRAME_COUNT,
        checkpoints: vec![
            DynamicCheckpoint {
                label: "flag_touch".into(),
                frame_index: 0,
            },
            DynamicCheckpoint {
                label: "scoreboard_window".into(),
                frame_index: 600,
            },
            DynamicCheckpoint {
                label: "transition_window".into(),
                frame_index: 1_200,
            },
            DynamicCheckpoint {
                label: "next_level_entry".into(),
                frame_index: 1_800,
            },
            DynamicCheckpoint {
                label: "next_level_survival_tail".into(),
                frame_index: FRAME_COUNT.saturating_sub(1),
            },
        ],
        start_position: Some(DynamicStartPosition { x: 1_552, y: 1_040 }),
        camera_position: Some(DynamicStartPosition { x: 1_280, y: 896 }),
        playtest_form: Some(5),
        initial_diamond_count: None,
        warp_to_level: Some(0x03),
        settle_oracle_warp: true,
    }
}

/// Focused #250 Highwater Pass 2 touch-grown shooter block audit.
///
/// Level 0x03's decoded layout contains `$E919` cells at `(20,9)` and `(23,9)`.
/// This starts Juggernaut just above the first cell so native/oracle can compare
/// ASM `loc_11430/sub_114F4`'s touch-block restore path before any flag or
/// traversal logic muddies the evidence.
pub fn highwater_pass2_grow_shooter_touch_audit_scenario() -> DynamicScenario {
    const FRAME_COUNT: usize = 180;

    let script = vec![0u8; FRAME_COUNT.saturating_add(1)];

    DynamicScenario {
        slug: "highwater_pass2_grow_shooter_touch_audit_03".into(),
        script,
        frame_count: FRAME_COUNT,
        checkpoints: vec![
            DynamicCheckpoint {
                label: "touch_probe_start_loc_11430".into(),
                frame_index: 0,
            },
            DynamicCheckpoint {
                label: "worker_wake_window_sub_114f4".into(),
                frame_index: 24,
            },
            DynamicCheckpoint {
                label: "shooter_scan_window_sub_f096".into(),
                frame_index: 60,
            },
            DynamicCheckpoint {
                label: "touch_probe_tail".into(),
                frame_index: FRAME_COUNT.saturating_sub(1),
            },
        ],
        start_position: Some(DynamicStartPosition { x: 328, y: 143 }),
        camera_position: Some(DynamicStartPosition { x: 176, y: 0 }),
        playtest_form: Some(5),
        initial_diamond_count: None,
        warp_to_level: Some(0x03),
        settle_oracle_warp: true,
    }
}

/// Focused #147 wall-grab "zoom to top" / "can't jump off a wall" audit.
///
/// A non-Kid form (Red Stealth, helmet 3) warps to Highwater Pass 1 (Level 0x02)
/// and walks right along the col 8-9 corridor into the full-height col-10 wall,
/// then pulses the jump button (B) against it. ASM `loc_AD7C`/`loc_AD98`/
/// `loc_ADAC` (kid.asm:13580-13600) dispatch on the `sub_902A` collision RESULT
/// d7 ∈ {-1,0,1,2}, NOT on `Current_Helmet`; the `collision_type=4 → loc_AA22`
/// re-stick at `loc_ADAC` needs d7>2 which `sub_902A` never returns, so floor
/// contact during wall-cling lands every form uniformly via `loc_A94E`. A
/// previous fix misread d7 as the helmet and re-armed wall-cling for helmet_id>2
/// forms on floor touch, holding them airborne with retained jump velocity. The
/// oracle here shows the correct trajectory: Red Stealth jumps off the floor and
/// lands in a clean parabola (MoveID 0 → jump → land) rather than being pinned to
/// the wall. This was the engine bug behind the re-confirmed #147 report.
pub fn issue_147_wall_side_jump_audit_scenario() -> DynamicScenario {
    const FRAME_COUNT: usize = 240;
    const RIGHT: u8 = 1 << 3;
    const JUMP: u8 = 1 << 4;
    const WALK_END: usize = 120;
    const JUMP_START: usize = 120;

    let mut script = vec![0u8; FRAME_COUNT.saturating_add(1)];
    // Walk right into the wall.
    apply_range(&mut script, 0, WALK_END.saturating_sub(1), RIGHT);
    // Pulse jumps against the wall: hold B for 2 frames, release for 1 (so the
    // jump button has a fresh rising edge each cycle) while still pushing right.
    let mut frame = JUMP_START;
    while frame + 2 < FRAME_COUNT {
        script[frame] = RIGHT | JUMP;
        script[frame + 1] = RIGHT | JUMP;
        script[frame + 2] = RIGHT;
        frame += 3;
    }

    DynamicScenario {
        slug: "issue_147_wall_side_jump_audit_02".into(),
        script,
        frame_count: FRAME_COUNT,
        checkpoints: vec![
            DynamicCheckpoint {
                label: "reach_wall".into(),
                frame_index: WALK_END.saturating_sub(1),
            },
            DynamicCheckpoint {
                label: "jump_into_wall".into(),
                frame_index: 180,
            },
            DynamicCheckpoint {
                label: "tail".into(),
                frame_index: FRAME_COUNT.saturating_sub(1),
            },
        ],
        start_position: None,
        camera_position: None,
        playtest_form: Some(3),
        initial_diamond_count: None,
        warp_to_level: Some(0x02),
        settle_oracle_warp: true,
    }
}

/// Focused #249 no-input wall-contact audit.
///
/// Starts from the same Highwater Pass 1 wall route as #147, but never presses
/// B/C. ASM `kid_flip -> loc_ABA4` gates The Kid's ledge pull-up on a fresh
/// `Ctrl_Pressed` B edge, so simply holding Right into the wall must not launch
/// Kid up the column.
fn issue_249_wall_no_jump_audit_scenario_for(
    slug: &str,
    playtest_form: Option<u16>,
) -> DynamicScenario {
    const FRAME_COUNT: usize = 240;
    const RIGHT: u8 = 1 << 3;
    const WALK_END: usize = 180;

    let mut script = vec![0u8; FRAME_COUNT.saturating_add(1)];
    apply_range(&mut script, 0, WALK_END.saturating_sub(1), RIGHT);

    DynamicScenario {
        slug: slug.into(),
        script,
        frame_count: FRAME_COUNT,
        checkpoints: vec![
            DynamicCheckpoint {
                label: "reach_wall_no_jump".into(),
                frame_index: 119,
            },
            DynamicCheckpoint {
                label: "hold_wall_no_jump".into(),
                frame_index: 180,
            },
            DynamicCheckpoint {
                label: "tail".into(),
                frame_index: FRAME_COUNT.saturating_sub(1),
            },
        ],
        start_position: None,
        camera_position: None,
        playtest_form,
        initial_diamond_count: None,
        warp_to_level: Some(0x02),
        settle_oracle_warp: true,
    }
}

pub fn issue_249_wall_no_jump_audit_scenario() -> DynamicScenario {
    issue_249_wall_no_jump_audit_scenario_for("issue_249_wall_no_jump_audit_02", None)
}

pub fn issue_249_juggernaut_wall_no_jump_audit_scenario() -> DynamicScenario {
    issue_249_wall_no_jump_audit_scenario_for("issue_249_juggernaut_wall_no_jump_audit_02", Some(5))
}

/// #249 variant for the user's Rhino/Juggernaut "jump on the wall to climb"
/// report. This mirrors the #147 jump pulse pattern but runs with helmet 5 so
/// we can compare the helmet-specific wall response against the oracle.
pub fn issue_249_juggernaut_wall_jump_audit_scenario() -> DynamicScenario {
    const FRAME_COUNT: usize = 240;
    const RIGHT: u8 = 1 << 3;
    const JUMP: u8 = 1 << 4;
    const WALK_END: usize = 120;
    const JUMP_START: usize = 120;

    let mut script = vec![0u8; FRAME_COUNT.saturating_add(1)];
    apply_range(&mut script, 0, WALK_END.saturating_sub(1), RIGHT);
    let mut frame = JUMP_START;
    while frame + 2 < FRAME_COUNT {
        script[frame] = RIGHT | JUMP;
        script[frame + 1] = RIGHT | JUMP;
        script[frame + 2] = RIGHT;
        frame += 3;
    }

    DynamicScenario {
        slug: "issue_249_juggernaut_wall_jump_audit_02".into(),
        script,
        frame_count: FRAME_COUNT,
        checkpoints: vec![
            DynamicCheckpoint {
                label: "reach_wall".into(),
                frame_index: WALK_END.saturating_sub(1),
            },
            DynamicCheckpoint {
                label: "jump_into_wall".into(),
                frame_index: 180,
            },
            DynamicCheckpoint {
                label: "tail".into(),
                frame_index: FRAME_COUNT.saturating_sub(1),
            },
        ],
        start_position: None,
        camera_position: None,
        playtest_form: Some(5),
        initial_diamond_count: None,
        warp_to_level: Some(0x02),
        settle_oracle_warp: true,
    }
}

fn issue_249_blw2_left_pit_wall_scenario_for(
    slug: &str,
    playtest_form: Option<u16>,
    pulse_jump: bool,
) -> DynamicScenario {
    const FRAME_COUNT: usize = 180;
    const LEFT: u8 = 1 << 2;
    const JUMP: u8 = 1 << 4;
    const WALL_HOLD_END: usize = 179;
    const JUMP_START: usize = 12;

    let mut script = vec![0u8; FRAME_COUNT.saturating_add(1)];
    apply_range(&mut script, 0, WALL_HOLD_END, LEFT);
    if pulse_jump {
        let mut frame = JUMP_START;
        while frame + 2 < FRAME_COUNT {
            script[frame] = LEFT | JUMP;
            script[frame + 1] = LEFT | JUMP;
            script[frame + 2] = LEFT;
            frame += 3;
        }
    }

    DynamicScenario {
        slug: slug.into(),
        script,
        frame_count: FRAME_COUNT,
        checkpoints: vec![
            DynamicCheckpoint {
                label: "left_pit_wall_contact_loc_a8de".into(),
                frame_index: 30,
            },
            DynamicCheckpoint {
                label: "left_pit_wall_hold_loc_aba4".into(),
                frame_index: 90,
            },
            DynamicCheckpoint {
                label: "tail".into(),
                frame_index: FRAME_COUNT.saturating_sub(1),
            },
        ],
        // Level 01's rubber-block drop pit has a long left wall at block col
        // 20, rows 10..35 (x=320, y=160..576). Seed near the top of that same
        // shaft: the lower y=560 probe lands on the rubber/bridge area and
        // immediately diverges before reaching the wall-cling logic.
        start_position: Some(DynamicStartPosition { x: 352, y: 192 }),
        camera_position: Some(DynamicStartPosition { x: 240, y: 80 }),
        playtest_form,
        initial_diamond_count: None,
        warp_to_level: Some(0x01),
        settle_oracle_warp: true,
    }
}

pub fn issue_249_blw2_left_pit_wall_no_jump_audit_scenario() -> DynamicScenario {
    issue_249_blw2_left_pit_wall_scenario_for(
        "issue_249_blw2_left_pit_wall_no_jump_audit_01",
        Some(5),
        false,
    )
}

pub fn issue_249_blw2_left_pit_wall_jump_audit_scenario() -> DynamicScenario {
    issue_249_blw2_left_pit_wall_scenario_for(
        "issue_249_blw2_left_pit_wall_jump_audit_01",
        Some(5),
        true,
    )
}

pub fn issue_249_blw2_left_pit_wall_kid_no_jump_audit_scenario() -> DynamicScenario {
    issue_249_blw2_left_pit_wall_scenario_for(
        "issue_249_blw2_left_pit_wall_kid_no_jump_audit_01",
        None,
        false,
    )
}

pub fn issue_249_blw2_left_pit_wall_red_stealth_no_jump_audit_scenario() -> DynamicScenario {
    issue_249_blw2_left_pit_wall_scenario_for(
        "issue_249_blw2_left_pit_wall_red_stealth_no_jump_audit_01",
        Some(3),
        false,
    )
}

/// Current #155/#213 Drips/metal-block probe.
///
/// This is intentionally not marked as the final human-visible repro: frontier
/// notes record that the direct Level 01 lower-floor start does not prove the
/// reported slime/metal-block interaction. It preserves the current
/// oracle-clean blocker window so future work has a durable named route while
/// a natural camera transition route is built.
pub fn issue_213_drips_metal_block_audit_scenario() -> DynamicScenario {
    const FRAME_COUNT: usize = 1_200;
    const B: u8 = 1 << 4;
    const JUMP_START: usize = 340;
    const JUMP_END: usize = 375;

    let mut script = vec![0u8; FRAME_COUNT.saturating_add(1)];
    apply_range(&mut script, JUMP_START, JUMP_END, B);

    DynamicScenario {
        slug: "issue_213_drips_metal_block_audit_01".into(),
        script,
        frame_count: FRAME_COUNT,
        checkpoints: vec![
            DynamicCheckpoint {
                label: "direct_probe_start".into(),
                frame_index: 0,
            },
            DynamicCheckpoint {
                label: "pre_jump".into(),
                frame_index: JUMP_START.saturating_sub(1),
            },
            DynamicCheckpoint {
                label: "jump_peak_window".into(),
                frame_index: 380,
            },
            DynamicCheckpoint {
                label: "contact_window".into(),
                frame_index: 460,
            },
            DynamicCheckpoint {
                label: "tail".into(),
                frame_index: FRAME_COUNT.saturating_sub(1),
            },
        ],
        start_position: Some(DynamicStartPosition { x: 568, y: 575 }),
        camera_position: Some(DynamicStartPosition { x: 240, y: 352 }),
        playtest_form: None,
        initial_diamond_count: None,
        warp_to_level: Some(0x01),
        settle_oracle_warp: true,
    }
}

/// Current #155/#213 Drips-drop healthy-prefix route window.
///
/// This route starts from the oracle-clean lower-floor probe, walks right,
/// delays the real B jump until Kid is close enough to activate the Drips,
/// and brakes before the Tank contact. The committed tail stops on the last
/// healthy frame before an ASM `Drips.asm loc_335D8`/`loc_33650` surface
/// crawler damages Kid. It is not the final slime/metal-block repro; it
/// preserves a narrower oracle-backed Drips projectile window without carrying
/// a death-windup tail.
pub fn issue_213_drips_delayed_b_drop_audit_scenario() -> DynamicScenario {
    const FRAME_COUNT: usize = 536;
    const LEFT: u8 = 1 << 2;
    const RIGHT: u8 = 1 << 3;
    const B: u8 = 1 << 4;

    let mut script = vec![0u8; FRAME_COUNT.saturating_add(1)];
    apply_range(&mut script, 0, 240, RIGHT);
    apply_range(&mut script, 60, 120, B);
    apply_range(&mut script, 190, 300, LEFT);
    apply_range(&mut script, 480, 530, LEFT);

    DynamicScenario {
        slug: "issue_213_drips_delayed_b_drop_audit_01".into(),
        script,
        frame_count: FRAME_COUNT,
        checkpoints: vec![
            DynamicCheckpoint {
                label: "lower_floor_start".into(),
                frame_index: 0,
            },
            DynamicCheckpoint {
                label: "delayed_jump_press".into(),
                frame_index: 60,
            },
            DynamicCheckpoint {
                label: "first_drips_drop_window".into(),
                frame_index: 108,
            },
            DynamicCheckpoint {
                label: "drop_tail_window".into(),
                frame_index: 473,
            },
            DynamicCheckpoint {
                label: "tank_evade_window".into(),
                frame_index: 497,
            },
            DynamicCheckpoint {
                label: "pre_crawler_contact_tail".into(),
                frame_index: FRAME_COUNT.saturating_sub(1),
            },
        ],
        start_position: Some(DynamicStartPosition { x: 344, y: 575 }),
        camera_position: Some(DynamicStartPosition { x: 192, y: 352 }),
        playtest_form: None,
        initial_diamond_count: None,
        warp_to_level: Some(0x01),
        settle_oracle_warp: true,
    }
}

/// Current #213/#260 Drips cleaner-prefix route window.
///
/// This preserves the first oracle-clean route variant that keeps Kid live after
/// two DripsSurfaceCrawler sightings without falling into the lower-pit
/// LevelTransition. It still does not reach the reported metal-block contact,
/// but it is a better extension seed than the older one-crawler healthy prefix.
pub fn issue_260_drips_cleaner_prefix_audit_scenario() -> DynamicScenario {
    const FRAME_COUNT: usize = 536;
    const LEFT: u8 = 1 << 2;
    const RIGHT: u8 = 1 << 3;
    const B: u8 = 1 << 4;

    let mut script = vec![0u8; FRAME_COUNT.saturating_add(1)];
    apply_range(&mut script, 0, 240, RIGHT);
    apply_range(&mut script, 60, 120, B);
    apply_range(&mut script, 170, 260, LEFT);

    DynamicScenario {
        slug: "issue_260_drips_cleaner_prefix_audit_01".into(),
        script,
        frame_count: FRAME_COUNT,
        checkpoints: vec![
            DynamicCheckpoint {
                label: "lower_floor_start".into(),
                frame_index: 0,
            },
            DynamicCheckpoint {
                label: "first_drips_drop_window".into(),
                frame_index: 108,
            },
            DynamicCheckpoint {
                label: "first_crawler_window".into(),
                frame_index: 376,
            },
            DynamicCheckpoint {
                label: "second_crawler_window".into(),
                frame_index: 518,
            },
            DynamicCheckpoint {
                label: "live_tail".into(),
                frame_index: FRAME_COUNT.saturating_sub(1),
            },
        ],
        start_position: Some(DynamicStartPosition { x: 344, y: 575 }),
        camera_position: Some(DynamicStartPosition { x: 192, y: 352 }),
        playtest_form: None,
        initial_diamond_count: None,
        warp_to_level: Some(0x01),
        settle_oracle_warp: true,
    }
}

/// #260 Drips route extension that survives the lower tank/gap approach.
///
/// This starts from the cleaner two-crawler prefix, then uses the corrected
/// physical B jump input and Right hold to carry Kid through the lower
/// RockTank/gap blocker into the rising right-hand tunnel. It is a climb seed
/// for the reported upper Drips/metal-block mutation, not a completed repro.
pub fn issue_260_drips_b_jump_climb_seed_scenario() -> DynamicScenario {
    const FRAME_COUNT: usize = 700;
    const LEFT: u8 = 1 << 2;
    const RIGHT: u8 = 1 << 3;
    const B: u8 = 1 << 4;

    let mut script = vec![0u8; FRAME_COUNT.saturating_add(1)];
    apply_range(&mut script, 0, 240, RIGHT);
    apply_range(&mut script, 60, 120, B);
    apply_range(&mut script, 170, 260, LEFT);
    apply_range(&mut script, 480, 700, RIGHT);
    apply_range(&mut script, 480, 560, B);

    DynamicScenario {
        slug: "issue_260_drips_b_jump_climb_seed_01".into(),
        script,
        frame_count: FRAME_COUNT,
        checkpoints: vec![
            DynamicCheckpoint {
                label: "lower_floor_start".into(),
                frame_index: 0,
            },
            DynamicCheckpoint {
                label: "first_crawler_window".into(),
                frame_index: 376,
            },
            DynamicCheckpoint {
                label: "second_crawler_window".into(),
                frame_index: 518,
            },
            DynamicCheckpoint {
                label: "b_jump_climb_start".into(),
                frame_index: 480,
            },
            DynamicCheckpoint {
                label: "rising_tunnel_entry".into(),
                frame_index: 640,
            },
            DynamicCheckpoint {
                label: "live_climb_tail".into(),
                frame_index: FRAME_COUNT,
            },
        ],
        start_position: Some(DynamicStartPosition { x: 344, y: 575 }),
        camera_position: Some(DynamicStartPosition { x: 192, y: 352 }),
        playtest_form: None,
        initial_diamond_count: None,
        warp_to_level: Some(0x01),
        settle_oracle_warp: true,
    }
}

/// #260 Drips route probe that isolates the closest early-jump approach.
///
/// This intentionally stops at the frame-408 closest approach to the upper
/// Drips target before the later death/LevelTransition tail. It is a narrower
/// oracle-backed seed for route search, not the human-visible metal-block repro.
pub fn issue_260_drips_early_jump_close_probe_scenario() -> DynamicScenario {
    const FRAME_COUNT: usize = 409;
    const LEFT: u8 = 1 << 2;
    const RIGHT: u8 = 1 << 3;
    const B: u8 = 1 << 4;

    let mut script = vec![0u8; FRAME_COUNT.saturating_add(1)];
    apply_range(&mut script, 0, 240, RIGHT);
    apply_range(&mut script, 60, 120, B);
    apply_range(&mut script, 170, 260, LEFT);
    apply_range(&mut script, 380, FRAME_COUNT, RIGHT | B);

    DynamicScenario {
        slug: "issue_260_drips_early_jump_close_probe_01".into(),
        script,
        frame_count: FRAME_COUNT,
        checkpoints: vec![
            DynamicCheckpoint {
                label: "lower_floor_start".into(),
                frame_index: 0,
            },
            DynamicCheckpoint {
                label: "first_crawler_window".into(),
                frame_index: 376,
            },
            DynamicCheckpoint {
                label: "early_b_jump_start".into(),
                frame_index: 380,
            },
            DynamicCheckpoint {
                label: "target_close_window".into(),
                frame_index: 408,
            },
            DynamicCheckpoint {
                label: "pre_death_tail".into(),
                frame_index: FRAME_COUNT,
            },
        ],
        start_position: Some(DynamicStartPosition { x: 344, y: 575 }),
        camera_position: Some(DynamicStartPosition { x: 192, y: 352 }),
        playtest_form: None,
        initial_diamond_count: None,
        warp_to_level: Some(0x01),
        settle_oracle_warp: true,
    }
}

/// Walk right with aggressive jump spam — higher frequency jumps than walk_right
/// to maximise enemy stomp and headbutt chances across arbitrary levels.
pub fn stomp_enemy_scenario(level_id: u8, walk_frames: usize) -> DynamicScenario {
    let settle_frames: usize = 60;
    let total_frames = settle_frames + walk_frames;
    let walk_start = settle_frames;
    let walk_end = total_frames.saturating_sub(1);

    let mut script = vec![0u8; total_frames.saturating_add(1)];
    // Hold right + run (C button) throughout the walk phase
    apply_range(&mut script, walk_start, walk_end, (1 << 3) | (1 << 5));
    // Aggressive jumps every 30 frames (~0.5s), 6-frame hold for higher arc
    add_periodic_jump(&mut script, walk_start + 15, walk_end, 30, 6);

    let checkpoints = vec![
        DynamicCheckpoint {
            label: "level_settle".into(),
            frame_index: settle_frames,
        },
        DynamicCheckpoint {
            label: "stomp_quarter".into(),
            frame_index: settle_frames + walk_frames / 4,
        },
        DynamicCheckpoint {
            label: "stomp_half".into(),
            frame_index: settle_frames + walk_frames / 2,
        },
        DynamicCheckpoint {
            label: "stomp_end".into(),
            frame_index: walk_end,
        },
    ];

    DynamicScenario {
        slug: format!("stomp_enemy_{level_id:02x}"),
        script,
        frame_count: total_frames,
        checkpoints,
        start_position: None,
        camera_position: None,
        playtest_form: None,
        initial_diamond_count: None,
        warp_to_level: Some(level_id),
        settle_oracle_warp: false,
    }
}

pub fn issue_271_level00_dragon_stomp_damage_audit_scenario() -> DynamicScenario {
    let mut scenario = stomp_enemy_scenario(0x00, 220);
    scenario.slug = "issue_271_level00_dragon_stomp_damage_audit_00".into();
    scenario.checkpoints = vec![
        DynamicCheckpoint {
            label: "level_settle".into(),
            frame_index: 60,
        },
        DynamicCheckpoint {
            label: "dragon_graze_damage_window_loc_1c76".into(),
            frame_index: 137,
        },
        DynamicCheckpoint {
            label: "post_graze_damage_window".into(),
            frame_index: 138,
        },
        DynamicCheckpoint {
            label: "route_tail".into(),
            frame_index: scenario.frame_count.saturating_sub(1),
        },
    ];
    scenario
}

pub fn issue_271_level01_rock_tank_stomp_route_audit_scenario() -> DynamicScenario {
    let mut scenario = stomp_enemy_scenario(0x01, 220);
    scenario.slug = "issue_271_level01_rock_tank_stomp_route_audit_01".into();
    scenario.checkpoints = vec![
        DynamicCheckpoint {
            label: "level_settle".into(),
            frame_index: 60,
        },
        DynamicCheckpoint {
            label: "tank_jump_start".into(),
            frame_index: 136,
        },
        DynamicCheckpoint {
            label: "closest_tank_pass_miss".into(),
            frame_index: 171,
        },
        DynamicCheckpoint {
            label: "route_tail".into(),
            frame_index: scenario.frame_count.saturating_sub(1),
        },
    ];
    scenario
}

impl Scenario {
    pub fn slug(self) -> &'static str {
        match self {
            Self::BootTitleIdle10s => "boot_title_idle_10s",
            Self::BootTitleStartLevel1 => "boot_title_start_level1",
            Self::BootTitleStartLevel1MoveRight5s => "boot_title_start_level1_move_right_5s",
            Self::Level1EnemyWindow => "level1_enemy_window",
            Self::WalkRight3s => "walk_right_3s",
            Self::JumpFromStandstill => "jump_from_standstill",
        }
    }

    pub fn frame_count(self) -> usize {
        match self {
            Self::BootTitleIdle10s => 600,
            Self::BootTitleStartLevel1 => 720,
            Self::BootTitleStartLevel1MoveRight5s => 1_020,
            Self::Level1EnemyWindow => 900,
            Self::WalkRight3s => 900,
            Self::JumpFromStandstill => 781,
        }
    }

    pub fn checkpoints(self) -> &'static [FrameCheckpoint] {
        match self {
            Self::BootTitleIdle10s => &BOOT_TITLE_IDLE_10S_CHECKPOINTS,
            Self::BootTitleStartLevel1 => &BOOT_TITLE_START_LEVEL_1_CHECKPOINTS,
            Self::BootTitleStartLevel1MoveRight5s => {
                &BOOT_TITLE_START_LEVEL_1_MOVE_RIGHT_5S_CHECKPOINTS
            }
            Self::Level1EnemyWindow => &LEVEL_1_ENEMY_WINDOW_CHECKPOINTS,
            Self::WalkRight3s => &WALK_RIGHT_3S_CHECKPOINTS,
            Self::JumpFromStandstill => &JUMP_FROM_STANDSTILL_CHECKPOINTS,
        }
    }

    pub fn script(self) -> Vec<u8> {
        let mut script = vec![0; self.frame_count().saturating_add(1)];
        match self {
            Self::BootTitleIdle10s => {}
            Self::BootTitleStartLevel1 => {
                apply_boot_to_level_1(&mut script);
            }
            Self::BootTitleStartLevel1MoveRight5s => {
                apply_boot_to_level_1(&mut script);
                apply_range(&mut script, 720, 1_019, 1 << 3);
            }
            Self::Level1EnemyWindow => {
                apply_boot_to_level_1(&mut script);
                apply_range(&mut script, 720, 899, 1 << 3);
            }
            Self::WalkRight3s => {
                apply_boot_to_level_1(&mut script);
                // Hold right only (no run) for 3 seconds from level settle
                apply_range(&mut script, 720, 899, 1 << 3);
            }
            Self::JumpFromStandstill => {
                apply_boot_to_level_1(&mut script);
                // Single jump press at level settle, no horizontal input
                apply_range(&mut script, 720, 723, 1 << 4);
            }
        }
        script
    }

    /// Convert a static scenario into the runtime DynamicScenario representation.
    #[cfg(test)]
    pub fn to_dynamic(self) -> DynamicScenario {
        DynamicScenario {
            slug: self.slug().to_string(),
            script: self.script(),
            frame_count: self.frame_count(),
            checkpoints: self
                .checkpoints()
                .iter()
                .map(|cp| DynamicCheckpoint {
                    label: cp.label.to_string(),
                    frame_index: cp.frame_index,
                })
                .collect(),
            start_position: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            warp_to_level: None,
            settle_oracle_warp: false,
        }
    }
}

impl GameplayScenario {
    pub fn slug(self) -> &'static str {
        match self {
            Self::PauseResumeLevel1 => "pause_resume_level1",
            Self::DeathRespawnLevel1 => "death_respawn_level1",
            Self::DeathRespawnStartResumeLevel1 => "death_respawn_start_resume_level1",
            Self::ProgressTwoTransitions => "progress_two_transitions",
            Self::SoakLevel10m => "soak_level_10m",
        }
    }

    pub fn frame_count(self) -> usize {
        match self {
            Self::PauseResumeLevel1 => 1_120,
            Self::DeathRespawnLevel1 => 22_001,
            Self::DeathRespawnStartResumeLevel1 => 1_951,
            Self::ProgressTwoTransitions => 3_301,
            Self::SoakLevel10m => 36_720,
        }
    }

    pub fn checkpoints(self) -> &'static [GameplayCheckpoint] {
        match self {
            Self::PauseResumeLevel1 => &PAUSE_RESUME_LEVEL_1_CHECKPOINTS,
            Self::DeathRespawnLevel1 => &DEATH_RESPAWN_LEVEL_1_CHECKPOINTS,
            Self::DeathRespawnStartResumeLevel1 => &DEATH_RESPAWN_START_RESUME_LEVEL_1_CHECKPOINTS,
            Self::ProgressTwoTransitions => &PROGRESS_TWO_TRANSITIONS_CHECKPOINTS,
            Self::SoakLevel10m => &SOAK_LEVEL_10M_CHECKPOINTS,
        }
    }

    pub fn uses_level_skip_cheat(self) -> bool {
        matches!(self, Self::ProgressTwoTransitions | Self::SoakLevel10m)
    }

    pub fn required_level_layout_range(self) -> (u8, u8) {
        match self {
            Self::PauseResumeLevel1
            | Self::DeathRespawnLevel1
            | Self::DeathRespawnStartResumeLevel1 => (0x00, 0x00),
            Self::ProgressTwoTransitions => (0x00, 0x02),
            Self::SoakLevel10m => (0x00, 0x1F),
        }
    }

    pub fn script(self) -> Vec<u8> {
        let mut script = vec![0; self.frame_count().saturating_add(1)];
        apply_boot_to_level_1(&mut script);
        match self {
            Self::PauseResumeLevel1 => {
                apply_range(&mut script, 600, 839, 1 << 3);
                apply_range(&mut script, 840, 840, 0x80);
                apply_range(&mut script, 841, 919, 1 << 3);
                apply_range(&mut script, 920, 920, 0x80);
                apply_range(&mut script, 960, 1_119, (1 << 3) | (1 << 6));
                add_periodic_jump(&mut script, 980, 1_119, 24, 4);
            }
            Self::DeathRespawnLevel1 => {
                apply_range(&mut script, 600, 899, 1 << 3);
                apply_range(&mut script, 900, 1_350, (1 << 3) | (1 << 6));
                add_periodic_jump(&mut script, 940, 1_350, 24, 4);
                apply_range(&mut script, 1_351, 1_559, 1 << 3);
                apply_range(&mut script, 1_560, 3_000, 1 << 3);
                add_periodic_jump(&mut script, 1_600, 3_000, 28, 3);
            }
            Self::DeathRespawnStartResumeLevel1 => {
                apply_range(&mut script, 600, 899, 1 << 3);
                apply_range(&mut script, 900, 1_350, (1 << 3) | (1 << 6));
                add_periodic_jump(&mut script, 940, 1_350, 24, 4);
                apply_range(&mut script, 1_351, 1_559, 1 << 3);
                apply_range(&mut script, 1_560, 1_950, 1 << 3);
                add_periodic_jump(&mut script, 1_600, 1_950, 28, 3);
                // ASM lose_life -> loc_6E2 re-enters the level-load shell.
                // Oracle probe `issue-248-death-start-resume-settled` shows
                // two Start-held frames in mode $0008 before live gameplay
                // resumes on the third.
                apply_range(&mut script, 1_920, 1_922, 0x80);
            }
            Self::ProgressTwoTransitions => {
                apply_range(&mut script, 600, 899, 1 << 3);
                apply_single_frames(
                    &mut script,
                    &[900, 2_100, 2_140, 2_180, 2_700],
                    (1 << 5) | (1 << 6) | 0x80,
                );
                apply_range(&mut script, 1_280, 1_499, 1 << 3);
                apply_range(&mut script, 1_920, 1_999, 1 << 3);
                apply_range(
                    &mut script,
                    2_480,
                    self.frame_count().saturating_sub(1),
                    1 << 3,
                );
            }
            Self::SoakLevel10m => {
                for frame in (900..self.frame_count().saturating_sub(600)).step_by(700) {
                    apply_single_frames(&mut script, &[frame], (1 << 5) | (1 << 6) | 0x80);
                }
            }
        }
        script
    }

    pub fn to_dynamic_window(
        self,
        frame_start: usize,
        frame_end: usize,
        checkpoint_interval: usize,
    ) -> Result<DynamicScenario> {
        anyhow::ensure!(
            frame_start <= frame_end,
            "gameplay window start {frame_start} exceeds end {frame_end}"
        );
        anyhow::ensure!(
            frame_end < self.frame_count(),
            "gameplay window end {frame_end} must be < scenario frame count {}",
            self.frame_count()
        );
        anyhow::ensure!(
            checkpoint_interval != 0,
            "gameplay window checkpoint interval must be non-zero"
        );

        let mut labels_by_frame = BTreeMap::new();
        for checkpoint in self
            .checkpoints()
            .iter()
            .filter(|checkpoint| (frame_start..=frame_end).contains(&checkpoint.frame_index))
        {
            labels_by_frame.insert(checkpoint.frame_index, checkpoint.label.to_string());
        }

        let mut frame = frame_start;
        while frame <= frame_end {
            labels_by_frame
                .entry(frame)
                .or_insert_with(|| format!("frame_{frame:05}"));
            if frame == frame_end {
                break;
            }
            frame = frame.saturating_add(checkpoint_interval);
            if frame > frame_end {
                labels_by_frame
                    .entry(frame_end)
                    .or_insert_with(|| format!("frame_{frame_end:05}"));
                break;
            }
        }

        let mut script = self.script();
        script.truncate(frame_end.saturating_add(2));

        Ok(DynamicScenario {
            slug: format!(
                "{}_window_{frame_start:05}_{frame_end:05}_step_{checkpoint_interval}",
                self.slug()
            ),
            frame_count: frame_end.saturating_add(1),
            script,
            checkpoints: labels_by_frame
                .into_iter()
                .map(|(frame_index, label)| DynamicCheckpoint { label, frame_index })
                .collect(),
            start_position: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            warp_to_level: None,
            settle_oracle_warp: false,
        })
    }

    /// Convert a static gameplay scenario into the runtime DynamicScenario representation.
    pub fn to_dynamic(self) -> DynamicScenario {
        DynamicScenario {
            slug: self.slug().to_string(),
            script: self.script(),
            frame_count: self.frame_count(),
            checkpoints: self
                .checkpoints()
                .iter()
                .map(|cp| DynamicCheckpoint {
                    label: cp.label.to_string(),
                    frame_index: cp.frame_index,
                })
                .collect(),
            start_position: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            warp_to_level: None,
            settle_oracle_warp: false,
        }
    }
}

fn apply_boot_to_level_1(script: &mut [u8]) {
    apply_single_frames(script, &kid_core::BOOT_TO_LEVEL_START_FRAMES, 0x80);
}

fn add_periodic_jump(script: &mut [u8], start: usize, end: usize, period: usize, width: usize) {
    let mut frame = start;
    while frame <= end {
        let jump_end = end.min(frame.saturating_add(width.saturating_sub(1)));
        apply_range(script, frame, jump_end, 1 << 4);
        frame = frame.saturating_add(period);
        if frame == usize::MAX {
            break;
        }
    }
}

fn apply_single_frames(script: &mut [u8], frames: &[usize], buttons: u8) {
    for frame in frames {
        if let Some(slot) = script.get_mut(*frame) {
            *slot |= buttons;
        }
    }
}

fn apply_range(script: &mut [u8], start: usize, end: usize, buttons: u8) {
    for slot in script.iter_mut().take(end.saturating_add(1)).skip(start) {
        *slot |= buttons;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DynamicCheckpoint, DynamicScenario, DynamicStartPosition, GameplayScenario, Scenario,
        diamond_power_circle_audit_scenario, diamond_power_invulnerability_audit_scenario,
        diamond_power_samurai_haze_audit_scenario, diamond_power_snake_audit_scenario,
        diamond_power_wall_audit_scenario, enemy_spawn_audit_scenario,
        idle_level_full_frame_scenario, idle_level_scenario,
        issue_147_wall_side_jump_audit_scenario, issue_213_drips_delayed_b_drop_audit_scenario,
        issue_213_drips_metal_block_audit_scenario, issue_216_prize_audio_audit_scenario,
        issue_249_blw2_left_pit_wall_kid_no_jump_audit_scenario,
        issue_249_blw2_left_pit_wall_red_stealth_no_jump_audit_scenario,
        issue_249_juggernaut_wall_jump_audit_scenario,
        issue_249_juggernaut_wall_no_jump_audit_scenario, issue_249_wall_no_jump_audit_scenario,
        issue_260_drips_b_jump_climb_seed_scenario, issue_260_drips_cleaner_prefix_audit_scenario,
        issue_260_drips_early_jump_close_probe_scenario,
        issue_271_level00_dragon_stomp_damage_audit_scenario,
        issue_271_level01_rock_tank_stomp_route_audit_scenario, lava_weather_audit_scenario,
        level_00_slope_right_probe_scenario, prize_block_scenario, stomp_enemy_scenario,
        walk_right_iron_knight_scenario, walk_right_juggernaut_scenario,
        walk_right_red_stealth_scenario, walk_right_scenario,
    };

    #[test]
    fn issue_147_wall_side_jump_audit_drives_red_stealth_into_level2_wall() {
        let scenario = issue_147_wall_side_jump_audit_scenario();
        assert_eq!(scenario.slug, "issue_147_wall_side_jump_audit_02");
        // Red Stealth (helmet 3) on Highwater Pass 1.
        assert_eq!(scenario.playtest_form, Some(3));
        assert_eq!(scenario.warp_to_level, Some(0x02));
        assert!(scenario.settle_oracle_warp);
        // Walks right (bit 3) into the wall, then pulses jump (bit 4) with a
        // fresh rising edge every third frame so the standing-jump can re-fire.
        const RIGHT: u8 = 1 << 3;
        const JUMP: u8 = 1 << 4;
        assert_eq!(scenario.script[60], RIGHT, "still walking before the wall");
        assert!(
            scenario.script[120] & JUMP != 0 && scenario.script[120] & RIGHT != 0,
            "jump pulse pushes right into the wall",
        );
        assert_eq!(
            scenario.script[122] & JUMP,
            0,
            "jump released for one frame so the next press is a rising edge",
        );
    }

    #[test]
    fn issue_213_drips_metal_block_audit_preserves_current_direct_probe() {
        let scenario = issue_213_drips_metal_block_audit_scenario();

        assert_eq!(scenario.slug, "issue_213_drips_metal_block_audit_01");
        assert_eq!(scenario.warp_to_level, Some(0x01));
        assert!(scenario.settle_oracle_warp);
        assert_eq!(
            scenario.start_position,
            Some(DynamicStartPosition { x: 568, y: 575 })
        );
        assert_eq!(
            scenario.camera_position,
            Some(DynamicStartPosition { x: 240, y: 352 })
        );
        assert_eq!(scenario.frame_count, 1_200);
        assert_eq!(scenario.script.len(), scenario.frame_count + 1);
        assert!(scenario.script[..340].iter().all(|buttons| *buttons == 0));
        assert!(
            scenario.script[340..=375]
                .iter()
                .all(|buttons| *buttons == 1 << 4),
            "the route holds B over the verified jump window"
        );
        assert!(scenario.script[376..].iter().all(|buttons| *buttons == 0));
        let labels: Vec<_> = scenario
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.label.as_str())
            .collect();
        assert_eq!(
            labels,
            vec![
                "direct_probe_start",
                "pre_jump",
                "jump_peak_window",
                "contact_window",
                "tail"
            ]
        );
    }

    #[test]
    fn issue_213_drips_delayed_b_drop_audit_preserves_oracle_backed_drop_window() {
        let scenario = issue_213_drips_delayed_b_drop_audit_scenario();

        assert_eq!(scenario.slug, "issue_213_drips_delayed_b_drop_audit_01");
        assert_eq!(scenario.warp_to_level, Some(0x01));
        assert!(scenario.settle_oracle_warp);
        assert_eq!(
            scenario.start_position,
            Some(DynamicStartPosition { x: 344, y: 575 })
        );
        assert_eq!(
            scenario.camera_position,
            Some(DynamicStartPosition { x: 192, y: 352 })
        );
        assert_eq!(scenario.frame_count, 536);
        assert_eq!(scenario.script.len(), scenario.frame_count + 1);

        assert_eq!(scenario.script[0], 1 << 3);
        assert_eq!(scenario.script[60], (1 << 3) | (1 << 4));
        assert_eq!(scenario.script[108], (1 << 3) | (1 << 4));
        assert_eq!(scenario.script[190], (1 << 2) | (1 << 3));
        assert_eq!(scenario.script[241], 1 << 2);
        assert_eq!(scenario.script[301], 0);
        assert_eq!(scenario.script[480], 1 << 2);
        assert_eq!(scenario.script[497], 1 << 2);
        assert_eq!(
            scenario.script[535], 0,
            "ASM Drips.asm loc_335D8/loc_33650: frame 535 is the last recorded healthy prefix frame before crawler damage at 537"
        );

        let labels: Vec<_> = scenario
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.label.as_str())
            .collect();
        assert_eq!(
            labels,
            vec![
                "lower_floor_start",
                "delayed_jump_press",
                "first_drips_drop_window",
                "drop_tail_window",
                "tank_evade_window",
                "pre_crawler_contact_tail"
            ]
        );
    }

    #[test]
    fn issue_260_drips_cleaner_prefix_audit_preserves_two_crawler_windows() {
        let scenario = issue_260_drips_cleaner_prefix_audit_scenario();

        assert_eq!(scenario.slug, "issue_260_drips_cleaner_prefix_audit_01");
        assert_eq!(scenario.warp_to_level, Some(0x01));
        assert!(scenario.settle_oracle_warp);
        assert_eq!(
            scenario.start_position,
            Some(DynamicStartPosition { x: 344, y: 575 })
        );
        assert_eq!(
            scenario.camera_position,
            Some(DynamicStartPosition { x: 192, y: 352 })
        );
        assert_eq!(scenario.frame_count, 536);
        assert_eq!(scenario.script.len(), scenario.frame_count + 1);

        assert_eq!(scenario.script[0], 1 << 3);
        assert_eq!(scenario.script[60], (1 << 3) | (1 << 4));
        assert_eq!(scenario.script[108], (1 << 3) | (1 << 4));
        assert_eq!(scenario.script[170], (1 << 2) | (1 << 3));
        assert_eq!(scenario.script[190], (1 << 2) | (1 << 3));
        assert_eq!(scenario.script[241], 1 << 2);
        assert_eq!(scenario.script[261], 0);
        assert_eq!(scenario.script[535], 0);

        let labels: Vec<_> = scenario
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.label.as_str())
            .collect();
        assert_eq!(
            labels,
            vec![
                "lower_floor_start",
                "first_drips_drop_window",
                "first_crawler_window",
                "second_crawler_window",
                "live_tail"
            ]
        );
    }

    #[test]
    fn issue_260_drips_b_jump_climb_seed_extends_prefix_with_live_right_tunnel_route() {
        let scenario = issue_260_drips_b_jump_climb_seed_scenario();

        assert_eq!(scenario.slug, "issue_260_drips_b_jump_climb_seed_01");
        assert_eq!(scenario.warp_to_level, Some(0x01));
        assert!(scenario.settle_oracle_warp);
        assert_eq!(
            scenario.start_position,
            Some(DynamicStartPosition { x: 344, y: 575 })
        );
        assert_eq!(
            scenario.camera_position,
            Some(DynamicStartPosition { x: 192, y: 352 })
        );
        assert_eq!(scenario.frame_count, 700);
        assert_eq!(scenario.script.len(), scenario.frame_count + 1);

        const LEFT: u8 = 1 << 2;
        const RIGHT: u8 = 1 << 3;
        const B: u8 = 1 << 4;
        assert_eq!(scenario.script[0], RIGHT);
        assert_eq!(scenario.script[60], RIGHT | B);
        assert_eq!(scenario.script[190], LEFT | RIGHT);
        assert_eq!(scenario.script[261], 0);
        assert_eq!(scenario.script[480], RIGHT | B);
        assert_eq!(scenario.script[560], RIGHT | B);
        assert_eq!(scenario.script[561], RIGHT);
        assert_eq!(scenario.script[700], RIGHT);

        let labels: Vec<_> = scenario
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.label.as_str())
            .collect();
        assert_eq!(
            labels,
            vec![
                "lower_floor_start",
                "first_crawler_window",
                "second_crawler_window",
                "b_jump_climb_start",
                "rising_tunnel_entry",
                "live_climb_tail"
            ]
        );
    }

    #[test]
    fn issue_260_drips_early_jump_close_probe_stops_before_death_tail() {
        let scenario = issue_260_drips_early_jump_close_probe_scenario();

        assert_eq!(scenario.slug, "issue_260_drips_early_jump_close_probe_01");
        assert_eq!(scenario.warp_to_level, Some(0x01));
        assert!(scenario.settle_oracle_warp);
        assert_eq!(
            scenario.start_position,
            Some(DynamicStartPosition { x: 344, y: 575 })
        );
        assert_eq!(
            scenario.camera_position,
            Some(DynamicStartPosition { x: 192, y: 352 })
        );
        assert_eq!(scenario.frame_count, 409);
        assert_eq!(scenario.script.len(), scenario.frame_count + 1);

        const LEFT: u8 = 1 << 2;
        const RIGHT: u8 = 1 << 3;
        const B: u8 = 1 << 4;
        assert_eq!(scenario.script[0], RIGHT);
        assert_eq!(scenario.script[60], RIGHT | B);
        assert_eq!(scenario.script[190], LEFT | RIGHT);
        assert_eq!(scenario.script[261], 0);
        assert_eq!(scenario.script[379], 0);
        assert_eq!(scenario.script[380], RIGHT | B);
        assert_eq!(scenario.script[408], RIGHT | B);
        assert_eq!(scenario.script[409], RIGHT | B);

        let labels: Vec<_> = scenario
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.label.as_str())
            .collect();
        assert_eq!(
            labels,
            vec![
                "lower_floor_start",
                "first_crawler_window",
                "early_b_jump_start",
                "target_close_window",
                "pre_death_tail"
            ]
        );
    }

    #[test]
    fn issue_249_wall_no_jump_audit_drives_the_kid_into_level2_wall_without_jump() {
        let scenario = issue_249_wall_no_jump_audit_scenario();
        assert_eq!(scenario.slug, "issue_249_wall_no_jump_audit_02");
        assert_eq!(scenario.playtest_form, None);
        assert_eq!(scenario.warp_to_level, Some(0x02));
        assert!(scenario.settle_oracle_warp);

        const RIGHT: u8 = 1 << 3;
        const JUMP: u8 = 1 << 4;
        const SPECIAL: u8 = 1 << 5;
        assert_eq!(scenario.script[60], RIGHT, "walks into the wall");
        assert_eq!(
            scenario.script[180] & (JUMP | SPECIAL),
            0,
            "ASM kid_flip -> loc_ABA4 needs a fresh B edge; this audit must not press jump or special"
        );
        assert_eq!(
            scenario.script[220] & RIGHT,
            0,
            "the tail samples post-contact settling after the held-right window"
        );
    }

    #[test]
    fn issue_249_juggernaut_wall_no_jump_audit_uses_rhino_form_without_jump() {
        let scenario = issue_249_juggernaut_wall_no_jump_audit_scenario();
        assert_eq!(scenario.slug, "issue_249_juggernaut_wall_no_jump_audit_02");
        assert_eq!(scenario.playtest_form, Some(5));
        assert_eq!(scenario.warp_to_level, Some(0x02));

        const JUMP: u8 = 1 << 4;
        const SPECIAL: u8 = 1 << 5;
        assert!(
            scenario
                .script
                .iter()
                .all(|buttons| buttons & (JUMP | SPECIAL) == 0),
            "the Juggernaut/Rhino variant must isolate wall contact from jump/special input"
        );
    }

    #[test]
    fn issue_249_blw2_left_pit_wall_kid_no_jump_audit_uses_reported_left_wall() {
        let scenario = issue_249_blw2_left_pit_wall_kid_no_jump_audit_scenario();
        assert_eq!(
            scenario.slug,
            "issue_249_blw2_left_pit_wall_kid_no_jump_audit_01"
        );
        assert_eq!(scenario.playtest_form, None);
        assert_eq!(scenario.warp_to_level, Some(0x01));
        assert_eq!(
            scenario.start_position,
            Some(DynamicStartPosition { x: 352, y: 192 })
        );
        assert_eq!(
            scenario.camera_position,
            Some(DynamicStartPosition { x: 240, y: 80 })
        );

        const LEFT: u8 = 1 << 2;
        const JUMP: u8 = 1 << 4;
        assert_eq!(scenario.script[60], LEFT);
        assert!(
            scenario.script.iter().all(|buttons| buttons & JUMP == 0),
            "Kid left-wall audit must isolate the latest no-jump launch report"
        );
    }

    #[test]
    fn issue_249_blw2_left_pit_wall_red_stealth_no_jump_audit_uses_samurai_form() {
        let scenario = issue_249_blw2_left_pit_wall_red_stealth_no_jump_audit_scenario();
        assert_eq!(
            scenario.slug,
            "issue_249_blw2_left_pit_wall_red_stealth_no_jump_audit_01"
        );
        assert_eq!(
            scenario.playtest_form,
            Some(3),
            "Red Stealth is the Samurai form named in the latest wall-launch report"
        );
        assert_eq!(scenario.warp_to_level, Some(0x01));

        const LEFT: u8 = 1 << 2;
        const JUMP: u8 = 1 << 4;
        assert_eq!(scenario.script[90], LEFT);
        assert!(
            scenario.script.iter().all(|buttons| buttons & JUMP == 0),
            "Samurai left-wall audit must not contain a jump edge"
        );
    }

    #[test]
    fn issue_249_juggernaut_wall_jump_audit_uses_rhino_form_with_jump_edges() {
        let scenario = issue_249_juggernaut_wall_jump_audit_scenario();
        assert_eq!(scenario.slug, "issue_249_juggernaut_wall_jump_audit_02");
        assert_eq!(scenario.playtest_form, Some(5));
        assert_eq!(scenario.warp_to_level, Some(0x02));

        const RIGHT: u8 = 1 << 3;
        const JUMP: u8 = 1 << 4;
        assert_eq!(scenario.script[119], RIGHT);
        assert_eq!(scenario.script[120] & (RIGHT | JUMP), RIGHT | JUMP);
        assert_eq!(
            scenario.script[122] & JUMP,
            0,
            "jump is pulsed to create repeated B edges on the wall"
        );
    }

    #[test]
    fn dynamic_scenario_deserialises_probe_start_position() {
        let scenario: DynamicScenario = serde_json::from_str(
            r#"{
                "slug": "issue-175-eol-flag-touch",
                "script": [0, 0],
                "frame_count": 2,
                "checkpoints": [{"label": "flag_touch", "frame_index": 0}],
                "warp_to_level": 0,
                "start_position": {"x": 3152, "y": 176},
                "camera_position": {"x": 3008, "y": 64},
                "playtest_form": 7,
                "initial_diamond_count": 20,
                "settle_oracle_warp": true
            }"#,
        )
        .expect("scenario");

        assert_eq!(
            scenario.start_position,
            Some(DynamicStartPosition { x: 3152, y: 176 })
        );
        assert_eq!(
            scenario.camera_position,
            Some(DynamicStartPosition { x: 3008, y: 64 })
        );
        assert_eq!(scenario.playtest_form, Some(7));
        assert_eq!(scenario.initial_diamond_count, Some(20));
        assert!(scenario.settle_oracle_warp);
    }

    #[test]
    fn dynamic_scenario_omits_absent_probe_start_position() {
        let scenario = DynamicScenario {
            slug: String::from("idle"),
            script: vec![0],
            frame_count: 1,
            checkpoints: vec![DynamicCheckpoint {
                label: String::from("start"),
                frame_index: 0,
            }],
            start_position: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            warp_to_level: Some(0),
            settle_oracle_warp: false,
        };

        let json = serde_json::to_string(&scenario).expect("json");

        assert!(
            !json.contains("start_position"),
            "legacy scenarios should keep their compact JSON shape"
        );
        assert!(
            !json.contains("camera_position"),
            "legacy scenarios should keep their compact JSON shape"
        );
        assert!(
            !json.contains("playtest_form"),
            "legacy scenarios should keep their compact JSON shape"
        );
        assert!(
            !json.contains("initial_diamond_count"),
            "legacy scenarios should keep their compact JSON shape"
        );
        assert!(
            !json.contains("settle_oracle_warp"),
            "legacy scenarios should keep their compact JSON shape"
        );
    }

    #[test]
    fn diamond_power_circle_audit_seeds_start_a_and_twenty_diamonds() {
        let scenario = diamond_power_circle_audit_scenario(0x00);

        assert_eq!(scenario.slug, "diamond_power_circle_audit_00");
        assert_eq!(scenario.playtest_form, Some(0));
        assert_eq!(scenario.initial_diamond_count, Some(20));
        assert_eq!(scenario.script[0], (1 << 7) | (1 << 6));
        assert!(scenario.settle_oracle_warp);
        assert_eq!(scenario.checkpoints[2].label, "diamond_power_slot_window");
        assert!(
            scenario
                .checkpoints
                .iter()
                .any(
                    |checkpoint| checkpoint.label == "diamond_power_tail_probe_040"
                        && checkpoint.frame_index == 40
                )
        );
        assert_eq!(
            scenario.checkpoints.last().unwrap().label,
            "diamond_power_tail"
        );
        assert_eq!(scenario.checkpoints.last().unwrap().frame_index, 119);
    }

    #[test]
    fn diamond_power_wall_audit_seeds_berzerker_tier2_start_a_and_fifty_diamonds() {
        let scenario = diamond_power_wall_audit_scenario(0x00);

        assert_eq!(scenario.slug, "diamond_power_wall_audit_00");
        assert_eq!(scenario.playtest_form, Some(7));
        assert_eq!(scenario.initial_diamond_count, Some(50));
        assert_eq!(scenario.script[0], (1 << 7) | (1 << 6));
        assert!(scenario.settle_oracle_warp);
        assert_eq!(scenario.checkpoints[2].label, "diamond_power_slot_window");
        assert!(
            scenario
                .checkpoints
                .iter()
                .any(
                    |checkpoint| checkpoint.label == "diamond_power_tail_probe_040"
                        && checkpoint.frame_index == 40
                )
        );
        assert!(
            scenario
                .checkpoints
                .iter()
                .any(
                    |checkpoint| checkpoint.label == "diamond_power_transition_probe_057"
                        && checkpoint.frame_index == 57
                )
        );
        assert_eq!(
            scenario.checkpoints.last().unwrap().label,
            "diamond_power_tail"
        );
        assert_eq!(scenario.checkpoints.last().unwrap().frame_index, 119);
    }

    #[test]
    fn diamond_power_snake_audit_seeds_micromax_tier1_start_a_and_twenty_diamonds() {
        let scenario = diamond_power_snake_audit_scenario(0x00);

        assert_eq!(scenario.slug, "diamond_power_snake_audit_00");
        assert_eq!(scenario.playtest_form, Some(9));
        assert_eq!(scenario.initial_diamond_count, Some(20));
        assert_eq!(scenario.script[0], (1 << 7) | (1 << 6));
        assert!(scenario.settle_oracle_warp);
        assert_eq!(scenario.checkpoints[2].label, "diamond_power_slot_window");
    }

    #[test]
    fn diamond_power_invulnerability_audit_seeds_skycutter_tier1_start_a_and_twenty_diamonds() {
        let scenario = diamond_power_invulnerability_audit_scenario(0x02);

        assert_eq!(scenario.slug, "diamond_power_invulnerability_audit_02");
        assert_eq!(scenario.playtest_form, Some(1));
        assert_eq!(scenario.initial_diamond_count, Some(20));
        assert_eq!(scenario.script[0], (1 << 7) | (1 << 6));
        assert!(scenario.settle_oracle_warp);
        assert_eq!(scenario.checkpoints[2].label, "diamond_power_slot_window");
    }

    #[test]
    fn diamond_power_samurai_haze_audit_seeds_red_stealth_tier1_start_a_and_twenty_diamonds() {
        let scenario = diamond_power_samurai_haze_audit_scenario(0x02);

        assert_eq!(scenario.slug, "diamond_power_samurai_haze_audit_02");
        assert_eq!(scenario.playtest_form, Some(3));
        assert_eq!(scenario.initial_diamond_count, Some(20));
        assert_eq!(scenario.script[0], (1 << 7) | (1 << 6));
        assert!(scenario.settle_oracle_warp);
        assert_eq!(scenario.checkpoints[2].label, "diamond_power_slot_window");
    }

    #[test]
    fn move_right_scenario_holds_right_for_five_seconds() {
        let script = Scenario::BootTitleStartLevel1MoveRight5s.script();
        assert_eq!(script[719], 0);
        assert_eq!(script[720], 1 << 3);
        assert_eq!(script[1_019], 1 << 3);
        assert_eq!(script[1_020], 0);
    }

    #[test]
    fn start_level_scenario_presses_start_during_boot_path() {
        let script = Scenario::BootTitleStartLevel1.script();
        assert_eq!(script[33], 0x80);
        assert_eq!(script[178], 0x80);
        assert_eq!(script[360], 0x80);
        assert_eq!(script[460], 0x80);
    }

    #[test]
    fn death_respawn_level_1_script_reuses_walk_only_opening_baseline() {
        let script = GameplayScenario::DeathRespawnLevel1.script();
        assert_eq!(script[460], 0x80);
        assert_eq!(script[462], 0);
        assert_eq!(script[600], 1 << 3);
    }

    #[test]
    fn gameplay_regression_pause_resume_scenario_pauses_and_resumes() {
        let script = GameplayScenario::PauseResumeLevel1.script();
        assert_eq!(script[840], 0x80);
        assert_eq!(script[920], 0x80);
        assert_eq!(script[980] & (1 << 4), 1 << 4);
    }

    #[test]
    fn gameplay_progress_scenario_holds_right_and_run() {
        let script = GameplayScenario::ProgressTwoTransitions.script();
        assert_eq!(script[720] & (1 << 3), 1 << 3);
        assert_eq!(script[900], (1 << 5) | (1 << 6) | 0x80);
        assert_eq!(script[2_700], (1 << 3) | (1 << 5) | (1 << 6) | 0x80);
    }

    #[test]
    fn gameplay_soak_scenario_idles_between_level_skip_inputs() {
        let script = GameplayScenario::SoakLevel10m.script();

        assert_eq!(script[720], 0);
        assert_eq!(script[900], (1 << 5) | (1 << 6) | 0x80);
        assert_eq!(script[1_600], (1 << 5) | (1 << 6) | 0x80);
        assert_eq!(script[1_601], 0);
    }

    #[test]
    fn walk_right_scenario_holds_right_for_three_seconds() {
        let script = Scenario::WalkRight3s.script();
        assert_eq!(script[719], 0);
        assert_eq!(script[720], 1 << 3);
        assert_eq!(script[899], 1 << 3);
        assert_eq!(script[900], 0);
    }

    #[test]
    fn level_1_enemy_window_scenario_tracks_the_first_enemy_slice() {
        let script = Scenario::Level1EnemyWindow.script();
        assert_eq!(script[719], 0);
        assert_eq!(script[720], 1 << 3);
        assert_eq!(script[899], 1 << 3);
        assert_eq!(script[900], 0);
        assert_eq!(
            Scenario::Level1EnemyWindow
                .checkpoints()
                .iter()
                .map(|checkpoint| checkpoint.label)
                .collect::<Vec<_>>(),
            vec![
                "level_settle",
                "enemy_window_open",
                "enemy_first_hit",
                "enemy_window_tail",
            ]
        );
    }

    #[test]
    fn jump_from_standstill_scenario_presses_jump_briefly() {
        let script = Scenario::JumpFromStandstill.script();
        assert_eq!(script[719], 0);
        assert_eq!(script[720], 1 << 4);
        assert_eq!(script[723], 1 << 4);
        assert_eq!(script[724], 0);
    }

    #[test]
    fn scenario_frame_count_covers_all_declared_checkpoints() {
        for scenario in [
            Scenario::BootTitleIdle10s,
            Scenario::BootTitleStartLevel1,
            Scenario::BootTitleStartLevel1MoveRight5s,
            Scenario::Level1EnemyWindow,
            Scenario::WalkRight3s,
            Scenario::JumpFromStandstill,
        ] {
            let max_checkpoint = scenario
                .checkpoints()
                .iter()
                .map(|checkpoint| checkpoint.frame_index)
                .max()
                .expect("scenario checkpoints");
            assert!(
                scenario.frame_count() > max_checkpoint,
                "{} frame_count {} should exceed max checkpoint {}",
                scenario.slug(),
                scenario.frame_count(),
                max_checkpoint
            );
        }
    }

    #[test]
    fn gameplay_death_scenario_keeps_the_death_window_free_of_start_presses() {
        let script = GameplayScenario::DeathRespawnLevel1.script();
        assert_eq!(script[1_400] & 0x80, 0);
        assert_eq!(script[1_460] & 0x80, 0);
        assert_eq!(script[1_520] & 0x80, 0);
        assert_eq!(script[1_560] & (1 << 3), 1 << 3);
        assert_eq!(script[3_000] & (1 << 3), 1 << 3);
        assert_eq!(script[3_001] & (1 << 3), 0);
        assert_eq!(script[4_000] & (1 << 3), 0);
        assert_eq!(script[4_500] & (1 << 3), 0);
        assert_eq!(script[5_000] & (1 << 3), 0);
        assert_eq!(script[5_500] & (1 << 3), 0);
        assert_eq!(script[6_000] & (1 << 3), 0);
        assert_eq!(script[6_500] & (1 << 3), 0);
        assert_eq!(script[7_000] & (1 << 3), 0);
        assert_eq!(script[7_500] & (1 << 3), 0);
        assert_eq!(script[8_000] & (1 << 3), 0);
        assert_eq!(script[8_500] & (1 << 3), 0);
        assert_eq!(script[9_000] & (1 << 3), 0);
        assert_eq!(script[9_500] & (1 << 3), 0);
        assert_eq!(script[10_500] & (1 << 3), 0);
        assert_eq!(script[11_000] & (1 << 3), 0);
        assert_eq!(script[11_500] & (1 << 3), 0);
        assert_eq!(script[12_200] & (1 << 3), 0);
    }

    #[test]
    fn gameplay_death_scenario_extends_past_the_hidden_respawn_bootstrap() {
        assert_eq!(GameplayScenario::DeathRespawnLevel1.frame_count(), 22_001);
        assert_eq!(
            GameplayScenario::DeathRespawnLevel1
                .checkpoints()
                .iter()
                .map(|checkpoint| (checkpoint.label, checkpoint.frame_index))
                .collect::<Vec<_>>(),
            vec![
                ("level_settle", 719),
                ("pre_death", 1_680),
                ("death_tail", 1_762),
                ("respawn_tail", 1_919),
                ("respawn_bootstrap_tail", 9_000),
                ("respawn_bootstrap_tail_9500", 9_500),
                ("respawn_bootstrap_tail_10500", 10_500),
                ("respawn_bootstrap_tail_12200", 12_200),
                ("respawn_bootstrap_tail_16000", 16_000),
                ("respawn_bootstrap_tail_22000", 22_000),
            ]
        );
    }

    #[test]
    fn gameplay_death_start_resume_scenario_holds_start_during_respawn_shell() {
        let script = GameplayScenario::DeathRespawnStartResumeLevel1.script();
        assert_eq!(
            GameplayScenario::DeathRespawnStartResumeLevel1.frame_count(),
            1_951
        );
        assert_eq!(script[1_919] & 0x80, 0);
        assert_eq!(script[1_920] & 0x80, 0x80);
        assert_eq!(script[1_921] & 0x80, 0x80);
        assert_eq!(script[1_922] & 0x80, 0x80);
        assert_eq!(script[1_923] & 0x80, 0);
        assert_eq!(
            GameplayScenario::DeathRespawnStartResumeLevel1
                .checkpoints()
                .iter()
                .map(|checkpoint| (checkpoint.label, checkpoint.frame_index))
                .collect::<Vec<_>>(),
            vec![
                ("level_settle", 719),
                ("pre_death", 1_254),
                ("respawn_hidden", 1_919),
                ("start_release", 1_922),
                ("resumed_tail", 1_950),
            ]
        );
    }

    #[test]
    fn gameplay_scenario_window_preserves_static_checkpoints_and_adds_frame_labels() {
        let dynamic = GameplayScenario::DeathRespawnLevel1
            .to_dynamic_window(1_216, 1_220, 2)
            .expect("dynamic gameplay window");
        assert_eq!(
            dynamic.slug,
            "death_respawn_level1_window_01216_01220_step_2"
        );
        assert_eq!(dynamic.frame_count, 1_221);
        assert_eq!(dynamic.script.len(), 1_222);
        assert_eq!(
            dynamic
                .checkpoints
                .iter()
                .map(|checkpoint| (checkpoint.label.as_str(), checkpoint.frame_index))
                .collect::<Vec<_>>(),
            vec![
                ("frame_01216", 1_216),
                ("frame_01218", 1_218),
                ("frame_01220", 1_220),
            ]
        );
    }

    #[test]
    fn gameplay_scenario_window_keeps_named_checkpoint_labels_when_they_land_in_range() {
        // Window [1679, 1681] spans the pre_death checkpoint at frame 1680.
        let dynamic = GameplayScenario::DeathRespawnLevel1
            .to_dynamic_window(1_679, 1_681, 1)
            .expect("dynamic gameplay window");
        assert_eq!(
            dynamic
                .checkpoints
                .iter()
                .map(|checkpoint| (checkpoint.label.as_str(), checkpoint.frame_index))
                .collect::<Vec<_>>(),
            vec![
                ("frame_01679", 1_679),
                ("pre_death", 1_680),
                ("frame_01681", 1_681),
            ]
        );
    }

    #[test]
    fn gameplay_scenarios_report_required_level_layout_ranges() {
        assert_eq!(
            GameplayScenario::PauseResumeLevel1.required_level_layout_range(),
            (0x00, 0x00)
        );
        assert_eq!(
            GameplayScenario::ProgressTwoTransitions.required_level_layout_range(),
            (0x00, 0x02)
        );
        assert_eq!(
            GameplayScenario::SoakLevel10m.required_level_layout_range(),
            (0x00, 0x1F)
        );
    }

    #[test]
    fn idle_level_0_scenario_uses_warp_to_level() {
        let scenario = idle_level_scenario(0x00, None);
        assert_eq!(scenario.slug, "idle_level_00");
        assert_eq!(scenario.warp_to_level, Some(0x00));
        // settle=60, idle=300, total=360
        assert_eq!(scenario.frame_count, 360);
        assert_eq!(scenario.checkpoints.len(), 2);
        assert_eq!(scenario.checkpoints[0].label, "level_settle");
        assert_eq!(scenario.checkpoints[0].frame_index, 60);
        assert_eq!(scenario.checkpoints[1].label, "idle_tail");
        assert_eq!(scenario.checkpoints[1].frame_index, 359);
        // Script is pure idle — no button presses
        assert!(scenario.script.iter().all(|&b| b == 0));
    }

    #[test]
    fn idle_level_5_scenario_warps_to_level_5() {
        let scenario = idle_level_scenario(0x05, Some(120));
        assert_eq!(scenario.slug, "idle_level_05");
        assert_eq!(scenario.warp_to_level, Some(0x05));
        // settle=60, idle=120, total=180
        assert_eq!(scenario.frame_count, 180);
        // Script is pure idle
        assert!(scenario.script.iter().all(|&b| b == 0));
    }

    #[test]
    fn idle_level_scenario_frame_count_covers_checkpoints() {
        for level_id in [0x00, 0x05, 0x10, 0x68] {
            let scenario = idle_level_scenario(level_id, None);
            let max_cp = scenario
                .checkpoints
                .iter()
                .map(|cp| cp.frame_index)
                .max()
                .expect("checkpoints");
            assert!(
                scenario.frame_count > max_cp,
                "level {level_id:#04x}: frame_count {} should exceed max checkpoint {max_cp}",
                scenario.frame_count
            );
        }
    }

    #[test]
    fn idle_level_scenario_script_length_matches_frame_count() {
        let scenario = idle_level_scenario(0x03, Some(200));
        assert_eq!(scenario.script.len(), scenario.frame_count + 1);
    }

    #[test]
    fn idle_full_frame_scenario_requests_settled_oracle_warp_explicitly() {
        let scenario = idle_level_full_frame_scenario(0x01);

        assert!(scenario.settle_oracle_warp);
    }

    #[test]
    fn idle_level_full_frame_scenario_adds_visual_keyframes() {
        let scenario = idle_level_full_frame_scenario(0x02);
        assert_eq!(scenario.slug, "idle_level_full_frames_02");
        assert_eq!(scenario.warp_to_level, Some(0x02));
        assert_eq!(scenario.frame_count, 360);
        assert_eq!(scenario.script.len(), scenario.frame_count + 1);
        assert!(scenario.script.iter().all(|&b| b == 0));

        let checkpoints: Vec<(&str, usize)> = scenario
            .checkpoints
            .iter()
            .map(|cp| (cp.label.as_str(), cp.frame_index))
            .collect();
        assert_eq!(
            checkpoints,
            vec![
                ("frame_000", 0),
                ("frame_030", 30),
                ("level_settle", 60),
                ("frame_120", 120),
                ("frame_300", 300),
                ("idle_tail", 359),
            ]
        );
    }

    #[test]
    fn scenario_to_dynamic_preserves_slug_and_checkpoints() {
        let dynamic = Scenario::BootTitleStartLevel1.to_dynamic();
        assert_eq!(dynamic.slug, "boot_title_start_level1");
        assert_eq!(dynamic.frame_count, 720);
        assert_eq!(dynamic.checkpoints.len(), 3);
        assert_eq!(dynamic.checkpoints[0].label, "title");
        assert_eq!(dynamic.checkpoints[2].label, "level_settle");
    }

    #[test]
    fn gameplay_scenario_to_dynamic_preserves_slug_and_checkpoints() {
        let dynamic = GameplayScenario::PauseResumeLevel1.to_dynamic();
        assert_eq!(dynamic.slug, "pause_resume_level1");
        assert_eq!(dynamic.frame_count, 1_120);
        assert_eq!(dynamic.checkpoints.len(), 4);
    }

    #[test]
    fn gameplay_progress_scenario_reaches_level_2_live_tail() {
        let dynamic = GameplayScenario::ProgressTwoTransitions.to_dynamic();
        assert_eq!(dynamic.frame_count, 3_301);
        assert_eq!(dynamic.checkpoints.len(), 6);
        assert_eq!(dynamic.checkpoints[4].label, "level_2_settle");
        assert_eq!(dynamic.checkpoints[4].frame_index, 2_701);
        assert_eq!(dynamic.checkpoints[5].label, "level_2_live_tail");
        assert_eq!(dynamic.checkpoints[5].frame_index, 3_300);
    }

    #[test]
    fn dynamic_scenario_round_trips_through_json() {
        let original = idle_level_scenario(0x0A, Some(60));
        let json = serde_json::to_string(&original).expect("serialize");
        let restored: DynamicScenario = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, restored);
    }

    #[test]
    fn walk_right_scenario_holds_right_with_periodic_jumps() {
        let scenario = walk_right_scenario(0x00, 300);
        assert_eq!(scenario.slug, "walk_right_00");
        assert_eq!(scenario.warp_to_level, Some(0x00));
        assert_eq!(scenario.frame_count, 360);
        // Settle phase is idle
        assert_eq!(scenario.script[59], 0);
        // Walk phase holds right
        assert_eq!(scenario.script[60] & (1 << 3), 1 << 3);
        assert_eq!(scenario.script[359] & (1 << 3), 1 << 3);
        // First jump at settle + 30 = frame 90
        assert_eq!(scenario.script[90] & (1 << 4), 1 << 4);
        // No jump at frame 89
        assert_eq!(scenario.script[89] & (1 << 4), 0);
        // Checkpoints cover the walk
        assert_eq!(scenario.checkpoints.len(), 4);
        assert_eq!(scenario.checkpoints[0].label, "level_settle");
        assert_eq!(scenario.checkpoints[3].label, "walk_end");
    }

    #[test]
    fn walk_right_form_scenarios_seed_playtest_forms() {
        let red_stealth = walk_right_red_stealth_scenario(0x00, 700);
        assert_eq!(red_stealth.slug, "walk_right_red_stealth_00");
        assert_eq!(red_stealth.playtest_form, Some(3));
        assert_eq!(red_stealth.frame_count, 760);
        assert_eq!(red_stealth.script[60] & (1 << 3), 1 << 3);

        let iron_knight = walk_right_iron_knight_scenario(0x00, 700);
        assert_eq!(iron_knight.slug, "walk_right_iron_knight_00");
        assert_eq!(iron_knight.playtest_form, Some(6));

        let juggernaut = walk_right_juggernaut_scenario(0x00, 700);
        assert_eq!(juggernaut.slug, "walk_right_juggernaut_00");
        assert_eq!(juggernaut.playtest_form, Some(5));
    }

    #[test]
    fn walk_right_scenario_frame_count_covers_checkpoints() {
        let scenario = walk_right_scenario(0x05, 180);
        let max_cp = scenario
            .checkpoints
            .iter()
            .map(|cp| cp.frame_index)
            .max()
            .expect("checkpoints");
        assert!(scenario.frame_count > max_cp);
        assert_eq!(scenario.script.len(), scenario.frame_count + 1);
    }

    #[test]
    fn level_00_slope_right_probe_starts_on_first_level_slope() {
        let scenario = level_00_slope_right_probe_scenario();

        assert_eq!(scenario.slug, "level_00_slope_right_probe");
        assert_eq!(scenario.warp_to_level, Some(0x00));
        assert!(scenario.settle_oracle_warp);
        assert_eq!(
            scenario.start_position,
            Some(DynamicStartPosition { x: 760, y: 231 })
        );
        assert_eq!(
            scenario.camera_position,
            Some(DynamicStartPosition { x: 608, y: 32 })
        );
        assert_eq!(scenario.frame_count, 120);
        assert_eq!(scenario.script.len(), scenario.frame_count + 1);
        assert!(
            scenario
                .script
                .iter()
                .take(scenario.frame_count)
                .all(|buttons| buttons & (1 << 3) != 0),
            "slope probe must hold Right throughout the sampled window"
        );
        assert_eq!(
            scenario
                .checkpoints
                .iter()
                .map(|checkpoint| checkpoint.label.as_str())
                .collect::<Vec<_>>(),
            vec![
                "slope_probe_start",
                "slope_first_step",
                "slope_anim_window",
                "slope_mid",
                "slope_tail",
            ]
        );
    }

    #[test]
    fn prize_block_scenario_scripts_walk_and_jumps() {
        let scenario = prize_block_scenario(0x00);
        assert_eq!(scenario.slug, "prize_block_00");
        assert_eq!(scenario.warp_to_level, Some(0x00));
        assert_eq!(scenario.frame_count, 360);
        // Settle phase is idle
        assert_eq!(scenario.script[59], 0);
        // Walking right in phase 1
        assert_eq!(scenario.script[60] & (1 << 3), 1 << 3);
        // First jump: right + B at frame 120
        assert_eq!(scenario.script[120], (1 << 3) | (1 << 4));
        // Second jump: right + B at frame 240
        assert_eq!(scenario.script[240], (1 << 3) | (1 << 4));
        // 6 checkpoints
        assert_eq!(scenario.checkpoints.len(), 6);
        assert_eq!(scenario.checkpoints[0].label, "level_settle");
        assert_eq!(scenario.checkpoints[2].label, "post_first_jump");
        assert_eq!(scenario.checkpoints[5].label, "scenario_end");
    }

    #[test]
    fn prize_block_scenario_frame_count_covers_checkpoints() {
        let scenario = prize_block_scenario(0x00);
        let max_cp = scenario
            .checkpoints
            .iter()
            .map(|cp| cp.frame_index)
            .max()
            .expect("checkpoints");
        assert!(scenario.frame_count > max_cp);
        assert_eq!(scenario.script.len(), scenario.frame_count + 1);
    }

    #[test]
    fn issue_216_prize_audio_audit_starts_under_clean_prize_cluster() {
        let scenario = issue_216_prize_audio_audit_scenario();
        assert_eq!(scenario.slug, "issue_216_prize_audio_audit_00");
        assert_eq!(scenario.warp_to_level, Some(0x00));
        assert_eq!(scenario.frame_count, 180);
        assert_eq!(
            scenario
                .start_position
                .map(|position| (position.x, position.y)),
            Some((230, 271)),
            "the #216 route avoids the old frame-109 damage contact before the block-hit cue"
        );
        assert_eq!(
            scenario
                .camera_position
                .map(|position| (position.x, position.y)),
            Some((16, 136))
        );
        assert_eq!(scenario.script[19], 0);
        assert_eq!(scenario.script[20], 1 << 4);
        assert_eq!(scenario.script[24], 1 << 4);
        assert_eq!(scenario.script[25], 0);
        assert!(
            scenario.script[25..].iter().all(|&buttons| buttons == 0),
            "the audit idles after the vertical jump so diamond pickup timing is not hidden by later movement"
        );
        assert_eq!(
            scenario
                .checkpoints
                .iter()
                .map(|checkpoint| checkpoint.label.as_str())
                .collect::<Vec<_>>(),
            vec![
                "route_start",
                "jump_input",
                "block_hit_window",
                "diamond_pickup_window",
                "route_tail",
            ]
        );
        assert!(scenario.settle_oracle_warp);
    }

    #[test]
    fn stomp_enemy_scenario_holds_right_run_with_aggressive_jumps() {
        let scenario = stomp_enemy_scenario(0x00, 300);
        assert_eq!(scenario.slug, "stomp_enemy_00");
        assert_eq!(scenario.warp_to_level, Some(0x00));
        assert_eq!(scenario.frame_count, 360);
        // Settle phase is idle
        assert_eq!(scenario.script[59], 0);
        // Walk phase holds right + run (C)
        assert_eq!(
            scenario.script[60] & ((1 << 3) | (1 << 5)),
            (1 << 3) | (1 << 5)
        );
        // First jump at settle + 15 = frame 75
        assert_eq!(scenario.script[75] & (1 << 4), 1 << 4);
        // Checkpoints cover the run
        assert_eq!(scenario.checkpoints.len(), 4);
        assert_eq!(scenario.checkpoints[0].label, "level_settle");
        assert_eq!(scenario.checkpoints[3].label, "stomp_end");
    }

    #[test]
    fn stomp_enemy_scenario_frame_count_covers_checkpoints() {
        let scenario = stomp_enemy_scenario(0x05, 240);
        let max_cp = scenario
            .checkpoints
            .iter()
            .map(|cp| cp.frame_index)
            .max()
            .expect("checkpoints");
        assert!(scenario.frame_count > max_cp);
        assert_eq!(scenario.script.len(), scenario.frame_count + 1);
    }

    #[test]
    fn issue_271_dragon_stomp_damage_audit_names_the_current_graze_window() {
        let scenario = issue_271_level00_dragon_stomp_damage_audit_scenario();

        assert_eq!(
            scenario.slug,
            "issue_271_level00_dragon_stomp_damage_audit_00"
        );
        assert_eq!(scenario.warp_to_level, Some(0x00));
        assert_eq!(scenario.frame_count, 280);
        assert!(!scenario.settle_oracle_warp);

        const RIGHT: u8 = 1 << 3;
        const JUMP: u8 = 1 << 4;
        const RUN: u8 = 1 << 5;
        assert_eq!(
            scenario.script[137] & (RIGHT | JUMP | RUN),
            RIGHT | JUMP | RUN
        );
        assert_eq!(scenario.script[141] & JUMP, 0);

        let labels: Vec<_> = scenario
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.label.as_str())
            .collect();
        assert_eq!(
            labels,
            vec![
                "level_settle",
                "dragon_graze_damage_window_loc_1c76",
                "post_graze_damage_window",
                "route_tail",
            ]
        );
    }

    #[test]
    fn issue_271_rock_tank_stomp_route_audit_names_the_current_miss_window() {
        let scenario = issue_271_level01_rock_tank_stomp_route_audit_scenario();

        assert_eq!(
            scenario.slug,
            "issue_271_level01_rock_tank_stomp_route_audit_01"
        );
        assert_eq!(scenario.warp_to_level, Some(0x01));
        assert_eq!(scenario.frame_count, 280);
        assert!(!scenario.settle_oracle_warp);

        const RIGHT: u8 = 1 << 3;
        const JUMP: u8 = 1 << 4;
        const RUN: u8 = 1 << 5;
        assert_eq!(
            scenario.script[136] & (RIGHT | JUMP | RUN),
            RIGHT | JUMP | RUN
        );
        assert_eq!(scenario.script[171] & (RIGHT | RUN), RIGHT | RUN);
        assert_eq!(scenario.script[171] & JUMP, 0);

        let closest = scenario
            .checkpoints
            .iter()
            .find(|checkpoint| checkpoint.label == "closest_tank_pass_miss")
            .expect("closest Tank miss checkpoint");
        assert_eq!(closest.frame_index, 171);
    }

    #[test]
    fn enemy_spawn_audit_heady_metal_targets_plethora_one() {
        // HeadyMetal (kind 0x20) is the Plethora 1 boss, level 0x33 — verified
        // via `kidcli inspect-level --rom kid.bin --level 33` 2026-04-25.
        let scenario = enemy_spawn_audit_scenario(0x20, 0x33);
        assert_eq!(scenario.slug, "enemy_spawn_audit_20_level_33");
        assert_eq!(scenario.warp_to_level, Some(0x33));
        // Pure idle — no input noise during the boss spawn window.
        assert!(scenario.script.iter().all(|byte| *byte == 0));
        assert_eq!(scenario.frame_count, 660);
        // Spawn-window checkpoint is past Enemy20_HeadyMetal_Init (~120f).
        let labels: Vec<_> = scenario
            .checkpoints
            .iter()
            .map(|cp| cp.label.clone())
            .collect();
        assert_eq!(
            labels,
            vec![
                "level_settle".to_string(),
                "enemy_spawn_window".into(),
                "enemy_visible_mid".into(),
                "enemy_visible_tail".into(),
            ]
        );
        assert_eq!(scenario.checkpoints[1].frame_index, 240);
    }

    #[test]
    fn enemy_spawn_audit_scenario_frame_count_covers_checkpoints() {
        let scenario = enemy_spawn_audit_scenario(0x20, 0x33);
        let max_cp = scenario
            .checkpoints
            .iter()
            .map(|cp| cp.frame_index)
            .max()
            .expect("checkpoints");
        assert!(scenario.frame_count > max_cp);
        assert_eq!(scenario.script.len(), scenario.frame_count + 1);
    }

    #[test]
    fn lava_weather_audit_scenario_targets_settled_lava_object_window() {
        let scenario = lava_weather_audit_scenario(0x05);

        assert_eq!(scenario.slug, "lava_weather_audit_05");
        assert_eq!(scenario.warp_to_level, Some(0x05));
        assert!(scenario.settle_oracle_warp);
        assert_eq!(scenario.frame_count, 180);
        assert_eq!(scenario.script.len(), scenario.frame_count + 1);
        assert!(scenario.script.iter().all(|&buttons| buttons == 0));
        assert_eq!(scenario.checkpoints[0].label, "weather_object_start");
        assert_eq!(scenario.checkpoints[0].frame_index, 0);
        assert_eq!(
            scenario.checkpoints.last().expect("tail checkpoint").label,
            "weather_object_tail"
        );
    }
}
