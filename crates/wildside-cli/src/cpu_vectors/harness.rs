//! Drive `r68k-emu` through one SingleStepTests vector and grade the result (#447).
//!
//! # Why this is an honest harness and not a faked one
//!
//! r68k exposes everything a per-instruction conformance run needs: `mem` is any
//! [`AddressBus`] implementation, every register/flag field on `ConfiguredCore`
//! is public, and `execute1()` runs exactly one instruction and returns the
//! cycles it charged. So state is injected and read back directly, with no
//! emulator-side stubbing.
//!
//! One structural mismatch is unavoidable and is treated as a scoped deviation
//! rather than hidden: **r68k has no prefetch queue.** Its `read_imm_u16` caches
//! a longword from `pc & !3` and re-derives words from it, whereas the real
//! 68000 holds two separately-fetched words (IRD/IRC) that the vectors publish
//! as `prefetch`. There is no seedable queue, so the harness writes those two
//! words into memory at the instruction address instead. That reconstructs real
//! memory rather than inventing it — on the publishing machine `prefetch[0]`
//! *is* the word at `pc` and `prefetch[1]` the word at `pc + 2` (the MAME set
//! lists both in `ram` too, and [`VectorOutcome::prefetch_ram_conflict`] counts
//! any vector where they disagree). Two consequences are reported, never masked:
//! the final prefetch queue is not comparable at all, and the *program*-space
//! fetch sequence is not comparable either, so bus comparison covers data-space
//! accesses only.

use std::cell::RefCell;

use wildside_md::r68k_emu::cpu::{ConfiguredCore, ProcessingState};
use wildside_md::r68k_emu::interrupts::AutoInterruptController;
use wildside_md::r68k_emu::ram::{ADDRBUS_MASK, AddressBus, AddressSpace};

use super::vector::{AccessKind, AccessSize, BusTransaction, CpuVectorSet, Vector};

/// 68000 external address bus: 24 bits, 16 MiB.
const RAM_LEN: usize = ADDRBUS_MASK as usize + 1;

/// A flat 16 MiB bus that records every access r68k performs.
///
/// Reused across vectors: `journal` remembers written addresses so a reset costs
/// the handful of bytes a vector touched rather than 16 MiB. Unspecified memory
/// reads as zero; [`VectorOutcome::read_outside_vector`] counts reads of
/// addresses the vector never listed, which is how a wrong effective address
/// shows up even when the final state happens to match.
pub(crate) struct VectorBus {
    ram: Box<[u8]>,
    journal: Vec<u32>,
    log: RefCell<Vec<BusTransaction>>,
    recording: bool,
}

impl VectorBus {
    pub(crate) fn new() -> Self {
        Self {
            ram: vec![0u8; RAM_LEN].into_boxed_slice(),
            journal: Vec::new(),
            log: RefCell::new(Vec::new()),
            recording: false,
        }
    }

    /// Store a setup byte: journalled for reset, but not logged as a bus access.
    fn poke(&mut self, addr: u32, value: u8) {
        let index = (addr & ADDRBUS_MASK) as usize;
        self.ram[index] = value;
        self.journal.push(addr & ADDRBUS_MASK);
    }

    fn poke_word(&mut self, addr: u32, value: u16) {
        self.poke(addr, (value >> 8) as u8);
        self.poke(addr.wrapping_add(1), (value & 0xff) as u8);
    }

    pub(crate) fn peek(&self, addr: u32) -> u8 {
        self.ram[(addr & ADDRBUS_MASK) as usize]
    }

    /// Return every byte the vector touched to zero and clear the access log.
    fn reset(&mut self) {
        for addr in self.journal.drain(..) {
            self.ram[addr as usize] = 0;
        }
        self.log.borrow_mut().clear();
        self.recording = false;
    }

    fn record(&self, transaction: BusTransaction) {
        if self.recording {
            self.log.borrow_mut().push(transaction);
        }
    }

    fn byte(&self, addr: u32) -> u32 {
        u32::from(self.ram[(addr & ADDRBUS_MASK) as usize])
    }

