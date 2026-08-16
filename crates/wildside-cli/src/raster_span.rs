//! Replay bounded HUD raster spans from the census' measured CPU timeline
//! (#409). A measurement tool per the #437 `gems_span` precedent — never a
//! CI gate. It tests whether the captured cumulative 68000 cycle span
//! reproduces the captured H40 beam and classifies the asynchronous wait
//! surface a native frame-local scheduler must own.
//!
//! Units: every `pixel` field in the census is an H40 **pixel index**
//! (0..419). A pixel index is an mclk *interval* (8/9/10 mclk wide under the
//! H40 divider), never an mclk offset into the 3,420-mclk scanline. The
//! interval bounds come from the engine's own divider
//! (`wildside_core::vdp::h40_pixel_mclk_bounds`) rather than a re-derivation
//! — re-deriving these constants in Python is exactly how the original
//! script acquired its units bug. `cpu_span_exact` is intersection of the
//! advanced entry interval with the observed interval; `residual_mclk` is
//! the signed gap when it misses (positive = replay stopped short of the
//! observed beam, negative = replay overshot it).

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::Path;
use wildside_core::vdp::h40_pixel_mclk_bounds;

const H40_MCLK_PER_SCANLINE: i64 = 3_420;
const H40_PIXELS_PER_SCANLINE: i64 = 420;
const NTSC_SCANLINES: i64 = 262;
const FRAME_MCLK: i64 = H40_MCLK_PER_SCANLINE * NTSC_SCANLINES;
const MCLK_PER_M68K_CYCLE: i64 = 7;
const DEFAULT_CLUSTERS: [&str; 2] = ["hud_diamond", "hud_life_icon"];

#[derive(Debug, Deserialize)]
struct CensusFile {
    clusters: Vec<CensusCluster>,
}

#[derive(Debug, Deserialize)]
struct CensusCluster {
    cluster: String,
    frames: Vec<CensusFrame>,
}

#[derive(Debug, Deserialize)]
struct CensusFrame {
    frame: u64,
    changed_pixels: u64,
    mode_entry: Position,
    mode_to_update_hud: CensusInterval,
    update_hud: Position,
    update_to_draw_hud: CensusInterval,
    first_fifo_commit: Position,
}

#[derive(Debug, Deserialize)]
struct CensusInterval {
    m68k_cycles: i64,
    z80_wait_cycles: i64,
}

/// Field order matches the Python report writer this port replaced;
/// the differential acceptance test compared bytes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
struct Position {
    scanline: i64,
    pixel: i64,
}

#[derive(Debug, Serialize)]
struct ReplayReport {
    schema_version: u32,
    pixel_units: &'static str,
    measurement_only: bool,
    clusters: Vec<ReplayCluster>,
    frame_count: usize,
    exact_cpu_span_count: usize,
    all_cpu_spans_exact: bool,
    witnesses: Vec<Witness>,
}

#[derive(Debug, Serialize)]
struct ReplayCluster {
    cluster: String,
    frames: Vec<ReplayRow>,
}

#[derive(Debug, Serialize)]
struct ReplayRow {
    frame: u64,
    wait_class: &'static str,
    observed_update_hud: Position,
    first_fifo_commit: Position,
    fifo_latency_mclk_from_update_hud: i64,
    replayed_update_hud: Position,
    cpu_span_exact: bool,
    residual_mclk: i64,
    entry_quantisation_mclk: i64,
}

#[derive(Debug, Serialize)]
struct Witness {
    kind: &'static str,
    frame: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    changed_pixels: Option<u64>,
}

/// Frame-absolute mclk interval a captured (scanline, pixel index) spans.
fn beam_mclk_interval(position: Position) -> Result<(i64, i64)> {
    let pixel = u16::try_from(position.pixel)
        .ok()
        .and_then(h40_pixel_mclk_bounds);
    let Some((lo, hi)) = pixel else {
        bail!(
            "pixel index out of range 0..{}: {}",
            H40_PIXELS_PER_SCANLINE - 1,
            position.pixel
        );
    };
    let base = position.scanline * H40_MCLK_PER_SCANLINE;
    Ok((base + i64::from(lo), base + i64::from(hi)))
}

fn mclk_to_position(frame_mclk: i64) -> Position {
    let wrapped = frame_mclk.rem_euclid(FRAME_MCLK);
    let scanline_mclk = wrapped % H40_MCLK_PER_SCANLINE;
    let pixel = (0..H40_PIXELS_PER_SCANLINE)
        .find(|&pixel| {
            h40_pixel_mclk_bounds(pixel as u16)
                .is_some_and(|(lo, hi)| (i64::from(lo)..=i64::from(hi)).contains(&scanline_mclk))
        })
        .expect("every scanline mclk maps to an H40 pixel index");
    Position {
        scanline: wrapped / H40_MCLK_PER_SCANLINE,
        pixel,
    }
}

