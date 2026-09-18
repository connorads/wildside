use super::*;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const NANODOLLARS_PER_TOKEN: u64 = 42;
const REQUEST_RESERVATION: u64 = 64_000 * NANODOLLARS_PER_TOKEN;
const RUN_LIMIT: u64 = 1_000_000_000;

#[derive(Default, serde::Serialize)]
struct Budget {
    nanodollars_upper_bound: u64,
    requests: usize,
}

impl Budget {
    fn reserve(&mut self) -> Result<()> {
        ensure!(
            self.nanodollars_upper_bound + REQUEST_RESERVATION <= RUN_LIMIT,
            "API budget exhausted"
        );
        self.nanodollars_upper_bound += REQUEST_RESERVATION;
        self.requests += 1;
        Ok(())
    }

    fn reconcile(&mut self, response: &Value) -> Result<()> {
        let tokens = response["usage"]["input_tokens"]
            .as_u64()
            .context("missing input usage")?;
        ensure!(tokens <= 64_000, "usage exceeds reserved model context");
        self.nanodollars_upper_bound -= REQUEST_RESERVATION - tokens * NANODOLLARS_PER_TOKEN;
        Ok(())
    }
}

fn action(response: &Value) -> Result<u8> {
    ensure!(response["model"] == "jev-1.13.0", "unexpected model");
    ensure!(
        response["answers"]["action"]["type"] == "choice",
        "unexpected answer type"
    );
    match response["answers"]["action"]["choice"].as_str() {
        Some("release") => Ok(0),
        Some("left") => Ok(4),
        Some("right") => Ok(8),
        Some("jump") => Ok(16),
        Some("left_jump") => Ok(20),
        Some("right_jump") => Ok(24),
        _ => bail!("unknown controller choice"),
    }
}

fn request(state: &Value) -> Result<Value> {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/jev_request.py");
    let mut child = Command::new("python3")
        .arg(script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Err(error) = (|| -> Result<()> {
        let mut input = child.stdin.take().context("helper stdin")?;
        serde_json::to_writer(&mut input, state)?;
        Ok(())
    })() {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    let output = child.wait_with_output()?;
    ensure!(
        output.status.success(),
        "helper exited {}: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).context("invalid Jev response JSON")
}

pub(super) fn run() -> Result<()> {
    ensure!(
        std::env::var("TYPESAFE_API_KEY").is_ok_and(|key| !key.is_empty()),
        "TYPESAFE_API_KEY is required"
    );
    let (output, manifest) = prepare()?;
    let started = Instant::now();
    let prior = std::env::var("JEV_PRIOR_NANODOLLARS")
        .unwrap_or_else(|_| "0".into())
        .parse()?;
    let attempts: usize = std::env::var("JEV_ATTEMPTS")
        .unwrap_or_else(|_| "3".into())
        .parse()?;
    ensure!(
        prior <= RUN_LIMIT && (1..=3).contains(&attempts),
        "invalid remaining pilot budget"
    );
    let mut budget = Budget {
        nanodollars_upper_bound: prior,
        requests: 0,
    };
    for attempt in 1..=attempts {
        let mut episode = Episode::new(&manifest, (8, 12))?;
        let mut journal = fs::File::create(output.join(format!("attempt-{attempt}-inputs.bin")))?;
        let mut decisions =
            fs::File::create(output.join(format!("attempt-{attempt}-decisions.jsonl")))?;
        let mut reason = String::from("frame_limit");
        let mut fatal = false;
        for _ in 0..600 {
            if started.elapsed() >= Duration::from_secs(1200) {
                reason = "collection_deadline".into();
                fatal = true;
                break;
            }
            if let Err(error) = budget.reserve() {
                reason = error.to_string();
                fatal = true;
                break;
            }
            fs::write(
                output.join("budget.json"),
                serde_json::to_vec_pretty(&budget)?,
            )?;
            let state = episode.observation();
            let call_started = Instant::now();
            let response = request(&state);
            let entry = match &response {
                Ok(value) => {
                    json!({"state": state, "response": value, "latency_ms": call_started.elapsed().as_millis()})
                }
                Err(error) => {
                    json!({"state": state, "error": error.to_string(), "latency_ms": call_started.elapsed().as_millis()})
                }
            };
            serde_json::to_writer(&mut decisions, &entry)?;
            writeln!(decisions)?;
            decisions.flush()?;
            let chosen = response.and_then(|response| {
                budget.reconcile(&response)?;
                action(&response)
            });
            fs::write(
                output.join("budget.json"),
                serde_json::to_vec_pretty(&budget)?,
            )?;
            let mask = match chosen {
                Ok(mask) => mask,
                Err(error) => {
                    reason = error.to_string();
                    fatal = true;
                    break;
                }
            };
            let first = episode.script.len();
            episode.advance(mask);
            journal.write_all(&episode.script[first..])?;
            journal.flush()?;
            println!(
                "attempt {attempt} frame {} action {mask:#04x} kid {} hit {}",
                episode.script.len(),
                episode.observation()["kid"],
                episode.encounter.first.is_some()
            );
            if episode.encounter.complete(episode.script.len() - 1) {
                reason = "animation_window_complete".into();
                break;
            }
            if episode.ended() {
                reason = "death_or_gameplay_exit".into();
                break;
            }
            if episode.script.len() >= MAX_FRAMES {
                break;
            }
        }
        let exercised = episode.encounter.first.is_some();
        fs::write(
            output.join(format!("attempt-{attempt}-outcome.json")),
            serde_json::to_vec_pretty(&json!({
                "reason": reason, "target_exercised": exercised, "fatal": fatal
            }))?,
        )?;
        if !episode.script.is_empty() {
            episode.save(
                &manifest,
                &output.join(format!("attempt-{attempt}")),
                &reason,
            )?;
        }
        if fatal || exercised {
            break;
        }
    }
    println!(
        "API cost upper bound: ${:.6}",
        budget.nanodollars_upper_bound as f64 / 1e9
    );
    Ok(())
}

#[test]
fn gameplay_regression_jev_budget_reserves_timeouts_and_rejects_overspend() {
    let mut budget = Budget {
        nanodollars_upper_bound: RUN_LIMIT - REQUEST_RESERVATION,
        requests: 0,
    };
    budget.reserve().unwrap();
    assert_eq!(budget.nanodollars_upper_bound, RUN_LIMIT);
    assert!(budget.reserve().is_err());
    assert_eq!(budget.requests, 1);
    assert!(budget.reconcile(&json!({})).is_err());
    assert_eq!(budget.nanodollars_upper_bound, RUN_LIMIT);
    budget
        .reconcile(&json!({"usage": {"input_tokens": 100}}))
        .unwrap();
    assert_eq!(
        budget.nanodollars_upper_bound,
        RUN_LIMIT - REQUEST_RESERVATION + 4_200
    );
}

#[test]
fn gameplay_regression_jev_rejects_unknown_actions_without_applying_input() {
    let mut response = json!({"model": "jev-1.13.0", "answers": {"action": {"type": "choice", "choice": "right_jump"}}});
    assert_eq!(action(&response).unwrap(), 24);
    response["answers"]["action"]["choice"] = json!("teleport");
    assert!(action(&response).is_err());
}
