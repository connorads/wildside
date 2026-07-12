//! Recording state for one deterministic input segment of a live play session.
//!
//! A segment starts at a deterministic runtime start (boot, `--warp-level`, or a
//! debug warp) and accumulates the per-frame script/trace/audio that
//! `save_recording_with_artifacts` needs to write a standard recording bundle.
//! Plain `--record` runs are a single always-save-worthy segment; playtest mode
//! rotates through several per session.

use crate::inspect::target_enemy_contact_from_frame;
use crate::recording::{
    RecordingBundleArtifacts, RecordingRouteTarget, RecordingStart, build_recording,
    recording_route_target_score_commands, recording_tas_frontier_command,
    save_recording_with_artifacts,
};
use crate::scenarios::{DynamicCheckpoint, DynamicScenario};
use anyhow::{Result, anyhow};
use std::path::PathBuf;
use wildside_core::NativeTraceFrame;
use wildside_data::AssetManifest;
use wildside_frame::native_audio_events_for_trace;

use super::NativePlayOptions;

#[derive(Debug, Clone)]
pub(super) struct SegmentRecorder {
    record_to: PathBuf,
    slug: String,
    start: RecordingStart,
    script: Vec<u8>,
    trace: Vec<NativeTraceFrame>,
    audio: Vec<f32>,
    audio_events: Vec<String>,
    final_frame_rgb: Option<Vec<u8>>,
    save_worthy: bool,
    target_contact_stop_tail_remaining: Option<usize>,
    target_contact_stop: Option<TargetEnemyContactStop>,
}

