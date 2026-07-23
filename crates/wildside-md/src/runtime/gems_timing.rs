use super::{
    GEMS_DRIVER_ADDR, GEMS_DRIVER_END_ADDR, GEMS_ENVELOPE_ADDR, GEMS_PATCH_ADDR, GEMS_QUEUE_ADDR,
    GEMS_QUEUE_PTR_ADDR, GEMS_QUEUE_SIZE, GEMS_SAMPLE_ADDR, GEMS_SEQUENCE_ADDR, Ym2612Shadow,
    Z80_RAM_LEN,
};
use anyhow::{Result, ensure};
use std::cell::{Cell, RefCell};
use z80::{Z80, Z80_io};

const M68K_DIVIDER: u64 = 7;
const Z80_DIVIDER: u64 = 15;
// Ym2612Shadow accepts raw 7.67 MHz input clocks (master / 7), while the
// jgenesis machine scheduler retains its externally visible YM tick phase
// modulo 42 master clocks. These are distinct clocks.
const YM2612_INPUT_DIVIDER: u64 = 7;
const YM2612_SCHEDULER_DIVIDER: u64 = 7 * 6;
const MASTER_CLOCKS_PER_SCANLINE: u64 = 3_420;
const NTSC_SCANLINES_PER_FRAME: u16 = 262;
const NTSC_ACTIVE_SCANLINES: u16 = 224;

/// One Z80 access to the banked 68000 address space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GemsBanked68kRead {
    /// Absolute timing-engine master clock at the 68000 instruction boundary
    /// where the Z80 tick executes the read.
    pub master_clock: u64,
    /// Z80 instruction address that issued the banked read.
    pub z80_pc: u16,
    pub address: u32,
    pub value: u8,
}

/// Result of one master-clock advance through the persistent GEMS machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GemsMasterClockAdvance {
    pub banked_68k_reads: Vec<GemsBanked68kRead>,
    /// Wait queued for the next 68000 scheduler step by banked Z80 accesses.
    pub pending_m68k_wait_cycles: u32,
}

/// Raster origin used to derive the Mega Drive Z80 VBlank interrupt line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GemsNtscTimelineOrigin {
    /// Native trace/audio frames begin on VDP scanline 224.
    VblankFrameBoundary,
    /// Oracle absolute master clocks begin on VDP scanline 0.
    PowerOnScanlineZero,
}

/// Native GEMS timing checkpoint at a 68000 scheduling boundary.
///
/// The self-modifying audio RAM is deliberately retained in full: equal PCs
/// and divider phases are insufficient when the driver has patched its own
/// sequence, envelope, and DAC worker state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GemsTimingCheckpoint {
    pub master_clock: u64,
    pub z80_pc: u16,
    /// `[IX, IY, SP]`, matching the oracle checkpoint order.
    pub z80_index_registers: [u16; 3],
    pub audio_ram: Vec<u8>,
    pub bank_address: u32,
    pub z80_mclk_counter: u64,
    pub z80_t_cycles_wait: u32,
    pub z80_wait_mclk_cycles: u64,
    pub z80_odd_access: bool,
    pub ym2612_mclk_counter: u64,
    pub ym2612_last_status_read: u8,
    pub ym2612_status_decay_clocks_remaining: u32,
    pub z80_halt: bool,
    pub vdp_owns_bus: bool,
    pub busreq: bool,
    pub reset: bool,
    pub pending_m68k_wait_cycles: u32,
    pub z80_interrupt_mode: u8,
    pub z80_irq_pending: u8,
    pub z80_halted: bool,
    pub z80_iff1: bool,
    pub z80_iff2: bool,
}

/// One of the six separately arbitrated GEMS initialisation writes in ASM
/// `StopMusic -> sub_E149C/sub_E14AC`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GemsStopMusicQueueTransaction {
    CommandMarker,
    DataPointersCommand,
    PatchPointer,
    EnvelopePointer,
    SequencePointer,
    SamplePointer,
}

/// Hardware-visible stages of ASM `StopMusic` (`$E14C8..$E150E`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GemsStopMusicControl {
    ReleaseDriverReset,
    BeginDriverBus,
    CopyDriver,
    AssertReset,
    ReleaseDriverBus,
    ReleaseReset,
    BeginQueueTransaction,
    WriteQueue(GemsStopMusicQueueTransaction),
    ReleaseQueueBus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GemsQueueCommand {
    StartSequence(u8),
    StopSequence(u8),
    MusicSpeedPrefix(u8),
    MusicSpeedCommit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GemsQueueCommandControl {
    BeginQueueBus,
    WriteQueueBytes(GemsQueueCommand),
    ReleaseQueueBus,
}

struct GemsTimingBus {
    ram: RefCell<Vec<u8>>,
    virtual_rom: Vec<u8>,
    bank_address: u32,
    ym2612_shadow: RefCell<Ym2612Shadow>,
    current_master_clock: Cell<u64>,
    current_z80_pc: Cell<u16>,
    banked_reads: RefCell<Vec<GemsBanked68kRead>>,
}

impl GemsTimingBus {
    fn new(driver: &[u8], virtual_rom: Vec<u8>) -> Self {
        let mut ram = vec![0; Z80_RAM_LEN];
        let copy_len = driver.len().min(Z80_RAM_LEN);
        ram[..copy_len].copy_from_slice(&driver[..copy_len]);
        Self {
            ram: RefCell::new(ram),
            virtual_rom,
            bank_address: 0,
            ym2612_shadow: RefCell::new(Ym2612Shadow::new()),
            current_master_clock: Cell::new(0),
            current_z80_pc: Cell::new(0),
            banked_reads: RefCell::new(Vec::new()),
        }
    }

    fn take_banked_reads(&self) -> Vec<GemsBanked68kRead> {
        std::mem::take(&mut *self.banked_reads.borrow_mut())
    }
}

impl Z80_io for GemsTimingBus {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.ram.borrow()[usize::from(address) & (Z80_RAM_LEN - 1)],
            0x4000..=0x5FFF => self
                .ym2612_shadow
                .borrow_mut()
                .read_port((address & 0x03) as u8),
            0x8000..=0xFFFF => {
                let banked_address = self
                    .bank_address
                    .wrapping_add(u32::from(address.wrapping_sub(0x8000)));
                let value = self
                    .virtual_rom
                    .get(banked_address as usize)
                    .copied()
                    .unwrap_or(0xFF);
                self.banked_reads.borrow_mut().push(GemsBanked68kRead {
                    master_clock: self.current_master_clock.get(),
                    z80_pc: self.current_z80_pc.get(),
                    address: banked_address,
                    value,
                });
                value
            }
            _ => 0xFF,
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.ram.get_mut()[usize::from(address) & (Z80_RAM_LEN - 1)] = value;
            }
            0x4000..=0x5FFF => self
                .ym2612_shadow
                .get_mut()
                .write_port((address & 0x03) as u8, value),
            0x6000..=0x60FF => {
                let current_bank = self.bank_address >> 15;
                let next_bank = ((current_bank >> 1) + (u32::from(value & 1) << 8)) & 0x01FF;
                self.bank_address = next_bank << 15;
            }
            _ => {}
        }
    }

    fn port_in(&self, _address: u16) -> u8 {
        0xFF
    }

    fn port_out(&mut self, _address: u16, _value: u8) {}
}

