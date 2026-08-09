//! `verify-cpu-vectors` (#447): validate `r68k-emu` against the published
//! SingleStepTests 68000 vector sets.
//!
//! Every timing claim in this repository sits on top of one git-pinned 68000
//! core that had no conformance evidence at all. This runner supplies it, in
//! two tiers of decreasing authority:
//!
//! 1. **State (primary verdict).** Registers, both stack pointers, PC, SR/CCR,
//!    and every published final memory byte. A failure here is a defect in the
//!    core's arithmetic, flags, addressing, or exception handling — i.e. a root
//!    cause for whole divergence classes, not a rounding detail.
//! 2. **Timing and bus (secondary, informational).** Total cycle count and the
//!    ordered data-space access list. A core can be state-exact and still
//!    charge the wrong cycles; that is precisely the debt the oracle-injection
//!    meter pays for, so it is measured but never conflated with tier 1.
//!
//! Where the two published sets disagree *with each other* about the same
//! opcode word's cycle count, that is recorded as a finding about the vectors
//! rather than a failure of r68k — see [`CycleDisagreement`].

pub(crate) mod harness;
pub(crate) mod vector;

use anyhow::{Context, Result, bail};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::bootstrap::{cpu_vector_repos, head_commit};
use harness::{VectorHarness, VectorOutcome};
use vector::{CpuVectorSet, load_vector_file, vector_files};

/// How many worst-offender opcode files the human summary names.
const WORST_FILE_LIMIT: usize = 15;
/// How many set-vs-set cycle disagreements the human summary names.
const DISAGREEMENT_LIMIT: usize = 20;
/// How many failing vector names each opcode file keeps as examples.
const EXAMPLE_FAILURE_CAP: usize = 3;

/// Per-opcode-file tallies. Counts, not rates: the ratchet compares integers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FileReport {
    pub(crate) vectors: u64,
    /// Vectors whose entire published final state matched (tier 1 pass).
    pub(crate) state_pass: u64,
    /// Vectors where the publishing core dispatched an exception. Reported as a
    /// partition of the verdict because r68k's exception *frames* diverge
    /// systematically, which would otherwise mask the arithmetic verdict.
    #[serde(default)]
    pub(crate) exception_vectors: u64,
    /// State passes among [`Self::exception_vectors`].
    #[serde(default)]
    pub(crate) exception_state_pass: u64,
    /// Trace-bit-only failures among [`Self::exception_vectors`].
    #[serde(default)]
    pub(crate) exception_trace_bit_only_fail: u64,
    /// Tier-1 failures whose only disagreement is r68k's absent trace bit.
    pub(crate) trace_bit_only_fail: u64,
    pub(crate) cycle_pass: u64,
    pub(crate) data_bus_pass: u64,
    pub(crate) data_register_fail: u64,
    pub(crate) address_register_fail: u64,
    pub(crate) stack_pointer_fail: u64,
    pub(crate) program_counter_fail: u64,
    pub(crate) status_register_fail: u64,
    pub(crate) memory_fail: u64,
    pub(crate) read_outside_vector: u64,
    pub(crate) prefetch_ram_conflict: u64,
    /// Vectors on which r68k panicked instead of raising an exception.
    #[serde(default)]
    pub(crate) panicked: u64,
    /// First few failing vector names, so a family's failure is diagnosable
    /// from the report without re-running. Capped at [`EXAMPLE_FAILURE_CAP`].
    #[serde(default)]
    pub(crate) example_failures: Vec<String>,
}

impl FileReport {
    pub(crate) fn state_fail(&self) -> u64 {
        self.vectors - self.state_pass
    }

    /// Vectors where no exception was dispatched — the arithmetic/addressing
    /// verdict, uncontaminated by r68k's divergent exception frames.
    pub(crate) fn plain_vectors(&self) -> u64 {
        self.vectors - self.exception_vectors
    }

    pub(crate) fn plain_state_pass(&self) -> u64 {
        self.state_pass - self.exception_state_pass
    }

    /// The **arithmetic verdict**: no exception dispatched, and r68k's absent
    /// trace bit forgiven. This is the number that says whether the core
    /// computes the right answer; the other two tiers say how it frames an
    /// exception and how long it claims to take.
    pub(crate) fn arithmetic_pass(&self) -> u64 {
        self.plain_state_pass() + (self.trace_bit_only_fail - self.exception_trace_bit_only_fail)
    }

