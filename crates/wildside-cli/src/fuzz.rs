//! `wildside fuzz`: random routes from each level start, replayed on native
//! and the ROM, with first divergences clustered by `Signature`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use wildside_data::AssetManifest;

use crate::input_gen::{Choices, InputProfile, extend_prefix, reduce_script};
use crate::parity::{OracleRunner, Signature, first_divergence, route_scenario};
use crate::scenarios::{DynamicScenario, NativeDynamicRunOptions, run_native_dynamic_scenario};

/// Frames kept after the divergence in a cluster representative.
const REPRO_TAIL: usize = 30;

/// Up, Down, Left, Right, B, C, A and the combinations the spike found
/// divergent; Start is excluded so routes never pause.
const FUZZ_INPUT: InputProfile = InputProfile {
    actions: &[
        0x08, 0x28, 0x18, 0x38, 0x04, 0x24, 0x14, 0x34, 0x10, 0x20, 0x02, 0x01, 0x40, 0x00, 0x48,
        0x44, 0x12, 0x11,
    ],
    run_lengths: &[
        4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
    ],
    switch_percent: 30,
};

const SUMMARY_PATH: &str = "docs/source-port/fuzz-summary.yaml";
const REPRO_DIR: &str = "tests/data/fuzz-repros";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct FuzzArgs {
    #[serde(with = "hex_levels")]
    pub(crate) levels: Vec<u8>,
    pub(crate) seeds: u64,
    pub(crate) frames: usize,
}

/// One route's outcome; `signature` is absent when native matched the ROM.
#[derive(Debug, Clone, Serialize)]
struct RouteResult {
    #[serde(with = "hex_level")]
    level: u8,
    seed: Option<u64>,
    frame: Option<usize>,
    signature: Option<Signature>,
    error: Option<String>,
    #[serde(skip)]
    script: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IdleStatus {
    /// The all-zero control matched for every frame.
    Clean,
    /// The idle control diverged, so random routes there would only restate it.
    StartState,
    /// The control could not run; see the run report.
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LevelSummary {
    #[serde(with = "hex_level")]
    pub(crate) level: u8,
    pub(crate) idle: IdleStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) idle_frame: Option<usize>,
    pub(crate) routes: usize,
    pub(crate) diverged: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Cluster {
    pub(crate) id: String,
    pub(crate) signature: Signature,
    pub(crate) count: usize,
    #[serde(with = "hex_levels")]
    pub(crate) levels: Vec<u8>,
    pub(crate) earliest_frame: usize,
    /// Representative route, truncated to the divergence plus `REPRO_TAIL`.
    pub(crate) repro: String,
}

/// The checked-in census, and the core of each run's `report.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FuzzSummary {
    pub(crate) revision: String,
    pub(crate) args: FuzzArgs,
    pub(crate) levels: Vec<LevelSummary>,
    pub(crate) clusters: Vec<Cluster>,
}

#[derive(Serialize)]
struct RunReport<'a> {
    summary: &'a FuzzSummary,
    idle: &'a [RouteResult],
    routes: &'a [RouteResult],
}

