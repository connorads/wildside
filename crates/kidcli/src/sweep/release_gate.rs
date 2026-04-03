use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct ReleaseGateFocusSummary {
    pub(super) target: String,
    pub(super) step_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) checkpoint: Option<String>,
    pub(super) field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) native_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) oracle_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) scenario_level_id: Option<String>,
    pub(super) matched_in_state_sweep: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) scenario_level_matched_in_state_sweep: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) checkpoint_matched_in_state_sweep: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) exact_value_pair_matched_in_state_sweep: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) total_diffs: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) levels: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) level_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) matched_checkpoints: Vec<SweepHistoryCheckpointCount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) exact_value_pair_checkpoints: Vec<SweepHistoryCheckpointCount>,
    #[serde(default, skip_serializing_if = "DivergenceSummary::is_empty")]
    pub(super) divergence_summary: DivergenceSummary,
    pub(super) summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ReleaseGateFieldFocus {
    pub(super) target: String,
    pub(super) step_name: String,
    pub(super) checkpoint: Option<String>,
    pub(super) field: String,
    pub(super) native_value: Option<String>,
    pub(super) oracle_value: Option<String>,
    pub(super) scenario_level_id: Option<String>,
}

fn compact_release_gate_step_name(step_name: &str) -> String {
    step_name
        .split_once(':')
        .map(|(_, tail)| tail)
        .unwrap_or(step_name)
        .to_owned()
}

pub(super) fn extract_release_gate_failed_field(detail: &str) -> Option<String> {
    let (_, tail) = detail.split_once(" diverged in ")?;
    let raw_field = tail
        .split(['(', ';'])
        .next()
        .map(str::trim)
        .filter(|field| !field.is_empty())?;
    let mut field = String::new();
    let mut previous_was_separator = false;
    for ch in raw_field.chars() {
        if ch.is_ascii_alphanumeric() {
            field.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator {
            field.push('_');
            previous_was_separator = true;
        }
    }
    let field = field.trim_matches('_').to_owned();
    if field.is_empty() { None } else { Some(field) }
}

pub(super) fn extract_release_gate_failed_checkpoint(detail: &str) -> Option<String> {
    let (_, tail) = detail.split_once("state ")?;
    let checkpoint = tail
        .split_once(" diverged in ")
        .map(|(checkpoint, _)| checkpoint.trim())
        .filter(|checkpoint| !checkpoint.is_empty())?;
    Some(checkpoint.to_owned())
}

pub(super) fn normalise_release_gate_value(raw: &str) -> String {
    let trimmed = raw.trim();
    let hex = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    if !hex.is_empty() && hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        format!("0x{}", hex.to_ascii_lowercase())
    } else {
        trimmed.to_ascii_lowercase()
    }
}

pub(super) fn extract_release_gate_failed_values(detail: &str) -> Option<(String, String)> {
    let (_, tail) = detail.split_once("(native ")?;
    let (native_raw, tail) = tail.split_once(", oracle ")?;
    let (oracle_raw, _) = tail.split_once(')')?;
    Some((
        normalise_release_gate_value(native_raw),
        normalise_release_gate_value(oracle_raw),
    ))
}

pub(super) fn decimal_level_index_to_bcd(level_index: usize) -> Option<u8> {
    if level_index > 68 {
        return None;
    }
    let tens = level_index / 10;
    let ones = level_index % 10;
    Some(((tens as u8) << 4) | ones as u8)
}