/// Persistent GEMS timing machine clocked in Mega Drive master clocks.
///
/// This deliberately retains the complete Z80 core, self-modifying 8 KiB
/// audio RAM, bank register, IRQ/bus signals, YM status, divider remainder,
/// outstanding 49/50-master-clock bank wait, and alternating access phase.
/// ASM GEMS `FILLDACFIFO -> XFER68K` (`$01FB`, `$0273`, `$0285`) therefore
/// emits banked-read events from machine state rather than route constants.
pub struct GemsMasterClockTimingEngine {
    z80: Z80<GemsTimingBus>,
    driver: Vec<u8>,
    ntsc_timeline_origin: GemsNtscTimelineOrigin,
    master_clock: u64,
    z80_mclk_counter: u64,
    z80_t_cycles_wait: u32,
    z80_wait_mclk_cycles: u64,
    z80_odd_access: bool,
    ym2612_mclk_counter: u64,
    ym2612_input_mclk_counter: u64,
    z80_halt: bool,
    vdp_owns_bus: bool,
    busreq: bool,
    reset: bool,
    pending_m68k_wait_cycles: u32,
    pending_banked_reads: Vec<GemsBanked68kRead>,
    initialised: bool,
}

impl GemsMasterClockTimingEngine {
    pub fn new(driver: &[u8], virtual_rom: Vec<u8>) -> Self {
        Self::new_with_ntsc_timeline_origin(
            driver,
            virtual_rom,
            GemsNtscTimelineOrigin::VblankFrameBoundary,
        )
    }

    pub fn new_with_ntsc_timeline_origin(
        driver: &[u8],
        virtual_rom: Vec<u8>,
        ntsc_timeline_origin: GemsNtscTimelineOrigin,
    ) -> Self {
        let mut z80 = Z80::new(GemsTimingBus::new(driver, virtual_rom));
        z80.reset();
        Self {
            z80,
            driver: driver.to_vec(),
            ntsc_timeline_origin,
            master_clock: 0,
            z80_mclk_counter: 0,
            z80_t_cycles_wait: 0,
            z80_wait_mclk_cycles: 0,
            z80_odd_access: false,
            ym2612_mclk_counter: 0,
            ym2612_input_mclk_counter: 0,
            z80_halt: false,
            vdp_owns_bus: false,
            busreq: false,
            reset: ntsc_timeline_origin == GemsNtscTimelineOrigin::PowerOnScanlineZero,
            pending_m68k_wait_cycles: 0,
            pending_banked_reads: Vec::new(),
            initialised: false,
        }
    }

    pub fn from_kid_rom(rom: &[u8]) -> Result<Self> {
        Self::from_kid_rom_with_ntsc_timeline_origin(
            rom,
            GemsNtscTimelineOrigin::VblankFrameBoundary,
        )
    }

    pub fn from_kid_rom_with_ntsc_timeline_origin(
        rom: &[u8],
        ntsc_timeline_origin: GemsNtscTimelineOrigin,
    ) -> Result<Self> {
        ensure!(
            rom.len() >= GEMS_DRIVER_END_ADDR as usize,
            "ROM is too small for Kid Chameleon GEMS driver"
        );
        let driver = &rom[GEMS_DRIVER_ADDR as usize..GEMS_DRIVER_END_ADDR as usize];
        Ok(Self::new_with_ntsc_timeline_origin(
            driver,
            rom.to_vec(),
            ntsc_timeline_origin,
        ))
    }

    pub const fn master_clock(&self) -> u64 {
        self.master_clock
    }

    pub const fn ntsc_timeline_origin(&self) -> GemsNtscTimelineOrigin {
        self.ntsc_timeline_origin
    }

    pub const fn pending_m68k_wait_cycles(&self) -> u32 {
        self.pending_m68k_wait_cycles
    }

    /// Drain CPU wait cycles queued by banked Z80 reads without advancing time.
    ///
    /// A shared 68000/VDP scheduler uses this to merge the wait with refresh
    /// work at the next instruction boundary before clocking that combined
    /// interval back through the GEMS machine.
    pub fn take_pending_m68k_wait_cycles(&mut self) -> u32 {
        std::mem::take(&mut self.pending_m68k_wait_cycles)
    }

    /// Set the wait ledger owned by an externally source-scheduled 68000
    /// Z80-RAM access. The caller must hold BUSREQ so no Z80 banked read can
    /// contribute an independent wait at the same instant.
    pub fn set_source_scheduled_m68k_wait_cycles(&mut self, cycles: u32) {
        assert!(self.busreq, "source-scheduled Z80-RAM waits require BUSREQ");
        self.pending_m68k_wait_cycles = cycles;
    }

    pub fn z80_pc(&self) -> u16 {
        self.z80.pc
    }

    /// Snapshot the persistent GEMS machine without advancing it.
    #[must_use]
    pub fn checkpoint(&self) -> GemsTimingCheckpoint {
        let ym2612 = self.z80.io.ym2612_shadow.borrow();
        GemsTimingCheckpoint {
            master_clock: self.master_clock,
            z80_pc: self.z80.pc,
            z80_index_registers: [self.z80.ix, self.z80.iy, self.z80.sp],
            audio_ram: self.z80.io.ram.borrow().clone(),
            bank_address: self.z80.io.bank_address,
            z80_mclk_counter: self.z80_mclk_counter,
            z80_t_cycles_wait: self.z80_t_cycles_wait,
            z80_wait_mclk_cycles: self.z80_wait_mclk_cycles,
            z80_odd_access: self.z80_odd_access,
            ym2612_mclk_counter: self.ym2612_mclk_counter,
            ym2612_last_status_read: ym2612.last_status_read,
            ym2612_status_decay_clocks_remaining: ym2612.status_decay_clocks_remaining,
            z80_halt: self.z80_halt,
            vdp_owns_bus: self.vdp_owns_bus,
            busreq: self.busreq,
            reset: self.reset,
            pending_m68k_wait_cycles: self.pending_m68k_wait_cycles,
            z80_interrupt_mode: self.z80.interrupt_mode,
            z80_irq_pending: self.z80.irq_pending,
            z80_halted: self.z80.halted,
            z80_iff1: self.z80.iff1,
            z80_iff2: self.z80.iff2,
        }
    }

    /// Advance an already-accounted hardware interval.
    ///
    /// `interrupt_low` is the VDP's Z80 INT line sampled before this interval,
    /// matching the oracle's instruction-boundary ordering. `vdp_owns_bus`
    /// covers memory-to-VRAM DMA and pending FIFO writes.
    pub fn advance_master_clocks(
        &mut self,
        master_clocks: u64,
        interrupt_low: bool,
        vdp_owns_bus: bool,
    ) -> GemsMasterClockAdvance {
        self.advance_continuous_master_clocks(master_clocks, interrupt_low, vdp_owns_bus, None)
    }

    /// Advance one already-timed 68000 instruction.
    ///
    /// A Z80 bank wait raised at this instruction's closing boundary cannot
    /// consume clocks that elapsed before the access. The following
    /// instruction interval consumes it. ASM GEMS `XFER68K $0273/$0285`
    /// reaches this path while `DecompressToVRAM loc_14314` feeds the FIFO.
    pub fn advance_m68k_instruction_master_clocks(
        &mut self,
        master_clocks: u64,
        interrupt_low: bool,
        vdp_owns_bus: bool,
    ) -> GemsMasterClockAdvance {
        let mut consumable_z80_wait_mclk_cycles = self.z80_wait_mclk_cycles;
        self.advance_continuous_master_clocks(
            master_clocks,
            interrupt_low,
            vdp_owns_bus,
            Some(&mut consumable_z80_wait_mclk_cycles),
        )
    }

    /// Advance one exact VBlank-to-VBlank 262-scanline NTSC frame.
    ///
    /// ASM `WaitForVint` hands the game frame to `V_Int` at scanline 224, so
    /// the trace/audio frame boundary begins with the one scanline where the
    /// Mega Drive VDP drives the Z80 INT line low. The remaining VBlank
    /// scanlines, visible scanlines 0..223, then lead back to the next VBlank.
    pub fn advance_ntsc_frame(&mut self) -> GemsMasterClockAdvance {
        self.advance_ntsc_master_clocks(
            MASTER_CLOCKS_PER_SCANLINE * u64::from(NTSC_SCANLINES_PER_FRAME),
        )
    }

