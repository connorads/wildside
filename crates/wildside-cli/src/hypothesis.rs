use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Parsed hypothesis TOML file.
#[derive(Debug, Deserialize)]
pub struct HypothesisFile {
    pub hypothesis: HypothesisHeader,
    pub oracle: Option<OracleSection>,
    pub assertions: Option<HashMap<String, String>>,
    pub comparison: Option<ComparisonSection>,
}

#[derive(Debug, Deserialize)]
pub struct HypothesisHeader {
    pub name: String,
    pub description: String,
    pub scenario: String,
    pub frame_range: [usize; 2],
}

#[derive(Debug, Deserialize)]
pub struct OracleSection {
    pub watches: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ComparisonSection {
    pub native_field: String,
    pub oracle_address: String,
    pub frame: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct HypothesisResult {
    pub name: String,
    pub result: HypothesisVerdict,
    pub evidence: HashMap<String, String>,
    pub timestamp: String,
    pub commit: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum HypothesisVerdict {
    Confirmed,
    Rejected,
    Inconclusive,
}

impl std::fmt::Display for HypothesisVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Confirmed => write!(f, "CONFIRMED"),
            Self::Rejected => write!(f, "REJECTED"),
            Self::Inconclusive => write!(f, "INCONCLUSIVE"),
        }
    }
}

/// Run a hypothesis test: load TOML, run oracle, evaluate assertions.
pub fn test_hypothesis(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &wildside_data::AssetManifest,
    hypothesis_path: &Path,
) -> Result<HypothesisResult> {
    let toml_str = std::fs::read_to_string(hypothesis_path)
        .with_context(|| format!("failed to read {}", hypothesis_path.display()))?;
    let hyp: HypothesisFile = toml::from_str(&toml_str).context("invalid hypothesis TOML")?;

    let name = hyp.hypothesis.name.clone();
    let [frame_start, frame_end] = hyp.hypothesis.frame_range;

    // Find scenario file
    let scenario_path = repo_root
        .join(".cache/tas-imports")
        .join(&hyp.hypothesis.scenario)
        .join("scenario.json");
    if !scenario_path.exists() {
        bail!(
            "scenario not found: {} (expected at {})",
            hyp.hypothesis.scenario,
            scenario_path.display()
        );
    }

    println!("hypothesis: {name}");
    println!(
        "  {}",
        hyp.hypothesis.description.lines().next().unwrap_or("")
    );
    println!("  frames: {frame_start}..{frame_end}");

    // Run oracle with watches if specified
    let mut evidence: HashMap<String, String> = HashMap::new();
    let mut oracle_data: HashMap<usize, HashMap<String, i64>> = HashMap::new();

    if let Some(ref oracle_section) = hyp.oracle {
        let watches_str = oracle_section.watches.join(",");

        let raw_json = std::fs::read_to_string(&scenario_path)?;
        let mut scenario: crate::scenarios::DynamicScenario = serde_json::from_str(&raw_json)?;
        scenario.frame_count = frame_end + 1;
        scenario.checkpoints = (frame_start..=frame_end)
            .map(|f| crate::scenarios::DynamicCheckpoint {
                label: format!("f{f}"),
                frame_index: f,
            })
            .collect();

        let run_dir = repo_root.join(".cache/hypothesis").join(&name);
        std::fs::create_dir_all(&run_dir)?;
        let scenario_file = run_dir.join("scenario.json");
        std::fs::write(&scenario_file, serde_json::to_vec_pretty(&scenario)?)?;

        let env = crate::oracle::ensure_oracles(repo_root)?;
        let oracle_dir = run_dir.join("oracle");
        std::fs::create_dir_all(&oracle_dir)?;

        let options = crate::oracle::OracleRunOptions {
            skip_frames_before: Some(frame_start),
            memory_watches: Some(watches_str),
            wram_dump_frames: None,
            vram_dump_frames: None,
            cram_dump_frames: None,
            vsram_dump_frames: None,
            vdp_pixel_samples: None,
            vdp_fifo_commit_frames: None,
            vdp_fifo_commit_address_range: None,
            m68k_pc_trace: None,
            capture_gems_state: false,
            capture_gems_frame_signature: false,
            capture_cpu_state: false,
            capture_settle_trace: false,
            capture_rng_stream: false,
        };
        let oracle_manifest = crate::oracle::run_jgenesis_capture_dynamic_with_options(
            &env,
            rom_path,
            &scenario_file,
            &oracle_dir,
            &options,
        )?;

        // Index oracle watch data by frame
        for watch_map in &oracle_manifest.memory_watches {
            let frame = watch_map.get("frame").copied().unwrap_or(-1) as usize;
            oracle_data.insert(frame, watch_map.clone());
        }
    }

    // Run native for the frame range
    let raw_json = std::fs::read_to_string(&scenario_path)?;
    let scenario: crate::scenarios::DynamicScenario = serde_json::from_str(&raw_json)?;
    let native_trace = wildside_core::run_script_with_trace(
        manifest.clone(),
        &scenario.script,
        frame_end + 1,
        scenario.warp_to_level,
    );

    // Evaluate assertions
    let mut all_pass = true;
    let mut any_evaluated = false;

    if let Some(ref assertions) = hyp.assertions {
        for (assertion_name, expr) in assertions {
            let result = evaluate_assertion(expr, &oracle_data, &native_trace);
            match result {
                Ok(true) => {
                    evidence.insert(assertion_name.clone(), "PASS".into());
                    println!("  {assertion_name}: PASS");
                    any_evaluated = true;
                }
                Ok(false) => {
                    evidence.insert(assertion_name.clone(), "FAIL".into());
                    println!("  {assertion_name}: FAIL");
                    all_pass = false;
                    any_evaluated = true;
                }
                Err(e) => {
                    evidence.insert(assertion_name.clone(), format!("ERROR: {e}"));
                    println!("  {assertion_name}: ERROR ({e})");
                }
            }
        }
    }

    // Evaluate comparison
    if let Some(ref comparison) = hyp.comparison {
        let native_val = native_trace.get(comparison.frame).and_then(|f| {
            match comparison.native_field.as_str() {
                "kid_y_pos" => f.kid_y_pos,
                "kid_x_pos" => f.kid_x_pos,
                _ => None,
            }
        });
        let oracle_val = oracle_data.get(&comparison.frame).and_then(|m| {
            let key = comparison.oracle_address.replace(':', "_");
            m.get(&key).copied()
        });

        if let (Some(n), Some(o)) = (native_val, oracle_val) {
            let matches = n as i64 == o;
            evidence.insert(
                "comparison".into(),
                format!("native={n:#010x} oracle={o:#010x} match={matches}"),
            );
            println!(
                "  comparison at frame {}: native={:#010x} oracle={:#010x} {}",
                comparison.frame,
                n,
                o,
                if matches { "MATCH" } else { "DIVERGE" }
            );
            if !matches {
                all_pass = false;
            }
            any_evaluated = true;
        }
    }

    let verdict = if !any_evaluated {
        HypothesisVerdict::Inconclusive
    } else if all_pass {
        HypothesisVerdict::Confirmed
    } else {
        HypothesisVerdict::Rejected
    };

    println!("  result: {verdict}");

    // Get git commit
    let commit = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
        .trim()
        .to_string();

    let result = HypothesisResult {
        name,
        result: verdict,
        evidence,
        timestamp: {
            let d = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            format!("{}s", d.as_secs())
        },
        commit,
    };

    // Append to log
    let log_path = repo_root.join("hypothesis_log.jsonl");
    let mut log_line = serde_json::to_string(&result)?;
    log_line.push('\n');
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?
        .write_all(log_line.as_bytes())?;
    println!("  logged to {}", log_path.display());

    Ok(result)
}