struct SpanReplay {
    replayed_update_hud: Position,
    cpu_span_exact: bool,
    residual_mclk: i64,
    entry_quantisation_mclk: i64,
}

fn replay_cpu_span(entry: Position, observed: Position, m68k_cycles: i64) -> Result<SpanReplay> {
    let (entry_lo, entry_hi) = beam_mclk_interval(entry)?;
    let advance = m68k_cycles * MCLK_PER_M68K_CYCLE;
    let (advanced_lo, advanced_hi) = (entry_lo + advance, entry_hi + advance);

    let (mut observed_lo, mut observed_hi) = beam_mclk_interval(observed)?;
    // The observed beam recurs every frame; compare against the occurrence
    // nearest the advanced interval so multi-frame spans replay correctly.
    // round_ties_even matches Python round() in the retired script.
    let frames_ahead = (((advanced_lo + advanced_hi) - (observed_lo + observed_hi)) as f64
        / (2 * FRAME_MCLK) as f64)
        .round_ties_even() as i64;
    observed_lo += frames_ahead * FRAME_MCLK;
    observed_hi += frames_ahead * FRAME_MCLK;

    let residual_mclk = if advanced_hi < observed_lo {
        observed_lo - advanced_hi
    } else if advanced_lo > observed_hi {
        observed_hi - advanced_lo
    } else {
        0
    };
    Ok(SpanReplay {
        replayed_update_hud: mclk_to_position(advanced_hi),
        cpu_span_exact: residual_mclk == 0,
        residual_mclk,
        entry_quantisation_mclk: entry_hi - entry_lo + 1,
    })
}

fn select_witnesses(clusters: &[&CensusCluster]) -> Vec<Witness> {
    let diamond = clusters
        .iter()
        .find(|cluster| cluster.cluster == "hud_diamond")
        .map(|cluster| cluster.frames.as_slice())
        .unwrap_or_default();
    let life = clusters
        .iter()
        .find(|cluster| cluster.cluster == "hud_life_icon")
        .map(|cluster| cluster.frames.as_slice())
        .unwrap_or_default();
    let mut witnesses = Vec::new();

    if let Some(wait_free) = diamond.iter().find(|frame| {
        frame.mode_to_update_hud.z80_wait_cycles == 0
            && frame.update_to_draw_hud.z80_wait_cycles == 0
    }) {
        witnesses.push(Witness {
            kind: "first_wait_free_diamond",
            frame: wait_free.frame,
            changed_pixels: None,
        });
    }
    if let Some(asynchronous) = diamond.iter().find(|frame| {
        frame.mode_to_update_hud.z80_wait_cycles != 0
            || frame.update_to_draw_hud.z80_wait_cycles != 0
    }) {
        witnesses.push(Witness {
            kind: "first_asynchronous_wait_diamond",
            frame: asynchronous.frame,
            changed_pixels: None,
        });
    }
    if let Some(worst_life) = life
        .iter()
        .max_by_key(|frame| (frame.changed_pixels, std::cmp::Reverse(frame.frame)))
    {
        witnesses.push(Witness {
            kind: "worst_life_icon",
            frame: worst_life.frame,
            changed_pixels: Some(worst_life.changed_pixels),
        });
    }
    witnesses
}

fn replay_census(census: &CensusFile, cluster_keys: &[String]) -> Result<ReplayReport> {
    let clusters: Vec<&CensusCluster> = census
        .clusters
        .iter()
        .filter(|cluster| cluster_keys.contains(&cluster.cluster))
        .collect();
    if clusters.is_empty() {
        let mut sorted = cluster_keys.to_vec();
        sorted.sort();
        bail!("no selected clusters found: {sorted:?}");
    }

    let mut output_clusters = Vec::new();
    let mut exact_cpu_spans = 0;
    let mut total_frames = 0;
    for cluster in &clusters {
        let mut rows = Vec::new();
        for frame in &cluster.frames {
            let span = replay_cpu_span(
                frame.mode_entry,
                frame.update_hud,
                frame.mode_to_update_hud.m68k_cycles,
            )
            .with_context(|| format!("frame {}", frame.frame))?;
            exact_cpu_spans += usize::from(span.cpu_span_exact);
            total_frames += 1;
            let fifo_latency_mclk = (beam_mclk_interval(frame.first_fifo_commit)?.0
                - beam_mclk_interval(frame.update_hud)?.0)
                .rem_euclid(FRAME_MCLK);
            rows.push(ReplayRow {
                frame: frame.frame,
                wait_class: if frame.mode_to_update_hud.z80_wait_cycles == 0
                    && frame.update_to_draw_hud.z80_wait_cycles == 0
                {
                    "wait_free"
                } else {
                    "asynchronous_z80_wait"
                },
                observed_update_hud: frame.update_hud,
                first_fifo_commit: frame.first_fifo_commit,
                fifo_latency_mclk_from_update_hud: fifo_latency_mclk,
                replayed_update_hud: span.replayed_update_hud,
                cpu_span_exact: span.cpu_span_exact,
                residual_mclk: span.residual_mclk,
                entry_quantisation_mclk: span.entry_quantisation_mclk,
            });
        }
        output_clusters.push(ReplayCluster {
            cluster: cluster.cluster.clone(),
            frames: rows,
        });
    }

    Ok(ReplayReport {
        schema_version: 2,
        pixel_units: "h40_pixel_index",
        measurement_only: true,
        clusters: output_clusters,
        frame_count: total_frames,
        exact_cpu_span_count: exact_cpu_spans,
        all_cpu_spans_exact: exact_cpu_spans == total_frames,
        witnesses: select_witnesses(&clusters),
    })
}