    fn absorb(&mut self, outcome: &VectorOutcome, name: &str, expects_exception: bool) {
        self.vectors += 1;
        let mismatch = outcome.mismatch;
        self.panicked += u64::from(outcome.panicked);
        let state_pass = mismatch.is_pass() && !outcome.panicked;
        let trace_only = mismatch.is_trace_bit_only() && !outcome.panicked && !state_pass;
        if expects_exception {
            self.exception_vectors += 1;
            self.exception_state_pass += u64::from(state_pass);
            self.exception_trace_bit_only_fail += u64::from(trace_only);
        }
        if state_pass {
            self.state_pass += 1;
        } else {
            self.trace_bit_only_fail += u64::from(trace_only);
            if self.example_failures.len() < EXAMPLE_FAILURE_CAP {
                self.example_failures.push(name.to_owned());
            }
        }
        if outcome.cycles_match() {
            self.cycle_pass += 1;
        }
        if outcome.data_bus_match {
            self.data_bus_pass += 1;
        }
        self.data_register_fail += u64::from(mismatch.data_registers);
        self.address_register_fail += u64::from(mismatch.address_registers);
        self.stack_pointer_fail +=
            u64::from(mismatch.user_stack_pointer || mismatch.supervisor_stack_pointer);
        self.program_counter_fail += u64::from(mismatch.program_counter);
        self.status_register_fail += u64::from(mismatch.status_register);
        self.memory_fail += u64::from(mismatch.memory);
        self.read_outside_vector += u64::from(outcome.read_outside_vector);
        self.prefetch_ram_conflict += u64::from(outcome.prefetch_ram_conflict);
    }

    fn merge(&mut self, other: &Self) {
        self.vectors += other.vectors;
        self.state_pass += other.state_pass;
        self.exception_vectors += other.exception_vectors;
        self.exception_state_pass += other.exception_state_pass;
        self.exception_trace_bit_only_fail += other.exception_trace_bit_only_fail;
        self.trace_bit_only_fail += other.trace_bit_only_fail;
        self.cycle_pass += other.cycle_pass;
        self.data_bus_pass += other.data_bus_pass;
        self.data_register_fail += other.data_register_fail;
        self.address_register_fail += other.address_register_fail;
        self.stack_pointer_fail += other.stack_pointer_fail;
        self.program_counter_fail += other.program_counter_fail;
        self.status_register_fail += other.status_register_fail;
        self.memory_fail += other.memory_fail;
        self.read_outside_vector += other.read_outside_vector;
        self.prefetch_ram_conflict += other.prefetch_ram_conflict;
        self.panicked += other.panicked;
        // Totals carry no examples: they belong to the per-file rows, where the
        // opcode family gives them meaning.
    }
}

/// One published set's results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SetReport {
    /// `680x0` or `m68000`.
    pub(crate) set: String,
    /// The vector repository's checked-out commit, so a baseline can refuse to
    /// compare across a data change.
    pub(crate) commit: Option<String>,
    pub(crate) totals: FileReport,
    pub(crate) files: BTreeMap<String, FileReport>,
    /// Observed cycle counts per instruction word, for the set-vs-set finding.
    #[serde(skip)]
    cycles_by_opcode: BTreeMap<u16, OpcodeCycles>,
}

/// Every distinct published cycle count seen for one instruction word, plus the
/// opcode file it came from and how many vectors contributed. Opcode files
/// partition the encoding space (one mnemonic and size each), so the file name
/// is unambiguous.
///
/// Only vectors that dispatched no exception contribute: an exception's cost is
/// not the instruction's cost, and a single address-error vector would otherwise
/// make an encoding look like a 50-cycle instruction.
#[derive(Debug, Clone, Default)]
struct OpcodeCycles {
    file: String,
    observed: Vec<u32>,
    samples: u32,
}

/// Minimum vectors per set before a cycle count counts as that set's position.
///
/// With one observation apiece, two independently randomised sets disagree
/// constantly for reasons that have nothing to do with the instruction — most
/// often because only one of them happened to pick operands that also branch.
const MIN_CYCLE_SAMPLES: u32 = 2;

