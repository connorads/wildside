//! SingleStepTests vector decoding for both published 68000 sets (#447).
//!
//! Two repositories publish per-opcode single-instruction vectors, in two
//! different encodings and two different program-counter conventions. Both are
//! normalised into [`Vector`] here so the harness compares one shape.
//!
//! - `SingleStepTests/680x0` (`68000/v1/<OP>.json.gz`): gzipped Tom Harte-format
//!   JSON. `pc` is the address of the instruction being executed, and the two
//!   prefetch-queue words are NOT in `ram` — they are only in `prefetch`.
//! - `SingleStepTests/m68000` (`v1/<OP>.json.bin`): a bespoke little-endian
//!   binary (upstream `decode.py` is the reference decoder). `pc` is MAME's
//!   `m_au`, the next-prefetch address, which is instruction address + 4. `ram`
//!   holds 16-bit words, and byte transactions post the word-aligned address
//!   with UDS/LDS naming the active half.

use anyhow::{Context, Result, anyhow, bail, ensure};
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Which published set a vector file came from. The variants differ in encoding
/// AND in program-counter convention, so this is not merely provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub(crate) enum CpuVectorSet {
    /// `SingleStepTests/680x0`, Tom Harte lineage.
    Harte,
    /// `SingleStepTests/m68000`, generated from MAME's microcoded core.
    Mame,
}

impl CpuVectorSet {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Harte => "680x0",
            Self::Mame => "m68000",
        }
    }

    /// Directory name under `.cache/references/`.
    pub(crate) fn dir_name(self) -> &'static str {
        match self {
            Self::Harte => "singlesteptests-680x0",
            Self::Mame => "singlesteptests-m68000",
        }
    }

    /// Path of the vector directory relative to the repository root.
    pub(crate) fn vector_dir(self) -> &'static str {
        match self {
            Self::Harte => "68000/v1",
            Self::Mame => "v1",
        }
    }

    pub(crate) fn file_suffix(self) -> &'static str {
        match self {
            Self::Harte => ".json.gz",
            Self::Mame => ".json.bin",
        }
    }

    /// How far the published `pc` sits ahead of the instruction's own address.
    ///
    /// The Harte set publishes the instruction address itself; the MAME set
    /// publishes `m_au`, which is the next prefetch address, four bytes ahead.
    /// Subtracting this bias yields the value r68k's `pc` must hold, because
    /// r68k has no prefetch queue: its `pc` is simply the next word to fetch.
    pub(crate) fn pc_bias(self) -> u32 {
        match self {
            Self::Harte => 0,
            Self::Mame => 4,
        }
    }

    pub(crate) const ALL: [Self; 2] = [Self::Harte, Self::Mame];
}

/// Processor + memory state at one end of a vector.
///
/// `a` holds a0-a6 only; the stack pointer is split into `usp`/`ssp` exactly as
/// published, because which one is live depends on the S bit in `sr`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VectorState {
    pub(crate) d: [u32; 8],
    pub(crate) a: [u32; 7],
    pub(crate) usp: u32,
    pub(crate) ssp: u32,
    pub(crate) sr: u16,
    /// As published — apply [`CpuVectorSet::pc_bias`] to reach r68k's `pc`.
    pub(crate) pc: u32,
    /// Prefetch queue: `[0]` was fetched first and is the instruction word.
    pub(crate) prefetch: [u16; 2],
    /// Sparse `(address, byte)` memory contents.
    pub(crate) ram: Vec<(u32, u8)>,
}

impl VectorState {
    /// The active stack pointer, per the S bit in `sr`.
    pub(crate) fn active_sp(&self) -> u32 {
        if self.supervisor() {
            self.ssp
        } else {
            self.usp
        }
    }