    /// Advance an arbitrary interval on the persistent VBlank-to-VBlank NTSC
    /// timeline, splitting it where the VDP changes the Z80 interrupt line.
    pub fn advance_ntsc_master_clocks(&mut self, master_clocks: u64) -> GemsMasterClockAdvance {
        self.advance_ntsc_master_clocks_with_vdp_ownership(master_clocks, false)
    }

    /// Advance an NTSC interval with constant VDP ownership of the 68000 bus.
    ///
    /// `loc_14314` FIFO retries use this for their one-cycle halted ticks.
    /// Z80 interrupt edges still follow the scanline-224 VBlank timeline.
    pub fn advance_ntsc_master_clocks_with_vdp_ownership(
        &mut self,
        master_clocks: u64,
        vdp_owns_bus: bool,
    ) -> GemsMasterClockAdvance {
        self.advance_ntsc_master_clocks_with_vdp_ownership_inner(master_clocks, vdp_owns_bus, None)
    }

    /// Advance one 68000 instruction while deriving Z80 INT from NTSC raster.
    ///
    /// A bank wait generated at this instruction's closing boundary remains
    /// pending for the following instruction.
    pub fn advance_ntsc_m68k_instruction_with_vdp_ownership(
        &mut self,
        master_clocks: u64,
        vdp_owns_bus: bool,
    ) -> GemsMasterClockAdvance {
        let mut consumable_z80_wait_mclk_cycles = self.z80_wait_mclk_cycles;
        self.advance_ntsc_master_clocks_with_vdp_ownership_inner(
            master_clocks,
            vdp_owns_bus,
            Some(&mut consumable_z80_wait_mclk_cycles),
        )
    }

    /// Advance one 68000 instruction and capture an exact checkpoint inside it.
    ///
    /// The same opening Z80 bank-wait budget is shared by both halves, so a
    /// bank read raised before the checkpoint remains pending for the next
    /// 68000 instruction exactly as it does in the unsplit path.
    pub fn advance_ntsc_m68k_instruction_with_checkpoint(
        &mut self,
        master_clocks: u64,
        vdp_owns_bus: bool,
        checkpoint_offset: u64,
    ) -> Option<(GemsMasterClockAdvance, GemsTimingCheckpoint)> {
        let (advance, mut checkpoints) = self.advance_ntsc_m68k_instruction_with_checkpoints(
            master_clocks,
            vdp_owns_bus,
            &[checkpoint_offset],
        )?;
        Some((advance, checkpoints.pop()?))
    }

    /// Advance one 68000 instruction and capture ordered checkpoints inside it.
    ///
    /// All segments share the instruction's opening Z80 bank-wait budget.
    pub fn advance_ntsc_m68k_instruction_with_checkpoints(
        &mut self,
        master_clocks: u64,
        vdp_owns_bus: bool,
        checkpoint_offsets: &[u64],
    ) -> Option<(GemsMasterClockAdvance, Vec<GemsTimingCheckpoint>)> {
        if checkpoint_offsets.last().copied().unwrap_or(0) > master_clocks
            || !checkpoint_offsets.is_sorted()
        {
            return None;
        }
        let mut consumable_z80_wait_mclk_cycles = self.z80_wait_mclk_cycles;
        let mut reads = Vec::new();
        let mut checkpoints = Vec::with_capacity(checkpoint_offsets.len());
        let mut previous_offset = 0;
        for &checkpoint_offset in checkpoint_offsets {
            reads.extend(
                self.advance_ntsc_master_clocks_with_vdp_ownership_inner(
                    checkpoint_offset - previous_offset,
                    vdp_owns_bus,
                    Some(&mut consumable_z80_wait_mclk_cycles),
                )
                .banked_68k_reads,
            );
            checkpoints.push(self.checkpoint());
            previous_offset = checkpoint_offset;
        }
        reads.extend(
            self.advance_ntsc_master_clocks_with_vdp_ownership_inner(
                master_clocks - previous_offset,
                vdp_owns_bus,
                Some(&mut consumable_z80_wait_mclk_cycles),
            )
            .banked_68k_reads,
        );
        Some((
            GemsMasterClockAdvance {
                banked_68k_reads: reads,
                pending_m68k_wait_cycles: self.pending_m68k_wait_cycles,
            },
            checkpoints,
        ))
    }

    fn advance_ntsc_master_clocks_with_vdp_ownership_inner(
        &mut self,
        mut master_clocks: u64,
        vdp_owns_bus: bool,
        mut consumable_z80_wait_mclk_cycles: Option<&mut u64>,
    ) -> GemsMasterClockAdvance {
        const MASTER_CLOCKS_PER_FRAME: u64 =
            MASTER_CLOCKS_PER_SCANLINE * NTSC_SCANLINES_PER_FRAME as u64;
        let mut reads = Vec::new();
        while master_clocks != 0 {
            let frame_mclk = self.master_clock % MASTER_CLOCKS_PER_FRAME;
            let frame_scanline = u16::try_from(frame_mclk / MASTER_CLOCKS_PER_SCANLINE)
                .expect("NTSC frame scanline fits u16");
            let scanline_mclk = frame_mclk % MASTER_CLOCKS_PER_SCANLINE;
            let until_next_scanline = MASTER_CLOCKS_PER_SCANLINE - scanline_mclk;
            let elapsed = master_clocks.min(until_next_scanline);
            let vdp_scanline = match self.ntsc_timeline_origin {
                GemsNtscTimelineOrigin::VblankFrameBoundary => {
                    (NTSC_ACTIVE_SCANLINES + frame_scanline) % NTSC_SCANLINES_PER_FRAME
                }
                GemsNtscTimelineOrigin::PowerOnScanlineZero => frame_scanline,
            };
            reads.extend(
                self.advance_continuous_master_clocks(
                    elapsed,
                    vdp_scanline == NTSC_ACTIVE_SCANLINES,
                    vdp_owns_bus,
                    consumable_z80_wait_mclk_cycles.as_deref_mut(),
                )
                .banked_68k_reads,
            );
            master_clocks -= elapsed;
        }
        GemsMasterClockAdvance {
            banked_68k_reads: reads,
            pending_m68k_wait_cycles: self.pending_m68k_wait_cycles,
        }
    }