pub(super) fn extract_release_gate_scenario_level_id(step_name: &str) -> Option<String> {
    let scenario = compact_release_gate_step_name(step_name);
    let marker = "level";
    let start = scenario.find(marker)? + marker.len();
    let digits = scenario[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    let human_level = digits.parse::<usize>().ok()?;
    let level_index = human_level.checked_sub(1)?;
    let bcd = decimal_level_index_to_bcd(level_index)?;
    if !is_valid_level_id(bcd) {
        return None;
    }
    Some(format!("{:#04x}", bcd))
}

pub(super) fn latest_release_gate_failed_fields(repo_root: &Path) -> Vec<ReleaseGateFieldFocus> {
    ["1p-playable", "1p-accurate"]
        .into_iter()
        .filter_map(|target| {
            let latest_path = repo_root
                .join(".cache/release-gate")
                .join(target)
                .join("latest.json");
            let contents = fs::read_to_string(&latest_path).ok()?;
            let report = serde_json::from_str::<serde_json::Value>(&contents).ok()?;
            if report
                .get("passed")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
            {
                return None;
            }
            let failed_step = report
                .get("steps")
                .and_then(serde_json::Value::as_array)
                .and_then(|steps| {
                    steps.iter().find(|step| {
                        step.get("status")
                            .and_then(serde_json::Value::as_str)
                            .is_some_and(|status| status == "failed")
                    })
                })?;
            let step_name = failed_step.get("name")?.as_str()?.to_owned();
            let detail = failed_step.get("detail")?.as_str()?;
            let field = extract_release_gate_failed_field(detail)?;
            let (native_value, oracle_value) = extract_release_gate_failed_values(detail)
                .map_or((None, None), |(native, oracle)| {
                    (Some(native), Some(oracle))
                });
            let scenario_level_id = extract_release_gate_scenario_level_id(&step_name);
            Some(ReleaseGateFieldFocus {
                target: target.to_owned(),
                step_name,
                checkpoint: extract_release_gate_failed_checkpoint(detail),
                field,
                native_value,
                oracle_value,
                scenario_level_id,
            })
        })
        .collect()
}

pub(super) fn format_release_gate_focus_summary(
    field_summary: &[SweepHistoryFieldEntry],
    focus: &ReleaseGateFieldFocus,
) -> String {
    match field_summary
        .iter()
        .find(|entry| entry.field == focus.field)
    {
        Some(entry) => {
            let scenario_level_note = focus
                .scenario_level_id
                .as_deref()
                .map(|level_id| {
                    if entry
                        .level_ids
                        .iter()
                        .any(|candidate| candidate == level_id)
                    {
                        format!(" [includes scenario level {level_id}]")
                    } else {
                        format!(" [excludes scenario level {level_id}]")
                    }
                })
                .unwrap_or_default();
            let checkpoint_note = focus
                .checkpoint
                .as_deref()
                .map(|checkpoint| {
                    if entry
                        .checkpoints
                        .iter()
                        .any(|candidate| candidate == checkpoint)
                    {
                        format!(" [includes checkpoint {checkpoint}]")
                    } else {
                        format!(" [excludes checkpoint {checkpoint}]")
                    }
                })
                .unwrap_or_default();
            let checkpoint_preview = format_checkpoint_count_preview(&entry.checkpoint_counts);

            format!(
                "{} failing field {}={} diffs across {} levels for {}{}{}{}{}{}{} (field-level coverage only; exact pair evidence lives in release_gate_focus, and the release-gate scenario still needs its own proof)",
                focus.target,
                entry.field,
                entry.total_diffs,
                entry.levels,
                compact_release_gate_step_name(&focus.step_name),
                focus
                    .checkpoint
                    .as_deref()
                    .map(|checkpoint| format!(" @ {checkpoint}"))
                    .unwrap_or_default(),
                format_field_category_summary(&entry.divergence_summary),
                format_field_level_preview(&entry.level_ids),
                scenario_level_note,
                checkpoint_note,
                checkpoint_preview
            )
        }
        None => format!(
            "{} failing field {} for {}{} not present in latest state sweep field_summary",
            focus.target,
            focus.field,
            compact_release_gate_step_name(&focus.step_name),
            focus
                .checkpoint
                .as_deref()
                .map(|checkpoint| format!(" @ {checkpoint}"))
                .unwrap_or_default()
        ),
    }
}

pub(super) fn summarise_release_gate_focuses(
    results: &[LevelSweepResult],
    field_summary: &[SweepHistoryFieldEntry],
    focuses: &[ReleaseGateFieldFocus],
) -> Vec<ReleaseGateFocusSummary> {
    focuses
        .iter()
        .map(|focus| {
            let matched = field_summary
                .iter()
                .find(|entry| entry.field == focus.field);
            let exact_value_pair_checkpoints = match (&focus.native_value, &focus.oracle_value) {
                (Some(native_value), Some(oracle_value)) => {
                    let mut counts: std::collections::BTreeMap<String, usize> =
                        std::collections::BTreeMap::new();
                    for result in results {
                        for diff in &result.state_diffs {
                            if diff.field == focus.field
                                && normalise_release_gate_value(&diff.native_value) == *native_value
                                && normalise_release_gate_value(&diff.oracle_value) == *oracle_value
                            {
                                *counts.entry(diff.checkpoint.clone()).or_default() += 1;
                            }
                        }
                    }
                    let mut entries = counts
                        .into_iter()
                        .map(|(checkpoint, diffs)| SweepHistoryCheckpointCount {
                            checkpoint,
                            diffs,
                        })
                        .collect::<Vec<_>>();
                    entries.sort_by(|left, right| {
                        right
                            .diffs
                            .cmp(&left.diffs)
                            .then_with(|| left.checkpoint.cmp(&right.checkpoint))
                    });
                    entries
                }
                _ => Vec::new(),
            };
            let scenario_level_matched_in_state_sweep =
                focus.scenario_level_id.as_ref().map(|level_id| {
                    matched.is_some_and(|entry| {
                        entry
                            .level_ids
                            .iter()
                            .any(|candidate| candidate == level_id)
                    })
                });
            let checkpoint_matched_in_state_sweep = focus.checkpoint.as_ref().map(|checkpoint| {
                matched.is_some_and(|entry| {
                    entry
                        .checkpoints
                        .iter()
                        .any(|candidate| candidate == checkpoint)
                })
            });
            let exact_value_pair_matched_in_state_sweep =
                if focus.native_value.is_some() && focus.oracle_value.is_some() {
                    Some(!exact_value_pair_checkpoints.is_empty())
                } else {
                    None
                };
            let exact_pair_note = match (&focus.native_value, &focus.oracle_value) {
                (Some(native_value), Some(oracle_value)) => {
                    if exact_value_pair_checkpoints.is_empty() {
                        format!(
                            " [exact pair {native_value}->{oracle_value} absent from latest state sweep; same field class, different concrete miss]"
                        )
                    } else {
                        format!(
                            " [exact pair {native_value}->{oracle_value} seen{}]",
                            format_checkpoint_count_preview(&exact_value_pair_checkpoints)
                        )
                    }
                }
                _ => String::new(),
            };
            let summary = format!(
                "{}{}",
                format_release_gate_focus_summary(field_summary, focus),
                exact_pair_note
            );
            ReleaseGateFocusSummary {
                target: focus.target.clone(),
                step_name: focus.step_name.clone(),
                checkpoint: focus.checkpoint.clone(),
                field: focus.field.clone(),
                native_value: focus.native_value.clone(),
                oracle_value: focus.oracle_value.clone(),
                scenario_level_id: focus.scenario_level_id.clone(),
                matched_in_state_sweep: matched.is_some(),
                scenario_level_matched_in_state_sweep,
                checkpoint_matched_in_state_sweep,
                exact_value_pair_matched_in_state_sweep,
                total_diffs: matched.map(|entry| entry.total_diffs),
                levels: matched.map(|entry| entry.levels),
                level_ids: matched
                    .map(|entry| entry.level_ids.clone())
                    .unwrap_or_default(),
                matched_checkpoints: matched
                    .map(|entry| entry.checkpoint_counts.clone())
                    .unwrap_or_default(),
                exact_value_pair_checkpoints,
                divergence_summary: matched
                    .map(|entry| entry.divergence_summary.clone())
                    .unwrap_or_default(),
                summary,
            }
        })
        .collect()
}