    pub(crate) fn supervisor(&self) -> bool {
        self.sr & 0x2000 != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AccessKind {
    Read,
    Write,
    /// TAS indivisible read-modify-write.
    Tas,
    /// MAME-set only: a read the real chip aborts without asserting AS.
    ReadAddressError,
    /// MAME-set only: a write the real chip aborts without asserting AS.
    WriteAddressError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AccessSize {
    Byte,
    Word,
}

/// One bus transaction, normalised across both sets.
///
/// `addr` is always the *effective byte address* — for MAME byte accesses the
/// posted address has A0 forced low, so the LDS flag supplies bit 0. `value`
/// is likewise normalised to the active bus half (0..=255 for a byte).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BusTransaction {
    pub(crate) kind: AccessKind,
    pub(crate) fc: u8,
    pub(crate) addr: u32,
    pub(crate) size: AccessSize,
    pub(crate) value: u32,
}

impl BusTransaction {
    /// Data-space accesses have FC0 set (user data 1, supervisor data 5).
    ///
    /// Program-space fetches are excluded from bus comparison: r68k prefetches
    /// a longword from `pc & !3` where the real chip fetches words at `pc`, so
    /// the program-fetch sequence is structurally incomparable by design.
    pub(crate) fn is_data_space(self) -> bool {
        self.fc & 1 != 0
    }
}

/// One decoded vector: a single instruction, fully specified at both ends.
#[derive(Debug, Clone)]
pub(crate) struct Vector {
    pub(crate) name: String,
    pub(crate) initial: VectorState,
    pub(crate) expected: VectorState,
    /// Total cycles the publishing core spent, assuming immediate DTACK.
    pub(crate) cycles: u32,
    /// Bus transactions in order; idle (`"n"`) entries are dropped, their
    /// duration already being part of `cycles`.
    pub(crate) transactions: Vec<BusTransaction>,
}

/// The 68000 exception vector table occupies the first 256 longwords.
const VECTOR_TABLE_END: u32 = 0x0400;

impl Vector {
    /// Whether the publishing core dispatched an exception during this vector.
    ///
    /// Read straight off the published transaction list: dispatching an
    /// exception means fetching its vector from the table in supervisor data
    /// space, so a data-space read below [`VECTOR_TABLE_END`] is the signal.
    /// This is a *reporting* partition, not a verdict — a random operand
    /// address inside the table would be misfiled, which is why it never
    /// changes whether a vector passes.
    pub(crate) fn expects_exception(&self) -> bool {
        self.transactions.iter().any(|access| {
            access.kind == AccessKind::Read
                && access.is_data_space()
                && access.addr < VECTOR_TABLE_END
        })
    }

    /// Address of the instruction word, i.e. r68k's initial `pc`.
    pub(crate) fn exec_pc(&self, set: CpuVectorSet) -> u32 {
        self.initial.pc.wrapping_sub(set.pc_bias())
    }

    /// The `pc` r68k must hold after the step.
    pub(crate) fn expected_pc(&self, set: CpuVectorSet) -> u32 {
        self.expected.pc.wrapping_sub(set.pc_bias())
    }
}

/// Enumerate a set's vector files, sorted by name so runs are reproducible.
pub(crate) fn vector_files(
    reference_root: &Path,
    set: CpuVectorSet,
) -> Result<Vec<(String, PathBuf)>> {
    let dir = reference_root.join(set.dir_name()).join(set.vector_dir());
    let entries = std::fs::read_dir(&dir)
        .with_context(|| format!("failed to read vector directory {}", dir.display()))?;
    let mut files = Vec::new();
    for entry in entries {
        let path = entry
            .with_context(|| format!("failed to list {}", dir.display()))?
            .path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if let Some(stem) = name.strip_suffix(set.file_suffix()) {
            files.push((stem.to_owned(), path));
        }
    }
    ensure!(
        !files.is_empty(),
        "no {} vector files in {} — run `mise run bootstrap-cpu-vectors`",
        set.label(),
        dir.display()
    );
    files.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(files)
}

/// Decode one opcode file, keeping at most `limit` vectors from the front.
///
/// The cap takes a prefix of the published order rather than a random sample:
/// the ratchet baseline has to be reproducible, and file order is stable.
pub(crate) fn load_vector_file(
    set: CpuVectorSet,
    path: &Path,
    limit: Option<usize>,
) -> Result<Vec<Vector>> {
    match set {
        CpuVectorSet::Harte => load_harte_file(path, limit),
        CpuVectorSet::Mame => load_mame_file(path, limit),
    }
}

// --- Harte set: gzipped JSON -------------------------------------------------

#[derive(Debug, Deserialize)]
struct HarteVector {
    name: String,
    initial: HarteState,
    #[serde(rename = "final")]
    expected: HarteState,
    length: u32,
    transactions: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
struct HarteState {
    d0: u32,
    d1: u32,
    d2: u32,
    d3: u32,
    d4: u32,
    d5: u32,
    d6: u32,
    d7: u32,
    a0: u32,
    a1: u32,
    a2: u32,
    a3: u32,
    a4: u32,
    a5: u32,
    a6: u32,
    usp: u32,
    ssp: u32,
    sr: u32,
    pc: u32,
    prefetch: [u32; 2],
    ram: Vec<(u32, u32)>,
}

impl From<HarteState> for VectorState {
    fn from(state: HarteState) -> Self {
        Self {
            d: [
                state.d0, state.d1, state.d2, state.d3, state.d4, state.d5, state.d6, state.d7,
            ],
            a: [
                state.a0, state.a1, state.a2, state.a3, state.a4, state.a5, state.a6,
            ],
            usp: state.usp,
            ssp: state.ssp,
            sr: state.sr as u16,
            pc: state.pc,
            prefetch: [state.prefetch[0] as u16, state.prefetch[1] as u16],
            ram: state
                .ram
                .into_iter()
                .map(|(addr, value)| (addr, value as u8))
                .collect(),
        }
    }
}

fn load_harte_file(path: &Path, limit: Option<usize>) -> Result<Vec<Vector>> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut text = String::new();
    GzDecoder::new(file)
        .read_to_string(&mut text)
        .with_context(|| format!("failed to gunzip {}", path.display()))?;
    let raw: Vec<HarteVector> = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse {}", path.display()))?;

    raw.into_iter()
        .take(limit.unwrap_or(usize::MAX))
        .map(|vector| {
            let transactions = vector
                .transactions
                .iter()
                .filter_map(|entry| parse_harte_transaction(entry).transpose())
                .collect::<Result<Vec<_>>>()
                .with_context(|| format!("{}: {}", path.display(), vector.name))?;
            Ok(Vector {
                name: vector.name,
                initial: vector.initial.into(),
                expected: vector.expected.into(),
                cycles: vector.length,
                transactions,
            })
        })
        .collect()
}

/// `["n", cycles]` -> `None`; `[kind, cycles, fc, addr, size, value]` -> access.
fn parse_harte_transaction(entry: &[serde_json::Value]) -> Result<Option<BusTransaction>> {
    let kind = entry
        .first()
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("transaction has no kind"))?;
    if kind == "n" {
        return Ok(None);
    }
    ensure!(
        entry.len() == 6,
        "expected 6 fields in a {kind} transaction, got {}",
        entry.len()
    );
    let kind = match kind {
        "r" => AccessKind::Read,
        "w" => AccessKind::Write,
        "t" => AccessKind::Tas,
        other => bail!("unknown transaction kind {other:?}"),
    };
    let number = |index: usize| -> Result<u32> {
        entry[index]
            .as_u64()
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| anyhow!("transaction field {index} is not a u32"))
    };
    let size = match entry[4].as_str() {
        Some(".b") => AccessSize::Byte,
        Some(".w") => AccessSize::Word,
        other => bail!("unknown access size {other:?}"),
    };
    Ok(Some(BusTransaction {
        kind,
        fc: number(2)? as u8,
        addr: number(3)?,
        size,
        value: number(5)?,
    }))
}