impl SegmentRecorder {
    pub(super) fn new(record_to: PathBuf, start: RecordingStart) -> Result<Self> {
        let slug = record_to
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                anyhow!(
                    "recording path has no valid file name: {}",
                    record_to.display()
                )
            })?
            .to_owned();
        Ok(Self {
            record_to,
            slug,
            start,
            script: Vec::new(),
            trace: Vec::new(),
            audio: Vec::new(),
            audio_events: Vec::new(),
            final_frame_rgb: None,
            save_worthy: false,
            target_contact_stop_tail_remaining: None,
            target_contact_stop: None,
        })
    }

    pub(super) fn mark_save_worthy(&mut self) {
        self.save_worthy = true;
    }

    pub(super) fn save_worthy(&self) -> bool {
        self.save_worthy
    }

    /// Record one stepped frame's input and trace. Returns true when the
    /// recording should stop (target-enemy contact tail elapsed or the
    /// `--record-frames` limit is reached).
    pub(super) fn record_frame(
        &mut self,
        buttons: u8,
        trace: &NativeTraceFrame,
        options: &NativePlayOptions,
    ) -> bool {
        self.script.push(buttons);
        self.trace.push(trace.clone());
        let mut should_stop = update_target_contact_auto_stop(
            options,
            trace,
            self.trace.len().saturating_sub(1),
            &mut self.target_contact_stop_tail_remaining,
            &mut self.target_contact_stop,
        );
        if recording_frame_limit_reached(options, self.trace.len()) {
            should_stop = true;
        }
        should_stop
    }

    pub(super) fn record_audio(&mut self, samples: &[f32]) {
        self.audio.extend_from_slice(samples);
    }

    pub(super) fn push_audio_event(&mut self, event: String) {
        self.audio_events.push(event);
    }

    pub(super) fn set_final_frame(&mut self, rgb: &[u8]) {
        self.final_frame_rgb = Some(rgb.to_vec());
    }

    pub(super) fn frame_len(&self) -> usize {
        self.trace.len()
    }

    #[cfg(test)]
    pub(super) fn set_target_contact_stop(&mut self, contact: TargetEnemyContactStop) {
        self.target_contact_stop = Some(contact);
    }

    /// Flush a save-worthy segment to its standard recording bundle, or drop a
    /// segment nothing marked worth keeping.
    pub(super) fn finalise(
        self,
        manifest: &AssetManifest,
        options: &NativePlayOptions,
        audio_diagnostics: Vec<String>,
        current_trace: NativeTraceFrame,
        audio_channels: u8,
        audio_sample_rate: u32,
    ) -> Result<()> {
        if !self.save_worthy {
            return Ok(());
        }
        self.flush(
            manifest,
            options,
            audio_diagnostics,
            current_trace,
            audio_channels,
            audio_sample_rate,
        )
    }

    fn flush(
        self,
        manifest: &AssetManifest,
        options: &NativePlayOptions,
        audio_diagnostics: Vec<String>,
        current_trace: NativeTraceFrame,
        audio_channels: u8,
        audio_sample_rate: u32,
    ) -> Result<()> {
        let mut recorded_script = self.script;
        let mut recorded_trace = self.trace;
        if recorded_trace.is_empty() {
            recorded_script.push(0);
            recorded_trace.push(current_trace);
        }
        if let Some(route) = &options.scripted_route {
            recorded_script = route
                .script
                .iter()
                .copied()
                .take(recorded_trace.len())
                .collect();
        }
        let recorded_audio_events = if options.scripted_route.is_some() {
            native_audio_events_for_trace(manifest, &recorded_trace)
        } else {
            self.audio_events
        };

        let slug = &self.slug;
        let mut recording =
            build_recording(manifest, slug, self.start, recorded_script, recorded_trace)?;
        recording.manifest.record_command = native_recording_command(slug, self.start, options);
        recording.manifest.tas_frontier_command = recording_tas_frontier_command(
            slug,
            options.record_frontier_kind.as_deref().unwrap_or("enemy"),
        );
        recording
            .manifest
            .route_target
            .clone_from(&options.record_route_target);
        if let Some(contact) = self.target_contact_stop.as_ref() {
            append_target_enemy_contact_checkpoint(&mut recording.manifest.scenario, contact);
        }
        if let Some(route) = &options.scripted_route {
            recording.manifest.scenario.start_position = route.start_position;
            recording.manifest.scenario.start_velocity = route.start_velocity;
            recording.manifest.scenario.camera_position = route.camera_position;
            recording.manifest.scenario.playtest_form = route.playtest_form;
            recording.manifest.scenario.initial_diamond_count = route.initial_diamond_count;
            recording.manifest.scenario.initial_lives = route.initial_lives;
        }
        let artifacts = RecordingBundleArtifacts {
            user_notes: options.record_notes.clone(),
            final_frame_rgb: self.final_frame_rgb,
            audio_samples: Some(self.audio),
            audio_sample_rate,
            audio_channels: u16::from(audio_channels),
            audio_events: recorded_audio_events,
            audio_diagnostics,
        };
        save_recording_with_artifacts(&self.record_to, &recording, &artifacts)?;
        println!(
            "recording saved to {}. verify with: cargo run -p wildside-cli -- verify-recording --slug {}",
            self.record_to.display(),
            slug
        );
        for command in native_recording_follow_up_commands(slug, options) {
            println!("{command}");
        }
        Ok(())
    }
}