pub(crate) fn run_fuzz(
    repo_root: &Path,
    rom: &Path,
    manifest: &AssetManifest,
    args: &FuzzArgs,
    jobs: Option<usize>,
    write: bool,
) -> Result<()> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let runs = repo_root.join(".cache/fuzz-runs");
    let run_dir = runs.join(stamp.to_string());
    let oracle = OracleRunner::new(repo_root, rom, &run_dir.join("jobs"))?;
    let mut pool = rayon::ThreadPoolBuilder::new();
    if let Some(jobs) = jobs {
        pool = pool.num_threads(jobs);
    }
    let pool = pool.build()?;
    let run = |level: u8, seed: Option<u64>| {
        let script = seed.map_or_else(
            || vec![0; args.frames],
            |seed| {
                extend_prefix(
                    &[],
                    args.frames,
                    &FUZZ_INPUT,
                    &mut Choices((u64::from(level) << 32) | seed),
                )
            },
        );
        run_route(manifest, &oracle, level, seed, script)
    };

    println!("fuzz: idle controls for {} levels", args.levels.len());
    let idle = pool.install(|| {
        args.levels
            .par_iter()
            .map(|&level| run(level, None))
            .collect::<Vec<_>>()
    });
    let clean = idle
        .iter()
        .filter(|result| result.error.is_none() && result.signature.is_none())
        .map(|result| result.level)
        .collect::<Vec<_>>();
    let jobs = clean
        .iter()
        .flat_map(|&level| (0..args.seeds).map(move |seed| (level, seed)))
        .collect::<Vec<_>>();
    println!(
        "fuzz: {} idle-clean levels, {} routes of {} frames",
        clean.len(),
        jobs.len(),
        args.frames
    );
    let routes = pool.install(|| {
        jobs.par_iter()
            .map(|&(level, seed)| run(level, Some(seed)))
            .collect::<Vec<_>>()
    });

    let summary = FuzzSummary {
        revision: git_revision(repo_root),
        args: args.clone(),
        levels: summarise_levels(args, &idle, &routes),
        clusters: cluster(&routes, |id| format!("{REPRO_DIR}/{id}.json")),
    };
    fs::create_dir_all(run_dir.join("repros"))?;
    for (cluster, scenario) in summary
        .clusters
        .iter()
        .zip(representatives(&summary.clusters, &routes))
    {
        fs::write(
            run_dir.join("repros").join(format!("{}.json", cluster.id)),
            serde_json::to_vec_pretty(&scenario)?,
        )?;
        if write {
            fs::create_dir_all(repo_root.join(REPRO_DIR))?;
            fs::write(
                repo_root.join(&cluster.repro),
                serde_json::to_vec_pretty(&scenario)?,
            )?;
        }
    }
    fs::write(
        run_dir.join("report.json"),
        serde_json::to_vec_pretty(&RunReport {
            summary: &summary,
            idle: &idle,
            routes: &routes,
        })?,
    )?;
    fs::write(run_dir.join("clusters.md"), clusters_markdown(&summary))?;
    fs::remove_dir_all(run_dir.join("jobs")).ok();
    let latest = runs.join("latest");
    let _ = fs::remove_file(&latest);
    #[cfg(unix)]
    std::os::unix::fs::symlink(&run_dir, &latest).ok();
    if write {
        write_summary(repo_root, &summary)?;
    }

    print!("{}", clusters_markdown(&summary));
    println!("fuzz report: {}", run_dir.join("report.json").display());
    let errors = idle
        .iter()
        .chain(&routes)
        .filter(|result| result.error.is_some())
        .count();
    anyhow::ensure!(errors == 0, "{errors} fuzz jobs failed; see the run report");
    Ok(())
}

/// Shrinks a cluster's repro while it keeps the same signature, writing the
/// minimal script in place: `tests/data/fuzz-repros/<id>.json`, else the
/// latest run's copy.
pub(crate) fn shrink_cluster(
    repo_root: &Path,
    rom: &Path,
    manifest: &AssetManifest,
    cluster: &str,
) -> Result<()> {
    let path = [
        repo_root.join(REPRO_DIR),
        repo_root.join(".cache/fuzz-runs/latest/repros"),
    ]
    .into_iter()
    .map(|dir| dir.join(format!("{cluster}.json")))
    .find(|path| path.is_file())
    .with_context(|| format!("no repro for cluster {cluster}"))?;
    let repro: DynamicScenario = serde_json::from_slice(&fs::read(&path)?)
        .with_context(|| format!("parse {}", path.display()))?;
    let level = repro
        .warp_to_level
        .context("fuzz repro has no warp level")?;
    let oracle = OracleRunner::new(
        repo_root,
        rom,
        &repo_root.join(".cache/fuzz-runs/shrink").join(cluster),
    )?;
    let divergence_of = |script: &[u8]| {
        let scenario = route_scenario(repro.slug.clone(), level, script.to_vec());
        let native = run_native_dynamic_scenario(
            manifest.clone(),
            &scenario,
            NativeDynamicRunOptions::default(),
        )?;
        let oracle = oracle.run(&scenario, cluster)?;
        Ok::<_, anyhow::Error>(first_divergence(&native, &oracle, level).map(|divergence| {
            let signature = Signature::new(&divergence, &native, &oracle, script);
            (divergence.frame, signature.id())
        }))
    };
    let Some((frame, id)) = divergence_of(&repro.script)? else {
        bail!("{} no longer diverges", path.display());
    };
    if id != cluster {
        bail!("{} now diverges as cluster {id}", path.display());
    }
    let mut evaluations = 0;
    let script = reduce_script(&repro.script, frame, |candidate| {
        evaluations += 1;
        Ok(divergence_of(candidate)?.is_some_and(|(_, id)| id == cluster))
    })?;
    let shrunk = route_scenario(repro.slug.clone(), level, script);
    fs::write(&path, serde_json::to_vec_pretty(&shrunk)?)?;
    println!(
        "shrank {cluster} from {} to {} frames ({} nonzero inputs) in {evaluations} replays: {}",
        repro.script.len(),
        shrunk.script.len(),
        shrunk.script.iter().filter(|&&mask| mask != 0).count(),
        path.display()
    );
    Ok(())
}