    fn set_byte(&mut self, addr: u32, value: u32) {
        let index = (addr & ADDRBUS_MASK) as usize;
        self.ram[index] = value as u8;
        if self.recording {
            self.journal.push(addr & ADDRBUS_MASK);
        }
    }
}

fn function_code(space: AddressSpace) -> u8 {
    space.fc() as u8
}

/// Longword accesses are split into two word transactions.
///
/// The real chip has a 16-bit data bus and never performs a 32-bit cycle, so
/// splitting is what makes r68k's coarser interface comparable to the published
/// transaction lists at all.
impl AddressBus for VectorBus {
    fn copy_from(&mut self, other: &Self) {
        self.ram.copy_from_slice(&other.ram);
    }

    fn read_byte(&self, address_space: AddressSpace, address: u32) -> u32 {
        let value = self.byte(address);
        self.record(BusTransaction {
            kind: AccessKind::Read,
            fc: function_code(address_space),
            addr: address & ADDRBUS_MASK,
            size: AccessSize::Byte,
            value,
        });
        value
    }

    fn read_word(&self, address_space: AddressSpace, address: u32) -> u32 {
        let value = (self.byte(address) << 8) | self.byte(address.wrapping_add(1));
        self.record(BusTransaction {
            kind: AccessKind::Read,
            fc: function_code(address_space),
            addr: address & ADDRBUS_MASK,
            size: AccessSize::Word,
            value,
        });
        value
    }

    fn read_long(&self, address_space: AddressSpace, address: u32) -> u32 {
        let high = self.read_word(address_space, address);
        let low = self.read_word(address_space, address.wrapping_add(2));
        (high << 16) | low
    }

    fn write_byte(&mut self, address_space: AddressSpace, address: u32, value: u32) {
        let value = value & 0xff;
        self.set_byte(address, value);
        self.record(BusTransaction {
            kind: AccessKind::Write,
            fc: function_code(address_space),
            addr: address & ADDRBUS_MASK,
            size: AccessSize::Byte,
            value,
        });
    }

    fn write_word(&mut self, address_space: AddressSpace, address: u32, value: u32) {
        let value = value & 0xffff;
        self.set_byte(address, value >> 8);
        self.set_byte(address.wrapping_add(1), value & 0xff);
        self.record(BusTransaction {
            kind: AccessKind::Write,
            fc: function_code(address_space),
            addr: address & ADDRBUS_MASK,
            size: AccessSize::Word,
            value,
        });
    }

    fn write_long(&mut self, address_space: AddressSpace, address: u32, value: u32) {
        self.write_word(address_space, address, value >> 16);
        self.write_word(address_space, address.wrapping_add(2), value & 0xffff);
    }
}

/// Which published fields disagreed with r68k. Empty means a state pass.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct StateMismatch {
    pub(crate) data_registers: bool,
    pub(crate) address_registers: bool,
    pub(crate) user_stack_pointer: bool,
    pub(crate) supervisor_stack_pointer: bool,
    pub(crate) program_counter: bool,
    /// SR differs somewhere other than the trace bit.
    pub(crate) status_register: bool,
    /// SR differs *only* in T. r68k models no trace bit and no trace exception,
    /// so this is one known gap rather than an arithmetic defect, and is counted
    /// apart from `status_register` to keep the rest of the SR verdict readable.
    pub(crate) trace_bit_only: bool,
    pub(crate) memory: bool,
}

impl StateMismatch {
    pub(crate) fn is_pass(self) -> bool {
        self == Self::default()
    }

    /// True when the only disagreement is r68k's missing trace bit.
    pub(crate) fn is_trace_bit_only(self) -> bool {
        self.trace_bit_only
            && Self {
                trace_bit_only: false,
                ..self
            } == Self::default()
    }
}

