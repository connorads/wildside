use anyhow::Result;
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Scenario {
    BootTitleIdle10s,
    BootTitleStartLevel1,
    BootTitleStartLevel1MoveRight5s,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameCheckpoint {
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

const BOOT_TITLE_START_LEVEL_1_MOVE_RIGHT_5S_CHECKPOINTS: [FrameCheckpoint; 4] = [
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
        frame_index: 650,
    },
    FrameCheckpoint {
        label: "move_end",
        frame_index: 799,
    },
];

impl Scenario {
    pub fn slug(self) -> &'static str {
        match self {
            Self::BootTitleIdle10s => "boot_title_idle_10s",
            Self::BootTitleStartLevel1 => "boot_title_start_level1",
            Self::BootTitleStartLevel1MoveRight5s => "boot_title_start_level1_move_right_5s",
        }
    }

    pub fn frame_count(self) -> usize {
        match self {
            Self::BootTitleIdle10s => 600,
            Self::BootTitleStartLevel1 => 720,
            Self::BootTitleStartLevel1MoveRight5s => 900,
        }
    }

    pub fn checkpoints(self) -> &'static [FrameCheckpoint] {
        match self {
            Self::BootTitleIdle10s => &BOOT_TITLE_IDLE_10S_CHECKPOINTS,
            Self::BootTitleStartLevel1 => &BOOT_TITLE_START_LEVEL_1_CHECKPOINTS,
            Self::BootTitleStartLevel1MoveRight5s => {
                &BOOT_TITLE_START_LEVEL_1_MOVE_RIGHT_5S_CHECKPOINTS
            }
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
                apply_range(&mut script, 500, 799, 1 << 3);
            }
        }
        Ok(script)
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
    use super::Scenario;

    #[test]
    fn move_right_scenario_holds_right_for_five_seconds() {
        let script = Scenario::BootTitleStartLevel1MoveRight5s
            .script()
            .expect("scenario script should build");
        assert_eq!(script[500], 1 << 3);
        assert_eq!(script[799], 1 << 3);
        assert_eq!(script[800], 0);
    }

    #[test]
    fn start_level_scenario_presses_start_during_boot_path() {
        let script = Scenario::BootTitleStartLevel1
            .script()
            .expect("scenario script should build");
        assert_eq!(script[120], 0x80);
        assert_eq!(script[460], 0x80);
    }
}