fn run_route(
    manifest: &AssetManifest,
    oracle: &OracleRunner,
    level: u8,
    seed: Option<u64>,
    script: Vec<u8>,
) -> RouteResult {
    let slug = seed.map_or_else(
        || format!("fuzz-{level:02x}-idle"),
        |seed| format!("fuzz-{level:02x}-{seed}"),
    );
    let scenario = route_scenario(slug.clone(), level, script);
    let outcome = (|| {
        let native = run_native_dynamic_scenario(
            manifest.clone(),
            &scenario,
            NativeDynamicRunOptions::default(),
        )?;
        let manifest = oracle.run(&scenario, &slug)?;
        Ok::<_, anyhow::Error>(
            first_divergence(&native, &manifest, level).map(|divergence| {
                let signature = Signature::new(&divergence, &native, &manifest, &scenario.script);
                (divergence.frame, signature)
            }),
        )
    })();
    let (frame, signature, error) = match outcome {
        Ok(Some((frame, signature))) => (Some(frame), Some(signature), None),
        Ok(None) => (None, None, None),
        Err(error) => (None, None, Some(format!("{error:#}"))),
    };
    println!(
        "{slug}: {}",
        match (&signature, &error) {
            (Some(signature), _) => format!("frame {} {signature}", frame.unwrap_or_default()),
            (None, Some(error)) => format!("error {error}"),
            (None, None) => "match".to_owned(),
        }
    );
    RouteResult {
        level,
        seed,
        frame,
        signature,
        error,
        script: scenario.script,
    }
}

fn summarise_levels(
    args: &FuzzArgs,
    idle: &[RouteResult],
    routes: &[RouteResult],
) -> Vec<LevelSummary> {
    args.levels
        .iter()
        .zip(idle)
        .map(|(&level, control)| {
            let level_routes = routes.iter().filter(|route| route.level == level);
            LevelSummary {
                level,
                idle: match (&control.error, &control.signature) {
                    (Some(_), _) => IdleStatus::Error,
                    (None, Some(_)) => IdleStatus::StartState,
                    (None, None) => IdleStatus::Clean,
                },
                idle_frame: control.frame,
                routes: level_routes.clone().count(),
                diverged: level_routes
                    .filter(|route| route.signature.is_some())
                    .count(),
            }
        })
        .collect()
}

/// Groups divergent routes; the representative is the earliest divergence,
/// ties broken by level then seed. Ordered by count, then earliest frame.
fn cluster(routes: &[RouteResult], repro_path: impl Fn(&str) -> String) -> Vec<Cluster> {
    let mut groups = BTreeMap::<&Signature, Vec<&RouteResult>>::new();
    for route in routes {
        if let Some(signature) = &route.signature {
            groups.entry(signature).or_default().push(route);
        }
    }
    let mut clusters = groups
        .into_iter()
        .map(|(signature, members)| {
            let id = signature.id();
            let mut levels = members.iter().map(|route| route.level).collect::<Vec<_>>();
            levels.sort_unstable();
            levels.dedup();
            Cluster {
                repro: repro_path(&id),
                id,
                signature: signature.clone(),
                count: members.len(),
                levels,
                earliest_frame: members
                    .iter()
                    .filter_map(|route| route.frame)
                    .min()
                    .unwrap_or_default(),
            }
        })
        .collect::<Vec<_>>();
    clusters.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then(left.earliest_frame.cmp(&right.earliest_frame))
            .then(left.id.cmp(&right.id))
    });
    clusters
}

fn representatives(clusters: &[Cluster], routes: &[RouteResult]) -> Vec<DynamicScenario> {
    clusters
        .iter()
        .map(|cluster| {
            let route = routes
                .iter()
                .filter(|route| route.signature.as_ref() == Some(&cluster.signature))
                .min_by_key(|route| (route.frame, route.level, route.seed))
                .expect("cluster has members");
            let frame = route.frame.expect("divergent route has a frame");
            let script = route.script[..route.script.len().min(frame + REPRO_TAIL)].to_vec();
            route_scenario(format!("fuzz-{}", cluster.id), route.level, script)
        })
        .collect()
}

fn clusters_markdown(summary: &FuzzSummary) -> String {
    let mut out = String::new();
    let routes: usize = summary.levels.iter().map(|level| level.routes).sum();
    let diverged: usize = summary.levels.iter().map(|level| level.diverged).sum();
    let _ = writeln!(
        out,
        "# Fuzz clusters\n\n{diverged}/{routes} routes diverged into {} clusters.\n",
        summary.clusters.len()
    );
    let start_state = summary
        .levels
        .iter()
        .filter(|level| level.idle != IdleStatus::Clean)
        .map(|level| format!("{:#04x}", level.level))
        .collect::<Vec<_>>();
    if !start_state.is_empty() {
        let _ = writeln!(out, "Idle control not clean: {}\n", start_state.join(", "));
    }
    let _ = writeln!(out, "| id | count | levels | earliest | signature |");
    let _ = writeln!(out, "|---|---|---|---|---|");
    for cluster in &summary.clusters {
        let levels = cluster
            .levels
            .iter()
            .map(|level| format!("{level:#04x}"))
            .collect::<Vec<_>>()
            .join(" ");
        let _ = writeln!(
            out,
            "| {} | {} | {levels} | {} | {} |",
            cluster.id, cluster.count, cluster.earliest_frame, cluster.signature
        );
    }
    out
}

