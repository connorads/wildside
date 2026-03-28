use anyhow::Result;
use clap::ValueEnum;
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

const DEATH_RESPAWN_LEVEL_1_CHECKPOINTS: [GameplayCheckpoint; 5] = [
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
        frame_index: 6_000,
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
    /// Target level to warp to before running the script.
    /// Both native and oracle boot to level 0 first, then advance
    /// to this level using their own mechanism (native: direct skip,
    /// oracle: memory write + cheat flag). The script runs after warping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warp_to_level: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicCheckpoint {
    pub label: String,
    pub frame_index: usize,
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
        warp_to_level: Some(level_id),
    }
}

/// Walk right for `walk_frames` with periodic jumps. Useful for verifying
/// movement, enemy encounters, and basic traversal on any level.
pub fn walk_right_scenario(level_id: u8, walk_frames: usize) -> DynamicScenario {
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
        slug: format!("walk_right_{level_id:02x}"),
        script,
        frame_count: total_frames,
        checkpoints,
        warp_to_level: Some(level_id),
    }
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
        warp_to_level: Some(level_id),
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
        warp_to_level: Some(level_id),
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
        Ok(script)
    }

    /// Convert a static scenario into the runtime DynamicScenario representation.
    #[cfg(test)]
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
            warp_to_level: None,
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
            Self::DeathRespawnLevel1 => 6_001,
            Self::ProgressTwoTransitions => 3_301,
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

        let mut script = self.script()?;
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
            warp_to_level: None,
        })
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
            warp_to_level: None,
        })
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
        DynamicScenario, GameplayScenario, Scenario, idle_level_scenario, prize_block_scenario,
        stomp_enemy_scenario, walk_right_scenario,
    };

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
        assert_eq!(script[33], 0x80);
        assert_eq!(script[178], 0x80);
        assert_eq!(script[360], 0x80);
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
        assert_eq!(script[2_700], (1 << 3) | (1 << 5) | (1 << 6) | 0x80);
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
    fn gameplay_death_scenario_keeps_the_death_window_free_of_start_presses() {
        let script = GameplayScenario::DeathRespawnLevel1
            .script()
            .expect("gameplay scenario script should build");
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
    }

    #[test]
    fn gameplay_death_scenario_extends_past_the_hidden_respawn_bootstrap() {
        assert_eq!(GameplayScenario::DeathRespawnLevel1.frame_count(), 6_001);
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
                ("respawn_bootstrap_tail", 6_000),
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
        let dynamic = GameplayScenario::DeathRespawnLevel1
            .to_dynamic_window(1_278, 1_280, 1)
            .expect("dynamic gameplay window");
        assert_eq!(
            dynamic
                .checkpoints
                .iter()
                .map(|checkpoint| (checkpoint.label.as_str(), checkpoint.frame_index))
                .collect::<Vec<_>>(),
            vec![
                ("frame_01278", 1_278),
                ("pre_death", 1_279),
                ("frame_01280", 1_280),
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
    fn gameplay_progress_scenario_reaches_level_2_live_tail() {
        let dynamic = GameplayScenario::ProgressTwoTransitions
            .to_dynamic()
            .expect("to_dynamic");
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
}