/// The two sets disagree about how long one opcode word takes.
///
/// Only reported where each set observed exactly one cycle count for that word
/// and the two differ — a data-dependent instruction legitimately shows several
/// counts, and comparing multisets of independently randomised operands would
/// manufacture noise.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CycleDisagreement {
    pub(crate) opcode: u16,
    pub(crate) file: String,
    pub(crate) harte_cycles: u32,
    pub(crate) mame_cycles: u32,
}

/// The two sets disagree about an opcode's *final state*, not its timing.
///
/// Inferred, not matched vector-by-vector: the two sets randomise operands
/// independently, so individual vectors never line up. But r68k is
/// deterministic and both sets sample the same encoding space, so one set
/// failing thousands of vectors on an opcode while the other passes every one
/// cannot be a property of r68k — the sets must expect different final state.
/// Such an opcode is a finding about the vectors, and the passing set exonerates
/// r68k on it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StateDisagreement {
    pub(crate) file: String,
    pub(crate) harte_arithmetic_fail: u64,
    pub(crate) harte_vectors: u64,
    pub(crate) mame_arithmetic_fail: u64,
    pub(crate) mame_vectors: u64,
}

impl StateDisagreement {
    /// Which set expects state r68k does not produce.
    pub(crate) fn disagreeing_set(&self) -> &'static str {
        if self.harte_arithmetic_fail > 0 {
            CpuVectorSet::Harte.label()
        } else {
            CpuVectorSet::Mame.label()
        }
    }
}

/// The whole run, as written to `.cache/audit/cpu-vectors.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CpuVectorReport {
    /// Always true in a written report: its absence is how a consumer detects a
    /// run that did not happen (see `scripts/cpu_vector_ratchet.py`).
    pub(crate) completed: bool,
    /// `None` for an uncapped run; otherwise the per-file prefix length.
    pub(crate) per_file_limit: Option<usize>,
    pub(crate) sets: Vec<SetReport>,
    pub(crate) cycle_disagreements: Vec<CycleDisagreement>,
    #[serde(default)]
    pub(crate) state_disagreements: Vec<StateDisagreement>,
}

/// Run both sets and produce the report.
///
/// The panic hook is silenced for the duration: r68k aborts on some
/// exception paths (see [`VectorOutcome::panicked`]), and thousands of default
/// hook messages would bury the report. The panics themselves are still
/// counted, and the hook is restored before returning.
pub(crate) fn verify_cpu_vectors(
    reference_root: &Path,
    sets: &[CpuVectorSet],
    per_file_limit: Option<usize>,
) -> Result<CpuVectorReport> {
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let outcome = run_sets(reference_root, sets, per_file_limit);
    std::panic::set_hook(previous_hook);
    outcome
}

fn run_sets(
    reference_root: &Path,
    sets: &[CpuVectorSet],
    per_file_limit: Option<usize>,
) -> Result<CpuVectorReport> {
    let mut reports = Vec::new();
    for &set in sets {
        reports.push(run_set(reference_root, set, per_file_limit)?);
    }
    let cycle_disagreements = cycle_disagreements(&reports);
    let state_disagreements = state_disagreements(&reports);
    Ok(CpuVectorReport {
        completed: true,
        per_file_limit,
        sets: reports,
        cycle_disagreements,
        state_disagreements,
    })
}

fn run_set(
    reference_root: &Path,
    set: CpuVectorSet,
    per_file_limit: Option<usize>,
) -> Result<SetReport> {
    let files = vector_files(reference_root, set)?;
    let per_file = files
        .par_iter()
        .map(|(stem, path)| run_file(set, stem, path, per_file_limit))
        .collect::<Result<Vec<_>>>()?;

    let mut totals = FileReport::default();
    let mut by_file = BTreeMap::new();
    let mut cycles_by_opcode: BTreeMap<u16, OpcodeCycles> = BTreeMap::new();
    for (stem, report, cycles) in per_file {
        totals.merge(&report);
        for (opcode, counts) in cycles {
            let entry = cycles_by_opcode.entry(opcode).or_default();
            entry.file.clone_from(&stem);
            entry.samples += counts.samples;
            for cycles in counts.observed {
                if !entry.observed.contains(&cycles) {
                    entry.observed.push(cycles);
                }
            }
        }
        by_file.insert(stem, report);
    }

    Ok(SetReport {
        set: set.label().to_owned(),
        commit: head_commit(&reference_root.join(set.dir_name())),
        totals,
        files: by_file,
        cycles_by_opcode,
    })
}