/// Simple assertion evaluator for expressions like "0xFF2500_u16[955] != 0".
fn evaluate_assertion(
    expr: &str,
    oracle_data: &HashMap<usize, HashMap<String, i64>>,
    _native_trace: &[wildside_core::NativeTraceFrame],
) -> Result<bool> {
    // Parse: "oracle.KEY[FRAME] OP VALUE" or "KEY[FRAME] OP VALUE"
    let expr = expr.trim();

    // Find operator
    let ops: &[(&str, fn(i64, i64) -> bool)] = &[
        ("!=", |a, b| a != b),
        (">=", |a, b| a >= b),
        ("<=", |a, b| a <= b),
        ("=", |a, b| a == b),
        (">", |a, b| a > b),
        ("<", |a, b| a < b),
    ];

    for (op_str, op_fn) in ops {
        if let Some((left, right)) = expr.split_once(op_str) {
            let left = left.trim();
            let right = right.trim();

            // Parse left: "oracle.KEY[FRAME]" or "KEY[FRAME]"
            let left = left.strip_prefix("oracle.").unwrap_or(left);
            let (key, frame) = parse_indexed_ref(left)?;

            let actual = oracle_data
                .get(&frame)
                .and_then(|m| m.get(&key).copied())
                .unwrap_or(0);

            let expected = parse_int_value(right)?;
            return Ok(op_fn(actual, expected));
        }
    }

    bail!("could not parse assertion: {expr}")
}