/// Graded result of one vector.
#[derive(Debug, Clone)]
pub(crate) struct VectorOutcome {
    pub(crate) mismatch: StateMismatch,
    pub(crate) expected_cycles: u32,
    pub(crate) actual_cycles: i32,
    /// Data-space accesses: r68k's sequence equals the published one.
    pub(crate) data_bus_match: bool,
    /// The vector's own `ram` disagreed with its `prefetch` at the instruction
    /// address, so the injected instruction words are not a faithful
    /// reconstruction of the publishing machine's memory.
    pub(crate) prefetch_ram_conflict: bool,
    /// r68k read an address the vector never specified.
    pub(crate) read_outside_vector: bool,
    /// r68k panicked instead of raising an exception. Several of its stack and
    /// vector helpers (`pop_32`, `jump_vector`, `push_sp`) `unwrap()` a
    /// `read_data_long`/`write_data_long` result, so an odd stack pointer or
    /// vector address aborts the process rather than taking the address-error
    /// exception the real chip takes. Always a state failure.
    pub(crate) panicked: bool,
    /// The instruction word, for per-opcode aggregation.
    pub(crate) opcode: u16,
}

impl VectorOutcome {
    pub(crate) fn cycles_match(&self) -> bool {
        i64::from(self.actual_cycles) == i64::from(self.expected_cycles)
    }
}

/// A reusable r68k core plus its bus.
///
/// Reuse is not an optimisation detail: `ConfiguredCore::new_with` builds the
/// 65,536-entry dispatch table on every call, which would dominate a
/// million-vector run.
pub(crate) struct VectorHarness {
    core: ConfiguredCore<AutoInterruptController, VectorBus>,
}

impl VectorHarness {
    pub(crate) fn new() -> Self {
        Self {
            core: ConfiguredCore::new_with(0, AutoInterruptController::new(), VectorBus::new()),
        }
    }

    /// Load a vector, step one instruction, and grade the outcome.
    pub(crate) fn run(&mut self, set: CpuVectorSet, vector: &Vector) -> VectorOutcome {
        self.core.mem.reset();

        let exec_pc = vector.exec_pc(set);
        for &(addr, value) in &vector.initial.ram {
            self.core.mem.poke(addr, value);
        }
        // See the module docs: no seedable prefetch queue, so the queue's two
        // words are reconstructed in memory at the instruction address.
        let prefetch_ram_conflict = vector.initial.ram.iter().any(|&(addr, value)| {
            let offset = addr.wrapping_sub(exec_pc);
            match offset {
                0 => value != (vector.initial.prefetch[0] >> 8) as u8,
                1 => value != (vector.initial.prefetch[0] & 0xff) as u8,
                2 => value != (vector.initial.prefetch[1] >> 8) as u8,
                3 => value != (vector.initial.prefetch[1] & 0xff) as u8,
                _ => false,
            }
        });
        self.core.mem.poke_word(exec_pc, vector.initial.prefetch[0]);
        self.core
            .mem
            .poke_word(exec_pc.wrapping_add(2), vector.initial.prefetch[1]);

        self.core.processing_state = ProcessingState::Normal;
        self.core.irq_level = 0;
        self.core.int_ctrl = AutoInterruptController::new();
        // `prefetch_addr` is odd, so `pc & !3` can never match it and the first
        // instruction fetch always goes to the bus (this is how `reset()` does
        // it upstream).
        self.core.prefetch_addr = 1;
        self.core.prefetch_data = 0;
        self.core.ir = 0;

        for (index, value) in vector.initial.d.iter().enumerate() {
            self.core.dar[index] = *value;
        }
        for (index, value) in vector.initial.a.iter().enumerate() {
            self.core.dar[8 + index] = *value;
        }
        // Set the flags first: `sr_to_flags` swaps the stack pointers when S
        // changes, so all three pointer fields must be assigned afterwards.
        // Both inactive pointers are written even though only one can be read
        // back, because an instruction may switch mode mid-step (RTE, MOVE to
        // SR) and then swap in whichever one it finds.
        self.core.sr_to_flags(vector.initial.sr);
        self.core.inactive_usp = vector.initial.usp;
        self.core.inactive_ssp = vector.initial.ssp;
        self.core.dar[15] = vector.initial.active_sp();
        self.core.pc = exec_pc;

        self.core.mem.recording = true;
        // r68k `unwrap()`s inside several exception helpers, so a vector that
        // pushes the core onto one of those paths aborts the thread. Catching
        // it turns an emulator defect into a reportable outcome instead of
        // ending the run; the next `run` resets bus and registers wholesale, so
        // no state survives the unwind.
        let stepped =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.core.execute1()));
        self.core.mem.recording = false;
        let (cycles, panicked) = match stepped {
            Ok(cycles) => (cycles.0, false),
            Err(_) => (0, true),
        };

        let mismatch = self.grade_state(set, vector);
        let log = self.core.mem.log.borrow();
        let data_bus_match = data_accesses(&log).eq(data_accesses(&vector.transactions));
        let read_outside_vector = log
            .iter()
            .filter(|access| access.kind == AccessKind::Read && access.is_data_space())
            .any(|access| !covers_specified_byte(vector, *access));

        VectorOutcome {
            mismatch,
            expected_cycles: vector.cycles,
            actual_cycles: cycles,
            data_bus_match,
            prefetch_ram_conflict,
            read_outside_vector,
            panicked,
            opcode: vector.initial.prefetch[0],
        }
    }

    fn grade_state(&self, set: CpuVectorSet, vector: &Vector) -> StateMismatch {
        let expected = &vector.expected;
        let actual_sr = self.core.status_register();
        // r68k stores no trace bit, so compare SR without it and account for T
        // separately.
        let sr_differs_below_trace = (actual_sr & 0x7fff) != (expected.sr & 0x7fff);
        let trace_differs = (actual_sr & 0x8000) != (expected.sr & 0x8000);

        StateMismatch {
            data_registers: (0..8).any(|index| self.core.dar[index] != expected.d[index]),
            address_registers: (0..7).any(|index| self.core.dar[8 + index] != expected.a[index]),
            user_stack_pointer: self.core.usp() != expected.usp,
            supervisor_stack_pointer: self.core.ssp() != expected.ssp,
            program_counter: self.core.pc != vector.expected_pc(set),
            status_register: sr_differs_below_trace,
            trace_bit_only: trace_differs && !sr_differs_below_trace,
            memory: expected
                .ram
                .iter()
                .any(|&(addr, value)| self.core.mem.peek(addr) != value),
        }
    }
}

