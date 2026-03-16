use anyhow::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

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

const LEVEL_1_ENEMY_WINDOW_CHECKPOINTS: [FrameCheckpoint; 3] = [
    FrameCheckpoint {
        label: "level_settle",
        frame_index: 719,
    },
    FrameCheckpoint {
        label: "enemy_window_open",
        frame_index: 730,
    },
    FrameCheckpoint {
        label: "enemy_window_tail",
        frame_index: 750,
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

const DEATH_RESPAWN_LEVEL_1_CHECKPOINTS: [GameplayCheckpoint; 4] = [
    GameplayCheckpoint {
        label: "level_settle",
        frame_index: 719,
    },
    GameplayCheckpoint {
        label: "pre_death",
        frame_index: 1_279,
    },
    GameplayCheckpoint {
        label: "death_tail",
        frame_index: 1_439,
    },
    GameplayCheckpoint {
        label: "respawn_tail",
        frame_index: 1_919,
    },
];

const PROGRESS_TWO_TRANSITIONS_CHECKPOINTS: [GameplayCheckpoint; 4] = [
    GameplayCheckpoint {
        label: "level_settle",
        frame_index: 719,
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
        label: "run_end",
        frame_index: 3_199,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicCheckpoint {
    pub label: String,
    pub frame_index: usize,
}

/// Boot → title → start game → idle on level 0x00 for 300 frames.
///
/// All idle-level scenarios share the same boot prefix (start presses to reach
/// gameplay). For level 0x00 (the default starting level) no warp is needed.
/// For other levels, the native runtime's debug-warp key (N) is pressed
/// repeatedly to advance. The oracle side must use the same key sequence.
///
/// `idle_frames` defaults to 300 (~5 s at 60 fps) when `None`.
pub fn idle_level_scenario(level_id: u8, idle_frames: Option<usize>) -> DynamicScenario {
    let idle = idle_frames.unwrap_or(300);

    // Boot prefix: press Start repeatedly to navigate title → game start.
    // Level 0x00 loads automatically after the boot sequence settles at frame 720.
    // For later levels we press the debug-warp key (N = Up, mapped to bit 0)
    // once per level advance. Each warp takes ~80 frames to settle.
    let warps_needed = u16::from(level_id);
    let warp_period: usize = 80;
    let post_boot_frame: usize = 720;
    let warp_end = post_boot_frame + (warps_needed as usize) * warp_period;
    let settle_start = warp_end;
    // 60 frames for the level to settle after last warp (or boot).
    let settle_frames: usize = 60;
    let idle_start = settle_start + settle_frames;
    let total_frames = idle_start + idle;

    let mut script = vec![0u8; total_frames.saturating_add(1)];

    // Boot to level 1: same as apply_boot_to_level_1
    apply_single_frames(&mut script, &[120, 200, 280, 360, 440], 0x80);
    apply_range(&mut script, 460, 460, 0x80);

    // Debug warp presses (Start+A+Up combo on a single frame per warp).
    // In the native runtime the N key triggers `debug_warp_next_level()`.
    // For the oracle, Start+A+Up (0x80|0x40|0x01 = 0xC1) is sent as input.
    for warp in 0..warps_needed {
        let frame = post_boot_frame + (warp as usize) * warp_period;
        if let Some(slot) = script.get_mut(frame) {
            *slot |= 0xC1; // Start + A + Up
        }
    }

    let checkpoints = vec![
        DynamicCheckpoint {
            label: "level_settle".into(),
            frame_index: idle_start,
        },
        DynamicCheckpoint {
            label: "idle_tail".into(),
            frame_index: total_frames.saturating_sub(1),
        },
    ];

    DynamicScenario {
        slug: format!("idle_level_{level_id:02x}"),
        script,
        frame_count: total_frames,
        checkpoints,
    }
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

    pub fn script(self) -> Result<Vec<u8>> {
        let mut script = vec![0; self.frame_count().saturating_add(1)];
        match self {
            Self::BootTitleIdle10s => {}
            Self::BootTitleStartLevel1 => {
                apply_single_frames(&mut script, &[120, 200, 280, 360, 440], 0x80);
                apply_range(&mut script, 460, 460, 0x80);
            }
            Self::BootTitleStartLevel1MoveRight5s => {
                apply_single_frames(&mut script, &[120, 200, 280, 360, 440], 0x80);
                apply_range(&mut script, 460, 460, 0x80);
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
        Ok(script)
    }

    /// Convert a static scenario into the runtime DynamicScenario representation.
    pub fn to_dynamic(self) -> Result<DynamicScenario> {
        Ok(DynamicScenario {
            slug: self.slug().to_string(),
            script: self.script()?,
            frame_count: self.frame_count(),
            checkpoints: self
                .checkpoints()
                .iter()
                .map(|cp| DynamicCheckpoint {
                    label: cp.label.to_string(),
                    frame_index: cp.frame_index,
                })
                .collect(),
        })
    }
}

impl GameplayScenario {
    pub fn slug(self) -> &'static str {
        match self {
            Self::PauseResumeLevel1 => "pause_resume_level1",
            Self::DeathRespawnLevel1 => "death_respawn_level1",
            Self::ProgressTwoTransitions => "progress_two_transitions",
            Self::SoakLevel10m => "soak_level_10m",
        }
    }

    pub fn frame_count(self) -> usize {
        match self {
            Self::PauseResumeLevel1 => 1_120,
            Self::DeathRespawnLevel1 => 1_920,
            Self::ProgressTwoTransitions => 3_200,
            Self::SoakLevel10m => 36_720,
        }
    }

    pub fn checkpoints(self) -> &'static [GameplayCheckpoint] {
        match self {
            Self::PauseResumeLevel1 => &PAUSE_RESUME_LEVEL_1_CHECKPOINTS,
            Self::DeathRespawnLevel1 => &DEATH_RESPAWN_LEVEL_1_CHECKPOINTS,
            Self::ProgressTwoTransitions => &PROGRESS_TWO_TRANSITIONS_CHECKPOINTS,
            Self::SoakLevel10m => &SOAK_LEVEL_10M_CHECKPOINTS,
        }
    }

    pub fn uses_level_skip_cheat(self) -> bool {
        matches!(self, Self::ProgressTwoTransitions | Self::SoakLevel10m)
    }

    pub fn required_level_layout_range(self) -> (u8, u8) {
        match self {
            Self::PauseResumeLevel1 | Self::DeathRespawnLevel1 => (0x00, 0x00),
            Self::ProgressTwoTransitions => (0x00, 0x02),
            Self::SoakLevel10m => (0x00, 0x1F),
        }
    }

    pub fn script(self) -> Result<Vec<u8>> {
        let mut script = vec![0; self.frame_count().saturating_add(1)];
        apply_boot_to_level_1(&mut script);
        match self {
            Self::PauseResumeLevel1 => {
                apply_range(&mut script, 720, 839, (1 << 3) | (1 << 6));
                apply_range(&mut script, 840, 840, 0x80);
                apply_range(&mut script, 841, 919, 1 << 3);
                apply_range(&mut script, 920, 920, 0x80);
                apply_range(&mut script, 960, 1_119, (1 << 3) | (1 << 6));
                add_periodic_jump(&mut script, 980, 1_119, 24, 4);
            }
            Self::DeathRespawnLevel1 => {
                apply_range(&mut script, 720, 1_350, (1 << 3) | (1 << 6));
                add_periodic_jump(&mut script, 760, 1_350, 24, 4);
                apply_single_frames(&mut script, &[1_400, 1_460, 1_520], 0x80);
                apply_range(&mut script, 1_560, 1_919, 1 << 3);
                add_periodic_jump(&mut script, 1_600, 1_919, 28, 3);
            }
            Self::ProgressTwoTransitions => {
                apply_range(&mut script, 720, 899, 1 << 3);
                apply_single_frames(
                    &mut script,
                    &[900, 2_100, 2_140, 2_180],
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
                apply_range(
                    &mut script,
                    720,
                    self.frame_count().saturating_sub(1),
                    1 << 3,
                );
                for frame in (900..self.frame_count().saturating_sub(600)).step_by(600) {
                    apply_single_frames(&mut script, &[frame], (1 << 5) | (1 << 6) | 0x80);
                }
            }
        }
        Ok(script)
    }

    /// Convert a static gameplay scenario into the runtime DynamicScenario representation.
    pub fn to_dynamic(self) -> Result<DynamicScenario> {
        Ok(DynamicScenario {
            slug: self.slug().to_string(),
            script: self.script()?,
            frame_count: self.frame_count(),
            checkpoints: self
                .checkpoints()
                .iter()
                .map(|cp| DynamicCheckpoint {
                    label: cp.label.to_string(),
                    frame_index: cp.frame_index,
                })
                .collect(),
        })
    }
}

fn apply_boot_to_level_1(script: &mut [u8]) {
    apply_single_frames(script, &[120, 200, 280, 360, 440], 0x80);
    apply_range(script, 460, 460, 0x80);
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
    use super::{DynamicScenario, GameplayScenario, Scenario, idle_level_scenario};

    #[test]
    fn move_right_scenario_holds_right_for_five_seconds() {
        let script = Scenario::BootTitleStartLevel1MoveRight5s
            .script()
            .expect("scenario script should build");
        assert_eq!(script[719], 0);
        assert_eq!(script[720], 1 << 3);
        assert_eq!(script[1_019], 1 << 3);
        assert_eq!(script[1_020], 0);
    }

    #[test]
    fn start_level_scenario_presses_start_during_boot_path() {
        let script = Scenario::BootTitleStartLevel1
            .script()
            .expect("scenario script should build");
        assert_eq!(script[120], 0x80);
        assert_eq!(script[460], 0x80);
    }

    #[test]
    fn gameplay_regression_pause_resume_scenario_pauses_and_resumes() {
        let script = GameplayScenario::PauseResumeLevel1
            .script()
            .expect("gameplay scenario script should build");
        assert_eq!(script[840], 0x80);
        assert_eq!(script[920], 0x80);
        assert_eq!(script[980] & (1 << 4), 1 << 4);
    }

    #[test]
    fn gameplay_progress_scenario_holds_right_and_run() {
        let script = GameplayScenario::ProgressTwoTransitions
            .script()
            .expect("gameplay scenario script should build");
        assert_eq!(script[720] & (1 << 3), 1 << 3);
        assert_eq!(script[900], (1 << 5) | (1 << 6) | 0x80);
    }

    #[test]
    fn walk_right_scenario_holds_right_for_three_seconds() {
        let script = Scenario::WalkRight3s
            .script()
            .expect("scenario script should build");
        assert_eq!(script[719], 0);
        assert_eq!(script[720], 1 << 3);
        assert_eq!(script[899], 1 << 3);
        assert_eq!(script[900], 0);
    }

    #[test]
    fn level_1_enemy_window_scenario_tracks_the_first_enemy_slice() {
        let script = Scenario::Level1EnemyWindow
            .script()
            .expect("scenario script should build");
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
            vec!["level_settle", "enemy_window_open", "enemy_window_tail"]
        );
    }

    #[test]
    fn jump_from_standstill_scenario_presses_jump_briefly() {
        let script = Scenario::JumpFromStandstill
            .script()
            .expect("scenario script should build");
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
    fn gameplay_death_scenario_requests_restart_after_life_loss() {
        let script = GameplayScenario::DeathRespawnLevel1
            .script()
            .expect("gameplay scenario script should build");
        assert_eq!(script[1_400], 0x80);
        assert_eq!(script[1_560] & (1 << 3), 1 << 3);
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
    fn idle_level_0_scenario_boots_to_level_1_and_idles() {
        let scenario = idle_level_scenario(0x00, None);
        assert_eq!(scenario.slug, "idle_level_00");
        // No warps needed — boot lands on level 0x00 directly.
        // settle at 720+60=780, idle for 300, total=1080
        assert_eq!(scenario.frame_count, 1_080);
        assert_eq!(scenario.checkpoints.len(), 2);
        assert_eq!(scenario.checkpoints[0].label, "level_settle");
        assert_eq!(scenario.checkpoints[0].frame_index, 780);
        assert_eq!(scenario.checkpoints[1].label, "idle_tail");
        assert_eq!(scenario.checkpoints[1].frame_index, 1_079);
        // Boot presses present
        assert_eq!(scenario.script[120], 0x80);
        assert_eq!(scenario.script[460], 0x80);
        // No warp presses at frame 720
        assert_eq!(scenario.script[720], 0);
    }

    #[test]
    fn idle_level_5_scenario_warps_five_times() {
        let scenario = idle_level_scenario(0x05, Some(120));
        assert_eq!(scenario.slug, "idle_level_05");
        // 5 warps * 80 frames = 400, post_boot=720, settle_start=1120
        // settle_frames=60, idle_start=1180, total=1180+120=1300
        assert_eq!(scenario.frame_count, 1_300);
        // Each warp is Start+A+Up (0xC1) at 720, 800, 880, 960, 1040
        assert_eq!(scenario.script[720], 0xC1);
        assert_eq!(scenario.script[800], 0xC1);
        assert_eq!(scenario.script[880], 0xC1);
        assert_eq!(scenario.script[960], 0xC1);
        assert_eq!(scenario.script[1_040], 0xC1);
        // No warp at frame 1120 (only 5 warps needed)
        assert_eq!(scenario.script[1_120], 0);
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
    fn scenario_to_dynamic_preserves_slug_and_checkpoints() {
        let dynamic = Scenario::BootTitleStartLevel1
            .to_dynamic()
            .expect("to_dynamic");
        assert_eq!(dynamic.slug, "boot_title_start_level1");
        assert_eq!(dynamic.frame_count, 720);
        assert_eq!(dynamic.checkpoints.len(), 3);
        assert_eq!(dynamic.checkpoints[0].label, "title");
        assert_eq!(dynamic.checkpoints[2].label, "level_settle");
    }

    #[test]
    fn gameplay_scenario_to_dynamic_preserves_slug_and_checkpoints() {
        let dynamic = GameplayScenario::PauseResumeLevel1
            .to_dynamic()
            .expect("to_dynamic");
        assert_eq!(dynamic.slug, "pause_resume_level1");
        assert_eq!(dynamic.frame_count, 1_120);
        assert_eq!(dynamic.checkpoints.len(), 4);
    }

    #[test]
    fn dynamic_scenario_round_trips_through_json() {
        let original = idle_level_scenario(0x0A, Some(60));
        let json = serde_json::to_string(&original).expect("serialize");
        let restored: DynamicScenario = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, restored);
    }
}