// --- MAME set: bespoke little-endian binary ---------------------------------

const MAME_FILE_MAGIC: u32 = 0x1A3F_5D71;
const MAME_TEST_MAGIC: u32 = 0xABC1_2367;
const MAME_NAME_MAGIC: u32 = 0x89AB_CDEF;
const MAME_STATE_MAGIC: u32 = 0x0123_4567;
const MAME_TRANSACTIONS_MAGIC: u32 = 0x4567_89AB;

/// Sequential little-endian reader over an in-memory `.json.bin`.
struct MameReader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> MameReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    fn u32(&mut self) -> Result<u32> {
        let end = self.at + 4;
        let slice = self
            .bytes
            .get(self.at..end)
            .ok_or_else(|| anyhow!("truncated at byte {}", self.at))?;
        self.at = end;
        Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
    }

    fn u16(&mut self) -> Result<u16> {
        let end = self.at + 2;
        let slice = self
            .bytes
            .get(self.at..end)
            .ok_or_else(|| anyhow!("truncated at byte {}", self.at))?;
        self.at = end;
        Ok(u16::from_le_bytes([slice[0], slice[1]]))
    }

    fn u8(&mut self) -> Result<u8> {
        let byte = *self
            .bytes
            .get(self.at)
            .ok_or_else(|| anyhow!("truncated at byte {}", self.at))?;
        self.at += 1;
        Ok(byte)
    }

    /// Every record is prefixed with a length and a magic number; the length is
    /// unused by this decoder (records are read field by field, as upstream's
    /// `decode.py` does) but the magic is asserted to catch a format change.
    fn record_header(&mut self, expected: u32) -> Result<()> {
        let _length = self.u32()?;
        let magic = self.u32()?;
        ensure!(
            magic == expected,
            "expected record magic {expected:#010x}, got {magic:#010x}"
        );
        Ok(())
    }

    fn name(&mut self) -> Result<String> {
        self.record_header(MAME_NAME_MAGIC)?;
        let length = self.u32()? as usize;
        let end = self.at + length;
        let slice = self
            .bytes
            .get(self.at..end)
            .ok_or_else(|| anyhow!("truncated name at byte {}", self.at))?;
        self.at = end;
        Ok(String::from_utf8_lossy(slice).into_owned())
    }

    fn state(&mut self) -> Result<VectorState> {
        self.record_header(MAME_STATE_MAGIC)?;
        let mut d = [0u32; 8];
        for register in &mut d {
            *register = self.u32()?;
        }
        let mut a = [0u32; 7];
        for register in &mut a {
            *register = self.u32()?;
        }
        let usp = self.u32()?;
        let ssp = self.u32()?;
        let sr = self.u32()? as u16;
        let pc = self.u32()?;
        let prefetch = [self.u32()? as u16, self.u32()? as u16];

        let word_count = self.u32()? as usize;
        let mut ram = Vec::with_capacity(word_count * 2);
        for _ in 0..word_count {
            let addr = self.u32()?;
            let word = self.u16()?;
            ensure!(addr < 0x0100_0000, "ram address {addr:#x} exceeds 16 MiB");
            ram.push((addr, (word >> 8) as u8));
            ram.push((addr | 1, (word & 0xff) as u8));
        }

        Ok(VectorState {
            d,
            a,
            usp,
            ssp,
            sr,
            pc,
            prefetch,
            ram,
        })
    }

    fn transactions(&mut self) -> Result<(Vec<BusTransaction>, u32)> {
        self.record_header(MAME_TRANSACTIONS_MAGIC)?;
        let cycles = self.u32()?;
        let count = self.u32()? as usize;
        let mut transactions = Vec::with_capacity(count);
        for _ in 0..count {
            let tag = self.u8()?;
            let _cycles = self.u32()?;
            if tag == 0 {
                continue; // idle
            }
            let fc = self.u32()?;
            let posted_addr = self.u32()?;
            let data_bus = self.u32()?;
            let upper = self.u32()?;
            let lower = self.u32()?;
            let kind = match tag {
                1 => AccessKind::Write,
                2 => AccessKind::Read,
                3 => AccessKind::Tas,
                4 => AccessKind::ReadAddressError,
                5 => AccessKind::WriteAddressError,
                other => bail!("unknown MAME transaction tag {other}"),
            };
            // The real chip has no A0 line: a byte access posts the even address
            // and asserts one data strobe. UDS selects the high half.
            let (size, addr, value) = match (upper, lower) {
                (1, 1) => (AccessSize::Word, posted_addr, data_bus & 0xffff),
                (1, 0) => (AccessSize::Byte, posted_addr, (data_bus >> 8) & 0xff),
                (0, 1) => (AccessSize::Byte, posted_addr | 1, data_bus & 0xff),
                (upper, lower) => bail!("no data strobe asserted (UDS {upper}, LDS {lower})"),
            };
            transactions.push(BusTransaction {
                kind,
                fc: fc as u8,
                addr,
                size,
                value,
            });
        }
        Ok((transactions, cycles))
    }
}