fn write_summary(repo_root: &Path, summary: &FuzzSummary) -> Result<()> {
    let path = repo_root.join(SUMMARY_PATH);
    let body = format!(
        "# Generated by `wildside fuzz run --write`; do not edit by hand.\n{}",
        serde_yaml::to_string(summary)?
    );
    fs::write(&path, body).with_context(|| format!("write {}", path.display()))
}

pub(crate) fn git_revision(repo_root: &Path) -> String {
    std::process::Command::new("git")
        .args(["describe", "--always", "--dirty", "--abbrev=12"])
        .current_dir(repo_root)
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map_or_else(|| "unknown".to_owned(), |text| text.trim().to_owned())
}

pub(crate) mod hex_level {
    use serde::{Deserialize, Deserializer, Serializer};

    #[expect(
        clippy::trivially_copy_pass_by_ref,
        reason = "serde `with` passes the field by reference"
    )]
    pub(crate) fn serialize<S: Serializer>(level: &u8, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{level:#04x}"))
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u8, D::Error> {
        let text = String::deserialize(deserializer)?;
        crate::inspect::parse_level_id(&text).map_err(serde::de::Error::custom)
    }
}

mod hex_levels {
    use serde::{Deserialize, Deserializer, Serializer, ser::SerializeSeq};

    pub(crate) fn serialize<S: Serializer>(
        levels: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(levels.len()))?;
        for level in levels {
            seq.serialize_element(&format!("{level:#04x}"))?;
        }
        seq.end()
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<u8>, D::Error> {
        Vec::<String>::deserialize(deserializer)?
            .iter()
            .map(|text| crate::inspect::parse_level_id(text).map_err(serde::de::Error::custom))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route(level: u8, seed: u64, frame: Option<usize>, field: &str) -> RouteResult {
        RouteResult {
            level,
            seed: Some(seed),
            frame,
            signature: frame.map(|_| Signature {
                fields: vec![field.to_owned()],
                native_movement: Some(1),
                oracle_movement: Some(2),
                native_on_ground: Some(true),
                inputs: [Some(2), Some(2)],
            }),
            error: None,
            script: vec![2; 600],
        }
    }

    #[test]
    fn clusters_rank_by_count_and_pick_the_earliest_representative() {
        let routes = [
            route(1, 0, Some(90), "kid_x_pos"),
            route(0, 3, Some(40), "kid_x_pos"),
            route(0, 1, None, ""),
            route(2, 0, Some(10), "number_objects"),
        ];
        let clusters = cluster(&routes, |id| format!("repros/{id}.json"));
        assert_eq!(clusters.len(), 2);
        assert_eq!((clusters[0].count, clusters[0].earliest_frame), (2, 40));
        assert_eq!(clusters[0].levels, [0, 1]);
        assert_eq!(clusters[0].repro, format!("repros/{}.json", clusters[0].id));
        let scenarios = representatives(&clusters, &routes);
        assert_eq!(scenarios[0].warp_to_level, Some(0));
        assert_eq!(scenarios[0].script.len(), 40 + REPRO_TAIL);
        assert_eq!(scenarios[0].frame_count, 41 + REPRO_TAIL);
        assert_eq!(scenarios[1].slug, format!("fuzz-{}", clusters[1].id));
    }

    #[test]
    fn the_summary_round_trips_through_yaml_with_hex_levels() {
        let routes = [route(0x0a, 0, Some(12), "kid_y_pos")];
        let summary = FuzzSummary {
            revision: "abc".to_owned(),
            args: FuzzArgs {
                levels: vec![0x0a],
                seeds: 1,
                frames: 600,
            },
            levels: summarise_levels(
                &FuzzArgs {
                    levels: vec![0x0a],
                    seeds: 1,
                    frames: 600,
                },
                &[route(0x0a, 0, None, "")],
                &routes,
            ),
            clusters: cluster(&routes, |id| format!("{REPRO_DIR}/{id}.json")),
        };
        let yaml = serde_yaml::to_string(&summary).unwrap();
        assert!(
            yaml.contains("- '0x0a'") || yaml.contains("- 0x0a"),
            "{yaml}"
        );
        let parsed: FuzzSummary = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.args, summary.args);
        assert_eq!(parsed.levels[0].idle, IdleStatus::Clean);
        assert_eq!(parsed.levels[0].diverged, 1);
        assert_eq!(parsed.clusters[0].signature, summary.clusters[0].signature);
    }
}
