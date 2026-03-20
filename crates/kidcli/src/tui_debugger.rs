use crate::oracle::OracleManifest;
use crate::sweep::find_first_crc_mismatch;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::ExecutableCommand;
use kid_core::NativeTraceFrame;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::io::stdout;

pub(crate) struct TuiDebugger {
    native_trace: Vec<NativeTraceFrame>,
    oracle_states: Vec<crate::oracle::OracleState>,
    native_crcs: Vec<u32>,
    oracle_crcs: Vec<u32>,
    current_frame: usize,
    first_divergence: Option<usize>,
}

impl TuiDebugger {
    pub(crate) fn new(native_trace: Vec<NativeTraceFrame>, oracle: &OracleManifest) -> Self {
        let native_crcs: Vec<u32> = native_trace
            .iter()
            .map(|f| f.authoritative_crc32())
            .collect();
        let first_divergence = find_first_crc_mismatch(&native_crcs, &oracle.frame_crcs);
        Self {
            native_trace,
            oracle_states: oracle.states.clone(),
            native_crcs,
            oracle_crcs: oracle.frame_crcs.clone(),
            current_frame: 0,
            first_divergence,
        }
    }

    pub(crate) fn run(mut self) -> Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        loop {
            terminal.draw(|frame| self.render(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Right | KeyCode::Char('l') => {
                        if self.current_frame + 1 < self.native_trace.len() {
                            self.current_frame += 1;
                        }
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        self.current_frame = self.current_frame.saturating_sub(1);
                    }
                    KeyCode::Char('d') => {
                        if let Some(div) = self.first_divergence {
                            self.current_frame = div;
                        }
                    }
                    KeyCode::Home => {
                        self.current_frame = 0;
                    }
                    KeyCode::End => {
                        self.current_frame = self.native_trace.len().saturating_sub(1);
                    }
                    KeyCode::PageDown => {
                        self.current_frame = (self.current_frame + 60)
                            .min(self.native_trace.len().saturating_sub(1));
                    }
                    KeyCode::PageUp => {
                        self.current_frame = self.current_frame.saturating_sub(60);
                    }
                    _ => {}
                }
            }
        }

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // header
                Constraint::Min(10),  // state comparison
                Constraint::Length(3), // footer / help
            ])
            .split(frame.area());

        // Header
        let native_frame = &self.native_trace[self.current_frame];
        let crc_match = self
            .native_crcs
            .get(self.current_frame)
            .zip(self.oracle_crcs.get(self.current_frame))
            .is_some_and(|(n, o)| n == o);
        let crc_indicator = if crc_match { "OK" } else { "DIFF" };
        let div_info = self
            .first_divergence
            .map(|f| format!(" | first divergence: frame {f}"))
            .unwrap_or_default();

        let header = Paragraph::new(format!(
            "Frame {}/{} | CRC: {} | Mode: {:?} | Level: {:02x}{}",
            self.current_frame,
            self.native_trace.len(),
            crc_indicator,
            native_frame.game_mode,
            native_frame.current_level.0,
            div_info,
        ))
        .block(Block::default().borders(Borders::ALL).title("Debug TUI"));
        frame.render_widget(header, chunks[0]);

        // State comparison: two columns
        let state_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Native state
        let native_lines = self.format_native_state(native_frame);
        let native_widget = Paragraph::new(native_lines)
            .block(Block::default().borders(Borders::ALL).title("Native"));
        frame.render_widget(native_widget, state_chunks[0]);

        // Oracle state (find nearest checkpoint)
        let oracle_lines = self.format_oracle_state();
        let oracle_widget = Paragraph::new(oracle_lines)
            .block(Block::default().borders(Borders::ALL).title("Oracle"));
        frame.render_widget(oracle_widget, state_chunks[1]);

        // Footer
        let footer = Paragraph::new(
            " <-/->: step | PgUp/PgDn: +/-60 | Home/End | d: jump to divergence | q: quit",
        )
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(footer, chunks[2]);
    }

    fn format_native_state(&self, f: &NativeTraceFrame) -> Vec<Line<'static>> {
        let phase = if f.kid_x_velocity.is_some() {
            "in-game"
        } else {
            "boot"
        };
        vec![
            Line::from(format!(
                "game_mode: {:#06x} ({:?})",
                f.game_mode_value, f.game_mode
            )),
            Line::from(format!("level: {:#04x}", f.current_level.0)),
            Line::from(format!("phase: {phase}")),
            Line::from(format!(
                "kid_x_pos: {:#010x}",
                f.kid_x_pos.unwrap_or(0)
            )),
            Line::from(format!(
                "kid_y_pos: {:#010x}",
                f.kid_y_pos.unwrap_or(0)
            )),
            Line::from(format!(
                "kid_x_vel: {}",
                f.kid_x_velocity
                    .map(|v| format!("{v:#010x}"))
                    .unwrap_or_else(|| "-".into())
            )),
            Line::from(format!(
                "kid_y_vel: {}",
                f.kid_y_velocity
                    .map(|v| format!("{v:#010x}"))
                    .unwrap_or_else(|| "-".into())
            )),
            Line::from(format!(
                "camera: ({:#010x}, {:#010x})",
                f.camera_x_pos, f.camera_y_pos
            )),
            Line::from(format!(
                "helmet: {} | hp: {} | lives: {}",
                f.current_helmet_id, f.number_hitpoints, f.number_lives
            )),
            Line::from(format!(
                "score: {} | diamonds: {} | power: {}",
                f.score, f.diamond_count, f.diamond_power_active
            )),
            Line::from(format!(
                "objects: {}/{} | enemies: {} | platforms: {}",
                f.number_objects,
                f.number_gfx_objects,
                f.active_enemy_object_count,
                f.active_platform_object_count
            )),
            Line::from(format!(
                "invuln: {} | paused: {}",
                f.invulnerable_frames, f.paused
            )),
            Line::from(format!("murder_wall: {:?}", f.murder_wall_x)),
            Line::from(format!(
                "CRC: {:#010x}",
                self.native_crcs
                    .get(self.current_frame)
                    .unwrap_or(&0)
            )),
        ]
    }

    fn format_oracle_state(&self) -> Vec<Line<'static>> {
        // Find the oracle state closest to (but not after) current frame
        let nearest = self
            .oracle_states
            .iter()
            .rev()
            .find(|s| s.frame_index <= self.current_frame);

        match nearest {
            Some(s) => {
                vec![
                    Line::from(format!(
                        "checkpoint: {} (frame {})",
                        s.label, s.frame_index
                    )),
                    Line::from(format!("game_mode: {:#06x}", s.game_mode)),
                    Line::from(format!("level: {:#04x}", s.current_level_id)),
                    Line::from(format!("kid_x_pos: {:#010x}", s.kid_x_pos)),
                    Line::from(format!("kid_y_pos: {:#010x}", s.kid_y_pos)),
                    Line::from(format!(
                        "kid_x_vel: {}",
                        s.kid_x_vel
                            .map(|v| format!("{v:#010x}"))
                            .unwrap_or_else(|| "-".into())
                    )),
                    Line::from(format!(
                        "kid_y_vel: {}",
                        s.kid_y_vel
                            .map(|v| format!("{v:#010x}"))
                            .unwrap_or_else(|| "-".into())
                    )),
                    Line::from(format!(
                        "camera: ({:#010x}, {:#010x})",
                        s.camera_x_pos, s.camera_y_pos
                    )),
                    Line::from(format!(
                        "helmet: {} | hp: {} | lives: {}",
                        s.current_helmet_id, s.number_hitpoints, s.number_lives
                    )),
                    Line::from(format!(
                        "score: {} | diamonds: {} | power: {}",
                        s.score, s.number_diamonds, s.diamond_power_active
                    )),
                    Line::from(format!(
                        "objects: {}/{}",
                        s.number_objects, s.number_gfx_objects
                    )),
                    Line::from(format!(
                        "invuln: {:?} | paused: {}",
                        s.invulnerable_frames, s.game_paused
                    )),
                    Line::from(format!("murder_wall: {:?}", s.murder_wall_x)),
                    Line::from(format!(
                        "CRC: {:#010x}",
                        self.oracle_crcs
                            .get(self.current_frame)
                            .unwrap_or(&0)
                    )),
                ]
            }
            None => vec![Line::from("(no oracle state before this frame)")],
        }
    }
}