pub(super) fn recording_frame_limit_reached(
    options: &NativePlayOptions,
    recorded_frames: usize,
) -> bool {
    options
        .record_frames
        .is_some_and(|limit| recorded_frames >= limit)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TargetEnemyContactStop {
    pub(super) frame_index: usize,
    pub(super) object_type: u8,
    pub(super) result: String,
    pub(super) gate: String,
    pub(super) score_px: i64,
}

pub(super) fn update_target_contact_auto_stop(
    options: &NativePlayOptions,
    trace: &NativeTraceFrame,
    replay_frame_index: usize,
    tail_remaining: &mut Option<usize>,
    detected_contact: &mut Option<TargetEnemyContactStop>,
) -> bool {
    if let Some(remaining) = tail_remaining {
        if *remaining == 0 {
            return true;
        }
        *remaining -= 1;
        return false;
    }

    let Some(target) = &options.record_route_target else {
        return false;
    };
    if !options.record_stop_on_target_enemy_contact {
        return false;
    }
    let Some(contact) = target
        .enemies
        .iter()
        .filter_map(|enemy| {
            target_enemy_contact_from_frame(std::slice::from_ref(trace), *enemy, 0)
                .map(|contact| (*enemy, contact))
        })
        .find(|(_, contact)| contact.result != "none")
    else {
        return false;
    };
    let (object_type, contact) = contact;
    detected_contact.get_or_insert_with(|| TargetEnemyContactStop {
        frame_index: replay_frame_index,
        object_type,
        result: contact.result.clone(),
        gate: contact.gate.clone(),
        score_px: contact.score_px,
    });
    let tail = options.record_contact_tail_frames;
    eprintln!(
        "recording target enemy contact detected: enemy=0x{object_type:02X} frame={replay_frame_index} result={} gate={} score_px={} tail_frames={}",
        contact.result, contact.gate, contact.score_px, tail
    );
    if tail == 0 {
        true
    } else {
        *tail_remaining = Some(tail - 1);
        false
    }
}

pub(super) fn append_target_enemy_contact_checkpoint(
    scenario: &mut DynamicScenario,
    contact: &TargetEnemyContactStop,
) {
    if contact.frame_index >= scenario.frame_count {
        return;
    }
    let label = format!(
        "target_enemy_contact_{:02X}_{}",
        contact.object_type, contact.result
    );
    if scenario.checkpoints.iter().any(|checkpoint| {
        checkpoint.label == label && checkpoint.frame_index == contact.frame_index
    }) {
        return;
    }
    scenario.checkpoints.push(DynamicCheckpoint {
        label,
        frame_index: contact.frame_index,
    });
    scenario
        .checkpoints
        .sort_by_key(|checkpoint| checkpoint.frame_index);
}

pub(super) fn native_recording_follow_up_commands(
    slug: &str,
    options: &NativePlayOptions,
) -> Vec<String> {
    let mut commands = vec![recording_tas_frontier_command(
        slug,
        options.record_frontier_kind.as_deref().unwrap_or("enemy"),
    )];
    if let Some(target) = &options.record_route_target {
        commands.extend(recording_route_target_score_commands(slug, target));
    }
    commands
}

fn native_recording_command(
    slug: &str,
    start: RecordingStart,
    options: &NativePlayOptions,
) -> String {
    use std::fmt::Write;

    let mut command = String::from("cargo run --release -p wildside-cli -- play-native");
    if let Some(level) = start.warp_level {
        let _ = write!(command, " --warp-level {level:02X}");
    }
    if start.settled {
        command.push_str(" --settled");
    }
    let _ = write!(command, " --record {slug}");
    if let Some(frame_limit) = start.record_frames {
        let _ = write!(command, " --record-frames {frame_limit}");
    }
    if options.record_stop_on_target_enemy_contact {
        command.push_str(" --record-stop-on-target-enemy-contact");
        if options.record_contact_tail_frames != 30 {
            let _ = write!(
                command,
                " --record-contact-tail-frames {}",
                options.record_contact_tail_frames
            );
        }
    }
    if let Some(target) = &options.record_route_target {
        append_record_route_target_args(&mut command, target);
    }
    if let Some(frontier_kind) = &options.record_frontier_kind {
        let _ = write!(command, " --record-frontier-kind {frontier_kind}");
    }
    command
}

fn append_record_route_target_args(command: &mut String, target: &RecordingRouteTarget) {
    use std::fmt::Write;

    if let Some(label) = &target.label {
        let _ = write!(command, " --record-target-label {label}");
    }
    if let Some(level) = target.level {
        let _ = write!(command, " --record-target-level {level:02X}");
    }
    for point in &target.points {
        let _ = write!(
            command,
            " --record-target-point {},{}",
            point.x_px, point.y_px
        );
    }
    for cell in &target.cells {
        let _ = write!(command, " --record-target-cell {},{}", cell.col, cell.row);
    }
    for cell in &target.cell_hitboxes {
        let _ = write!(
            command,
            " --record-target-cell-hitbox {},{}",
            cell.col, cell.row
        );
    }
    for enemy in &target.enemies {
        let _ = write!(command, " --record-target-enemy {enemy:02X}");
    }
}