/// Parse "KEY[FRAME]" → (key, frame_index)
fn parse_indexed_ref(s: &str) -> Result<(String, usize)> {
    let (key, rest) = s.split_once('[').context("expected KEY[FRAME] format")?;
    let frame_str = rest
        .strip_suffix(']')
        .context("expected KEY[FRAME] format")?;
    let frame: usize = frame_str.parse().context("invalid frame index")?;
    Ok((key.to_string(), frame))
}

fn parse_int_value(s: &str) -> Result<i64> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        Ok(i64::from_str_radix(hex, 16)?)
    } else {
        Ok(s.parse::<i64>()?)
    }
}

use std::io::Write;

#[cfg(test)]
mod tests {
    use super::*;

    fn oracle_data(entries: &[(usize, &str, i64)]) -> HashMap<usize, HashMap<String, i64>> {
        let mut oracle_data = HashMap::new();
        for &(frame, key, value) in entries {
            oracle_data
                .entry(frame)
                .or_insert_with(HashMap::new)
                .insert(key.to_string(), value);
        }
        oracle_data
    }

    fn evaluate(expr: &str, oracle_data: &HashMap<usize, HashMap<String, i64>>) -> Result<bool> {
        evaluate_assertion(expr, oracle_data, &[])
    }

    #[test]
    fn assertion_evaluator_supports_each_comparison_operator() {
        let data = oracle_data(&[(12, "0xFF2500_u16", 5)]);
        let cases = [
            ("oracle.0xFF2500_u16[12] != 0", true),
            ("oracle.0xFF2500_u16[12] != 5", false),
            ("oracle.0xFF2500_u16[12] >= 5", true),
            ("oracle.0xFF2500_u16[12] >= 6", false),
            ("oracle.0xFF2500_u16[12] <= 5", true),
            ("oracle.0xFF2500_u16[12] <= 4", false),
            ("oracle.0xFF2500_u16[12] = 5", true),
            ("oracle.0xFF2500_u16[12] = 4", false),
            ("oracle.0xFF2500_u16[12] > 4", true),
            ("oracle.0xFF2500_u16[12] > 5", false),
            ("oracle.0xFF2500_u16[12] < 6", true),
            ("oracle.0xFF2500_u16[12] < 5", false),
        ];

        for (expr, expected) in cases {
            assert_eq!(evaluate(expr, &data).unwrap(), expected, "{expr}");
        }
    }

    #[test]
    fn assertion_evaluator_accepts_unprefixed_keys_and_defaults_missing_watches_to_zero() {
        let data = oracle_data(&[(3, "watch", 9)]);

        assert!(evaluate("watch[3] = 9", &data).unwrap());
        assert!(evaluate("missing[3] = 0", &data).unwrap());
    }

    #[test]
    fn assertion_evaluator_reports_unparsable_expressions() {
        let data = oracle_data(&[(3, "watch", 9)]);
        let error = evaluate("watch[3] approximately 9", &data).unwrap_err();

        assert!(error.to_string().contains("could not parse assertion"));
    }

    #[test]
    fn parse_indexed_ref_returns_key_and_frame() {
        assert_eq!(
            parse_indexed_ref("0xFF2500_u16[955]").unwrap(),
            ("0xFF2500_u16".to_string(), 955)
        );
    }

    #[test]
    fn parse_indexed_ref_rejects_missing_brackets_and_invalid_frames() {
        let missing_open = parse_indexed_ref("0xFF2500_u16").unwrap_err();
        assert!(missing_open.to_string().contains("expected KEY[FRAME]"));

        let missing_close = parse_indexed_ref("0xFF2500_u16[955").unwrap_err();
        assert!(missing_close.to_string().contains("expected KEY[FRAME]"));

        let invalid_frame = parse_indexed_ref("0xFF2500_u16[later]").unwrap_err();
        assert!(invalid_frame.to_string().contains("invalid frame index"));
    }

    #[test]
    fn parse_int_value_accepts_decimal_and_hex_literals() {
        assert_eq!(parse_int_value("42").unwrap(), 42);
        assert_eq!(parse_int_value("-7").unwrap(), -7);
        assert_eq!(parse_int_value("0x2A").unwrap(), 42);
        assert_eq!(parse_int_value("0X2a").unwrap(), 42);
    }

    #[test]
    fn parse_int_value_rejects_invalid_literals() {
        assert!(parse_int_value("forty-two").is_err());
        assert!(parse_int_value("0xnothex").is_err());
    }
}
