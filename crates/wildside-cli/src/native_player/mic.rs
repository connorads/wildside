//! Session-long microphone capture for playtest narration.
//!
//! Spawns a configurable recorder subprocess (ffmpeg by default) writing a
//! whisper-friendly mono 16 kHz WAV next to `session.json`. The child is
//! SIGTERMed on normal exit so the WAV header finalises, and hard-killed on
//! drop so a player crash never orphans a recording process;
//! `playtest-ingest` tolerates a truncated WAV either way.

use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Mono 16 kHz signed 16-bit: what whisper.cpp/mlx-whisper resample to anyway.
pub(crate) const DEFAULT_MIC_COMMAND: &str = "ffmpeg -hide_banner -loglevel error \
     -f avfoundation -i :0 -ac 1 -ar 16000 -sample_fmt s16 -y {wav}";

pub(crate) struct MicCapture {
    child: Child,
    command_line: String,
}

impl MicCapture {
    /// Spawn the capture command with `{wav}` substituted (or the path
    /// appended when the template has no placeholder). Whitespace-split, no
    /// shell quoting: capture commands are simple flag lists.
    pub(crate) fn spawn(template: &str, wav_path: &Path) -> Result<Self> {
        let argv = mic_command_argv(template, wav_path);
        let Some((program, args)) = argv.split_first() else {
            bail!("mic capture command is empty");
        };
        let child = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .spawn()
            .with_context(|| format!("failed to spawn mic capture: {}", argv.join(" ")))?;
        Ok(Self {
            child,
            command_line: argv.join(" "),
        })
    }

    pub(crate) fn command_line(&self) -> &str {
        &self.command_line
    }

    /// Graceful stop: SIGTERM (ffmpeg finalises the WAV header on it), then a
    /// short wait, then the Drop hard-kill as backstop.
    pub(crate) fn stop(mut self) {
        let _ = Command::new("kill")
            .args(["-TERM", &self.child.id().to_string()])
            .status();
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            match self.child.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) => std::thread::sleep(Duration::from_millis(50)),
                Err(_) => break,
            }
        }
        // Drop takes over: hard kill.
    }
}

impl Drop for MicCapture {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub(crate) fn mic_command_argv(template: &str, wav_path: &Path) -> Vec<String> {
    let wav = wav_path.display().to_string();
    let mut argv: Vec<String> = template
        .split_whitespace()
        .map(|part| part.replace("{wav}", &wav))
        .collect();
    if !template.contains("{wav}") {
        argv.push(wav);
    }
    argv
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn command_builder_substitutes_wav_placeholder() {
        let argv = mic_command_argv(DEFAULT_MIC_COMMAND, &PathBuf::from("/tmp/s/mic.wav"));
        assert_eq!(argv[0], "ffmpeg");
        assert_eq!(argv.last().map(String::as_str), Some("/tmp/s/mic.wav"));
        assert!(
            !argv.iter().any(|part| part.contains("{wav}")),
            "placeholder must be substituted: {argv:?}"
        );
    }

    #[test]
    fn command_builder_appends_wav_when_template_has_no_placeholder() {
        let argv = mic_command_argv("rec -q", &PathBuf::from("mic.wav"));
        assert_eq!(argv, vec!["rec", "-q", "mic.wav"]);
    }
}