/// Data-space reads and writes only, in order — see the module docs on why
/// program-space fetches are excluded.
fn data_accesses(transactions: &[BusTransaction]) -> impl Iterator<Item = BusTransaction> + '_ {
    transactions
        .iter()
        .copied()
        .filter(|access| access.is_data_space())
}

/// Whether any byte the access covers appears in the vector's initial memory.
///
/// A read that covers nothing the vector specified means r68k computed an
/// effective address the publishing core never visited — visible even when the
/// final state happens to agree.
fn covers_specified_byte(vector: &Vector, access: BusTransaction) -> bool {
    let width = match access.size {
        AccessSize::Byte => 1,
        AccessSize::Word => 2,
    };
    (0..width).any(|offset| {
        let addr = access.addr.wrapping_add(offset) & ADDRBUS_MASK;
        vector
            .initial
            .ram
            .iter()
            .any(|&(listed, _)| listed & ADDRBUS_MASK == addr)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu_vectors::vector::VectorState;

    fn state(pc: u32, sr: u16, opcode: u16, next: u16) -> VectorState {
        VectorState {
            d: [0; 8],
            a: [0; 7],
            usp: 0x0001_0000,
            ssp: 0x0000_2000,
            sr,
            pc,
            prefetch: [opcode, next],
            ram: Vec::new(),
        }
    }

    /// `NOP` (0x4E71) is 4 cycles, touches nothing, and advances PC by 2 — the
    /// smallest end-to-end proof that the harness injects and reads back state.
    #[test]
    fn harness_steps_a_nop_and_grades_it_as_a_state_pass() {
        let mut vector = Vector {
            name: "4e71 [NOP]".to_owned(),
            initial: state(0x0c00, 0x2700, 0x4e71, 0x4e71),
            expected: state(0x0c02, 0x2700, 0x4e71, 0x4e71),
            cycles: 4,
            transactions: Vec::new(),
        };
        vector.initial.ram = vec![(0x0c04, 0x4e), (0x0c05, 0x71)];
        vector.expected.ram = vector.initial.ram.clone();

        let mut harness = VectorHarness::new();
        let outcome = harness.run(CpuVectorSet::Harte, &vector);

        assert!(
            outcome.mismatch.is_pass(),
            "unexpected mismatch: {:?}",
            outcome.mismatch
        );
        assert_eq!(outcome.actual_cycles, 4);
        assert!(outcome.cycles_match());
        assert!(!outcome.prefetch_ram_conflict);
        assert_eq!(outcome.opcode, 0x4e71);
    }

    /// `MOVE.w #$1234,(A0)+` writes through the bus, so this pins the write
    /// transaction shape and the final-memory comparison at once.
    #[test]
    fn harness_records_the_data_write_of_a_move_to_memory() {
        let mut vector = Vector {
            name: "30fc [MOVE.w #$1234, (A0)+]".to_owned(),
            initial: state(0x0c00, 0x2700, 0x30fc, 0x1234),
            // $1234 is positive and non-zero, so MOVE leaves CCR clear.
            expected: state(0x0c04, 0x2700, 0x30fc, 0x1234),
            cycles: 12,
            transactions: vec![BusTransaction {
                kind: AccessKind::Write,
                fc: 5,
                addr: 0x4000,
                size: AccessSize::Word,
                value: 0x1234,
            }],
        };
        vector.initial.a[0] = 0x4000;
        vector.expected.a[0] = 0x4002;
        vector.expected.ram = vec![(0x4000, 0x12), (0x4001, 0x34)];

        let mut harness = VectorHarness::new();
        let outcome = harness.run(CpuVectorSet::Harte, &vector);

        assert!(
            outcome.mismatch.is_pass(),
            "unexpected mismatch: {:?}",
            outcome.mismatch
        );
        assert!(outcome.data_bus_match, "write transaction should match");
    }

    /// A wrong expectation must be reported per field, not collapsed into one
    /// boolean — otherwise "worst opcode families" carries no diagnosis.
    #[test]
    fn grading_names_the_fields_that_disagree() {
        let mut vector = Vector {
            name: "4e71 [NOP] mis-expected".to_owned(),
            initial: state(0x0c00, 0x2700, 0x4e71, 0x4e71),
            expected: state(0x0c02, 0x2700, 0x4e71, 0x4e71),
            cycles: 4,
            transactions: Vec::new(),
        };
        vector.expected.d[3] = 0xdead_beef;
        vector.expected.a[2] = 0x1234;
        vector.expected.pc = 0x0c08;
        vector.expected.ram = vec![(0x8000, 0x99)];

        let mut harness = VectorHarness::new();
        let mismatch = harness.run(CpuVectorSet::Harte, &vector).mismatch;

        assert!(mismatch.data_registers);
        assert!(mismatch.address_registers);
        assert!(mismatch.program_counter);
        assert!(mismatch.memory);
        assert!(!mismatch.status_register);
        assert!(!mismatch.is_pass());
    }

    /// r68k drops the trace bit entirely; roughly half the MAME set sets it, so
    /// the classification has to separate that gap from real SR defects.
    #[test]
    fn trace_bit_only_divergence_is_classified_apart_from_other_sr_bits() {
        let mut vector = Vector {
            name: "4e71 [NOP] with T set".to_owned(),
            initial: state(0x0c00, 0xa700, 0x4e71, 0x4e71),
            expected: state(0x0c02, 0xa700, 0x4e71, 0x4e71),
            cycles: 4,
            transactions: Vec::new(),
        };
        vector.initial.ram = vec![(0x0c04, 0x4e), (0x0c05, 0x71)];
        vector.expected.ram = vector.initial.ram.clone();

        let mut harness = VectorHarness::new();
        let mismatch = harness.run(CpuVectorSet::Harte, &vector).mismatch;

        assert!(mismatch.trace_bit_only);
        assert!(!mismatch.status_register);
        assert!(mismatch.is_trace_bit_only());
        assert!(!mismatch.is_pass());
    }

    /// The bus is reused across vectors, so a stale byte from an earlier vector
    /// would silently satisfy a later memory comparison.
    #[test]
    fn bus_reset_clears_bytes_written_by_the_previous_vector() {
        let mut first = Vector {
            name: "30fc [MOVE.w #$1234, (A0)+]".to_owned(),
            initial: state(0x0c00, 0x2700, 0x30fc, 0x1234),
            // $1234 is positive and non-zero, so MOVE leaves CCR clear.
            expected: state(0x0c04, 0x2700, 0x30fc, 0x1234),
            cycles: 12,
            transactions: Vec::new(),
        };
        first.initial.a[0] = 0x4000;
        first.expected.a[0] = 0x4002;

        let mut harness = VectorHarness::new();
        harness.run(CpuVectorSet::Harte, &first);
        assert_eq!(harness.core.mem.peek(0x4000), 0x12);

        let second = Vector {
            name: "4e71 [NOP]".to_owned(),
            initial: state(0x0c00, 0x2700, 0x4e71, 0x4e71),
            expected: state(0x0c02, 0x2700, 0x4e71, 0x4e71),
            cycles: 4,
            transactions: Vec::new(),
        };
        harness.run(CpuVectorSet::Harte, &second);
        assert_eq!(
            harness.core.mem.peek(0x4000),
            0,
            "the previous vector's write must not survive"
        );
    }

    /// The MAME set publishes `pc` four bytes ahead, so the bias has to be
    /// applied on both the injected and the expected side.
    #[test]
    fn mame_pc_bias_is_applied_to_both_ends_of_the_step() {
        let mut vector = Vector {
            name: "4e71 [NOP] mame".to_owned(),
            initial: state(0x0c04, 0x2700, 0x4e71, 0x4e71),
            expected: state(0x0c06, 0x2700, 0x4e71, 0x4e71),
            cycles: 4,
            transactions: Vec::new(),
        };
        vector.initial.ram = vec![
            (0x0c00, 0x4e),
            (0x0c01, 0x71),
            (0x0c02, 0x4e),
            (0x0c03, 0x71),
        ];
        vector.expected.ram = vector.initial.ram.clone();

        let mut harness = VectorHarness::new();
        let outcome = harness.run(CpuVectorSet::Mame, &vector);

        assert!(
            outcome.mismatch.is_pass(),
            "unexpected mismatch: {:?}",
            outcome.mismatch
        );
        assert!(!outcome.prefetch_ram_conflict);
    }

    /// `RTS` with an odd stack pointer reaches `pop_32`, which `unwrap()`s the
    /// address error instead of raising it. The harness must survive that and
    /// classify it, or one bad vector ends a million-vector run.
    #[test]
    fn a_panicking_exception_path_is_caught_and_classified() {
        let mut vector = Vector {
            name: "4e75 [RTS] odd sp".to_owned(),
            initial: state(0x0c00, 0x2700, 0x4e75, 0x4e71),
            expected: state(0x0c02, 0x2700, 0x4e71, 0x4e71),
            cycles: 16,
            transactions: Vec::new(),
        };
        vector.initial.ssp = 0x2001;
        vector.expected.ssp = 0x2001;

        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let mut harness = VectorHarness::new();
        let outcome = harness.run(CpuVectorSet::Harte, &vector);
        std::panic::set_hook(previous_hook);

        assert!(outcome.panicked, "r68k pop_32 should abort on an odd SP");
        assert_eq!(outcome.actual_cycles, 0);
    }

    /// A vector whose `ram` contradicts its own `prefetch` cannot be
    /// reconstructed faithfully; that must be surfaced, not absorbed.
    #[test]
    fn prefetch_ram_conflict_is_reported() {
        let mut vector = Vector {
            name: "4e71 [NOP] contradictory".to_owned(),
            initial: state(0x0c00, 0x2700, 0x4e71, 0x4e71),
            expected: state(0x0c02, 0x2700, 0x4e71, 0x4e71),
            cycles: 4,
            transactions: Vec::new(),
        };
        vector.initial.ram = vec![(0x0c00, 0x00)];

        let mut harness = VectorHarness::new();
        assert!(
            harness
                .run(CpuVectorSet::Harte, &vector)
                .prefetch_ram_conflict
        );
    }
}