    fn advance_continuous_master_clocks(
        &mut self,
        mut master_clocks: u64,
        interrupt_low: bool,
        vdp_owns_bus: bool,
        mut consumable_z80_wait_mclk_cycles: Option<&mut u64>,
    ) -> GemsMasterClockAdvance {
        if interrupt_low {
            self.z80.assert_irq(0xFF);
        } else {
            self.z80.clr_irq();
        }
        self.vdp_owns_bus = vdp_owns_bus;
        self.z80_halt &= vdp_owns_bus;
        while master_clocks != 0 {
            let consumable_wait = consumable_z80_wait_mclk_cycles
                .as_deref()
                .copied()
                .unwrap_or(self.z80_wait_mclk_cycles);
            if !vdp_owns_bus && consumable_wait != 0 {
                let elapsed = master_clocks.min(consumable_wait);
                self.advance_clock_counters(elapsed);
                self.z80_wait_mclk_cycles -= elapsed;
                if let Some(consumable) = consumable_z80_wait_mclk_cycles.as_deref_mut() {
                    *consumable -= elapsed;
                }
                master_clocks -= elapsed;
                continue;
            }

            let clocks_to_tick = Z80_DIVIDER - self.z80_mclk_counter;
            if master_clocks < clocks_to_tick {
                self.advance_clock_counters(master_clocks);
                self.z80_mclk_counter += master_clocks;
                master_clocks = 0;
                continue;
            }

            self.advance_clock_counters(clocks_to_tick);
            master_clocks -= clocks_to_tick;
            self.z80_mclk_counter = 0;
            if self.z80_halt {
                // jgenesis stops ticking the Z80 entirely after a banked
                // 68000 access collides with VDP ownership. The current
                // instruction's remaining T cycles resume only after the
                // FIFO/DMA releases the bus.
            } else if self.z80_t_cycles_wait != 0 {
                self.z80_t_cycles_wait -= 1;
            } else if self.reset {
                // jgenesis `Z80::execute_instruction`: RESET is sampled
                // only at an instruction boundary and occupies the normal
                // four-T-cycle minimum slot.
                self.z80.reset();
                self.z80_t_cycles_wait = 3;
            } else if self.busreq {
                // jgenesis BUSREQ completes the current instruction, then
                // returns four-cycle stalled slots until the bus releases.
                self.z80_t_cycles_wait = 3;
            } else {
                self.z80.io.current_master_clock.set(self.master_clock);
                self.z80.io.current_z80_pc.set(self.z80.pc);
                let reads_before = self.z80.io.banked_reads.borrow().len();
                let elapsed = self.step_z80_instruction();
                self.z80_t_cycles_wait = elapsed.saturating_sub(1);
                let access_count = self.z80.io.banked_reads.borrow().len() - reads_before;
                for _ in 0..access_count {
                    self.record_z80_68k_bus_access();
                }
            }
        }

        self.advance_ym2612();
        let banked_68k_reads = self.z80.io.take_banked_reads();
        self.pending_banked_reads
            .extend_from_slice(&banked_68k_reads);
        GemsMasterClockAdvance {
            banked_68k_reads,
            pending_m68k_wait_cycles: self.pending_m68k_wait_cycles,
        }
    }

    pub fn take_banked_68k_reads(&mut self) -> Vec<GemsBanked68kRead> {
        std::mem::take(&mut self.pending_banked_reads)
    }

    /// Queue the ROM's `$0B` data-pointer command without consuming hidden
    /// time. The timing engine is clocked continuously by its caller, so the
    /// real driver boot and command consumption remain on the game timeline.
    pub fn init(&mut self) {
        if self.initialised {
            return;
        }
        for byte in [0xFF, 0x0B] {
            self.queue_byte(byte);
        }
        for address in [
            GEMS_PATCH_ADDR,
            GEMS_ENVELOPE_ADDR,
            GEMS_SEQUENCE_ADDR,
            GEMS_SAMPLE_ADDR,
        ] {
            self.queue_24bit_addr(address);
        }
        self.initialised = true;
    }

    pub fn start_sequence(&mut self, sound_id: u8) {
        self.ensure_initialised();
        self.queue_byte(0xFF);
        self.queue_byte(0x10);
        self.queue_byte(sound_id);
    }

    pub fn stop_sequence(&mut self, sound_id: u8) {
        self.ensure_initialised();
        self.queue_byte(0xFF);
        self.queue_byte(0x12);
        self.queue_byte(sound_id);
    }

    pub fn set_music_speed(&mut self, speed: u8) {
        self.ensure_initialised();
        for byte in [0xFF, 0x05, speed, 0xFF, 0x0D] {
            self.queue_byte(byte);
        }
    }

    pub fn stop_all_sequences(&mut self) {
        self.ensure_initialised();
        for byte in [0xFF, 0x16, 0xFF] {
            self.queue_byte(byte);
        }
    }

    pub fn stop_music(&mut self) {
        self.apply_stop_music_control(GemsStopMusicControl::ReleaseDriverReset);
        self.apply_stop_music_control(GemsStopMusicControl::BeginDriverBus);
        self.apply_stop_music_control(GemsStopMusicControl::CopyDriver);
        self.apply_stop_music_control(GemsStopMusicControl::AssertReset);
        self.apply_stop_music_control(GemsStopMusicControl::ReleaseDriverBus);
        self.apply_stop_music_control(GemsStopMusicControl::ReleaseReset);
        for transaction in [
            GemsStopMusicQueueTransaction::CommandMarker,
            GemsStopMusicQueueTransaction::DataPointersCommand,
            GemsStopMusicQueueTransaction::PatchPointer,
            GemsStopMusicQueueTransaction::EnvelopePointer,
            GemsStopMusicQueueTransaction::SequencePointer,
            GemsStopMusicQueueTransaction::SamplePointer,
        ] {
            self.apply_stop_music_control(GemsStopMusicControl::BeginQueueTransaction);
            self.apply_stop_music_control(GemsStopMusicControl::WriteQueue(transaction));
            self.apply_stop_music_control(GemsStopMusicControl::ReleaseQueueBus);
        }
    }

    pub fn reload_driver(&mut self) {
        self.copy_driver_ram();
        self.z80.io.ym2612_shadow.get_mut().reset();
        self.z80.reset();
        self.z80_t_cycles_wait = 0;
        self.pending_banked_reads.clear();
    }

    /// Apply one source-positioned control event while the caller advances the
    /// intervening master clocks.
    pub fn apply_stop_music_control(&mut self, control: GemsStopMusicControl) {
        match control {
            GemsStopMusicControl::ReleaseDriverReset => {
                assert!(
                    !self.busreq,
                    "StopMusic releases power-on reset before requesting the Z80 bus"
                );
                self.set_z80_control(false, false);
            }
            GemsStopMusicControl::BeginDriverBus => {
                assert!(
                    !self.busreq && !self.reset,
                    "StopMusic driver bus request must begin from a running Z80"
                );
                self.set_z80_control(true, false);
            }
            GemsStopMusicControl::CopyDriver => {
                assert!(
                    self.busreq && !self.reset,
                    "StopMusic copies the driver only after the Z80 acknowledges BUSREQ"
                );
                self.copy_driver_ram();
            }
            GemsStopMusicControl::AssertReset => {
                assert!(
                    self.busreq && !self.reset,
                    "StopMusic reset is asserted while the driver-copy bus window is held"
                );
                self.set_z80_control(true, true);
            }
            GemsStopMusicControl::ReleaseDriverBus => {
                assert!(
                    self.busreq && self.reset,
                    "StopMusic releases the driver-copy bus while reset remains asserted"
                );
                self.set_z80_control(false, true);
            }
            GemsStopMusicControl::ReleaseReset => {
                assert!(
                    !self.busreq && self.reset,
                    "StopMusic releases reset after releasing the driver-copy bus"
                );
                self.set_z80_control(false, false);
            }
            GemsStopMusicControl::BeginQueueTransaction => {
                assert!(
                    !self.busreq && !self.reset,
                    "StopMusic queue transaction must begin from a running Z80"
                );
                self.set_z80_control(true, false);
            }
            GemsStopMusicControl::WriteQueue(transaction) => {
                assert!(
                    self.busreq && !self.reset,
                    "StopMusic writes each queue transaction while holding the Z80 bus"
                );
                match transaction {
                    GemsStopMusicQueueTransaction::CommandMarker => self.queue_byte(0xFF),
                    GemsStopMusicQueueTransaction::DataPointersCommand => self.queue_byte(0x0B),
                    GemsStopMusicQueueTransaction::PatchPointer => {
                        self.queue_24bit_addr(GEMS_PATCH_ADDR);
                    }
                    GemsStopMusicQueueTransaction::EnvelopePointer => {
                        self.queue_24bit_addr(GEMS_ENVELOPE_ADDR);
                    }
                    GemsStopMusicQueueTransaction::SequencePointer => {
                        self.queue_24bit_addr(GEMS_SEQUENCE_ADDR);
                    }
                    GemsStopMusicQueueTransaction::SamplePointer => {
                        self.queue_24bit_addr(GEMS_SAMPLE_ADDR);
                        self.initialised = true;
                    }
                }
            }
            GemsStopMusicControl::ReleaseQueueBus => {
                assert!(
                    self.busreq && !self.reset,
                    "StopMusic releases the bus after each queue transaction"
                );
                self.set_z80_control(false, false);
            }
        }
    }