fn load_mame_file(path: &Path, limit: Option<usize>) -> Result<Vec<Vector>> {
    let bytes =
        std::fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut reader = MameReader::new(&bytes);
    let magic = reader.u32()?;
    ensure!(
        magic == MAME_FILE_MAGIC,
        "{}: expected file magic {MAME_FILE_MAGIC:#010x}, got {magic:#010x}",
        path.display()
    );
    let count = reader.u32()? as usize;
    let wanted = limit.unwrap_or(usize::MAX).min(count);

    let mut vectors = Vec::with_capacity(wanted);
    for index in 0..wanted {
        reader
            .record_header(MAME_TEST_MAGIC)
            .with_context(|| format!("{}: test {index}", path.display()))?;
        let name = reader.name()?;
        let initial = reader.state()?;
        let expected = reader.state()?;
        let (transactions, cycles) = reader.transactions()?;
        vectors.push(Vector {
            name,
            initial,
            expected,
            cycles,
            transactions,
        });
    }
    Ok(vectors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pc_bias_maps_each_sets_published_pc_onto_r68k_pc() {
        // Harte publishes the instruction address; MAME publishes m_au, which
        // sits four bytes ahead (README: "it's +4 from where the test starts").
        assert_eq!(CpuVectorSet::Harte.pc_bias(), 0);
        assert_eq!(CpuVectorSet::Mame.pc_bias(), 4);
    }

    fn state_with(sr: u16) -> VectorState {
        VectorState {
            d: [0; 8],
            a: [0; 7],
            usp: 0x1000,
            ssp: 0x2000,
            sr,
            pc: 0x0c00,
            prefetch: [0x4e71, 0x4e71],
            ram: Vec::new(),
        }
    }

    #[test]
    fn active_stack_pointer_follows_the_supervisor_bit() {
        assert_eq!(state_with(0x2700).active_sp(), 0x2000);
        assert_eq!(state_with(0x0700).active_sp(), 0x1000);
        assert!(state_with(0x2700).supervisor());
        assert!(!state_with(0x0700).supervisor());
    }

    #[test]
    fn exec_pc_and_expected_pc_apply_the_set_bias() {
        let vector = Vector {
            name: "probe".to_owned(),
            initial: state_with(0x2700),
            expected: state_with(0x2700),
            cycles: 4,
            transactions: Vec::new(),
        };
        assert_eq!(vector.exec_pc(CpuVectorSet::Harte), 0x0c00);
        assert_eq!(vector.exec_pc(CpuVectorSet::Mame), 0x0bfc);
        assert_eq!(vector.expected_pc(CpuVectorSet::Mame), 0x0bfc);
    }

    /// The exception partition is read off the published transaction list, so
    /// only a supervisor/user *data* read inside the vector table counts — a
    /// program fetch from low memory is ordinary execution.
    #[test]
    fn exception_dispatch_is_detected_from_a_vector_table_data_read() {
        let read = |fc, addr| BusTransaction {
            kind: AccessKind::Read,
            fc,
            addr,
            size: AccessSize::Word,
            value: 0,
        };
        let vector_with = |transactions: Vec<BusTransaction>| Vector {
            name: "probe".to_owned(),
            initial: state_with(0x2700),
            expected: state_with(0x2700),
            cycles: 4,
            transactions,
        };

        // Address-error vector 3 lives at 0x0C, fetched in supervisor data.
        assert!(vector_with(vec![read(5, 0x0c)]).expects_exception());
        // Ordinary operand read well above the table.
        assert!(!vector_with(vec![read(5, 0x4000)]).expects_exception());
        // Program fetch from low memory is not a dispatch.
        assert!(!vector_with(vec![read(6, 0x0c)]).expects_exception());
        assert!(!vector_with(Vec::new()).expects_exception());
    }

    #[test]
    fn harte_idle_transactions_are_dropped_and_accesses_normalised() {
        let idle: Vec<serde_json::Value> = serde_json::from_str(r#"["n", 122]"#).expect("idle");
        assert_eq!(parse_harte_transaction(&idle).expect("idle parses"), None);

        let read: Vec<serde_json::Value> =
            serde_json::from_str(r#"["r", 4, 6, 3076, ".w", 10786]"#).expect("read");
        assert_eq!(
            parse_harte_transaction(&read).expect("read parses"),
            Some(BusTransaction {
                kind: AccessKind::Read,
                fc: 6,
                addr: 3076,
                size: AccessSize::Word,
                value: 10786,
            })
        );
    }

    #[test]
    fn program_space_function_codes_are_excluded_from_data_comparison() {
        // FC 2 = user program, 6 = supervisor program: FC0 clear.
        for (fc, is_data) in [(1, true), (2, false), (5, true), (6, false)] {
            let transaction = BusTransaction {
                kind: AccessKind::Read,
                fc,
                addr: 0,
                size: AccessSize::Word,
                value: 0,
            };
            assert_eq!(transaction.is_data_space(), is_data, "fc {fc}");
        }
    }

    #[test]
    fn harte_transaction_parse_rejects_unknown_kinds_and_short_rows() {
        let short: Vec<serde_json::Value> =
            serde_json::from_str(r#"["r", 4, 6, 3076]"#).expect("short");
        assert!(parse_harte_transaction(&short).is_err());

        let unknown: Vec<serde_json::Value> =
            serde_json::from_str(r#"["q", 4, 6, 3076, ".w", 1]"#).expect("unknown");
        assert!(parse_harte_transaction(&unknown).is_err());
    }
}