type FileRun = (String, FileReport, BTreeMap<u16, OpcodeCycles>);

fn run_file(
    set: CpuVectorSet,
    stem: &str,
    path: &Path,
    per_file_limit: Option<usize>,
) -> Result<FileRun> {
    let vectors = load_vector_file(set, path, per_file_limit)?;
    let mut harness = VectorHarness::new();
    let mut report = FileReport::default();
    let mut expected_cycles: BTreeMap<u16, OpcodeCycles> = BTreeMap::new();
    for vector in &vectors {
        let expects_exception = vector.expects_exception();
        let outcome = harness.run(set, vector);
        report.absorb(&outcome, &vector.name, expects_exception);
        if !expects_exception {
            let entry = expected_cycles.entry(outcome.opcode).or_default();
            entry.samples += 1;
            if !entry.observed.contains(&outcome.expected_cycles) {
                entry.observed.push(outcome.expected_cycles);
            }
        }
    }
    Ok((stem.to_owned(), report, expected_cycles))
}

/// Minimum plain vectors on both sides before a 0%-vs-nonzero split is a claim.
const MIN_DISAGREEMENT_VECTORS: u64 = 100;
/// Failure rate the disagreeing side must exceed, so a handful of stragglers in
/// an otherwise-clean opcode is not dressed up as a vector-set dispute.
const MIN_DISAGREEMENT_RATE: f64 = 1.0;

/// Opcode files where one set's arithmetic verdict is clean and the other's is
/// not — see [`StateDisagreement`] for why that implies the sets disagree.
fn state_disagreements(reports: &[SetReport]) -> Vec<StateDisagreement> {
    let (Some(harte), Some(mame)) = both_sets(reports) else {
        return Vec::new();
    };

    let mut disagreements = Vec::new();
    for (name, harte_file) in &harte.files {
        let Some(mame_file) = mame.files.get(name) else {
            continue;
        };
        let sides = [
            (
                harte_file.plain_vectors() - harte_file.arithmetic_pass(),
                harte_file.plain_vectors(),
            ),
            (
                mame_file.plain_vectors() - mame_file.arithmetic_pass(),
                mame_file.plain_vectors(),
            ),
        ];
        if sides
            .iter()
            .any(|&(_, total)| total < MIN_DISAGREEMENT_VECTORS)
        {
            continue;
        }
        let clean = sides.iter().any(|&(fail, _)| fail == 0);
        let dirty = sides
            .iter()
            .any(|&(fail, total)| percent(fail, total) >= MIN_DISAGREEMENT_RATE);
        if clean && dirty {
            disagreements.push(StateDisagreement {
                file: name.clone(),
                harte_arithmetic_fail: sides[0].0,
                harte_vectors: sides[0].1,
                mame_arithmetic_fail: sides[1].0,
                mame_vectors: sides[1].1,
            });
        }
    }
    disagreements
}

fn both_sets(reports: &[SetReport]) -> (Option<&SetReport>, Option<&SetReport>) {
    let find = |set: CpuVectorSet| reports.iter().find(|report| report.set == set.label());
    (find(CpuVectorSet::Harte), find(CpuVectorSet::Mame))
}

/// Opcode words where each set is internally unanimous but the two disagree.
fn cycle_disagreements(reports: &[SetReport]) -> Vec<CycleDisagreement> {
    let (Some(harte), Some(mame)) = both_sets(reports) else {
        return Vec::new();
    };

    let mut disagreements = Vec::new();
    for (opcode, harte_cycles) in &harte.cycles_by_opcode {
        let Some(mame_cycles) = mame.cycles_by_opcode.get(opcode) else {
            continue;
        };
        if harte_cycles.samples < MIN_CYCLE_SAMPLES || mame_cycles.samples < MIN_CYCLE_SAMPLES {
            continue;
        }
        if let ([harte_only], [mame_only]) = (
            harte_cycles.observed.as_slice(),
            mame_cycles.observed.as_slice(),
        ) && harte_only != mame_only
        {
            disagreements.push(CycleDisagreement {
                opcode: *opcode,
                file: harte_cycles.file.clone(),
                harte_cycles: *harte_only,
                mame_cycles: *mame_only,
            });
        }
    }
    disagreements
}

