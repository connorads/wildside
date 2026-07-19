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
// Ym2612Shadow accepts raw 7.67 MHz input clocks. jgenesis' YM core consumes
// an internally prescaled tick every 42 master clocks; copying that divider
// here under-clocks this shadow by six. Mega Drive YM input is master / 7.
const YM2612_DIVIDER: u64 = 7;
const MASTER_CLOCKS_PER_SCANLINE: u64 = 3_420;
const NTSC_SCANLINES_PER_FRAME: u16 = 262;
const NTSC_ACTIVE_SCANLINES: u16 = 224;

/// One Z80 access to the banked 68000 address space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GemsBanked68kRead {
    /// Absolute timing-engine master clock at the 68000 instruction boundary
    /// where the Z80 tick executes the read.
    pub master_clock: u64,
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
    BeginDriverCopy,
    AssertReset,
    ReleaseDriverBus,
    ReleaseReset,
    BeginQueueTransaction,
    WriteQueue(GemsStopMusicQueueTransaction),
    ReleaseQueueBus,
}

struct GemsTimingBus {
    ram: RefCell<Vec<u8>>,
    virtual_rom: Vec<u8>,
    bank_address: u32,
    ym2612_shadow: RefCell<Ym2612Shadow>,
    current_master_clock: Cell<u64>,
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
    master_clock: u64,
    z80_mclk_counter: u64,
    z80_t_cycles_wait: u32,
    z80_wait_mclk_cycles: u64,
    z80_odd_access: bool,
    ym2612_mclk_counter: u64,
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
        let mut z80 = Z80::new(GemsTimingBus::new(driver, virtual_rom));
        z80.reset();
        Self {
            z80,
            driver: driver.to_vec(),
            master_clock: 0,
            z80_mclk_counter: 0,
            z80_t_cycles_wait: 0,
            z80_wait_mclk_cycles: 0,
            z80_odd_access: false,
            ym2612_mclk_counter: 0,
            z80_halt: false,
            vdp_owns_bus: false,
            busreq: false,
            reset: false,
            pending_m68k_wait_cycles: 0,
            pending_banked_reads: Vec::new(),
            initialised: false,
        }
    }

    pub fn from_kid_rom(rom: &[u8]) -> Result<Self> {
        ensure!(
            rom.len() >= GEMS_DRIVER_END_ADDR as usize,
            "ROM is too small for Kid Chameleon GEMS driver"
        );
        let driver = &rom[GEMS_DRIVER_ADDR as usize..GEMS_DRIVER_END_ADDR as usize];
        Ok(Self::new(driver, rom.to_vec()))
    }

    pub const fn master_clock(&self) -> u64 {
        self.master_clock
    }

    pub const fn pending_m68k_wait_cycles(&self) -> u32 {
        self.pending_m68k_wait_cycles
    }

    pub fn z80_pc(&self) -> u16 {
        self.z80.pc
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
        self.advance_continuous_master_clocks(master_clocks, interrupt_low, vdp_owns_bus)
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
    pub fn advance_ntsc_master_clocks(&mut self, mut master_clocks: u64) -> GemsMasterClockAdvance {
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
            let vdp_scanline = (NTSC_ACTIVE_SCANLINES + frame_scanline) % NTSC_SCANLINES_PER_FRAME;
            reads.extend(
                self.advance_continuous_master_clocks(
                    elapsed,
                    vdp_scanline == NTSC_ACTIVE_SCANLINES,
                    false,
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
    ) -> GemsMasterClockAdvance {
        if interrupt_low {
            self.z80.assert_irq(0xFF);
        } else {
            self.z80.clr_irq();
        }
        self.vdp_owns_bus = vdp_owns_bus;
        self.z80_halt &= vdp_owns_bus;
        while master_clocks != 0 {
            if !vdp_owns_bus && self.z80_wait_mclk_cycles != 0 {
                let elapsed = master_clocks.min(self.z80_wait_mclk_cycles);
                self.advance_clock_counters(elapsed);
                self.z80_wait_mclk_cycles -= elapsed;
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
            if self.z80_t_cycles_wait != 0 {
                self.z80_t_cycles_wait -= 1;
            } else if !self.z80_halt && !self.busreq && !self.reset {
                self.z80.io.current_master_clock.set(self.master_clock);
                let reads_before = self.z80.io.banked_reads.borrow().len();
                let elapsed = self.z80.step();
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
        self.apply_stop_music_control(GemsStopMusicControl::BeginDriverCopy);
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
            GemsStopMusicControl::BeginDriverCopy => {
                assert!(
                    !self.busreq && !self.reset,
                    "StopMusic driver copy must begin from a running Z80"
                );
                self.set_z80_control(true, false);
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

    fn copy_driver_ram(&mut self) {
        let copy_len = self.driver.len().min(Z80_RAM_LEN);
        {
            let ram = self.z80.io.ram.get_mut();
            ram.fill(0);
            ram[..copy_len].copy_from_slice(&self.driver[..copy_len]);
        }
        self.initialised = false;
    }

    pub fn set_z80_control(&mut self, busreq: bool, reset: bool) {
        if reset && !self.reset {
            // ASM sub_E1416 asserts Mega Drive Z80 reset, which also resets YM2612.
            self.z80.io.ym2612_shadow.get_mut().reset();
            self.z80.reset();
            self.z80_t_cycles_wait = 0;
        }
        self.busreq = busreq;
        self.reset = reset;
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

    fn advance_clock_counters(&mut self, master_clocks: u64) {
        self.master_clock = self.master_clock.wrapping_add(master_clocks);
        self.ym2612_mclk_counter = self.ym2612_mclk_counter.wrapping_add(master_clocks);
    }

    fn advance_ym2612(&mut self) {
        let ym_ticks = self.ym2612_mclk_counter / YM2612_DIVIDER;
        self.ym2612_mclk_counter %= YM2612_DIVIDER;
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

        let advance = engine.advance_ntsc_frame();

        assert_eq!(advance.banked_68k_reads.len(), 1);
        assert!(
            advance.banked_68k_reads[0].master_clock < MASTER_CLOCKS_PER_SCANLINE,
            "the Z80 must service the scanline-224 interrupt at the trace/audio frame boundary"
        );
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
        assert_eq!(
            engine
                .z80
                .io
                .ym2612_shadow
                .borrow()
                .status_decay_clocks_remaining,
            9,
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
        engine.z80.io.ram.get_mut().fill(0xA5);

        engine.apply_stop_music_control(GemsStopMusicControl::BeginDriverCopy);
        assert!(engine.busreq);
        assert!(!engine.reset);
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
        engine.advance_master_clocks(60, false, true);
        assert_eq!(engine.z80_pc(), halted_pc);
        engine.advance_master_clocks(60, false, false);
        assert!(!engine.z80_halt);
    }
}