    /// Apply one source-positioned GEMS queue-command control event.
    pub fn apply_queue_command_control(&mut self, control: GemsQueueCommandControl) {
        match control {
            GemsQueueCommandControl::BeginQueueBus => {
                assert!(
                    !self.busreq && !self.reset,
                    "GEMS command transaction must begin from a running Z80"
                );
                self.set_z80_control(true, false);
            }
            GemsQueueCommandControl::WriteQueueBytes(command) => {
                assert!(
                    self.busreq && !self.reset,
                    "GEMS command bytes are written while holding the Z80 bus"
                );
                assert!(
                    self.initialised,
                    "GEMS command transaction requires StopMusic pointer initialisation"
                );
                match command {
                    GemsQueueCommand::StartSequence(sound_id) => {
                        for byte in [0xFF, 0x10, sound_id] {
                            self.queue_byte(byte);
                        }
                    }
                    GemsQueueCommand::StopSequence(sound_id) => {
                        for byte in [0xFF, 0x12, sound_id] {
                            self.queue_byte(byte);
                        }
                    }
                    GemsQueueCommand::MusicSpeedPrefix(speed) => {
                        for byte in [0xFF, 0x05, speed] {
                            self.queue_byte(byte);
                        }
                    }
                    GemsQueueCommand::MusicSpeedCommit => {
                        for byte in [0xFF, 0x0D] {
                            self.queue_byte(byte);
                        }
                    }
                }
            }
            GemsQueueCommandControl::ReleaseQueueBus => {
                assert!(
                    self.busreq && !self.reset,
                    "GEMS command releases a held Z80 bus"
                );
                self.set_z80_control(false, false);
            }
        }
    }

    fn copy_driver_ram(&mut self) {
        let copy_len = self.driver.len().min(Z80_RAM_LEN);
        {
            let ram = self.z80.io.ram.get_mut();
            ram.fill(0);
            ram[..copy_len].copy_from_slice(&self.driver[..copy_len]);
        }
        self.initialised = false;
    }

    /// Materialise the completed prefix of ASM `sub_E13DA`'s progressive
    /// `$A00000..$A01FFF` driver copy while the 68000 owns the Z80 bus.
    pub fn apply_stop_music_driver_copy_progress(
        &mut self,
        copied_driver_bytes: usize,
        cleared_tail_bytes: usize,
    ) {
        assert!(
            self.busreq && !self.reset,
            "StopMusic driver bytes are written only while the acknowledged bus is held"
        );
        assert!(copied_driver_bytes <= self.driver.len());
        assert!(self.driver.len() + cleared_tail_bytes <= Z80_RAM_LEN);
        let ram = self.z80.io.ram.get_mut();
        ram[..copied_driver_bytes].copy_from_slice(&self.driver[..copied_driver_bytes]);
        ram[self.driver.len()..self.driver.len() + cleared_tail_bytes].fill(0);
        self.initialised = false;
    }

    pub fn set_z80_control(&mut self, busreq: bool, reset: bool) {
        if reset && !self.reset {
            // ASM sub_E1416 asserts Mega Drive Z80 reset, which also resets YM2612.
            self.z80.io.ym2612_shadow.get_mut().reset();
        }
        self.busreq = busreq;
        self.reset = reset;
    }

    /// Copy the ROM's short power-on Z80 bootstrap while the 68000 owns the
    /// Z80 bus. ASM `EntryPoint Z80InitLoop $0500` writes the bytes labelled
    /// `Z80StartupCodeBegin`; the later `StopMusic -> sub_E13DA` replaces
    /// them with the GEMS driver.
    pub fn copy_power_on_startup_ram(&mut self, startup_code: &[u8]) {
        assert!(
            self.ntsc_timeline_origin == GemsNtscTimelineOrigin::PowerOnScanlineZero,
            "power-on startup code requires the absolute raster origin"
        );
        assert!(
            self.busreq,
            "EntryPoint copies Z80 startup code only while BUSREQ is held"
        );
        let copy_len = startup_code.len().min(Z80_RAM_LEN);
        let ram = self.z80.io.ram.get_mut();
        ram.fill(0);
        ram[..copy_len].copy_from_slice(&startup_code[..copy_len]);
    }

    fn ensure_initialised(&mut self) {
        self.init();
    }

    fn queue_byte(&mut self, value: u8) {
        let ram = self.z80.io.ram.get_mut();
        let ptr = ram[usize::from(GEMS_QUEUE_PTR_ADDR)];
        ram[usize::from(GEMS_QUEUE_ADDR) + usize::from(ptr)] = value;
        ram[usize::from(GEMS_QUEUE_PTR_ADDR)] = ptr.wrapping_add(1) & (GEMS_QUEUE_SIZE - 1);
    }

    fn queue_24bit_addr(&mut self, address: u32) {
        self.queue_byte(address as u8);
        self.queue_byte((address >> 8) as u8);
        self.queue_byte((address >> 16) as u8);
    }

    fn record_z80_68k_bus_access(&mut self) {
        self.z80_wait_mclk_cycles += 49 + u64::from(self.z80_odd_access);
        self.z80_odd_access = !self.z80_odd_access;
        self.z80_halt |= self.vdp_owns_bus;
        if !self.vdp_owns_bus {
            self.pending_m68k_wait_cycles = self.pending_m68k_wait_cycles.saturating_add(11);
        }
    }

    fn step_z80_instruction(&mut self) -> u32 {
        if self.z80.irq_pending != 0 && self.z80.iff1 && self.z80.iff_delay == 0 {
            assert_eq!(
                self.z80.interrupt_mode, 1,
                "Kid GEMS enables IM1 before accepting a maskable interrupt"
            );
            self.z80.halted = false;
            self.z80.iff1 = false;
            self.z80.iff2 = false;
            self.z80.r = (self.z80.r.wrapping_add(1) & 0x7F) | (self.z80.r & 0x80);
            let return_pc = self.z80.pc;
            self.z80.sp = self.z80.sp.wrapping_sub(1);
            self.z80.io.write_byte(self.z80.sp, (return_pc >> 8) as u8);
            self.z80.sp = self.z80.sp.wrapping_sub(1);
            self.z80.io.write_byte(self.z80.sp, return_pc as u8);
            self.z80.pc = 0x0038;
            return 13;
        }

        // The `z80` crate polls IRQ after executing an opcode. Mega Drive
        // hardware and jgenesis poll at the next instruction boundary, so the
        // wrapper owns IRQ service and hides the level from the core step.
        let irq_asserted = self.z80.irq_pending != 0;
        let irq_data = self.z80.irq_data;
        self.z80.clr_irq();
        let elapsed = self.z80.step();
        if irq_asserted {
            self.z80.assert_irq(irq_data);
        }
        elapsed
    }

    fn advance_clock_counters(&mut self, master_clocks: u64) {
        self.master_clock = self.master_clock.wrapping_add(master_clocks);
        self.ym2612_mclk_counter = self.ym2612_mclk_counter.wrapping_add(master_clocks);
        self.ym2612_input_mclk_counter = self.ym2612_input_mclk_counter.wrapping_add(master_clocks);
    }

    fn advance_ym2612(&mut self) {
        self.ym2612_mclk_counter %= YM2612_SCHEDULER_DIVIDER;
        let ym_ticks = self.ym2612_input_mclk_counter / YM2612_INPUT_DIVIDER;
        self.ym2612_input_mclk_counter %= YM2612_INPUT_DIVIDER;
        self.z80
            .io
            .ym2612_shadow
            .get_mut()
            .advance_clocks(u32::try_from(ym_ticks).unwrap_or(u32::MAX));
    }