pub(crate) fn run_raster_span_replay(
    census_path: &Path,
    cluster_keys: &[String],
    output: Option<&Path>,
) -> Result<()> {
    let census: CensusFile = serde_json::from_str(
        &std::fs::read_to_string(census_path)
            .with_context(|| format!("failed to read {}", census_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", census_path.display()))?;
    let keys: Vec<String> = if cluster_keys.is_empty() {
        DEFAULT_CLUSTERS.iter().map(ToString::to_string).collect()
    } else {
        cluster_keys.to_vec()
    };
    let report = replay_census(&census, &keys)?;
    let rendered = serde_json::to_string_pretty(&report)? + "\n";
    match output {
        Some(path) => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, &rendered)
                .with_context(|| format!("failed to write {}", path.display()))?;
        }
        None => print!("{rendered}"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real capture: hud_diamond frame 1730 from the HUD census. Entry pixel
    /// 204 spans mclk 1632..1639 on scanline 224; 9,282 measured 68000 cycles
    /// (x7 mclk) land the advanced interval on scanline 243 mclk
    /// 832686..832693, intersecting observed pixel 204 (832692..832699).
    fn census_frame_1730() -> CensusFrame {
        CensusFrame {
            frame: 1730,
            changed_pixels: 2,
            mode_entry: Position {
                scanline: 224,
                pixel: 204,
            },
            mode_to_update_hud: CensusInterval {
                m68k_cycles: 9_282,
                z80_wait_cycles: 0,
            },
            update_hud: Position {
                scanline: 243,
                pixel: 204,
            },
            update_to_draw_hud: CensusInterval {
                m68k_cycles: 18_630,
                z80_wait_cycles: 0,
            },
            first_fifo_commit: Position {
                scanline: 20,
                pixel: 380,
            },
        }
    }

    fn census(frames: Vec<CensusFrame>) -> CensusFile {
        CensusFile {
            clusters: vec![CensusCluster {
                cluster: "hud_diamond".to_string(),
                frames,
            }],
        }
    }

    fn default_keys() -> Vec<String> {
        DEFAULT_CLUSTERS.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn frame_1730_replays_exact_with_zero_residual() {
        let report = replay_census(&census(vec![census_frame_1730()]), &default_keys()).unwrap();

        assert!(report.all_cpu_spans_exact);
        let row = &report.clusters[0].frames[0];
        assert!(row.cpu_span_exact);
        assert_eq!(row.residual_mclk, 0);
        assert_eq!(row.entry_quantisation_mclk, 8);
        assert_eq!(
            row.replayed_update_hud,
            Position {
                scanline: 243,
                pixel: 204
            }
        );
        // update_hud {243,204} -> mclk 832692; first commit {20,380} ->
        // mclk 71460 (pixel 380 sits in the HSYNC divider region, 3,060
        // mclk into the scanline, NOT 380). Wrapped distance = 134,808.
        assert_eq!(row.fifo_latency_mclk_from_update_hud, 134_808);
    }

    #[test]
    fn frame_1730_discriminates_at_one_cpu_cycle() {
        let mut frame = census_frame_1730();
        frame.mode_to_update_hud.m68k_cycles -= 1;
        let report = replay_census(&census(vec![frame]), &default_keys()).unwrap();
        let row = &report.clusters[0].frames[0];
        assert!(!row.cpu_span_exact);
        assert_eq!(row.residual_mclk, 6);

        let mut frame = census_frame_1730();
        frame.mode_to_update_hud.m68k_cycles += 10;
        let report = replay_census(&census(vec![frame]), &default_keys()).unwrap();
        let row = &report.clusters[0].frames[0];
        assert!(!row.cpu_span_exact);
        assert_eq!(row.residual_mclk, -57);
    }

    #[test]
    fn rejects_out_of_range_pixel_index() {
        let mut frame = census_frame_1730();
        frame.mode_entry.pixel = 3_400; // mclk offset, not a pixel index
        let error = replay_census(&census(vec![frame]), &default_keys()).unwrap_err();
        assert!(format!("{error:#}").contains("pixel index out of range"));
    }

    #[test]
    fn rejects_missing_selected_clusters() {
        let empty = CensusFile { clusters: vec![] };
        let error = replay_census(&empty, &default_keys()).unwrap_err();
        assert!(error.to_string().contains("no selected clusters"));
    }
}