/// Write the machine-readable report next to the other audit artefacts.
pub(crate) fn write_report(repo_root: &Path, report: &CpuVectorReport) -> Result<PathBuf> {
    let path = repo_root.join(".cache/audit/cpu-vectors.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(report)?;
    std::fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(path)
}

/// Fail closed when a vector set has not been downloaded.
pub(crate) fn ensure_vectors_present(reference_root: &Path, sets: &[CpuVectorSet]) -> Result<()> {
    for &set in sets {
        let dir = reference_root.join(set.dir_name());
        if !dir.is_dir() {
            let names = cpu_vector_repos()
                .iter()
                .map(|repo| repo.name)
                .collect::<Vec<_>>()
                .join(", ");
            bail!(
                "missing {} vectors at {} — run `mise run bootstrap-cpu-vectors` ({names})",
                set.label(),
                dir.display()
            );
        }
    }
    Ok(())
}

fn percent(part: u64, whole: u64) -> f64 {
    if whole == 0 {
        return 0.0;
    }
    (part as f64) * 100.0 / (whole as f64)
}

/// Human summary: headline rates, then the worst opcode files, then findings.
pub(crate) fn format_report(report: &CpuVectorReport) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();

    let limit = report
        .per_file_limit
        .map_or_else(|| "all".to_owned(), |limit| limit.to_string());
    writeln!(&mut out, "per-file vector limit: {limit}").expect("string write");

    for set in &report.sets {
        let totals = &set.totals;
        writeln!(&mut out).expect("string write");
        writeln!(
            &mut out,
            "== {} @ {} ==",
            set.set,
            set.commit.as_deref().unwrap_or("unknown")
        )
        .expect("string write");
        writeln!(
            &mut out,
            "vectors: {}  files: {}",
            totals.vectors,
            set.files.len()
        )
        .expect("string write");
        writeln!(
            &mut out,
            "state pass: {} ({:.4}%)  fail: {}  of which trace-bit-only: {}",
            totals.state_pass,
            percent(totals.state_pass, totals.vectors),
            totals.state_fail(),
            totals.trace_bit_only_fail
        )
        .expect("string write");
        writeln!(
            &mut out,
            "  arithmetic verdict (no exception, trace bit forgiven): {}/{} pass ({:.4}%)",
            totals.arithmetic_pass(),
            totals.plain_vectors(),
            percent(totals.arithmetic_pass(), totals.plain_vectors())
        )
        .expect("string write");
        writeln!(
            &mut out,
            "  no exception dispatched: {}/{} pass ({:.4}%)",
            totals.plain_state_pass(),
            totals.plain_vectors(),
            percent(totals.plain_state_pass(), totals.plain_vectors())
        )
        .expect("string write");
        writeln!(
            &mut out,
            "  exception dispatched:    {}/{} pass ({:.4}%)",
            totals.exception_state_pass,
            totals.exception_vectors,
            percent(totals.exception_state_pass, totals.exception_vectors)
        )
        .expect("string write");
        writeln!(
            &mut out,
            "cycle pass: {} ({:.4}%)   data-bus pass: {} ({:.4}%)",
            totals.cycle_pass,
            percent(totals.cycle_pass, totals.vectors),
            totals.data_bus_pass,
            percent(totals.data_bus_pass, totals.vectors)
        )
        .expect("string write");
        writeln!(
            &mut out,
            "field fails — d: {}  a: {}  sp: {}  pc: {}  sr: {}  mem: {}",
            totals.data_register_fail,
            totals.address_register_fail,
            totals.stack_pointer_fail,
            totals.program_counter_fail,
            totals.status_register_fail,
            totals.memory_fail
        )
        .expect("string write");
        writeln!(
            &mut out,
            "unspecified-address reads: {}  prefetch/ram conflicts: {}  core panics: {}",
            totals.read_outside_vector, totals.prefetch_ram_conflict, totals.panicked
        )
        .expect("string write");

        // Ranked by the arithmetic verdict, not by raw failures: an opcode file
        // whose only failures are exception frames or the trace bit is already
        // explained by a known gap, and would otherwise crowd out real defects.
        let mut worst: Vec<(&String, &FileReport)> = set
            .files
            .iter()
            .filter(|(_, file)| file.arithmetic_pass() < file.plain_vectors())
            .collect();
        worst.sort_by(|left, right| {
            percent(
                right.1.plain_vectors() - right.1.arithmetic_pass(),
                right.1.plain_vectors(),
            )
            .total_cmp(&percent(
                left.1.plain_vectors() - left.1.arithmetic_pass(),
                left.1.plain_vectors(),
            ))
            .then_with(|| left.0.cmp(right.0))
        });
        writeln!(
            &mut out,
            "worst arithmetic-fail files ({} of {} with any arithmetic failure):",
            worst.len().min(WORST_FILE_LIMIT),
            worst.len()
        )
        .expect("string write");
        for (name, file) in worst.iter().take(WORST_FILE_LIMIT) {
            writeln!(
                &mut out,
                "  {name:<16} {}/{} arithmetic fail ({:.2}%)  exception {}/{} pass  cycles {}/{}",
                file.plain_vectors() - file.arithmetic_pass(),
                file.plain_vectors(),
                percent(
                    file.plain_vectors() - file.arithmetic_pass(),
                    file.plain_vectors()
                ),
                file.exception_state_pass,
                file.exception_vectors,
                file.cycle_pass,
                file.vectors
            )
            .expect("string write");
            for example in &file.example_failures {
                writeln!(&mut out, "      e.g. {example}").expect("string write");
            }
        }

        let mut cycle_worst: Vec<(&String, &FileReport)> = set
            .files
            .iter()
            .filter(|(_, file)| file.cycle_pass < file.vectors)
            .collect();
        cycle_worst.sort_by(|left, right| {
            (right.1.vectors - right.1.cycle_pass)
                .cmp(&(left.1.vectors - left.1.cycle_pass))
                .then_with(|| left.0.cmp(right.0))
        });
        writeln!(
            &mut out,
            "worst cycle-count files ({} with any mismatch):",
            cycle_worst.len()
        )
        .expect("string write");
        for (name, file) in cycle_worst.iter().take(WORST_FILE_LIMIT) {
            writeln!(
                &mut out,
                "  {name:<16} {}/{} cycle mismatches",
                file.vectors - file.cycle_pass,
                file.vectors
            )
            .expect("string write");
        }
    }

    writeln!(&mut out).expect("string write");
    writeln!(
        &mut out,
        "set-vs-set FINAL-STATE disagreements ({} opcode files — one set clean, the other not,",
        report.state_disagreements.len()
    )
    .expect("string write");
    writeln!(
        &mut out,
        "so the sets expect different state and the clean side exonerates r68k):"
    )
    .expect("string write");
    for disagreement in &report.state_disagreements {
        writeln!(
            &mut out,
            "  {:<16} 680x0 {}/{} fail, m68000 {}/{} fail — {} disagrees",
            disagreement.file,
            disagreement.harte_arithmetic_fail,
            disagreement.harte_vectors,
            disagreement.mame_arithmetic_fail,
            disagreement.mame_vectors,
            disagreement.disagreeing_set()
        )
        .expect("string write");
    }

    writeln!(&mut out).expect("string write");
    writeln!(
        &mut out,
        "set-vs-set cycle disagreements (unanimous within each set): {}",
        report.cycle_disagreements.len()
    )
    .expect("string write");
    for disagreement in report.cycle_disagreements.iter().take(DISAGREEMENT_LIMIT) {
        writeln!(
            &mut out,
            "  opcode {:#06x}: 680x0 {} cycles vs m68000 {} cycles",
            disagreement.opcode, disagreement.harte_cycles, disagreement.mame_cycles
        )
        .expect("string write");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use harness::StateMismatch;

    fn outcome(mismatch: StateMismatch, expected: u32, actual: i32, bus: bool) -> VectorOutcome {
        VectorOutcome {
            mismatch,
            expected_cycles: expected,
            actual_cycles: actual,
            data_bus_match: bus,
            prefetch_ram_conflict: false,
            read_outside_vector: false,
            panicked: false,
            opcode: 0x4e71,
        }
    }

    #[test]
    fn absorb_separates_the_three_tiers_and_the_trace_bit_gap() {
        let mut report = FileReport::default();
        report.absorb(
            &outcome(StateMismatch::default(), 4, 4, true),
            "4e71 [NOP]",
            false,
        );
        report.absorb(
            &outcome(
                StateMismatch {
                    trace_bit_only: true,
                    ..StateMismatch::default()
                },
                4,
                4,
                true,
            ),
            "4e71 [NOP] traced",
            false,
        );
        report.absorb(
            &outcome(
                StateMismatch {
                    data_registers: true,
                    ..StateMismatch::default()
                },
                70,
                54,
                false,
            ),
            "c0c0 [MULU D0, D0]",
            true,
        );

        assert_eq!(report.vectors, 3);
        assert_eq!(report.state_pass, 1);
        assert_eq!(report.state_fail(), 2);
        assert_eq!(report.trace_bit_only_fail, 1);
        // The exception partition splits the verdict without changing it.
        assert_eq!(report.exception_vectors, 1);
        assert_eq!(report.exception_state_pass, 0);
        assert_eq!(report.plain_vectors(), 2);
        assert_eq!(report.plain_state_pass(), 1);
        // A state failure with the right cycle count still counts as a cycle
        // pass: the tiers are graded independently on purpose.
        assert_eq!(report.cycle_pass, 2);
        assert_eq!(report.data_bus_pass, 2);
        assert_eq!(report.data_register_fail, 1);
        assert_eq!(
            report.example_failures,
            vec!["4e71 [NOP] traced", "c0c0 [MULU D0, D0]"],
            "only failures are exampled, and the pass is not"
        );
    }

    fn set_report(set: CpuVectorSet, cycles: &[(u16, &[u32], u32)]) -> SetReport {
        SetReport {
            set: set.label().to_owned(),
            commit: Some("deadbeef".to_owned()),
            totals: FileReport::default(),
            files: BTreeMap::new(),
            cycles_by_opcode: cycles
                .iter()
                .map(|(opcode, observed, samples)| {
                    (
                        *opcode,
                        OpcodeCycles {
                            file: "PROBE".to_owned(),
                            observed: observed.to_vec(),
                            samples: *samples,
                        },
                    )
                })
                .collect(),
        }
    }

    #[test]
    fn set_disagreement_needs_both_sets_unanimous_on_the_same_opcode() {
        let reports = vec![
            set_report(
                CpuVectorSet::Harte,
                &[
                    // Unanimous in both sets and different: a real disagreement.
                    (0xc0c0, &[70], 5),
                    // Data-dependent in one set: not comparable, so silent.
                    (0xc1c1, &[54], 5),
                    // Agreeing: silent.
                    (0x4e71, &[4], 5),
                    // Absent from the other set: silent.
                    (0x4e72, &[4], 5),
                    // One observation apiece: too thin to call, so silent.
                    (0x2011, &[50], 1),
                ],
            ),
            set_report(
                CpuVectorSet::Mame,
                &[
                    (0xc0c0, &[54], 5),
                    (0xc1c1, &[38, 42, 54], 9),
                    (0x4e71, &[4], 5),
                    (0x2011, &[12], 1),
                ],
            ),
        ];

        let disagreements = cycle_disagreements(&reports);
        assert_eq!(disagreements.len(), 1);
        assert_eq!(disagreements[0].opcode, 0xc0c0);
        assert_eq!(disagreements[0].harte_cycles, 70);
        assert_eq!(disagreements[0].mame_cycles, 54);
        assert_eq!(disagreements[0].file, "PROBE");
    }

    #[test]
    fn disagreements_are_empty_unless_both_sets_ran() {
        let only_harte = vec![set_report(CpuVectorSet::Harte, &[(0xc0c0, &[70], 5)])];
        assert!(cycle_disagreements(&only_harte).is_empty());
        let only_mame = vec![set_report(CpuVectorSet::Mame, &[(0xc0c0, &[54], 5)])];
        assert!(cycle_disagreements(&only_mame).is_empty());
    }

    #[test]
    fn missing_vector_directory_names_the_bootstrap_task() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let error = ensure_vectors_present(temp.path(), &CpuVectorSet::ALL)
            .expect_err("absent vectors must fail closed");
        assert!(
            error.to_string().contains("bootstrap-cpu-vectors"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn summary_reports_rates_and_findings() {
        let mut files = BTreeMap::new();
        files.insert(
            "MULU".to_owned(),
            FileReport {
                vectors: 100,
                state_pass: 100,
                cycle_pass: 3,
                data_bus_pass: 100,
                ..FileReport::default()
            },
        );
        let report = CpuVectorReport {
            completed: true,
            per_file_limit: Some(100),
            sets: vec![SetReport {
                set: CpuVectorSet::Harte.label().to_owned(),
                commit: Some("e0d5ece".to_owned()),
                totals: FileReport {
                    vectors: 100,
                    state_pass: 100,
                    cycle_pass: 3,
                    data_bus_pass: 100,
                    ..FileReport::default()
                },
                files,
                cycles_by_opcode: BTreeMap::new(),
            }],
            cycle_disagreements: vec![CycleDisagreement {
                opcode: 0xc0c0,
                file: "MULU".to_owned(),
                harte_cycles: 70,
                mame_cycles: 54,
            }],
            state_disagreements: vec![StateDisagreement {
                file: "ASR.b".to_owned(),
                harte_arithmetic_fail: 1642,
                harte_vectors: 8065,
                mame_arithmetic_fail: 0,
                mame_vectors: 2500,
            }],
        };

        let text = format_report(&report);
        assert!(text.contains("per-file vector limit: 100"));
        assert!(text.contains("state pass: 100 (100.0000%)"));
        assert!(text.contains("97/100 cycle mismatches"));
        assert!(text.contains("680x0 70 cycles vs m68000 54 cycles"));
        assert!(text.contains("680x0 1642/8065 fail, m68000 0/2500 fail — 680x0 disagrees"));
    }

    fn file_report(plain_vectors: u64, arithmetic_fail: u64) -> FileReport {
        FileReport {
            vectors: plain_vectors,
            state_pass: plain_vectors - arithmetic_fail,
            ..FileReport::default()
        }
    }

    fn set_with_files(set: CpuVectorSet, files: &[(&str, FileReport)]) -> SetReport {
        SetReport {
            set: set.label().to_owned(),
            commit: None,
            totals: FileReport::default(),
            files: files
                .iter()
                .map(|(name, report)| ((*name).to_owned(), report.clone()))
                .collect(),
            cycles_by_opcode: BTreeMap::new(),
        }
    }

    #[test]
    fn state_disagreement_needs_one_clean_side_and_a_material_other_side() {
        let reports = vec![
            set_with_files(
                CpuVectorSet::Harte,
                &[
                    // Clean vs 20%: the sets expect different state.
                    ("ASR.b", file_report(8000, 1600)),
                    // Both dirty: r68k's own defect, not a vector dispute.
                    ("DIVS", file_report(5000, 1900)),
                    // Both clean.
                    ("ADD.b", file_report(8000, 0)),
                    // Clean vs 0.05%: too thin to call a dispute.
                    ("Bcc", file_report(8000, 4)),
                    // Sample too small on the other side to claim anything.
                    ("CHK", file_report(8000, 800)),
                ],
            ),
            set_with_files(
                CpuVectorSet::Mame,
                &[
                    ("ASR.b", file_report(2500, 0)),
                    ("DIVS", file_report(1500, 1000)),
                    ("ADD.b", file_report(2500, 0)),
                    ("Bcc", file_report(2000, 0)),
                    ("CHK", file_report(99, 0)),
                    // Absent from the other set entirely: no comparison exists.
                    ("STOP", file_report(1200, 1200)),
                ],
            ),
        ];

        let disagreements = state_disagreements(&reports);
        assert_eq!(
            disagreements
                .iter()
                .map(|entry| entry.file.as_str())
                .collect::<Vec<_>>(),
            vec!["ASR.b"]
        );
        assert_eq!(disagreements[0].disagreeing_set(), "680x0");
    }

    #[test]
    fn state_disagreements_are_empty_unless_both_sets_ran() {
        let only_harte = vec![set_with_files(
            CpuVectorSet::Harte,
            &[("ASR.b", file_report(8000, 1600))],
        )];
        assert!(state_disagreements(&only_harte).is_empty());
    }
}