    /// Consume the accumulated 68000 wait as a real scheduler interval.
    pub fn advance_pending_m68k_wait(
        &mut self,
        interrupt_low: bool,
        vdp_owns_bus: bool,
    ) -> GemsMasterClockAdvance {
        let wait_cycles = std::mem::take(&mut self.pending_m68k_wait_cycles);
        self.advance_master_clocks(
            u64::from(wait_cycles) * M68K_DIVIDER,
            interrupt_low,
            vdp_owns_bus,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_retains_self_modifying_gems_state_and_scheduler_phase() {
        let driver = [0x00, 0x76];
        let mut engine = GemsMasterClockTimingEngine::new(&driver, vec![]);
        engine.z80.io.ram.borrow_mut()[0x1FFF] = 0xA5;
        engine.z80.io.bank_address = 0x0180_0000;
        engine.advance_master_clocks(14, false, false);

        let checkpoint = engine.checkpoint();

        assert_eq!(checkpoint.master_clock, 14);
        assert_eq!(checkpoint.z80_pc, 0);
        assert_eq!(checkpoint.audio_ram.len(), Z80_RAM_LEN);
        assert_eq!(&checkpoint.audio_ram[..2], &driver);
        assert_eq!(checkpoint.audio_ram[0x1FFF], 0xA5);
        assert_eq!(checkpoint.bank_address, 0x0180_0000);
        assert_eq!(checkpoint.z80_mclk_counter, 14);
        assert_eq!(checkpoint.ym2612_mclk_counter, 14);
    }

    #[test]
    fn banked_reads_persist_cpu_ram_and_alternating_wait_phase() {
        // LD A,($8000); LD ($1F00),A; JP $0000
        let driver = [0x3A, 0x00, 0x80, 0x32, 0x00, 0x1F, 0xC3, 0x00, 0x00];
        let mut rom = vec![0; 0x8001];
        rom[0] = 0x5A;
        let mut engine = GemsMasterClockTimingEngine::new(&driver, rom);

        let first = engine.advance_master_clocks(15, false, false);
        assert_eq!(
            first.banked_68k_reads,
            vec![GemsBanked68kRead {
                master_clock: 15,
                z80_pc: 0,
                address: 0,
                value: 0x5A,
            }],
            "GEMS XFER68K-style bank read is emitted at its Z80 instruction boundary",
        );
        assert_eq!(first.pending_m68k_wait_cycles, 11);

        let wait = engine.advance_pending_m68k_wait(false, false);
        assert!(wait.banked_68k_reads.is_empty());
        assert_eq!(engine.pending_m68k_wait_cycles(), 0);

        let mut reads = Vec::new();
        for _ in 0..64 {
            reads.extend(
                engine
                    .advance_master_clocks(15, false, false)
                    .banked_68k_reads,
            );
            if !reads.is_empty() {
                break;
            }
        }
        assert_eq!(reads.len(), 1);
        assert_eq!(engine.pending_m68k_wait_cycles(), 11);
        assert_eq!(engine.z80.io.ram.borrow()[0x1F00], 0x5A);
        assert!(
            !engine.z80_odd_access,
            "two bank reads must retain the alternating 49/50-master-clock phase",
        );
    }

    #[test]
    fn pending_m68k_wait_can_be_drained_for_shared_cpu_scheduler() {
        // LD A,($8000); HALT
        let mut rom = vec![0; 0x8001];
        rom[0] = 0x5A;
        let mut engine = GemsMasterClockTimingEngine::new(&[0x3A, 0x00, 0x80, 0x76], rom);

        engine.advance_master_clocks(15, false, false);
        let master_clock_before_drain = engine.master_clock();

        assert_eq!(engine.take_pending_m68k_wait_cycles(), 11);
        assert_eq!(engine.take_pending_m68k_wait_cycles(), 0);
        assert_eq!(
            engine.master_clock(),
            master_clock_before_drain,
            "the outer CPU scheduler owns the drained interval"
        );
    }

    #[test]
    fn long_interval_consumes_new_banked_read_wait_before_next_z80_tick() {
        // LD A,($8000); JP $0000
        let driver = [0x3A, 0x00, 0x80, 0xC3, 0x00, 0x00];
        let mut rom = vec![0; 0x8001];
        rom[0] = 0x5A;
        let mut engine = GemsMasterClockTimingEngine::new(&driver, rom);

        let advance = engine.advance_master_clocks(1_000, false, false);

        assert!(advance.banked_68k_reads.len() >= 2);
        assert_eq!(advance.banked_68k_reads[0].master_clock, 15);
        for reads in advance.banked_68k_reads.windows(2) {
            assert!(
                reads[1].master_clock >= reads[0].master_clock + 49 + Z80_DIVIDER,
                "the access wait must elapse before the next Z80 instruction boundary",
            );
        }
    }

    #[test]
    fn m68k_instruction_defers_bank_wait_generated_inside_its_interval() {
        // GEMS XFER68K $0273/$0285: jgenesis records the complete 68000
        // instruction before ticking the Z80. A bank wait raised during that
        // catch-up cannot consume clocks from the instruction that preceded
        // the access.
        let driver = [0x3A, 0x00, 0x80, 0xC3, 0x00, 0x00];
        let mut rom = vec![0; 0x8001];
        rom[0] = 0x5A;
        let mut instruction = GemsMasterClockTimingEngine::new(&driver, rom.clone());
        let mut continuous = GemsMasterClockTimingEngine::new(&driver, rom);

        instruction.advance_m68k_instruction_master_clocks(84, false, false);
        continuous.advance_master_clocks(84, false, false);

        assert_eq!(instruction.checkpoint().z80_wait_mclk_cycles, 49);
        assert_eq!(continuous.checkpoint().z80_wait_mclk_cycles, 0);
        assert_ne!(
            instruction.checkpoint().z80_t_cycles_wait,
            continuous.checkpoint().z80_t_cycles_wait,
        );
    }

    #[test]
    fn ntsc_frame_begins_with_the_z80_vblank_interrupt_scanline() {
        // Mega Drive VDP `z80_interrupt_line`: INT is low only on scanline
        // 224. ASM WaitForVint hands the trace frame to V_Int on that same
        // boundary, so a frame-clocked GEMS sink must expose the interrupt
        // before advancing the remaining VBlank and visible scanlines.
        let mut driver = vec![0; 0x3D];
        driver[..4].copy_from_slice(&[0xFB, 0x00, 0x76, 0x76]); // EI; NOP; HALT
        driver[0x38..0x3D].copy_from_slice(&[0x3A, 0x00, 0x80, 0x76, 0x76]);
        let mut rom = vec![0; 0x8001];
        rom[0] = 0x5A;
        let mut engine = GemsMasterClockTimingEngine::new(&driver, rom);
        engine.z80.interrupt_mode = 1;

        let advance = engine.advance_ntsc_frame();

        assert_eq!(advance.banked_68k_reads.len(), 1);
        assert!(
            advance.banked_68k_reads[0].master_clock < MASTER_CLOCKS_PER_SCANLINE,
            "the Z80 must service the scanline-224 interrupt at the trace/audio frame boundary"
        );
    }

    #[test]
    fn power_on_timeline_asserts_z80_interrupt_on_vdp_scanline_224() {
        // jgenesis `Vdp::z80_interrupt_line`: an absolute power-on master
        // clock reaches the low INT line only at VDP scanline 224.
        let mut driver = vec![0; 0x3D];
        driver[..4].copy_from_slice(&[0xFB, 0x00, 0x76, 0x76]); // EI; NOP; HALT
        driver[0x38..0x3D].copy_from_slice(&[0x3A, 0x00, 0x80, 0x76, 0x76]);
        let mut rom = vec![0; 0x8001];
        rom[0] = 0x5A;
        let mut engine = GemsMasterClockTimingEngine::new_with_ntsc_timeline_origin(
            &driver,
            rom,
            GemsNtscTimelineOrigin::PowerOnScanlineZero,
        );
        engine.z80.interrupt_mode = 1;
        assert!(engine.checkpoint().reset);
        engine.set_z80_control(false, false);

        let advance = engine.advance_ntsc_frame();
        let read = advance
            .banked_68k_reads
            .first()
            .expect("scanline-224 interrupt handler bank read");

        assert!(read.master_clock >= 224 * MASTER_CLOCKS_PER_SCANLINE);
        assert!(read.master_clock < 225 * MASTER_CLOCKS_PER_SCANLINE);
    }

    #[test]
    fn ntsc_frame_retains_exact_master_clock_and_divider_phase() {
        let mut engine = GemsMasterClockTimingEngine::new(&[0x76], vec![]);
        let advance = engine.advance_ntsc_frame();
        assert!(advance.banked_68k_reads.is_empty());
        assert_eq!(
            engine.master_clock(),
            MASTER_CLOCKS_PER_SCANLINE * u64::from(NTSC_SCANLINES_PER_FRAME),
        );
        assert_eq!(engine.z80_mclk_counter, 0);
    }

    #[test]
    fn arbitrary_ntsc_advance_retains_vblank_interrupt_and_scanline_cursor() {
        let mut driver = vec![0; 0x3D];
        driver[..4].copy_from_slice(&[0xFB, 0x00, 0x76, 0x76]);
        driver[0x38..0x3D].copy_from_slice(&[0x3A, 0x00, 0x80, 0x76, 0x76]);
        let mut rom = vec![0; 0x8001];
        rom[0] = 0x5A;
        let mut whole = GemsMasterClockTimingEngine::new(&driver, rom.clone());
        let mut split = GemsMasterClockTimingEngine::new(&driver, rom);
        whole.z80.interrupt_mode = 1;
        split.z80.interrupt_mode = 1;
        let frame_mclk = MASTER_CLOCKS_PER_SCANLINE * u64::from(NTSC_SCANLINES_PER_FRAME);

        let whole_reads = whole
            .advance_ntsc_master_clocks(frame_mclk)
            .banked_68k_reads;
        let mut split_reads = split
            .advance_ntsc_master_clocks(MASTER_CLOCKS_PER_SCANLINE / 2)
            .banked_68k_reads;
        split_reads.extend(
            split
                .advance_ntsc_master_clocks(frame_mclk - MASTER_CLOCKS_PER_SCANLINE / 2)
                .banked_68k_reads,
        );

        assert_eq!(split_reads, whole_reads);
        assert_eq!(split.master_clock(), whole.master_clock());
        assert_eq!(split.z80_pc(), whole.z80_pc());
    }

    #[test]
    fn ntsc_vdp_owned_interval_halts_banked_z80_without_68000_wait() {
        // GEMS XFER68K $0273/$0285 during ASM loc_14314: jgenesis records
        // the bank read but halts the Z80 instead of stalling the 68000 while
        // the VDP owns the bus.
        let driver = [0x3A, 0x00, 0x80, 0xC3, 0x00, 0x00];
        let mut rom = vec![0; 0x8001];
        rom[0] = 0x5A;
        let mut engine = GemsMasterClockTimingEngine::new(&driver, rom);

        let owned = engine.advance_ntsc_master_clocks_with_vdp_ownership(1_000, true);

        assert!(!owned.banked_68k_reads.is_empty());
        assert_eq!(owned.pending_m68k_wait_cycles, 0);
        assert!(engine.checkpoint().z80_halt);
        engine.advance_ntsc_master_clocks_with_vdp_ownership(15, false);
        assert!(!engine.checkpoint().z80_halt);
    }

    #[test]
    fn ym_shadow_uses_raw_master_divider_seven_for_gems_filldacfifo() {
        // GEMS FILLDACFIFO ($01FB) is paced by YM status/timer state.
        // Ym2612Shadow::advance_clocks accepts the raw master/7 input clock,
        // not jgenesis' internally prescaled master/42 YM tick.
        let mut engine = GemsMasterClockTimingEngine::new(&[0x76], vec![]);
        engine
            .z80
            .io
            .ym2612_shadow
            .get_mut()
            .status_decay_clocks_remaining = 10;

        engine.advance_master_clocks(6, false, false);
        assert_eq!(engine.checkpoint().ym2612_mclk_counter, 6);
        assert_eq!(
            engine
                .z80
                .io
                .ym2612_shadow
                .borrow()
                .status_decay_clocks_remaining,
            10,
        );
        engine.advance_master_clocks(1, false, false);
        assert_eq!(engine.checkpoint().ym2612_mclk_counter, 7);
        assert_eq!(
            engine
                .z80
                .io
                .ym2612_shadow
                .borrow()
                .status_decay_clocks_remaining,
            9,
        );
        engine.advance_master_clocks(35, false, false);
        assert_eq!(
            engine.checkpoint().ym2612_mclk_counter,
            0,
            "jgenesis retains the YM scheduler phase modulo master/42",
        );
    }

    #[test]
    fn gems_im1_vector_0038_preempts_next_opcode_at_instruction_boundary() {
        // GEMS vector $0038 stores the interrupted SP before RETI. The Z80
        // samples INT before the next opcode, so that opcode's address is the
        // return address pushed by the IM1 service.
        let mut driver = vec![0; 0x40];
        driver[0x10] = 0x00;
        driver[0x38..0x3E].copy_from_slice(&[0xED, 0x73, 0x3E, 0x00, 0xED, 0x4D]);
        let mut engine = GemsMasterClockTimingEngine::new(&driver, vec![]);
        engine.z80.pc = 0x0010;
        engine.z80.sp = 0x0100;
        engine.z80.interrupt_mode = 1;
        engine.z80.iff1 = true;
        engine.z80.iff2 = true;

        engine.advance_master_clocks(15, true, false);

        let checkpoint = engine.checkpoint();
        assert_eq!(checkpoint.z80_pc, 0x0038);
        assert_eq!(checkpoint.z80_index_registers[2], 0x00FE);
        assert_eq!(
            &checkpoint.audio_ram[0x00FE..=0x00FF],
            &[0x10, 0x00],
            "INT preempts the NOP at $0010 instead of pushing $0011 afterwards",
        );
    }

    #[test]
    fn driver_reload_preserves_hardware_scheduler_and_bank_state() {
        let driver = [0x3A, 0x00, 0x80, 0x76];
        let mut rom = vec![0; 0x8001];
        rom[0] = 0xA5;
        let mut engine = GemsMasterClockTimingEngine::new(&driver, rom);

        let first = engine.advance_master_clocks(15, false, false);
        assert_eq!(first.pending_m68k_wait_cycles, 11);
        assert!(engine.z80_odd_access);
        assert_eq!(engine.z80_wait_mclk_cycles, 49);
        engine.z80.io.bank_address = 0x0001_8000;

        engine.reload_driver();

        assert!(engine.z80_odd_access);
        assert_eq!(engine.z80_wait_mclk_cycles, 49);
        assert_eq!(engine.pending_m68k_wait_cycles(), 11);
        assert_eq!(
            engine.z80.io.bank_address, 0x0001_8000,
            "Z80 reset does not itself rewrite the Mega Drive bank register",
        );
    }

    #[test]
    fn stop_music_control_events_hold_z80_across_copy_reset_and_queue_windows_e14c8() {
        let driver = [0x00, 0xC3, 0x00, 0x00];
        let mut engine = GemsMasterClockTimingEngine::new(&driver, vec![]);
        engine.advance_master_clocks(15, false, false);
        assert_eq!(engine.z80_pc(), 1);
        engine.z80.io.ram.get_mut().fill(0xA5);

        engine.apply_stop_music_control(GemsStopMusicControl::BeginDriverBus);
        assert!(engine.busreq);
        assert!(!engine.reset);
        assert!(
            engine.z80.io.ram.borrow().iter().all(|&byte| byte == 0xA5),
            "BUSREQ write precedes both BUSACK and the first driver copy"
        );
        engine.advance_master_clocks(60, false, false);
        assert_eq!(
            engine.z80_pc(),
            1,
            "the in-flight Z80 instruction completes before BUSACK"
        );
        engine.apply_stop_music_control(GemsStopMusicControl::CopyDriver);
        assert_eq!(&engine.z80.io.ram.borrow()[..4], &driver);
        assert!(
            engine.z80.io.ram.borrow()[4..]
                .iter()
                .all(|&byte| byte == 0)
        );
        let held_pc = engine.z80_pc();
        engine.advance_master_clocks(150, false, false);
        assert_eq!(engine.z80_pc(), held_pc);

        engine.apply_stop_music_control(GemsStopMusicControl::AssertReset);
        engine.apply_stop_music_control(GemsStopMusicControl::ReleaseDriverBus);
        engine.advance_master_clocks(150, false, false);
        assert_eq!(engine.z80_pc(), 0, "ASM sub_E1416 keeps reset asserted");
        engine.apply_stop_music_control(GemsStopMusicControl::ReleaseReset);

        for transaction in [
            GemsStopMusicQueueTransaction::CommandMarker,
            GemsStopMusicQueueTransaction::DataPointersCommand,
            GemsStopMusicQueueTransaction::PatchPointer,
            GemsStopMusicQueueTransaction::EnvelopePointer,
            GemsStopMusicQueueTransaction::SequencePointer,
            GemsStopMusicQueueTransaction::SamplePointer,
        ] {
            engine.apply_stop_music_control(GemsStopMusicControl::BeginQueueTransaction);
            engine.apply_stop_music_control(GemsStopMusicControl::WriteQueue(transaction));
            engine.apply_stop_music_control(GemsStopMusicControl::ReleaseQueueBus);
        }

        let ram = engine.z80.io.ram.borrow();
        assert_eq!(
            &ram[usize::from(GEMS_QUEUE_ADDR)..usize::from(GEMS_QUEUE_ADDR) + 14],
            &[
                0xFF,
                0x0B,
                GEMS_PATCH_ADDR as u8,
                (GEMS_PATCH_ADDR >> 8) as u8,
                (GEMS_PATCH_ADDR >> 16) as u8,
                GEMS_ENVELOPE_ADDR as u8,
                (GEMS_ENVELOPE_ADDR >> 8) as u8,
                (GEMS_ENVELOPE_ADDR >> 16) as u8,
                GEMS_SEQUENCE_ADDR as u8,
                (GEMS_SEQUENCE_ADDR >> 8) as u8,
                (GEMS_SEQUENCE_ADDR >> 16) as u8,
                GEMS_SAMPLE_ADDR as u8,
                (GEMS_SAMPLE_ADDR >> 8) as u8,
                (GEMS_SAMPLE_ADDR >> 16) as u8,
            ],
            "ASM sub_E149C/sub_E14AC queue FF 0B and four low-to-high pointers"
        );
        assert!(engine.initialised);
    }

    #[test]
    fn queue_command_controls_hold_z80_and_write_asm_command_bytes() {
        let mut engine = GemsMasterClockTimingEngine::new(&[0x76], vec![]);
        engine.initialised = true;

        engine.apply_queue_command_control(GemsQueueCommandControl::BeginQueueBus);
        let held_pc = engine.z80_pc();
        engine.advance_master_clocks(150, false, false);
        assert_eq!(engine.z80_pc(), held_pc);
        engine.apply_queue_command_control(GemsQueueCommandControl::WriteQueueBytes(
            GemsQueueCommand::StartSequence(0x17),
        ));
        engine.apply_queue_command_control(GemsQueueCommandControl::ReleaseQueueBus);

        assert_eq!(
            &engine.z80.io.ram.borrow()
                [usize::from(GEMS_QUEUE_ADDR)..usize::from(GEMS_QUEUE_ADDR) + 3],
            &[0xFF, 0x10, 0x17],
            "ASM sub_E151C queues command marker, start-sequence byte, and id"
        );
    }

    #[test]
    fn reset_and_busreq_are_sampled_in_four_t_cycle_slots() {
        // jgenesis `Z80::tick`: the current instruction finishes before
        // BUSREQ stalls, while RESET is sampled only when the next instruction
        // would begin. Both states then occupy four-T-cycle minimum slots.
        let mut engine = GemsMasterClockTimingEngine::new(&[0x00, 0x00, 0x76], vec![]);
        engine.advance_master_clocks(15, false, false);
        assert_eq!(engine.z80_pc(), 1);
        assert_eq!(engine.checkpoint().z80_t_cycles_wait, 3);

        engine.set_z80_control(true, false);
        engine.advance_master_clocks(45, false, false);
        assert_eq!(engine.z80_pc(), 1);
        assert_eq!(engine.checkpoint().z80_t_cycles_wait, 0);
        engine.advance_master_clocks(15, false, false);
        assert_eq!(engine.checkpoint().z80_t_cycles_wait, 3);

        engine.set_z80_control(false, true);
        engine.advance_master_clocks(60, false, false);
        assert_eq!(engine.z80_pc(), 0);
        assert_eq!(engine.checkpoint().z80_t_cycles_wait, 3);
    }

    #[test]
    fn vdp_owned_bank_read_halts_z80_without_queuing_68000_wait() {
        let driver = [0x3A, 0x00, 0x80, 0x76];
        let mut rom = vec![0; 0x8001];
        rom[0] = 0x5A;
        let mut engine = GemsMasterClockTimingEngine::new(&driver, rom);

        let access = engine.advance_master_clocks(15, false, true);
        assert_eq!(access.banked_68k_reads.len(), 1);
        assert_eq!(access.pending_m68k_wait_cycles, 0);
        assert!(engine.z80_halt);

        let halted_pc = engine.z80_pc();
        let halted_t_cycles = engine.z80_t_cycles_wait;
        engine.advance_master_clocks(60, false, true);
        assert_eq!(engine.z80_pc(), halted_pc);
        assert_eq!(
            engine.z80_t_cycles_wait, halted_t_cycles,
            "VDP ownership freezes the current GEMS instruction, not only its PC",
        );
        engine.advance_master_clocks(60, false, false);
        assert!(!engine.z80_halt);
    }

    #[test]
    fn checkpoint_split_preserves_one_m68k_instruction_bank_wait_semantics() {
        let driver = [0x3A, 0x00, 0x80, 0x76];
        let mut rom = vec![0; 0x8001];
        rom[0] = 0x5A;
        let mut unsplit = GemsMasterClockTimingEngine::new(&driver, rom.clone());
        let mut split = GemsMasterClockTimingEngine::new(&driver, rom.clone());
        let mut multi_split = GemsMasterClockTimingEngine::new(&driver, rom);

        let expected = unsplit.advance_ntsc_m68k_instruction_with_vdp_ownership(105, false);
        let (actual, checkpoint) = split
            .advance_ntsc_m68k_instruction_with_checkpoint(105, false, 37)
            .expect("checkpoint falls inside the instruction");

        assert_eq!(checkpoint.master_clock, 37);
        assert_eq!(actual, expected);
        assert_eq!(split.checkpoint(), unsplit.checkpoint());

        let (multi_actual, checkpoints) = multi_split
            .advance_ntsc_m68k_instruction_with_checkpoints(105, false, &[19, 37, 88])
            .expect("ordered checkpoints fall inside the instruction");
        assert_eq!(
            checkpoints
                .iter()
                .map(|checkpoint| checkpoint.master_clock)
                .collect::<Vec<_>>(),
            [19, 37, 88],
        );
        assert_eq!(multi_actual, expected);
        assert_eq!(multi_split.checkpoint(), unsplit.checkpoint());
    }
}
