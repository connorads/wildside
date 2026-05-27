use anyhow::Result;
use r68k_emu::cpu::{ConfiguredCore, Cycles};
use r68k_emu::interrupts::AutoInterruptController;
use r68k_emu::ram::{AddressBus, AddressSpace};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use thiserror::Error;
use ymfm_sys::Ym2612;
use z80::{Z80, Z80_io};

const ADDRESS_MASK: u32 = 0x00FF_FFFF;
const WORK_RAM_BASE: u32 = 0x00E0_0000;
const WORK_RAM_END: u32 = 0x00FF_FFFF;
const WORK_RAM_LEN: usize = 0x1_0000;
const Z80_RAM_BASE: u32 = 0x00A0_0000;
const Z80_RAM_END: u32 = 0x00A0_FFFF;
const Z80_RAM_LEN: usize = 0x2000;
// NTSC Genesis frame timing: 3420 master clocks/scanline * 262 scanlines / 7 m68k clocks.
const SYNTHETIC_VBLANK_CYCLES: i32 = 128_006;
const CONTROLLER_FLIP_COUNTER_CYCLES: u32 = 12_150;
const CONTROLLER_TH_HIGH_DELAY_CYCLES: u32 = 30;
const CONTROLLER_TH_BIT: u8 = 6;
const V_INT_COUNTER_ADDR: u32 = 0x00FF_F806;
const V_INT_DONE_ADDR: u32 = 0x00FF_F80A;
const CAMERA_X_POS_ADDR: u32 = 0x00FF_F81C;
const CAMERA_Y_POS_ADDR: u32 = 0x00FF_F820;
const NUMBER_OBJECTS_ADDR: u32 = 0x00FF_F84A;
const NUMBER_GFX_OBJECTS_ADDR: u32 = 0x00FF_F85C;
const DEMO_MODE_FLAG_ADDR: u32 = 0x00FF_FBC9;
const GAME_MODE_ADDR: u32 = 0x00FF_FBCA;
const CURRENT_PLAYER_ADDR: u32 = 0x00FF_FC39;
const NUMBER_LIVES_ADDR: u32 = 0x00FF_FC3E;
const NUMBER_HITPOINTS_ADDR: u32 = 0x00FF_FC40;
const LEVEL_SKIP_CHEAT_ADDR: u32 = 0x00FF_FC37;
const CURRENT_LEVEL_ID_ADDR: u32 = 0x00FF_FC44;
const CURRENT_HELMET_ID_ADDR: u32 = 0x00FF_FC46;
const PLAYER_1_LEVEL_ID_ADDR: u32 = 0x00FF_FC5A;
const KID_X_POS_ADDR: u32 = 0x00FF_24FA;
const KID_Y_POS_ADDR: u32 = 0x00FF_24FE;
const KID_X_VEL_ADDR: u32 = 0x00FF_2506;
const KID_Y_VEL_ADDR: u32 = 0x00FF_250A;
const KID_COLLISION_TYPE_ADDR: u32 = 0x00FF_2518;
const CHARACTER_MOVEMENT_ADDR: u32 = 0x00FF_FA56;
const INVULNERABLE_FRAMES_ADDR: u32 = 0x00FF_FA2B;
const JUST_RECEIVED_DAMAGE_ADDR: u32 = 0x00FF_FA73;
const VRAM_LEN: usize = 0x1_0000;
const CRAM_LEN: usize = 0x80;
const VSRAM_LEN: usize = 0x50;
const TRACE_LIMIT: usize = 256;
const PSG_CLOCK_HZ: f64 = 3_579_545.0;
const Z80_CLOCK_HZ: f64 = 3_579_545.0;
const Z80_VBLANK_IRQ_CYCLES: u32 = 228;
const YM2612_GAIN: f32 = 1.0;
const PSG_GAIN: f32 = 0.446_683_6;
const AUDIO_DC_BLOCK_CUTOFF_HZ: f64 = 5.0;
const GENESIS_LPF_CUTOFF_HZ: f64 = 3_390.0;
const YM2612_SECOND_LPF_CUTOFF_HZ: f64 = 7_973.0;
const DEFAULT_OUTPUT_SAMPLE_RATE_HZ: f64 = 44_100.0;
const PSG_DIVIDER: f64 = 16.0;
const PSG_DC_BLOCK_COEFFICIENT: f32 = 0.995;
// jgenesis models mirrored YM2612 status reads decaying after roughly 12_000 output samples.
// Convert that to input clocks so the Z80 sound driver does not see dead reads on $4001-$4003.
const YM2612_STATUS_DECAY_CLOCKS: u32 = 1_728_000;
const PSG_ATTENUATION_TO_VOLUME: [f32; 16] = [
    1.0,
    0.794_328_2,
    0.630_957_37,
    0.501_187_2,
    0.398_107_17,
    0.316_227_76,
    0.251_188_64,
    0.199_526_24,
    0.158_489_32,
    0.125_892_53,
    0.1,
    0.079_432_82,
    0.063_095_73,
    0.050_118_72,
    0.039_810_72,
    0.0,
];
const PSG_INITIAL_LFSR: u16 = 0x8000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioLevels {
    pub master: f32,
    pub ym2612: f32,
    pub psg: f32,
}

impl Default for AudioLevels {
    fn default() -> Self {
        Self {
            master: 1.0,
            ym2612: 1.0,
            psg: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameState {
    pub current_level_id: u16,
    pub current_helmet_id: u16,
    pub number_lives: u16,
    pub number_hitpoints: u16,
    pub kid_x_pos: u32,
    pub kid_y_pos: u32,
    pub camera_x_pos: u32,
    pub camera_y_pos: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuProbe {
    pub reset_sp: u32,
    pub reset_pc: u32,
    pub final_pc: u32,
    pub final_sp: u32,
    pub synthetic_vblanks: u32,
    pub v_int_counter: u32,
    pub v_int_done: u8,
    pub demo_mode_flag: u8,
    pub game_mode: u16,
    pub current_player: u8,
    pub number_lives: u16,
    pub number_hitpoints: u16,
    pub current_level_id: u16,
    pub player_1_level_id: u16,
    pub current_helmet_id: u16,
    pub camera_x_pos: u32,
    pub camera_y_pos: u32,
    pub kid_x_pos: u32,
    pub kid_y_pos: u32,
    pub kid_x_vel: i32,
    pub kid_y_vel: i32,
    pub character_movement: u16,
    pub kid_collision_type: u16,
    pub invulnerable_frames: u8,
    pub just_received_damage: bool,
    pub number_objects: u16,
    pub number_gfx_objects: u16,
    pub z80_interrupt_mode: u8,
    pub vram_nonzero_bytes: usize,
    pub cram_nonzero_bytes: usize,
    pub vsram_nonzero_bytes: usize,
    pub z80_reset_count: usize,
    pub ym2612_write_count: usize,
    pub ym2612_key_on_count: usize,
    pub psg_write_count: usize,
    pub ym2612_writes: Vec<(u16, u8)>,
    pub psg_writes: Vec<u8>,
    pub banked_68k_reads: Vec<(u32, u8)>,
    pub banked_68k_writes: Vec<(u32, u8)>,
    pub vdp: VdpSnapshot,
    pub steps: Vec<CpuStep>,
    pub accesses: Vec<BusAccess>,
    pub vdp_control_writes: Vec<u16>,
    pub vdp_data_writes: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VdpSnapshot {
    pub registers: [u8; 0x20],
    pub vram: Vec<u8>,
    pub cram: Vec<u8>,
    pub vsram: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub rgb: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AudioEvent {
    Ym2612Write {
        z80_cycles: u64,
        port: u8,
        value: u8,
    },
    PsgWrite {
        z80_cycles: u64,
        value: u8,
    },
}

#[derive(Debug, Clone)]
struct FirstOrderFilter {
    b0: f32,
    b1: f32,
    a1: f32,
    x1: f32,
    y1: f32,
}

impl FirstOrderFilter {
    fn low_pass(cutoff_hz: f64, sample_rate_hz: f64) -> Self {
        let k = (std::f64::consts::PI * cutoff_hz / sample_rate_hz).tan();
        let norm = 1.0 / (1.0 + k);
        Self {
            b0: (k * norm) as f32,
            b1: (k * norm) as f32,
            a1: ((k - 1.0) * norm) as f32,
            x1: 0.0,
            y1: 0.0,
        }
    }

    fn high_pass(cutoff_hz: f64, sample_rate_hz: f64) -> Self {
        let k = (std::f64::consts::PI * cutoff_hz / sample_rate_hz).tan();
        let norm = 1.0 / (1.0 + k);
        Self {
            b0: norm as f32,
            b1: (-norm) as f32,
            a1: ((k - 1.0) * norm) as f32,
            x1: 0.0,
            y1: 0.0,
        }
    }

    fn filter(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 - self.a1 * self.y1;
        self.x1 = input;
        self.y1 = output;
        output
    }
}

#[derive(Debug, Clone)]
struct SecondOrderFilter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl SecondOrderFilter {
    fn low_pass(cutoff_hz: f64, sample_rate_hz: f64) -> Self {
        let k = (std::f64::consts::PI * cutoff_hz / sample_rate_hz).tan();
        let norm = 1.0 / (1.0 + std::f64::consts::SQRT_2 * k + k * k);
        let b0 = (k * k * norm) as f32;
        Self {
            b0,
            b1: 2.0 * b0,
            b2: b0,
            a1: (2.0 * (k * k - 1.0) * norm) as f32,
            a2: ((1.0 - std::f64::consts::SQRT_2 * k + k * k) * norm) as f32,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    fn filter(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }
}

impl AudioEvent {
    fn z80_cycles(self) -> u64 {
        match self {
            Self::Ym2612Write { z80_cycles, .. } | Self::PsgWrite { z80_cycles, .. } => z80_cycles,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WindowRegion {
    left: usize,
    top: usize,
    width: usize,
    height: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScrollPlane {
    base: usize,
    width_cells: usize,
    height_cells: usize,
    hscroll: i32,
    vscroll: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WindowPlane {
    base: usize,
    region: Option<WindowRegion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Z80Shared {
    ram: Vec<u8>,
    m68k_work_ram: Vec<u8>,
    busreq: bool,
    reset: bool,
    bank_address: u32,
    z80_cycles: u64,
    z80_reset_count: usize,
    ym2612_write_count: usize,
    ym2612_key_on_count: usize,
    psg_write_count: usize,
    ym2612_port_latches: [Option<u16>; 2],
    audio_events: VecDeque<AudioEvent>,
    banked_68k_reads: Vec<(u32, u8)>,
    banked_68k_writes: Vec<(u32, u8)>,
    ym2612_writes: Vec<(u16, u8)>,
    psg_writes: Vec<u8>,
    psg_registers: PsgRegisters,
}

impl Default for Z80Shared {
    fn default() -> Self {
        Self {
            ram: vec![0; Z80_RAM_LEN],
            m68k_work_ram: vec![0; WORK_RAM_LEN],
            busreq: false,
            reset: true,
            bank_address: 0,
            z80_cycles: 0,
            z80_reset_count: 0,
            ym2612_write_count: 0,
            ym2612_key_on_count: 0,
            psg_write_count: 0,
            ym2612_port_latches: [None, None],
            audio_events: VecDeque::new(),
            banked_68k_reads: Vec::new(),
            banked_68k_writes: Vec::new(),
            ym2612_writes: Vec::new(),
            psg_writes: Vec::new(),
            psg_registers: PsgRegisters::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PsgRegisters {
    latched_register: u8,
    tone_periods: [u16; 3],
    volumes: [u8; 4],
    noise_control: u8,
}

impl Default for PsgRegisters {
    fn default() -> Self {
        Self {
            latched_register: 0,
            tone_periods: [1; 3],
            volumes: [0x0F; 4],
            noise_control: 0,
        }
    }
}

impl PsgRegisters {
    fn apply_write(&mut self, value: u8) -> bool {
        if value & 0x80 != 0 {
            self.latched_register = (value >> 4) & 0x07;
            self.apply_latched_data(self.latched_register, value & 0x0F)
        } else {
            self.apply_data(self.latched_register, value & 0x3F)
        }
    }

    fn apply_latched_data(&mut self, register: u8, data: u8) -> bool {
        match register {
            0 | 2 | 4 => {
                let channel = usize::from(register / 2);
                self.tone_periods[channel] = (self.tone_periods[channel] & !0x0F) + u16::from(data);
                false
            }
            1 | 3 | 5 | 7 => {
                self.volumes[usize::from(register / 2)] = data & 0x0F;
                false
            }
            6 => {
                self.noise_control = data & 0x07;
                true
            }
            _ => false,
        }
    }

    fn apply_data(&mut self, register: u8, data: u8) -> bool {
        match register {
            0 | 2 | 4 => {
                let channel = usize::from(register / 2);
                self.tone_periods[channel] =
                    (self.tone_periods[channel] & 0x0F) + (u16::from(data & 0x3F) << 4);
                false
            }
            6 => {
                self.noise_control = data & 0x07;
                true
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
struct PsgSynth {
    tone_counters: [u16; 3],
    tone_outputs_high: [bool; 3],
    noise_counter: u16,
    noise_clock_high: bool,
    noise_lfsr: u16,
    noise_output_high: bool,
    pending_psg_clocks: f64,
    dc_previous_input: f32,
    dc_previous_output: f32,
}

impl Default for PsgSynth {
    fn default() -> Self {
        Self {
            tone_counters: [1; 3],
            tone_outputs_high: [false; 3],
            noise_counter: 0x10,
            noise_clock_high: false,
            noise_lfsr: PSG_INITIAL_LFSR,
            noise_output_high: false,
            pending_psg_clocks: 0.0,
            dc_previous_input: 0.0,
            dc_previous_output: 0.0,
        }
    }
}

impl PsgSynth {
    fn reset_noise_lfsr(&mut self) {
        self.noise_lfsr = PSG_INITIAL_LFSR;
        self.noise_output_high = false;
    }
}

struct Z80Bus {
    shared: Rc<RefCell<Z80Shared>>,
    ym2612_shadow: Rc<RefCell<Ym2612Shadow>>,
    rom: Vec<u8>,
}

struct AudioMixer {
    psg_synth: PsgSynth,
    psg_registers: PsgRegisters,
    ym2612: Ym2612,
    levels: AudioLevels,
    pending_events: VecDeque<AudioEvent>,
    output_samples_emitted: u64,
    ym_samples_emitted: u64,
    last_ym_sample: [f32; 2],
    ym_dc_filters: [FirstOrderFilter; 2],
    ym_low_pass_filters: [FirstOrderFilter; 2],
    ym_second_low_pass_filters: [SecondOrderFilter; 2],
    psg_low_pass_filter: FirstOrderFilter,
    psg_output_sample_rate: u32,
}

struct Ym2612Shadow {
    chip: Ym2612,
    last_status_read: u8,
    status_decay_clocks_remaining: u32,
}

impl Ym2612Shadow {
    fn new() -> Self {
        Self {
            chip: Ym2612::default(),
            last_status_read: 0,
            status_decay_clocks_remaining: 0,
        }
    }

    fn reset(&mut self) {
        self.chip.reset();
        self.last_status_read = 0;
        self.status_decay_clocks_remaining = 0;
    }

    fn read_port(&mut self, port: u8) -> u8 {
        match port & 0x03 {
            0 => {
                let status = self.chip.read_port(0);
                self.last_status_read = status;
                self.status_decay_clocks_remaining = YM2612_STATUS_DECAY_CLOCKS;
                status
            }
            1..=3 => {
                if self.status_decay_clocks_remaining == 0 {
                    0
                } else {
                    self.last_status_read
                }
            }
            _ => unreachable!("port is masked to two bits"),
        }
    }

    fn advance_clocks(&mut self, clocks: u32) {
        self.chip.advance_clocks(clocks);
        self.status_decay_clocks_remaining =
            self.status_decay_clocks_remaining.saturating_sub(clocks);
    }

    fn write_port(&mut self, port: u8, value: u8) {
        self.chip.write_port(port, value);
    }
}

impl Default for AudioMixer {
    fn default() -> Self {
        let ym2612 = Ym2612::default();
        let ym_sample_rate_hz = f64::from(ym2612.sample_rate_hz());
        Self {
            psg_synth: PsgSynth::default(),
            psg_registers: PsgRegisters::default(),
            ym2612,
            levels: AudioLevels::default(),
            pending_events: VecDeque::new(),
            output_samples_emitted: 0,
            ym_samples_emitted: 0,
            last_ym_sample: [0.0, 0.0],
            ym_dc_filters: std::array::from_fn(|_| {
                FirstOrderFilter::high_pass(AUDIO_DC_BLOCK_CUTOFF_HZ, ym_sample_rate_hz)
            }),
            ym_low_pass_filters: std::array::from_fn(|_| {
                FirstOrderFilter::low_pass(GENESIS_LPF_CUTOFF_HZ, ym_sample_rate_hz)
            }),
            ym_second_low_pass_filters: std::array::from_fn(|_| {
                SecondOrderFilter::low_pass(YM2612_SECOND_LPF_CUTOFF_HZ, ym_sample_rate_hz)
            }),
            psg_low_pass_filter: FirstOrderFilter::low_pass(
                GENESIS_LPF_CUTOFF_HZ,
                DEFAULT_OUTPUT_SAMPLE_RATE_HZ,
            ),
            psg_output_sample_rate: DEFAULT_OUTPUT_SAMPLE_RATE_HZ as u32,
        }
    }
}

pub struct MdRuntime {
    core: ConfiguredCore<AutoInterruptController, MdBus>,
    z80: Z80<Z80Bus>,
    z80_reset_asserted: bool,
    z80_vblank_irq_cycles_remaining: u32,
    audio_mixer: AudioMixer,
    cycles_until_vblank: i32,
    synthetic_vblanks: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuStep {
    pub index: u32,
    pub pc_before: u32,
    pub pc_after: u32,
    pub cycles: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BusAccess {
    pub kind: BusAccessKind,
    pub address: u32,
    pub value: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusAccessKind {
    ReadByte,
    ReadWord,
    WriteByte,
    WriteWord,
}

#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("ROM is too small to contain a reset vector")]
    RomTooSmall,
}

pub fn probe_rom(rom: &[u8], step_count: u32) -> Result<CpuProbe> {
    probe_rom_with_script(rom, step_count, &[])
}

pub fn probe_rom_with_script(
    rom: &[u8],
    step_count: u32,
    controller_1_script: &[u8],
) -> Result<CpuProbe> {
    let mut runtime = MdRuntime::new(rom)?;
    let reset_sp = runtime.core.dar[15];
    let reset_pc = runtime.core.pc;
    let mut frame_index = 0usize;
    runtime
        .core
        .mem
        .set_controller_1(controller_1_script.first().copied().unwrap_or(0));

    let mut steps = Vec::with_capacity(step_count as usize);
    for index in 0..step_count {
        let pc_before = runtime.core.pc;
        let cycles = runtime.step();
        let pc_after = runtime.core.pc;
        let rendered_frame_count = runtime.synthetic_vblanks as usize;
        if rendered_frame_count != frame_index {
            frame_index = rendered_frame_count;
            runtime
                .core
                .mem
                .set_controller_1(controller_1_script.get(frame_index).copied().unwrap_or(0));
        }
        steps.push(CpuStep {
            index,
            pc_before,
            pc_after,
            cycles,
        });
    }

    Ok(CpuProbe {
        reset_sp,
        reset_pc,
        final_pc: runtime.core.pc,
        final_sp: runtime.core.dar[15],
        synthetic_vblanks: runtime.synthetic_vblanks,
        v_int_counter: runtime.core.mem.read_work_ram_long(V_INT_COUNTER_ADDR),
        v_int_done: runtime.core.mem.read_work_ram_byte(V_INT_DONE_ADDR),
        demo_mode_flag: runtime.core.mem.read_work_ram_byte(DEMO_MODE_FLAG_ADDR),
        game_mode: runtime.core.mem.read_work_ram_word(GAME_MODE_ADDR),
        current_player: runtime.core.mem.read_work_ram_byte(CURRENT_PLAYER_ADDR),
        number_lives: runtime.core.mem.read_work_ram_word(NUMBER_LIVES_ADDR),
        number_hitpoints: runtime.core.mem.read_work_ram_word(NUMBER_HITPOINTS_ADDR),
        current_level_id: runtime.core.mem.read_work_ram_word(CURRENT_LEVEL_ID_ADDR),
        player_1_level_id: runtime.core.mem.read_work_ram_word(PLAYER_1_LEVEL_ID_ADDR),
        current_helmet_id: runtime.core.mem.read_work_ram_word(CURRENT_HELMET_ID_ADDR),
        camera_x_pos: runtime.core.mem.read_work_ram_long(CAMERA_X_POS_ADDR),
        camera_y_pos: runtime.core.mem.read_work_ram_long(CAMERA_Y_POS_ADDR),
        kid_x_pos: runtime.core.mem.read_work_ram_long(KID_X_POS_ADDR),
        kid_y_pos: runtime.core.mem.read_work_ram_long(KID_Y_POS_ADDR),
        kid_x_vel: runtime.core.mem.read_work_ram_long(KID_X_VEL_ADDR) as i32,
        kid_y_vel: runtime.core.mem.read_work_ram_long(KID_Y_VEL_ADDR) as i32,
        character_movement: runtime.core.mem.read_work_ram_word(CHARACTER_MOVEMENT_ADDR),
        kid_collision_type: runtime.core.mem.read_work_ram_word(KID_COLLISION_TYPE_ADDR),
        invulnerable_frames: runtime
            .core
            .mem
            .read_work_ram_byte(INVULNERABLE_FRAMES_ADDR),
        just_received_damage: runtime
            .core
            .mem
            .read_work_ram_byte(JUST_RECEIVED_DAMAGE_ADDR)
            != 0,
        number_objects: runtime.core.mem.read_work_ram_word(NUMBER_OBJECTS_ADDR),
        number_gfx_objects: runtime.core.mem.read_work_ram_word(NUMBER_GFX_OBJECTS_ADDR),
        z80_interrupt_mode: runtime.z80.interrupt_mode,
        vram_nonzero_bytes: runtime.core.mem.vram_nonzero_bytes(),
        cram_nonzero_bytes: runtime.core.mem.cram_nonzero_bytes(),
        vsram_nonzero_bytes: runtime.core.mem.vsram_nonzero_bytes(),
        z80_reset_count: runtime.core.mem.z80_reset_count(),
        ym2612_write_count: runtime.core.mem.ym2612_write_count(),
        ym2612_key_on_count: runtime.core.mem.ym2612_key_on_count(),
        psg_write_count: runtime.core.mem.psg_write_count(),
        ym2612_writes: runtime.core.mem.ym2612_writes(),
        psg_writes: runtime.core.mem.psg_writes(),
        banked_68k_reads: runtime.core.mem.banked_68k_reads(),
        banked_68k_writes: runtime.core.mem.banked_68k_writes(),
        vdp: runtime.core.mem.vdp_snapshot(),
        steps,
        accesses: runtime.core.mem.accesses(),
        vdp_control_writes: runtime.core.mem.vdp_control_writes(),
        vdp_data_writes: runtime.core.mem.vdp_data_writes(),
    })
}

impl MdRuntime {
    pub fn new(rom: &[u8]) -> Result<Self> {
        if rom.len() < 8 {
            return Err(ProbeError::RomTooSmall.into());
        }

        let z80_shared = Rc::new(RefCell::new(Z80Shared::default()));
        let ym2612_shadow = Rc::new(RefCell::new(Ym2612Shadow::new()));
        let mut core = ConfiguredCore::new_with(
            0,
            AutoInterruptController::new(),
            MdBus::new(rom, z80_shared.clone(), ym2612_shadow.clone()),
        );
        core.reset();
        let mut z80 = Z80::new(Z80Bus {
            shared: z80_shared,
            ym2612_shadow,
            rom: rom.to_vec(),
        });
        z80.reset();
        Ok(Self {
            core,
            z80,
            z80_reset_asserted: false,
            z80_vblank_irq_cycles_remaining: 0,
            audio_mixer: AudioMixer::default(),
            cycles_until_vblank: SYNTHETIC_VBLANK_CYCLES,
            synthetic_vblanks: 0,
        })
    }

    pub fn set_controller_1(&mut self, buttons: u8) {
        self.core.mem.set_controller_1(buttons);
    }

    pub fn set_level_skip_cheat_enabled(&mut self, enabled: bool) {
        self.core
            .mem
            .write_work_ram_byte(LEVEL_SKIP_CHEAT_ADDR, if enabled { u8::MAX } else { 0 });
    }

    pub fn set_current_level(&mut self, level_id: u16) {
        self.core
            .mem
            .write_work_ram_word(CURRENT_LEVEL_ID_ADDR, level_id);
        self.core
            .mem
            .write_work_ram_word(PLAYER_1_LEVEL_ID_ADDR, level_id);
    }

    pub fn set_game_mode(&mut self, mode: u16) {
        self.core.mem.write_work_ram_word(GAME_MODE_ADDR, mode);
    }

    pub fn set_current_helmet(&mut self, helmet_id: u16) {
        self.core
            .mem
            .write_work_ram_word(CURRENT_HELMET_ID_ADDR, helmet_id);
    }

    pub fn set_lives(&mut self, lives: u16) {
        self.core.mem.write_work_ram_word(NUMBER_LIVES_ADDR, lives);
    }

    pub fn set_hitpoints(&mut self, hitpoints: u16) {
        self.core
            .mem
            .write_work_ram_word(NUMBER_HITPOINTS_ADDR, hitpoints);
    }

    /// Read a range of work RAM bytes (big-endian, as stored on 68k).
    pub fn read_work_ram_range(&self, address: u32, len: usize) -> Vec<u8> {
        (0..len)
            .map(|i| {
                self.core
                    .mem
                    .read_work_ram_byte(address.wrapping_add(i as u32))
            })
            .collect()
    }

    pub fn read_work_ram_word(&self, address: u32) -> u16 {
        self.core.mem.read_work_ram_word(address)
    }

    pub fn game_state(&self) -> GameState {
        GameState {
            current_level_id: self.core.mem.read_work_ram_word(CURRENT_LEVEL_ID_ADDR),
            current_helmet_id: self.core.mem.read_work_ram_word(CURRENT_HELMET_ID_ADDR),
            number_lives: self.core.mem.read_work_ram_word(NUMBER_LIVES_ADDR),
            number_hitpoints: self.core.mem.read_work_ram_word(NUMBER_HITPOINTS_ADDR),
            kid_x_pos: self.core.mem.read_work_ram_long(KID_X_POS_ADDR),
            kid_y_pos: self.core.mem.read_work_ram_long(KID_Y_POS_ADDR),
            camera_x_pos: self.core.mem.read_work_ram_long(CAMERA_X_POS_ADDR),
            camera_y_pos: self.core.mem.read_work_ram_long(CAMERA_Y_POS_ADDR),
        }
    }

    pub fn step(&mut self) -> i32 {
        let Cycles(cycles) = self.core.execute1();
        self.core.mem.advance_ym2612_shadow(cycles.max(0) as u32);
        self.core.mem.tick_controller_ports(cycles.max(0) as u32);
        self.step_z80(cycles);
        self.cycles_until_vblank -= cycles;
        while self.cycles_until_vblank <= 0 {
            self.cycles_until_vblank += SYNTHETIC_VBLANK_CYCLES;
            self.z80_vblank_irq_cycles_remaining = Z80_VBLANK_IRQ_CYCLES;
            // Count every synthetic display frame even if the ROM masks VInt;
            // capture/verification frame indices track rendered frames, not
            // only interrupt-delivering frames.
            self.synthetic_vblanks += 1;
            if self.core.mem.vint_enabled() {
                self.core.int_ctrl.request_interrupt(6);
            }
        }
        cycles
    }

    pub fn run_until_vblank(&mut self, max_steps: u32) -> bool {
        let starting_vint_counter = self.core.mem.read_work_ram_long(V_INT_COUNTER_ADDR);
        for _ in 0..max_steps {
            self.step();
            if self.core.mem.read_work_ram_long(V_INT_COUNTER_ADDR) != starting_vint_counter {
                return true;
            }
        }
        false
    }

    pub fn run_until_frame_rendered(&mut self, max_steps: u32) -> bool {
        let starting_synthetic_vblanks = self.synthetic_vblanks;
        for _ in 0..max_steps {
            self.step();
            if self.synthetic_vblanks != starting_synthetic_vblanks {
                return true;
            }
        }
        false
    }

    pub fn snapshot(&self) -> CpuProbe {
        CpuProbe {
            reset_sp: 0,
            reset_pc: 0,
            final_pc: self.core.pc,
            final_sp: self.core.dar[15],
            synthetic_vblanks: self.synthetic_vblanks,
            v_int_counter: self.core.mem.read_work_ram_long(V_INT_COUNTER_ADDR),
            v_int_done: self.core.mem.read_work_ram_byte(V_INT_DONE_ADDR),
            demo_mode_flag: self.core.mem.read_work_ram_byte(DEMO_MODE_FLAG_ADDR),
            game_mode: self.core.mem.read_work_ram_word(GAME_MODE_ADDR),
            current_player: self.core.mem.read_work_ram_byte(CURRENT_PLAYER_ADDR),
            number_lives: self.core.mem.read_work_ram_word(NUMBER_LIVES_ADDR),
            number_hitpoints: self.core.mem.read_work_ram_word(NUMBER_HITPOINTS_ADDR),
            current_level_id: self.core.mem.read_work_ram_word(CURRENT_LEVEL_ID_ADDR),
            player_1_level_id: self.core.mem.read_work_ram_word(PLAYER_1_LEVEL_ID_ADDR),
            current_helmet_id: self.core.mem.read_work_ram_word(CURRENT_HELMET_ID_ADDR),
            camera_x_pos: self.core.mem.read_work_ram_long(CAMERA_X_POS_ADDR),
            camera_y_pos: self.core.mem.read_work_ram_long(CAMERA_Y_POS_ADDR),
            kid_x_pos: self.core.mem.read_work_ram_long(KID_X_POS_ADDR),
            kid_y_pos: self.core.mem.read_work_ram_long(KID_Y_POS_ADDR),
            kid_x_vel: self.core.mem.read_work_ram_long(KID_X_VEL_ADDR) as i32,
            kid_y_vel: self.core.mem.read_work_ram_long(KID_Y_VEL_ADDR) as i32,
            character_movement: self.core.mem.read_work_ram_word(CHARACTER_MOVEMENT_ADDR),
            kid_collision_type: self.core.mem.read_work_ram_word(KID_COLLISION_TYPE_ADDR),
            invulnerable_frames: self.core.mem.read_work_ram_byte(INVULNERABLE_FRAMES_ADDR),
            just_received_damage: self.core.mem.read_work_ram_byte(JUST_RECEIVED_DAMAGE_ADDR) != 0,
            number_objects: self.core.mem.read_work_ram_word(NUMBER_OBJECTS_ADDR),
            number_gfx_objects: self.core.mem.read_work_ram_word(NUMBER_GFX_OBJECTS_ADDR),
            z80_interrupt_mode: self.z80.interrupt_mode,
            vram_nonzero_bytes: self.core.mem.vram_nonzero_bytes(),
            cram_nonzero_bytes: self.core.mem.cram_nonzero_bytes(),
            vsram_nonzero_bytes: self.core.mem.vsram_nonzero_bytes(),
            z80_reset_count: self.core.mem.z80_reset_count(),
            ym2612_write_count: self.core.mem.ym2612_write_count(),
            ym2612_key_on_count: self.core.mem.ym2612_key_on_count(),
            psg_write_count: self.core.mem.psg_write_count(),
            ym2612_writes: self.core.mem.ym2612_writes(),
            psg_writes: self.core.mem.psg_writes(),
            banked_68k_reads: self.core.mem.banked_68k_reads(),
            banked_68k_writes: self.core.mem.banked_68k_writes(),
            vdp: self.core.mem.vdp_snapshot(),
            steps: Vec::new(),
            accesses: self.core.mem.accesses(),
            vdp_control_writes: self.core.mem.vdp_control_writes(),
            vdp_data_writes: self.core.mem.vdp_data_writes(),
        }
    }

    pub fn render_into(&self, rgb: &mut [u8]) {
        render_vdp_into(&self.core.mem.vdp_snapshot(), rgb);
    }

    pub fn generate_audio_samples(&mut self, sample_rate: u32, sample_count: usize) -> Vec<f32> {
        self.audio_mixer
            .push_events(self.core.mem.drain_audio_events());
        self.audio_mixer.generate(sample_rate, sample_count)
    }

    pub fn set_audio_levels(&mut self, levels: AudioLevels) {
        self.audio_mixer.set_levels(levels);
    }

    fn step_z80(&mut self, m68k_cycles: i32) {
        let (busreq, reset) = {
            let shared = self.core.mem.z80_shared.borrow();
            (shared.busreq, shared.reset)
        };
        let mut remaining_cycles = ((m68k_cycles.max(0) as u32) * 7) / 15;
        if remaining_cycles == 0 {
            remaining_cycles = 1;
        }
        let mut audio_cycle_cursor = self.core.mem.z80_shared.borrow().z80_cycles;
        let audio_cycle_end = audio_cycle_cursor.saturating_add(u64::from(remaining_cycles));

        if reset {
            self.core.mem.z80_shared.borrow_mut().z80_cycles = audio_cycle_end;
            if !self.z80_reset_asserted {
                self.z80.reset();
                self.z80_reset_asserted = true;
            }
            self.z80_vblank_irq_cycles_remaining = 0;
            self.z80.clr_irq();
            return;
        }
        self.z80_reset_asserted = false;

        if busreq {
            self.core.mem.z80_shared.borrow_mut().z80_cycles = audio_cycle_end;
            self.z80.clr_irq();
            return;
        }
        while remaining_cycles > 0 {
            self.core.mem.z80_shared.borrow_mut().z80_cycles = audio_cycle_cursor;
            if self.z80_vblank_irq_cycles_remaining != 0 {
                self.z80.assert_irq(0xFF);
            } else {
                self.z80.clr_irq();
            }
            let elapsed = self.z80.step();
            let elapsed_budget = elapsed.min(remaining_cycles);
            audio_cycle_cursor = audio_cycle_cursor.saturating_add(u64::from(elapsed_budget));
            self.z80_vblank_irq_cycles_remaining = self
                .z80_vblank_irq_cycles_remaining
                .saturating_sub(elapsed_budget);
            if elapsed_budget >= remaining_cycles {
                break;
            }
            remaining_cycles -= elapsed_budget;
        }
        self.core.mem.z80_shared.borrow_mut().z80_cycles = audio_cycle_end;
        if self.z80_vblank_irq_cycles_remaining == 0 {
            self.z80.clr_irq();
        }
    }
}

pub fn render_frame(probe: &CpuProbe) -> FrameBuffer {
    let width = 320usize;
    let height = 224usize;
    let mut rgb = vec![0; width * height * 3];
    render_vdp_into(&probe.vdp, &mut rgb);
    FrameBuffer { width, height, rgb }
}

pub fn render_vdp_into(vdp: &VdpSnapshot, rgb: &mut [u8]) {
    let width = 320usize;
    let height = 224usize;
    let background = cram_rgb(&vdp.cram, usize::from(vdp.registers[7] & 0x3F));
    for pixel in rgb.chunks_exact_mut(3) {
        pixel.copy_from_slice(&background);
    }

    let plane_a_base = (usize::from(vdp.registers[2] & 0x38)) << 10;
    let plane_b_base = (usize::from(vdp.registers[4] & 0x07)) << 13;
    let window_base = window_plane_base(&vdp.registers);
    let sprite_table_base = sprite_table_base(&vdp.registers);
    let hscroll_base = usize::from(vdp.registers[0x0D]) << 10;
    let vscroll_a = i32::from(read_u16_wrapped(&vdp.vsram, 0) as i16);
    let vscroll_b = i32::from(read_u16_wrapped(&vdp.vsram, 2) as i16);
    let (plane_width_cells, plane_height_cells) = plane_dimensions(vdp.registers[0x10]);
    let window_region = window_region(&vdp.registers, width, height);
    let sprite_scan = SpriteLineScan::build(&vdp.vram, sprite_table_base, height);

    for y in 0..height {
        let plane_a = ScrollPlane {
            base: plane_a_base,
            width_cells: plane_width_cells,
            height_cells: plane_height_cells,
            hscroll: hscroll_for_line(&vdp.vram, hscroll_base, y, 0, vdp.registers[0x0B]),
            vscroll: vscroll_a,
        };
        let plane_b = ScrollPlane {
            base: plane_b_base,
            width_cells: plane_width_cells,
            height_cells: plane_height_cells,
            hscroll: hscroll_for_line(&vdp.vram, hscroll_base, y, 2, vdp.registers[0x0B]),
            vscroll: vscroll_b,
        };
        let window_plane = WindowPlane {
            base: window_base,
            region: window_region,
        };
        let line_sprites = sprite_scan.sprites_on_line(y);
        for x in 0..width {
            for priority in [false, true] {
                if let Some(colour) = plane_pixel_rgb(vdp, &plane_b, x, y, priority) {
                    write_rgb(rgb, width, x, y, colour);
                }
                if let Some(colour) =
                    plane_a_or_window_pixel_rgb(vdp, &plane_a, &window_plane, x, y, priority)
                {
                    write_rgb(rgb, width, x, y, colour);
                }
                if let Some(colour) =
                    scanline_sprite_pixel_rgb(vdp, &sprite_scan, line_sprites, x, y, priority)
                {
                    write_rgb(rgb, width, x, y, colour);
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IoState {
    version_register: u8,
    p1_port: ControllerPortState,
    p2_port: ControllerPortState,
    z80_busreq: bool,
    z80_reset: bool,
}

impl Default for IoState {
    fn default() -> Self {
        Self {
            version_register: 0x20,
            p1_port: ControllerPortState::new(ControllerType::SixButton),
            p2_port: ControllerPortState::new(ControllerType::None),
            z80_busreq: false,
            z80_reset: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControllerType {
    None,
    SixButton,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ControllerPortState {
    controller_type: ControllerType,
    buttons: u8,
    last_data_write: u8,
    last_ctrl_write: u8,
    th_flip_count: u8,
    flip_reset_counter: u32,
    cycles_until_th_high: u32,
    controller_th: bool,
}

impl ControllerPortState {
    fn new(controller_type: ControllerType) -> Self {
        Self {
            controller_type,
            buttons: 0,
            last_data_write: 0,
            last_ctrl_write: 0,
            th_flip_count: 0,
            flip_reset_counter: 0,
            cycles_until_th_high: 0,
            controller_th: true,
        }
    }

    fn read_data(&self) -> u8 {
        if matches!(self.controller_type, ControllerType::None) {
            return 0xFF;
        }

        let mut controller_byte = match (self.th_flip_count, self.controller_th) {
            (0..=2, true) => {
                (u8::from(!button_pressed(self.buttons, 5)) << 5)
                    | (u8::from(!button_pressed(self.buttons, 4)) << 4)
                    | (u8::from(!button_pressed(self.buttons, 3)) << 3)
                    | (u8::from(!button_pressed(self.buttons, 2)) << 2)
                    | (u8::from(!button_pressed(self.buttons, 1)) << 1)
                    | u8::from(!button_pressed(self.buttons, 0))
            }
            (0..=1, false) => {
                (u8::from(!button_pressed(self.buttons, 7)) << 5)
                    | (u8::from(!button_pressed(self.buttons, 6)) << 4)
                    | (u8::from(!button_pressed(self.buttons, 1)) << 1)
                    | u8::from(!button_pressed(self.buttons, 0))
            }
            (3, true) => {
                (u8::from(!button_pressed(self.buttons, 5)) << 5)
                    | (u8::from(!button_pressed(self.buttons, 4)) << 4)
                    | 0b0000_1111
            }
            (2, false) => {
                (u8::from(!button_pressed(self.buttons, 7)) << 5)
                    | (u8::from(!button_pressed(self.buttons, 6)) << 4)
            }
            (3, false) => {
                (u8::from(!button_pressed(self.buttons, 7)) << 5)
                    | (u8::from(!button_pressed(self.buttons, 6)) << 4)
                    | 0b0000_1111
            }
            _ => 0x3F,
        };
        controller_byte |= u8::from(self.controller_th) << CONTROLLER_TH_BIT;
        controller_byte &= !self.last_ctrl_write;
        let outputs_byte = self.last_data_write & (self.last_ctrl_write | 0x80);
        controller_byte | outputs_byte
    }

    fn read_ctrl(&self) -> u8 {
        self.last_ctrl_write
    }

    fn write_data(&mut self, value: u8) {
        self.last_data_write = value;
        self.maybe_set_th();
    }

    fn write_ctrl(&mut self, value: u8) {
        self.last_ctrl_write = value;
        self.maybe_set_th();
        self.cycles_until_th_high = if ctrl_bit_set(value, CONTROLLER_TH_BIT) {
            0
        } else {
            CONTROLLER_TH_HIGH_DELAY_CYCLES
        };
    }

    fn set_buttons(&mut self, buttons: u8) {
        self.buttons = buttons;
    }

    fn tick(&mut self, m68k_cycles: u32) {
        self.flip_reset_counter = self.flip_reset_counter.saturating_sub(m68k_cycles);
        if self.flip_reset_counter == 0 {
            self.th_flip_count = 0;
        }

        if self.cycles_until_th_high != 0 {
            self.cycles_until_th_high = self.cycles_until_th_high.saturating_sub(m68k_cycles);
            if self.cycles_until_th_high == 0 {
                self.controller_th = true;
            }
        }
    }

    fn maybe_set_th(&mut self) {
        if !ctrl_bit_set(self.last_ctrl_write, CONTROLLER_TH_BIT) {
            return;
        }

        let th = ctrl_bit_set(self.last_data_write, CONTROLLER_TH_BIT);
        if matches!(self.controller_type, ControllerType::SixButton) && !self.controller_th && th {
            self.th_flip_count = (self.th_flip_count + 1) % 4;
            self.flip_reset_counter = CONTROLLER_FLIP_COUNTER_CYCLES;
        }
        self.controller_th = th;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VdpState {
    status: u16,
    registers: [u8; 0x20],
    vram: Vec<u8>,
    cram: Vec<u8>,
    vsram: Vec<u8>,
    pending_control_word: Option<u16>,
    data_port_address: u16,
    data_port_code: u8,
    control_writes: Vec<u16>,
    data_writes: Vec<u16>,
}

impl Default for VdpState {
    fn default() -> Self {
        Self {
            status: 0x3400,
            registers: [0; 0x20],
            vram: vec![0; VRAM_LEN],
            cram: vec![0; CRAM_LEN],
            vsram: vec![0; VSRAM_LEN],
            pending_control_word: None,
            data_port_address: 0,
            data_port_code: 0x01,
            control_writes: Vec::new(),
            data_writes: Vec::new(),
        }
    }
}

#[derive(Clone)]
struct MdBus {
    rom: Vec<u8>,
    work_ram: Vec<u8>,
    z80_shared: Rc<RefCell<Z80Shared>>,
    ym2612_shadow: Rc<RefCell<Ym2612Shadow>>,
    io: IoState,
    vdp: VdpState,
    accesses: RefCell<Vec<BusAccess>>,
}

impl MdBus {
    fn new(
        rom: &[u8],
        z80_shared: Rc<RefCell<Z80Shared>>,
        ym2612_shadow: Rc<RefCell<Ym2612Shadow>>,
    ) -> Self {
        Self {
            rom: rom.to_vec(),
            work_ram: vec![0; WORK_RAM_LEN],
            z80_shared,
            ym2612_shadow,
            io: IoState::default(),
            vdp: VdpState::default(),
            accesses: RefCell::new(Vec::new()),
        }
    }

    #[cfg(test)]
    fn with_fresh_z80(rom: &[u8]) -> Self {
        Self::new(
            rom,
            Rc::new(RefCell::new(Z80Shared::default())),
            Rc::new(RefCell::new(Ym2612Shadow::new())),
        )
    }

    fn accesses(&self) -> Vec<BusAccess> {
        self.accesses.borrow().clone()
    }

    fn vdp_control_writes(&self) -> Vec<u16> {
        self.vdp.control_writes.clone()
    }

    fn vdp_data_writes(&self) -> Vec<u16> {
        self.vdp.data_writes.clone()
    }

    fn vdp_snapshot(&self) -> VdpSnapshot {
        VdpSnapshot {
            registers: self.vdp.registers,
            vram: self.vdp.vram.clone(),
            cram: self.vdp.cram.clone(),
            vsram: self.vdp.vsram.clone(),
        }
    }

    fn ym2612_write_count(&self) -> usize {
        self.z80_shared.borrow().ym2612_write_count
    }

    fn ym2612_key_on_count(&self) -> usize {
        self.z80_shared.borrow().ym2612_key_on_count
    }

    fn psg_write_count(&self) -> usize {
        self.z80_shared.borrow().psg_write_count
    }

    fn z80_reset_count(&self) -> usize {
        self.z80_shared.borrow().z80_reset_count
    }

    fn ym2612_writes(&self) -> Vec<(u16, u8)> {
        self.z80_shared.borrow().ym2612_writes.clone()
    }

    fn psg_writes(&self) -> Vec<u8> {
        self.z80_shared.borrow().psg_writes.clone()
    }

    fn banked_68k_reads(&self) -> Vec<(u32, u8)> {
        self.z80_shared.borrow().banked_68k_reads.clone()
    }

    fn banked_68k_writes(&self) -> Vec<(u32, u8)> {
        self.z80_shared.borrow().banked_68k_writes.clone()
    }

    fn drain_audio_events(&self) -> VecDeque<AudioEvent> {
        std::mem::take(&mut self.z80_shared.borrow_mut().audio_events)
    }

    fn vint_enabled(&self) -> bool {
        self.vdp.registers[1] & 0x20 != 0
    }

    fn vram_nonzero_bytes(&self) -> usize {
        self.vdp.vram.iter().filter(|byte| **byte != 0).count()
    }

    fn cram_nonzero_bytes(&self) -> usize {
        self.vdp.cram.iter().filter(|byte| **byte != 0).count()
    }

    fn vsram_nonzero_bytes(&self) -> usize {
        self.vdp.vsram.iter().filter(|byte| **byte != 0).count()
    }

    fn record_access(&self, kind: BusAccessKind, address: u32, value: u32) {
        let mut accesses = self.accesses.borrow_mut();
        if accesses.len() < TRACE_LIMIT {
            accesses.push(BusAccess {
                kind,
                address: address & ADDRESS_MASK,
                value,
            });
        }
    }

    fn read_rom_byte(&self, address: u32) -> u8 {
        self.rom.get(address as usize).copied().unwrap_or(0xFF)
    }

    fn work_ram_index(address: u32) -> usize {
        (address as usize) & (WORK_RAM_LEN - 1)
    }

    fn z80_ram_index(address: u32) -> usize {
        (address as usize) & (Z80_RAM_LEN - 1)
    }

    fn read_work_ram_byte(&self, address: u32) -> u8 {
        self.work_ram[Self::work_ram_index(address)]
    }

    fn read_work_ram_word(&self, address: u32) -> u16 {
        u16::from_be_bytes([
            self.read_work_ram_byte(address),
            self.read_work_ram_byte(address.wrapping_add(1)),
        ])
    }

    fn read_work_ram_long(&self, address: u32) -> u32 {
        (u32::from(self.read_work_ram_word(address)) << 16)
            | u32::from(self.read_work_ram_word(address.wrapping_add(2)))
    }

    fn write_work_ram_byte(&mut self, address: u32, value: u8) {
        let index = Self::work_ram_index(address);
        self.work_ram[index] = value;
        self.z80_shared.borrow_mut().m68k_work_ram[index] = value;
    }

    fn write_work_ram_word(&mut self, address: u32, value: u16) {
        self.write_work_ram_byte(address, (value >> 8) as u8);
        self.write_work_ram_byte(address.wrapping_add(1), value as u8);
    }

    fn read_io_register(&self, address: u32) -> u8 {
        match address {
            0x00A1_0000 | 0x00A1_0001 => self.io.version_register,
            0x00A1_0002 | 0x00A1_0003 => self.io.p1_port.read_data(),
            0x00A1_0004 | 0x00A1_0005 => self.io.p2_port.read_data(),
            0x00A1_0008 | 0x00A1_0009 => self.io.p1_port.read_ctrl(),
            0x00A1_000A | 0x00A1_000B => self.io.p2_port.read_ctrl(),
            0x00A1_000E | 0x00A1_000F | 0x00A1_0014 | 0x00A1_0015 | 0x00A1_001A | 0x00A1_001B => {
                0xFF
            }
            _ => 0x00,
        }
    }

    fn write_io_register(&mut self, address: u32, value: u8) {
        match address {
            0x00A1_0002 | 0x00A1_0003 => self.io.p1_port.write_data(value),
            0x00A1_0004 | 0x00A1_0005 => self.io.p2_port.write_data(value),
            0x00A1_0008 | 0x00A1_0009 => self.io.p1_port.write_ctrl(value),
            0x00A1_000A | 0x00A1_000B => self.io.p2_port.write_ctrl(value),
            _ => {}
        }
    }

    fn set_controller_1(&mut self, buttons: u8) {
        self.io.p1_port.set_buttons(buttons);
    }

    fn tick_controller_ports(&mut self, m68k_cycles: u32) {
        self.io.p1_port.tick(m68k_cycles);
        self.io.p2_port.tick(m68k_cycles);
    }

    fn advance_ym2612_shadow(&mut self, clocks: u32) {
        self.ym2612_shadow.borrow_mut().advance_clocks(clocks);
    }

    fn set_z80_busreq(&mut self, requested: bool) {
        self.io.z80_busreq = requested;
        self.z80_shared.borrow_mut().busreq = requested;
    }

    fn set_z80_reset(&mut self, asserted: bool) {
        let was_asserted = self.io.z80_reset;
        self.io.z80_reset = asserted;
        let mut shared = self.z80_shared.borrow_mut();
        shared.reset = asserted;
        if !was_asserted && asserted {
            shared.z80_reset_count = shared.z80_reset_count.saturating_add(1);
            shared.ym2612_port_latches = [None, None];
        }
    }

    fn z80_busack(&self) -> bool {
        self.io.z80_busreq && !self.io.z80_reset
    }

    fn read_vdp_byte(&self, address: u32) -> u8 {
        match address & 0x1F {
            0x04 | 0x06 => (self.vdp.status >> 8) as u8,
            0x05 | 0x07 => self.vdp.status as u8,
            0x08..=0x0B => 0x00,
            0x0C..=0x1F => 0xFF,
            _ => 0x00,
        }
    }

    fn write_vdp_byte(&mut self, address: u32, value: u8) {
        let duplicated = u16::from_be_bytes([value, value]);
        match address & 0x1F {
            0x00..=0x03 => self.write_vdp_data_word(duplicated),
            0x04..=0x07 => self.apply_vdp_control_write(duplicated),
            _ => {}
        }
    }

    fn apply_vdp_control_write(&mut self, value: u16) {
        self.vdp.control_writes.push(value);
        if value & 0xC000 == 0x8000 {
            let register = ((value >> 8) & 0x1F) as usize;
            self.vdp.registers[register] = value as u8;
            self.vdp.pending_control_word = None;
            return;
        }

        if let Some(first_word) = self.vdp.pending_control_word.take() {
            self.configure_vdp_command(first_word, value);
        } else {
            self.vdp.pending_control_word = Some(value);
        }
    }

    fn configure_vdp_command(&mut self, first_word: u16, second_word: u16) {
        let command = (u32::from(first_word) << 16) | u32::from(second_word);
        let code = (((command >> 30) & 0x3) as u8) | (((command >> 2) & 0xFC) as u8);
        let address = (((command >> 16) & 0x3FFF) as u16) | (((command & 0x3) as u16) << 14);
        self.vdp.data_port_code = code;
        self.vdp.data_port_address = address;

        if Self::is_dma_command(code) {
            self.execute_vdp_dma();
        }
    }

    fn auto_increment(&self) -> u16 {
        let increment = self.vdp.registers[0x0F];
        if increment == 0 {
            2
        } else {
            u16::from(increment)
        }
    }

    fn dma_length_words(&self) -> u32 {
        let length =
            u16::from_be_bytes([self.vdp.registers[0x14], self.vdp.registers[0x13]]) as u32;
        if length == 0 { 0x1_0000 } else { length }
    }

    fn dma_source_address(&self) -> u32 {
        (((u32::from(self.vdp.registers[0x17] & 0x7F)) << 16)
            | (u32::from(self.vdp.registers[0x16]) << 8)
            | u32::from(self.vdp.registers[0x15]))
            << 1
    }

    fn dma_mode(&self) -> u8 {
        self.vdp.registers[0x17] >> 6
    }

    fn is_dma_command(code: u8) -> bool {
        matches!(code, 0x21 | 0x23 | 0x25)
    }

    fn write_vdp_data_word(&mut self, value: u16) {
        self.vdp.data_writes.push(value);
        self.write_vdp_data_raw(value);
        self.vdp.data_port_address = self
            .vdp
            .data_port_address
            .wrapping_add(self.auto_increment());
    }

    fn write_vdp_data_raw(&mut self, value: u16) {
        match self.vdp.data_port_code & 0x0F {
            0x01 => {
                Self::write_vdp_word_slice(&mut self.vdp.vram, self.vdp.data_port_address, value);
            }
            0x03 => {
                Self::write_vdp_word_slice(&mut self.vdp.cram, self.vdp.data_port_address, value);
            }
            0x05 => {
                Self::write_vdp_word_slice(&mut self.vdp.vsram, self.vdp.data_port_address, value);
            }
            _ => {}
        }
    }

    fn write_vdp_word_slice(slice: &mut [u8], address: u16, value: u16) {
        if slice.is_empty() {
            return;
        }

        let index = usize::from(address) % slice.len();
        let [hi, lo] = value.to_be_bytes();
        slice[index] = hi;
        slice[(index + 1) % slice.len()] = lo;
    }

    fn execute_vdp_dma(&mut self) {
        match self.dma_mode() {
            0 | 1 => {
                let mut source = self.dma_source_address();
                let length = self.dma_length_words();
                for _ in 0..length {
                    let value = u16::from_be_bytes([
                        self.read_byte_internal(source),
                        self.read_byte_internal(source.wrapping_add(1)),
                    ]);
                    self.write_vdp_data_raw(value);
                    self.vdp.data_port_address = self
                        .vdp
                        .data_port_address
                        .wrapping_add(self.auto_increment());
                    source = source.wrapping_add(2);
                }
            }
            2 => {
                let length = self.dma_length_words();
                let fill_value = self.vdp.data_writes.last().copied().unwrap_or(0);
                for _ in 0..length {
                    self.write_vdp_data_raw(fill_value);
                    self.vdp.data_port_address = self
                        .vdp
                        .data_port_address
                        .wrapping_add(self.auto_increment());
                }
            }
            _ => {}
        }
    }

    fn read_byte_internal(&self, address: u32) -> u8 {
        match address {
            0x000000..=0x009F_FFFF => self.read_rom_byte(address),
            Z80_RAM_BASE..=Z80_RAM_END => {
                if self.z80_busack() {
                    self.z80_shared.borrow().ram[Self::z80_ram_index(address)]
                } else {
                    0xFF
                }
            }
            0x00A1_0000..=0x00A1_001F => self.read_io_register(address),
            0x00A1_1100..=0x00A1_1101 => u8::from(!self.z80_busack()),
            0x00A1_1200..=0x00A1_1201 => u8::from(!self.io.z80_reset),
            0x00C0_0000..=0x00C0_001F => self.read_vdp_byte(address),
            WORK_RAM_BASE..=WORK_RAM_END => self.work_ram[Self::work_ram_index(address)],
            _ => 0xFF,
        }
    }

    fn write_byte_internal(&mut self, address: u32, value: u8) {
        match address {
            Z80_RAM_BASE..=Z80_RAM_END => {
                if self.z80_busack() {
                    let index = Self::z80_ram_index(address);
                    self.z80_shared.borrow_mut().ram[index] = value;
                }
            }
            0x00A1_0000..=0x00A1_001F => self.write_io_register(address, value),
            0x00A1_1100..=0x00A1_1101 => self.set_z80_busreq(value & 1 != 0),
            0x00A1_1200..=0x00A1_1201 => self.set_z80_reset(value & 1 == 0),
            0x00C0_0000..=0x00C0_001F => self.write_vdp_byte(address, value),
            WORK_RAM_BASE..=WORK_RAM_END => {
                self.write_work_ram_byte(address, value);
            }
            _ => {}
        }
    }

    fn read_word_internal(&self, address: u32) -> u16 {
        match address {
            Z80_RAM_BASE..=Z80_RAM_END => {
                if self.z80_busack() {
                    let byte = self.z80_shared.borrow().ram[Self::z80_ram_index(address)];
                    u16::from_be_bytes([byte, byte])
                } else {
                    0xFF00
                }
            }
            0x00A1_0000..=0x00A1_001F => u16::from(self.read_io_register(address)),
            0x00A1_1100..=0x00A1_1101 => {
                let byte = u8::from(!self.z80_busack());
                u16::from_be_bytes([byte, byte])
            }
            0x00A1_1200..=0x00A1_1201 => {
                let byte = u8::from(!self.io.z80_reset);
                u16::from_be_bytes([byte, byte])
            }
            0x00C0_0000..=0x00C0_0003 => 0,
            0x00C0_0004..=0x00C0_0007 => self.vdp.status,
            0x00C0_0008..=0x00C0_000F => 0,
            _ => u16::from_be_bytes([
                self.read_byte_internal(address),
                self.read_byte_internal(address.wrapping_add(1)),
            ]),
        }
    }

    fn write_word_internal(&mut self, address: u32, value: u16) {
        match address {
            Z80_RAM_BASE..=Z80_RAM_END => self.write_byte_internal(address, (value >> 8) as u8),
            0x00A1_0000..=0x00A1_001F => self.write_io_register(address, value as u8),
            0x00A1_1100..=0x00A1_1101 => self.set_z80_busreq((value & 0x0100) != 0),
            0x00A1_1200..=0x00A1_1201 => self.set_z80_reset((value & 0x0100) == 0),
            0x00C0_0000..=0x00C0_0003 => self.write_vdp_data_word(value),
            0x00C0_0004..=0x00C0_0007 => self.apply_vdp_control_write(value),
            WORK_RAM_BASE..=WORK_RAM_END => {
                self.write_work_ram_word(address, value);
            }
            _ => {
                self.write_byte_internal(address, (value >> 8) as u8);
                self.write_byte_internal(address.wrapping_add(1), value as u8);
            }
        }
    }
}

impl Z80Bus {
    fn read_banked_68k_byte(&self, address: u16) -> u8 {
        let mut shared = self.shared.borrow_mut();
        let banked_address = shared
            .bank_address
            .wrapping_add(u32::from(address.wrapping_sub(0x8000)));
        let value = if (WORK_RAM_BASE..=WORK_RAM_END).contains(&banked_address) {
            shared.m68k_work_ram[MdBus::work_ram_index(banked_address)]
        } else {
            self.rom
                .get(banked_address as usize)
                .copied()
                .unwrap_or(0xFF)
        };
        if shared.banked_68k_reads.len() < TRACE_LIMIT {
            shared.banked_68k_reads.push((banked_address, value));
        }
        value
    }

    fn write_banked_68k_byte(&self, address: u16, value: u8) {
        let mut shared = self.shared.borrow_mut();
        let banked_address = shared
            .bank_address
            .wrapping_add(u32::from(address.wrapping_sub(0x8000)));
        if (WORK_RAM_BASE..=WORK_RAM_END).contains(&banked_address) {
            let index = MdBus::work_ram_index(banked_address);
            shared.m68k_work_ram[index] = value;
        }
        if shared.banked_68k_writes.len() < TRACE_LIMIT {
            shared.banked_68k_writes.push((banked_address, value));
        }
    }

    fn record_ym2612_write(&self, address: u16, value: u8) {
        self.ym2612_shadow
            .borrow_mut()
            .write_port((address & 0x0003) as u8, value);
        let mut shared = self.shared.borrow_mut();
        let z80_cycles = shared.z80_cycles;
        match address & 0x0003 {
            0 => shared.ym2612_port_latches[0] = Some(u16::from(value)),
            1 => {
                if shared.ym2612_port_latches[0] == Some(0x0028) {
                    shared.ym2612_key_on_count = shared.ym2612_key_on_count.saturating_add(1);
                }
            }
            2 => shared.ym2612_port_latches[1] = Some(0x0100 | u16::from(value)),
            3 => {
                if shared.ym2612_port_latches[1] == Some(0x0128) {
                    shared.ym2612_key_on_count = shared.ym2612_key_on_count.saturating_add(1);
                }
            }
            _ => {}
        }
        shared.ym2612_write_count = shared.ym2612_write_count.saturating_add(1);
        if shared.ym2612_writes.len() < TRACE_LIMIT {
            shared.ym2612_writes.push((address, value));
        }
        shared.audio_events.push_back(AudioEvent::Ym2612Write {
            z80_cycles,
            port: (address & 0x0003) as u8,
            value,
        });
    }

    fn record_psg_write(&self, value: u8) {
        let mut shared = self.shared.borrow_mut();
        let z80_cycles = shared.z80_cycles;
        shared.psg_write_count = shared.psg_write_count.saturating_add(1);
        if shared.psg_writes.len() < TRACE_LIMIT {
            shared.psg_writes.push(value);
        }
        let _ = shared.psg_registers.apply_write(value);
        shared
            .audio_events
            .push_back(AudioEvent::PsgWrite { z80_cycles, value });
    }

    fn write_bank_register(&self, value: u8) {
        let mut shared = self.shared.borrow_mut();
        let current_bank = shared.bank_address >> 15;
        let next_bank = ((current_bank >> 1) | (u32::from(value & 1) << 8)) & 0x01FF;
        shared.bank_address = next_bank << 15;
    }
}

impl Z80_io for Z80Bus {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.shared.borrow().ram[usize::from(address) & (Z80_RAM_LEN - 1)],
            0x4000..=0x5FFF => self
                .ym2612_shadow
                .borrow_mut()
                .read_port((address & 0x0003) as u8),
            0x8000..=0xFFFF => self.read_banked_68k_byte(address),
            _ => 0xFF,
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.shared.borrow_mut().ram[usize::from(address) & (Z80_RAM_LEN - 1)] = value;
            }
            0x4000..=0x5FFF => self.record_ym2612_write(address & 0x0003, value),
            0x6000..=0x60FF => self.write_bank_register(value),
            0x7F11 => self.record_psg_write(value),
            0x8000..=0xFFFF => self.write_banked_68k_byte(address, value),
            _ => {}
        }
    }

    fn port_in(&self, _address: u16) -> u8 {
        0xFF
    }

    fn port_out(&mut self, address: u16, value: u8) {
        let _ = (address, value);
    }
}

impl PsgSynth {
    fn render_sample(&mut self, registers: &PsgRegisters, sample_rate: u32) -> f32 {
        let psg_clocks_per_sample = (PSG_CLOCK_HZ / PSG_DIVIDER) / f64::from(sample_rate);
        self.pending_psg_clocks += psg_clocks_per_sample;
        while self.pending_psg_clocks >= 1.0 {
            self.pending_psg_clocks -= 1.0;
            self.clock(registers);
        }

        let mixed = self.sample(registers);
        self.apply_dc_block(mixed).clamp(-0.25, 0.25)
    }

    fn clock(&mut self, registers: &PsgRegisters) {
        for channel in 0..3 {
            self.tone_counters[channel] = self.tone_counters[channel].saturating_sub(1);
            if self.tone_counters[channel] == 0 {
                self.tone_counters[channel] = registers.tone_periods[channel].max(1);
                self.tone_outputs_high[channel] = !self.tone_outputs_high[channel];
            }
        }

        self.noise_counter = self.noise_counter.saturating_sub(1);
        if self.noise_counter != 0 {
            return;
        }

        self.noise_counter = noise_reload(registers);
        self.noise_clock_high = !self.noise_clock_high;
        if self.noise_clock_high {
            self.shift_noise_lfsr(registers.noise_control);
        }
    }

    fn shift_noise_lfsr(&mut self, noise_control: u8) {
        self.noise_output_high = self.noise_lfsr & 1 != 0;
        let feedback = if noise_control & 0x04 != 0 {
            (self.noise_lfsr & 1) ^ ((self.noise_lfsr >> 3) & 1)
        } else {
            self.noise_lfsr & 1
        };
        self.noise_lfsr = (self.noise_lfsr >> 1) | (feedback << 15);
    }

    fn sample(&self, registers: &PsgRegisters) -> f32 {
        let mut mixed = 0.0f32;
        for channel in 0..3 {
            let volume = psg_volume(registers.volumes[channel]);
            if volume == 0.0 {
                continue;
            }
            mixed += if self.tone_outputs_high[channel] {
                volume
            } else {
                -volume
            };
        }
        let noise_volume = psg_volume(registers.volumes[3]);
        if noise_volume != 0.0 {
            mixed += if self.noise_output_high {
                noise_volume
            } else {
                -noise_volume
            };
        }

        mixed / 4.0
    }

    fn apply_dc_block(&mut self, input: f32) -> f32 {
        let output =
            input - self.dc_previous_input + (PSG_DC_BLOCK_COEFFICIENT * self.dc_previous_output);
        self.dc_previous_input = input;
        self.dc_previous_output = output;
        output
    }
}

impl AudioMixer {
    fn set_levels(&mut self, levels: AudioLevels) {
        self.levels = AudioLevels {
            master: levels.master.clamp(0.0, 1.0),
            ym2612: levels.ym2612.clamp(0.0, 1.0),
            psg: levels.psg.clamp(0.0, 1.0),
        };
    }

    fn push_events(&mut self, events: VecDeque<AudioEvent>) {
        self.pending_events.extend(events);
    }

    fn ensure_psg_filter(&mut self, sample_rate: u32) {
        if self.psg_output_sample_rate == sample_rate {
            return;
        }
        self.psg_low_pass_filter =
            FirstOrderFilter::low_pass(GENESIS_LPF_CUTOFF_HZ, f64::from(sample_rate));
        self.psg_output_sample_rate = sample_rate;
    }

    fn generate(&mut self, sample_rate: u32, sample_count: usize) -> Vec<f32> {
        if sample_rate == 0 || sample_count == 0 {
            return Vec::new();
        }

        self.ensure_psg_filter(sample_rate);
        let ym_sample_rate = f64::from(self.ym2612.sample_rate_hz());
        let z80_cycles_per_output = Z80_CLOCK_HZ / f64::from(sample_rate);
        let z80_cycles_per_ym = Z80_CLOCK_HZ / ym_sample_rate;
        let mut mixed = Vec::with_capacity(sample_count * 2);

        for _ in 0..sample_count {
            let output_cycle_end =
                (((self.output_samples_emitted + 1) as f64) * z80_cycles_per_output).floor() as u64;

            loop {
                let ym_cycle_end =
                    (((self.ym_samples_emitted + 1) as f64) * z80_cycles_per_ym).floor() as u64;
                if ym_cycle_end > output_cycle_end {
                    break;
                }

                self.apply_events_up_to(ym_cycle_end);
                self.last_ym_sample = self.generate_ym_sample();
                self.ym_samples_emitted += 1;
            }

            self.apply_events_up_to(output_cycle_end);
            let ym_sample = self.last_ym_sample;
            let psg_sample = self.psg_low_pass_filter.filter(
                self.psg_synth
                    .render_sample(&self.psg_registers, sample_rate),
            );
            let left = ((ym_sample[0] * YM2612_GAIN * self.levels.ym2612)
                + (psg_sample * PSG_GAIN * self.levels.psg))
                * self.levels.master;
            let right = ((ym_sample[1] * YM2612_GAIN * self.levels.ym2612)
                + (psg_sample * PSG_GAIN * self.levels.psg))
                * self.levels.master;
            let left = left.clamp(-1.0, 1.0);
            let right = right.clamp(-1.0, 1.0);
            mixed.push(left);
            mixed.push(right);
            self.output_samples_emitted += 1;
        }

        mixed
    }

    fn apply_events_up_to(&mut self, z80_cycles: u64) {
        loop {
            let Some(event) = self.pending_events.front().copied() else {
                break;
            };
            if event.z80_cycles() > z80_cycles {
                break;
            }

            match event {
                AudioEvent::Ym2612Write { port, value, .. } => self.ym2612.write_port(port, value),
                AudioEvent::PsgWrite { value, .. } => {
                    let noise_reset = self.psg_registers.apply_write(value);
                    if noise_reset {
                        self.psg_synth.reset_noise_lfsr();
                    }
                }
            }
            self.pending_events.pop_front();
        }
    }

    fn generate_ym_sample(&mut self) -> [f32; 2] {
        let mut left = [0i32; 1];
        let mut right = [0i32; 1];
        self.ym2612.generate(&mut left, &mut right);
        [
            self.apply_ym_filters(0, left[0] as f32 / 32768.0),
            self.apply_ym_filters(1, right[0] as f32 / 32768.0),
        ]
    }

    fn apply_ym_filters(&mut self, channel: usize, input: f32) -> f32 {
        let sample = self.ym_dc_filters[channel].filter(input);
        let sample = self.ym_low_pass_filters[channel].filter(sample);
        self.ym_second_low_pass_filters[channel].filter(sample)
    }
}

fn psg_volume(volume: u8) -> f32 {
    PSG_ATTENUATION_TO_VOLUME[usize::from(volume.min(0x0F))]
}

fn noise_reload(registers: &PsgRegisters) -> u16 {
    match registers.noise_control & 0x03 {
        0 => 0x10,
        1 => 0x20,
        2 => 0x40,
        _ => registers.tone_periods[2].max(1),
    }
}

fn plane_dimensions(size_register: u8) -> (usize, usize) {
    let width_cells = match size_register & 0x03 {
        0x00 => 32,
        0x01 => 64,
        0x03 => 128,
        _ => 32,
    };
    let height_cells = match (size_register >> 4) & 0x03 {
        0x00 => 32,
        0x01 => 64,
        0x03 => 128,
        _ => 32,
    };
    (width_cells, height_cells)
}

fn hscroll_for_line(
    vram: &[u8],
    base: usize,
    line: usize,
    plane_offset: usize,
    mode_register: u8,
) -> i32 {
    let offset = match mode_register & 0x03 {
        0x00 => base + plane_offset,
        0x01 => base + 4 * (line & 0x07) + plane_offset,
        0x02 => base + 32 * (line / 8) + plane_offset,
        0x03 => base + 4 * line + plane_offset,
        _ => unreachable!("mode register is masked to two bits"),
    };
    i32::from(read_u16_wrapped(vram, offset) & 0x03FF)
}

#[inline]
fn plane_pixel_rgb(
    vdp: &VdpSnapshot,
    plane: &ScrollPlane,
    screen_x: usize,
    screen_y: usize,
    priority: bool,
) -> Option<[u8; 3]> {
    let plane_width_pixels = (plane.width_cells * 8) as i32;
    let plane_height_pixels = (plane.height_cells * 8) as i32;
    let world_x = (screen_x as i32 - plane.hscroll).rem_euclid(plane_width_pixels);
    let world_y = (screen_y as i32 + plane.vscroll).rem_euclid(plane_height_pixels);
    let cell_x = (world_x as usize) / 8;
    let cell_y = (world_y as usize) / 8;
    let cell_index = cell_y * plane.width_cells + cell_x;
    let entry = read_u16_wrapped(&vdp.vram, plane.base + cell_index * 2);
    if (entry & 0x8000 != 0) != priority {
        return None;
    }
    tile_pixel_rgb(vdp, entry, (world_x as usize) & 7, (world_y as usize) & 7)
}

fn plane_a_or_window_pixel_rgb(
    vdp: &VdpSnapshot,
    plane_a: &ScrollPlane,
    window: &WindowPlane,
    screen_x: usize,
    screen_y: usize,
    priority: bool,
) -> Option<[u8; 3]> {
    if let Some(region) = window.region
        && screen_x >= region.left
        && screen_x < region.left + region.width
        && screen_y >= region.top
        && screen_y < region.top + region.height
    {
        let window_x = screen_x - region.left;
        let window_y = screen_y - region.top;
        return window_pixel_rgb(vdp, window.base, window_x, window_y, priority);
    }

    plane_pixel_rgb(vdp, plane_a, screen_x, screen_y, priority)
}

fn window_pixel_rgb(
    vdp: &VdpSnapshot,
    window_base: usize,
    screen_x: usize,
    screen_y: usize,
    priority: bool,
) -> Option<[u8; 3]> {
    let map_width_cells = if is_h40_mode(vdp.registers[0x0C]) {
        64
    } else {
        32
    };
    let cell_x = screen_x / 8;
    let cell_y = screen_y / 8;
    let cell_index = cell_y * map_width_cells + cell_x;
    let entry = read_u16_wrapped(&vdp.vram, window_base + cell_index * 2);
    if (entry & 0x8000 != 0) != priority {
        return None;
    }

    tile_pixel_rgb(vdp, entry, screen_x & 7, screen_y & 7)
}

const MAX_SPRITES: usize = 80;
// H40 mode allows 20 sprites per scanline; H32 allows 16. Use 20 as the upper bound.
const MAX_SPRITES_PER_LINE: usize = 20;

struct SpriteEntry {
    xpos: i32,
    ypos: i32,
    width_pixels: i32,
    height_pixels: i32,
    width_tiles: usize,
    height_tiles: usize,
    pattern: u16,
}

struct SpriteLineScan {
    entries: Vec<SpriteEntry>,
    // Per-scanline: indices into `entries` for sprites visible on that line.
    line_indices: Vec<Vec<u8>>,
}

impl SpriteLineScan {
    fn build(vram: &[u8], sprite_table_base: usize, screen_height: usize) -> Self {
        let mut entries = Vec::with_capacity(MAX_SPRITES);
        let mut line_indices = vec![Vec::new(); screen_height];
        let mut sprite_index = 0usize;
        let mut visited = [false; MAX_SPRITES];

        for _ in 0..MAX_SPRITES {
            if sprite_index >= MAX_SPRITES || visited[sprite_index] {
                break;
            }
            visited[sprite_index] = true;

            let entry_base = sprite_table_base + sprite_index * 8;
            let ypos = i32::from(read_u16_wrapped(vram, entry_base)) - 128;
            let size_link = read_u16_wrapped(vram, entry_base + 2);
            let pattern = read_u16_wrapped(vram, entry_base + 4);
            let xpos = i32::from(read_u16_wrapped(vram, entry_base + 6)) - 128;
            let next_link = usize::from(size_link & 0x7F);
            let width_tiles = usize::from(((size_link >> 10) & 0x03) + 1);
            let height_tiles = usize::from(((size_link >> 8) & 0x03) + 1);
            let width_pixels = (width_tiles * 8) as i32;
            let height_pixels = (height_tiles * 8) as i32;

            let entry_idx = entries.len() as u8;
            entries.push(SpriteEntry {
                xpos,
                ypos,
                width_pixels,
                height_pixels,
                width_tiles,
                height_tiles,
                pattern,
            });

            let top = ypos.max(0) as usize;
            let bottom = ((ypos + height_pixels) as usize).min(screen_height);
            for line in &mut line_indices[top.min(bottom)..bottom] {
                if line.len() < MAX_SPRITES_PER_LINE {
                    line.push(entry_idx);
                }
            }

            if next_link == 0 {
                break;
            }
            sprite_index = next_link;
        }

        Self {
            entries,
            line_indices,
        }
    }

    fn sprites_on_line(&self, y: usize) -> &[u8] {
        &self.line_indices[y]
    }
}

fn scanline_sprite_pixel_rgb(
    vdp: &VdpSnapshot,
    scan: &SpriteLineScan,
    line_sprites: &[u8],
    screen_x: usize,
    screen_y: usize,
    priority: bool,
) -> Option<[u8; 3]> {
    let screen_x_i = screen_x as i32;
    let screen_y_i = screen_y as i32;
    for &entry_idx in line_sprites {
        let entry = &scan.entries[entry_idx as usize];
        if ((entry.pattern & 0x8000) != 0) == priority
            && screen_x_i >= entry.xpos
            && screen_x_i < entry.xpos + entry.width_pixels
            && screen_y_i >= entry.ypos
            && screen_y_i < entry.ypos + entry.height_pixels
        {
            let local_x = (screen_x_i - entry.xpos) as usize;
            let local_y = (screen_y_i - entry.ypos) as usize;
            if let Some(colour) = sprite_tile_pixel_rgb(
                vdp,
                entry.pattern,
                entry.width_tiles,
                entry.height_tiles,
                local_x,
                local_y,
            ) {
                return Some(colour);
            }
        }
    }
    None
}

fn sprite_tile_pixel_rgb(
    vdp: &VdpSnapshot,
    pattern: u16,
    width_tiles: usize,
    height_tiles: usize,
    local_x: usize,
    local_y: usize,
) -> Option<[u8; 3]> {
    let hflip = pattern & 0x0800 != 0;
    let vflip = pattern & 0x1000 != 0;
    let palette = usize::from((pattern >> 13) & 0x03);
    let base_tile = usize::from(pattern & 0x07FF);
    let local_x = if hflip {
        width_tiles * 8 - 1 - local_x
    } else {
        local_x
    };
    let local_y = if vflip {
        height_tiles * 8 - 1 - local_y
    } else {
        local_y
    };
    let tile_col = local_x / 8;
    let tile_row = local_y / 8;
    let tile_index = base_tile + tile_col * height_tiles + tile_row;
    let pixel_x = local_x & 7;
    let pixel_y = local_y & 7;
    tile_pixel_rgb_from_parts(vdp, tile_index, palette, pixel_x, pixel_y)
}

#[inline]
fn tile_pixel_rgb(vdp: &VdpSnapshot, entry: u16, tile_x: usize, tile_y: usize) -> Option<[u8; 3]> {
    let tile_index = usize::from(entry & 0x07FF);
    let hflip = entry & 0x0800 != 0;
    let vflip = entry & 0x1000 != 0;
    let palette = usize::from((entry >> 13) & 0x03);
    let mut tile_x = tile_x & 7;
    let mut tile_y = tile_y & 7;
    if hflip {
        tile_x = 7 - tile_x;
    }
    if vflip {
        tile_y = 7 - tile_y;
    }
    tile_pixel_rgb_from_parts(vdp, tile_index, palette, tile_x, tile_y)
}

#[inline]
fn tile_pixel_rgb_from_parts(
    vdp: &VdpSnapshot,
    tile_index: usize,
    palette: usize,
    tile_x: usize,
    tile_y: usize,
) -> Option<[u8; 3]> {
    let tile_byte = read_u8_wrapped(&vdp.vram, tile_index * 32 + tile_y * 4 + tile_x / 2);
    let colour_index = if tile_x & 1 == 0 {
        tile_byte >> 4
    } else {
        tile_byte & 0x0F
    };
    if colour_index == 0 {
        None
    } else {
        Some(cram_rgb(
            &vdp.cram,
            palette * 16 + usize::from(colour_index),
        ))
    }
}

fn window_plane_base(registers: &[u8; 0x20]) -> usize {
    let mask = if is_h40_mode(registers[0x0C]) {
        0x3E
    } else {
        0x3C
    };
    (usize::from(registers[3] & mask)) << 10
}

fn sprite_table_base(registers: &[u8; 0x20]) -> usize {
    let mask = if is_h40_mode(registers[0x0C]) {
        0x7E
    } else {
        0x7F
    };
    (usize::from(registers[5] & mask)) << 9
}

fn window_region(registers: &[u8; 0x20], width: usize, height: usize) -> Option<WindowRegion> {
    let horizontal_pos = usize::from(registers[0x11] & 0x1F) * 16;
    let vertical_pos = usize::from(registers[0x12] & 0x1F) * 8;
    let (left, window_width) = if registers[0x11] & 0x80 == 0 {
        (0, horizontal_pos.min(width))
    } else {
        let left = horizontal_pos.min(width);
        (left, width.saturating_sub(left))
    };
    let (top, window_height) = if registers[0x12] & 0x80 == 0 {
        (0, vertical_pos.min(height))
    } else {
        let top = vertical_pos.min(height);
        (top, height.saturating_sub(top))
    };
    if window_width == 0 || window_height == 0 {
        None
    } else {
        Some(WindowRegion {
            left,
            top,
            width: window_width,
            height: window_height,
        })
    }
}

fn is_h40_mode(mode_register: u8) -> bool {
    mode_register & 0x81 == 0x81
}

#[inline]
fn cram_rgb(cram: &[u8], palette_index: usize) -> [u8; 3] {
    let colour = read_u16_wrapped(cram, palette_index * 2);
    let red = ((colour >> 1) & 0x7) as u8;
    let green = ((colour >> 5) & 0x7) as u8;
    let blue = ((colour >> 9) & 0x7) as u8;
    [
        scale_channel(red),
        scale_channel(green),
        scale_channel(blue),
    ]
}

#[inline]
fn scale_channel(value: u8) -> u8 {
    ((u16::from(value) * 255 + 3) / 7) as u8
}

#[inline]
fn write_rgb(rgb: &mut [u8], width: usize, x: usize, y: usize, colour: [u8; 3]) {
    let index = (y * width + x) * 3;
    rgb[index..index + 3].copy_from_slice(&colour);
}

#[inline]
fn read_u8_wrapped(slice: &[u8], index: usize) -> u8 {
    if slice.is_empty() {
        0
    } else {
        slice[index % slice.len()]
    }
}

#[inline]
fn read_u16_wrapped(slice: &[u8], index: usize) -> u16 {
    u16::from_be_bytes([
        read_u8_wrapped(slice, index),
        read_u8_wrapped(slice, index + 1),
    ])
}

fn button_pressed(buttons: u8, button_bit: u8) -> bool {
    buttons & (1 << button_bit) != 0
}

fn ctrl_bit_set(value: u8, bit: u8) -> bool {
    value & (1 << bit) != 0
}

impl AddressBus for MdBus {
    fn copy_from(&mut self, other: &Self) {
        self.rom.clone_from(&other.rom);
        self.work_ram.clone_from(&other.work_ram);
        self.z80_shared.replace(other.z80_shared.borrow().clone());
        self.ym2612_shadow.borrow_mut().reset();
        {
            let writes = other.z80_shared.borrow().ym2612_writes.clone();
            let mut shadow = self.ym2612_shadow.borrow_mut();
            for (port, value) in writes {
                shadow.write_port((port & 0x0003) as u8, value);
            }
        }
        self.io = other.io.clone();
        self.vdp = other.vdp.clone();
        self.accesses.replace(other.accesses());
    }

    fn read_byte(&self, _address_space: AddressSpace, address: u32) -> u32 {
        let address = address & ADDRESS_MASK;
        let value = self.read_byte_internal(address);
        self.record_access(BusAccessKind::ReadByte, address, u32::from(value));
        u32::from(value)
    }

    fn read_word(&self, _address_space: AddressSpace, address: u32) -> u32 {
        let address = address & ADDRESS_MASK;
        let value = self.read_word_internal(address);
        self.record_access(BusAccessKind::ReadWord, address, u32::from(value));
        u32::from(value)
    }

    fn read_long(&self, _address_space: AddressSpace, address: u32) -> u32 {
        let address = address & ADDRESS_MASK;
        let msw = self.read_word_internal(address);
        let lsw = self.read_word_internal(address.wrapping_add(2));
        let value = (u32::from(msw) << 16) | u32::from(lsw);
        self.record_access(BusAccessKind::ReadWord, address, u32::from(msw));
        self.record_access(
            BusAccessKind::ReadWord,
            address.wrapping_add(2) & ADDRESS_MASK,
            u32::from(lsw),
        );
        value
    }

    fn write_byte(&mut self, _address_space: AddressSpace, address: u32, value: u32) {
        let address = address & ADDRESS_MASK;
        let value = value as u8;
        self.record_access(BusAccessKind::WriteByte, address, u32::from(value));
        self.write_byte_internal(address, value);
    }

    fn write_word(&mut self, _address_space: AddressSpace, address: u32, value: u32) {
        let address = address & ADDRESS_MASK;
        let value = value as u16;
        self.record_access(BusAccessKind::WriteWord, address, u32::from(value));
        self.write_word_internal(address, value);
    }

    fn write_long(&mut self, _address_space: AddressSpace, address: u32, value: u32) {
        let address = address & ADDRESS_MASK;
        self.record_access(BusAccessKind::WriteWord, address, value >> 16);
        self.record_access(
            BusAccessKind::WriteWord,
            address.wrapping_add(2) & ADDRESS_MASK,
            value & 0xFFFF,
        );
        self.write_word_internal(address, (value >> 16) as u16);
        self.write_word_internal(address.wrapping_add(2), value as u16);
    }
}

// --- Standalone GEMS Audio Engine ---

/// ROM addresses where Kid Chameleon's GEMS sound data lives.
const GEMS_PATCH_ADDR: u32 = 0x0E_160E;
const GEMS_ENVELOPE_ADDR: u32 = 0x0E_25EA;
const GEMS_SEQUENCE_ADDR: u32 = 0x0E_2766;
const GEMS_SAMPLE_ADDR: u32 = 0x0E_8716;
/// GEMS command queue: 64-byte circular buffer in Z80 RAM.
const GEMS_QUEUE_ADDR: u16 = 0x1A40;
/// Queue write pointer location in Z80 RAM.
const GEMS_QUEUE_PTR_ADDR: u16 = 0x0036;
const GEMS_QUEUE_SIZE: u8 = 64;

/// Z80 bus for the standalone GEMS audio engine.
/// Provides: Z80 RAM (driver code), virtual ROM via bank switching, YM2612 + PSG I/O.
struct GemsBus {
    ram: Vec<u8>,
    /// Virtual ROM built from extracted sound data files.
    virtual_rom: Vec<u8>,
    bank_address: u32,
    ym2612_shadow: RefCell<Ym2612Shadow>,
    z80_cycles: u64,
    audio_events: VecDeque<AudioEvent>,
}

impl GemsBus {
    fn new(driver: &[u8], virtual_rom: Vec<u8>) -> Self {
        let mut ram = vec![0u8; Z80_RAM_LEN];
        // Load GEMS driver into Z80 RAM at address 0
        let copy_len = driver.len().min(Z80_RAM_LEN);
        ram[..copy_len].copy_from_slice(&driver[..copy_len]);
        Self {
            ram,
            virtual_rom,
            bank_address: 0,
            ym2612_shadow: RefCell::new(Ym2612Shadow::new()),
            z80_cycles: 0,
            audio_events: VecDeque::new(),
        }
    }

    fn reload_driver(&mut self, driver: &[u8]) {
        self.ram.fill(0);
        let copy_len = driver.len().min(Z80_RAM_LEN);
        self.ram[..copy_len].copy_from_slice(&driver[..copy_len]);
        self.bank_address = 0;
        self.z80_cycles = 0;
        self.audio_events.clear();
        self.ym2612_shadow.borrow_mut().reset();
    }

    fn record_ym2612_write(&mut self, port: u8, value: u8) {
        self.ym2612_shadow.borrow_mut().write_port(port, value);
        self.audio_events.push_back(AudioEvent::Ym2612Write {
            z80_cycles: self.z80_cycles,
            port,
            value,
        });
    }

    fn record_psg_write(&mut self, value: u8) {
        self.audio_events.push_back(AudioEvent::PsgWrite {
            z80_cycles: self.z80_cycles,
            value,
        });
    }
}

impl Z80_io for GemsBus {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.ram[usize::from(address) & (Z80_RAM_LEN - 1)],
            0x4000..=0x5FFF => self
                .ym2612_shadow
                .borrow_mut()
                .read_port((address & 0x03) as u8),
            0x8000..=0xFFFF => {
                let banked = self
                    .bank_address
                    .wrapping_add(u32::from(address.wrapping_sub(0x8000)));
                self.virtual_rom
                    .get(banked as usize)
                    .copied()
                    .unwrap_or(0xFF)
            }
            _ => 0xFF,
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.ram[usize::from(address) & (Z80_RAM_LEN - 1)] = value;
            }
            0x4000..=0x5FFF => self.record_ym2612_write((address & 0x03) as u8, value),
            0x6000..=0x60FF => {
                // Bank register write
                let current_bank = self.bank_address >> 15;
                let next_bank = ((current_bank >> 1) | (u32::from(value & 1) << 8)) & 0x01FF;
                self.bank_address = next_bank << 15;
            }
            0x7F11 => self.record_psg_write(value),
            _ => {}
        }
    }

    fn port_in(&self, _address: u16) -> u8 {
        0xFF
    }

    fn port_out(&mut self, _address: u16, _value: u8) {}
}

/// Standalone GEMS audio engine — runs the real Z80 GEMS sound driver
/// with YM2612 FM synthesis and PSG, independent of the full M68K emulator.
///
/// Usage:
/// 1. `GemsAudioEngine::new(driver, patches, envelopes, sequences, samples)`
/// 2. `engine.play_sound(sfx_id)` or `engine.play_music(bgm_id)`
/// 3. `engine.step_frame()` each game frame (advances Z80 by one VBlank period)
/// 4. `engine.generate_samples(735)` to pull 735 stereo samples (1 frame @ 44.1kHz)
pub struct GemsAudioEngine {
    z80: Z80<GemsBus>,
    mixer: AudioMixer,
    vblank_irq_cycles_remaining: u32,
    initialised: bool,
    driver: Vec<u8>,
}

/// Z80 cycles per VBlank period (NTSC: ~59,659 Z80 cycles per frame).
/// 3,579,545 Hz / 59.92 Hz ≈ 59,742 cycles.
const GEMS_Z80_CYCLES_PER_FRAME: u32 = 59_659;

impl GemsAudioEngine {
    /// Create a new GEMS audio engine from extracted sound data.
    ///
    /// - `driver`: GEMS_Sounddriver.bin (loaded into Z80 RAM at address 0)
    /// - `patches`: instruments.bin (patch/instrument definitions)
    /// - `envelopes`: envelopes.bin
    /// - `sequences`: sequences.bin (song/SFX sequence data)
    /// - `samples`: samples.bin (PCM sample data)
    pub fn new(
        driver: &[u8],
        patches: &[u8],
        envelopes: &[u8],
        sequences: &[u8],
        samples: &[u8],
    ) -> Self {
        // Build virtual ROM: place data at original ROM addresses
        let max_addr = GEMS_SAMPLE_ADDR as usize + samples.len();
        let mut virtual_rom = vec![0u8; max_addr];
        Self::place_data(&mut virtual_rom, GEMS_PATCH_ADDR, patches);
        Self::place_data(&mut virtual_rom, GEMS_ENVELOPE_ADDR, envelopes);
        Self::place_data(&mut virtual_rom, GEMS_SEQUENCE_ADDR, sequences);
        Self::place_data(&mut virtual_rom, GEMS_SAMPLE_ADDR, samples);

        let bus = GemsBus::new(driver, virtual_rom);
        let mut z80 = Z80::new(bus);
        z80.reset();

        Self {
            z80,
            mixer: AudioMixer::default(),
            vblank_irq_cycles_remaining: 0,
            initialised: false,
            driver: driver.to_vec(),
        }
    }

    fn place_data(rom: &mut [u8], base_addr: u32, data: &[u8]) {
        let start = base_addr as usize;
        let end = start + data.len();
        if end <= rom.len() {
            rom[start..end].copy_from_slice(data);
        }
    }

    /// Initialise GEMS: send command 0x0B with data pointers.
    /// Must be called once before playing any sounds.
    pub fn init(&mut self) {
        if self.initialised {
            return;
        }
        // Run a few frames to let the driver boot
        for _ in 0..4 {
            self.step_z80_frame();
        }
        // Queue init command: FF 0B <patch_ptr> <env_ptr> <seq_ptr> <sample_ptr>
        self.queue_byte(0xFF);
        self.queue_byte(0x0B);
        self.queue_24bit_addr(GEMS_PATCH_ADDR);
        self.queue_24bit_addr(GEMS_ENVELOPE_ADDR);
        self.queue_24bit_addr(GEMS_SEQUENCE_ADDR);
        self.queue_24bit_addr(GEMS_SAMPLE_ADDR);
        // Run a few more frames for GEMS to process the init
        for _ in 0..4 {
            self.step_z80_frame();
        }
        self.initialised = true;
    }

    /// Match ASM `StopMusic` (kid.asm:49727): reload/reset the GEMS driver,
    /// then queue the `$0B` sound-data pointer command. This is the ROM path
    /// used before changing music contexts; `$16` stop-all is not what Kid's
    /// 68k wrapper emits.
    pub fn stop_music(&mut self) {
        let levels = self.mixer.levels;
        self.z80.io.reload_driver(&self.driver);
        self.z80.reset();
        self.mixer = AudioMixer::default();
        self.mixer.set_levels(levels);
        self.vblank_irq_cycles_remaining = 0;
        self.initialised = false;
        self.init();
    }

    /// Queue a single byte to the GEMS command circular buffer.
    fn queue_byte(&mut self, value: u8) {
        let ptr = self.z80.io.ram[usize::from(GEMS_QUEUE_PTR_ADDR)];
        self.z80.io.ram[usize::from(GEMS_QUEUE_ADDR) + usize::from(ptr)] = value;
        let next = (ptr.wrapping_add(1)) & (GEMS_QUEUE_SIZE - 1);
        self.z80.io.ram[usize::from(GEMS_QUEUE_PTR_ADDR)] = next;
    }

    /// Queue a 24-bit ROM address in the low-to-high order used by ASM sub_E14AC.
    fn queue_24bit_addr(&mut self, addr: u32) {
        self.queue_byte((addr & 0xFF) as u8);
        self.queue_byte(((addr >> 8) & 0xFF) as u8);
        self.queue_byte(((addr >> 16) & 0xFF) as u8);
    }

    /// Play a sound effect by ID (GEMS command $10).
    pub fn play_sound(&mut self, sound_id: u8) {
        if !self.initialised {
            self.init();
        }
        self.queue_byte(0xFF);
        self.queue_byte(0x10);
        self.queue_byte(sound_id);
    }

    /// Play a sound via command $12 (alternate PlaySound).
    pub fn play_sound2(&mut self, sound_id: u8) {
        if !self.initialised {
            self.init();
        }
        self.queue_byte(0xFF);
        self.queue_byte(0x12);
        self.queue_byte(sound_id);
    }

    /// Stop all currently running sequences (GEMS command $16 with sequence $FF).
    pub fn stop_all_sequences(&mut self) {
        if !self.initialised {
            self.init();
        }
        self.queue_byte(0xFF);
        self.queue_byte(0x16);
        self.queue_byte(0xFF);
    }

    /// Advance Z80 by one frame's worth of cycles.
    /// Call once per game frame (60 Hz).
    pub fn step_frame(&mut self) {
        self.step_z80_frame();
        // Drain audio events from bus into mixer
        let events = std::mem::take(&mut self.z80.io.audio_events);
        self.mixer.push_events(events);
    }

    fn step_z80_frame(&mut self) {
        let mut remaining = GEMS_Z80_CYCLES_PER_FRAME;
        // Fire VBlank IRQ at start of frame
        self.vblank_irq_cycles_remaining = Z80_VBLANK_IRQ_CYCLES;

        while remaining > 0 {
            self.z80.io.z80_cycles = self.z80.io.z80_cycles.wrapping_add(0);
            if self.vblank_irq_cycles_remaining > 0 {
                self.z80.assert_irq(0xFF);
            } else {
                self.z80.clr_irq();
            }
            let elapsed = self.z80.step();
            let budget = elapsed.min(remaining);
            self.z80
                .io
                .ym2612_shadow
                .borrow_mut()
                .advance_clocks(budget);
            self.z80.io.z80_cycles = self.z80.io.z80_cycles.wrapping_add(u64::from(budget));
            self.vblank_irq_cycles_remaining =
                self.vblank_irq_cycles_remaining.saturating_sub(budget);
            if budget >= remaining {
                break;
            }
            remaining -= budget;
        }
    }

    /// Generate audio samples. Returns interleaved stereo f32 (L, R, L, R, ...).
    /// For 60fps at 44.1kHz, request 735 samples per frame.
    pub fn generate_samples(&mut self, sample_count: usize) -> Vec<f32> {
        self.mixer.generate(44_100, sample_count)
    }

    /// Set audio volume levels.
    pub fn set_levels(&mut self, levels: AudioLevels) {
        self.mixer.set_levels(levels);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AudioEvent, AudioLevels, AudioMixer, BusAccessKind, FirstOrderFilter, GEMS_QUEUE_ADDR,
        GemsBus, MdBus, MdRuntime, PsgRegisters, PsgSynth, SecondOrderFilter, WORK_RAM_BASE,
        YM2612_STATUS_DECAY_CLOCKS, Ym2612Shadow, Z80_RAM_BASE, Z80Bus, Z80Shared, cram_rgb,
        probe_rom, scale_channel,
    };
    use r68k_emu::ram::{AddressBus, SUPERVISOR_DATA};
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::rc::Rc;
    use z80::Z80_io;

    fn nop_rom() -> Vec<u8> {
        let mut rom = vec![0x4E; 0x20_0000];
        rom[1] = 0x71;
        rom[0..4].copy_from_slice(&0x0000_1000u32.to_be_bytes());
        rom[4..8].copy_from_slice(&0x0000_0008u32.to_be_bytes());
        for chunk in rom[8..].chunks_exact_mut(2) {
            chunk.copy_from_slice(&[0x4E, 0x71]);
        }
        rom
    }

    fn rom_with_code_at_reset(code: &[u8]) -> Vec<u8> {
        let mut rom = nop_rom();
        rom[8..8 + code.len()].copy_from_slice(code);
        rom
    }

    fn assert_close(actual: f32, expected: f32, epsilon: f32) {
        assert!(
            (actual - expected).abs() <= epsilon,
            "expected {actual} to be within {epsilon} of {expected}",
        );
    }

    #[test]
    fn probes_reset_vector_and_steps_nops() {
        let rom = [
            0x00, 0x00, 0x10, 0x00, // initial SP
            0x00, 0x00, 0x00, 0x08, // initial PC
            0x4E, 0x71, // NOP
            0x4E, 0x71, // NOP
        ];

        let probe = probe_rom(&rom, 2).expect("probe should succeed");
        assert_eq!(probe.reset_sp, 0x0000_1000);
        assert_eq!(probe.reset_pc, 0x0000_0008);
        assert_eq!(probe.final_pc, 0x0000_000C);
        assert_eq!(probe.steps.len(), 2);
        assert!(!probe.accesses.is_empty());
    }

    #[test]
    fn probe_rom_reports_nonzero_just_received_damage_as_true() {
        // Probe ROM: `move.b #1,($FFFFFA73).l; nop`. $FFFFFA73 is the
        // `Just_received_damage` byte exported by CpuProbe.
        let rom =
            rom_with_code_at_reset(&[0x13, 0xFC, 0x00, 0x01, 0x00, 0xFF, 0xFA, 0x73, 0x4E, 0x71]);

        let probe = probe_rom(&rom, 2).expect("probe should execute byte write");

        assert!(probe.just_received_damage);
    }

    #[test]
    fn probe_rom_controller_script_advances_once_per_rendered_frame() {
        // Probe ROM:
        //   move.b #$7E,($00A10009).l        ; controller ctrl masks every data bit except bit 0
        // loop:
        //   move.b ($00A10003).l,($FFFFFA73).l
        //   bra.s loop
        //
        // With active-low controller input, script frame 0 (no buttons) writes 1
        // and script frame 1 (button bit 0 held) writes 0. The run length is
        // chosen to cross exactly one synthetic rendered frame, so advancing the
        // script too early or more than once leaves the final byte non-zero.
        let rom = rom_with_code_at_reset(&[
            0x13, 0xFC, 0x00, 0x7E, 0x00, 0xA1, 0x00, 0x09, 0x13, 0xF9, 0x00, 0xA1, 0x00, 0x03,
            0x00, 0xFF, 0xFA, 0x73, 0x60, 0xF4,
        ]);

        let probe = super::probe_rom_with_script(&rom, 12_500, &[0x00, 0x01])
            .expect("probe should run through one rendered frame");

        assert_eq!(probe.synthetic_vblanks, 1);
        assert!(
            !probe.just_received_damage,
            "script frame 1 should hold button bit 0 after exactly one rendered frame"
        );
    }

    #[test]
    fn frame_render_boundary_is_not_gated_on_vint_enable() {
        // ASM `Title_InputLoop` writes `Game_Mode=$0008`, and `loc_1AE2E`
        // does not advance to `$000C` until later visible level-entry frames.
        // Capture must therefore keep counting rendered frames even when the
        // ROM temporarily masks VInt during that mode-8 window.
        let mut runtime = MdRuntime::new(&nop_rom()).expect("runtime should boot a NOP ROM");
        runtime.core.mem.vdp.registers[1] &= !0x20;

        assert!(
            runtime.run_until_frame_rendered(200_000),
            "frame counter should advance even while VInt is masked"
        );
    }

    #[test]
    fn run_until_vblank_reports_timeout_and_counter_change() {
        // Probe ROM: `move.l #1,($FFFFF806).l; nop`. $FFFFF806 is the
        // VInt counter observed by MdRuntime::run_until_vblank.
        let rom = rom_with_code_at_reset(&[
            0x23, 0xFC, 0x00, 0x00, 0x00, 0x01, 0x00, 0xFF, 0xF8, 0x06, 0x4E, 0x71,
        ]);
        let mut runtime = MdRuntime::new(&rom).expect("runtime should boot counter probe ROM");

        assert!(!runtime.run_until_vblank(0));
        assert!(runtime.run_until_vblank(1));
    }

    #[test]
    fn run_until_frame_rendered_reports_timeout_and_synthetic_frame() {
        let mut runtime = MdRuntime::new(&nop_rom()).expect("runtime should boot a NOP ROM");

        assert!(!runtime.run_until_frame_rendered(0));
        assert!(!runtime.run_until_frame_rendered(1));
        assert!(runtime.run_until_frame_rendered(200_000));
    }

    #[test]
    fn cpu_probe_reports_nonzero_just_received_damage_as_true() {
        let mut runtime = MdRuntime::new(&nop_rom()).expect("runtime should boot a NOP ROM");
        runtime
            .core
            .mem
            .write_work_ram_byte(super::JUST_RECEIVED_DAMAGE_ADDR, 1);

        assert!(runtime.snapshot().just_received_damage);
    }

    #[test]
    fn render_into_uses_runtime_vdp_background_colour() {
        let mut runtime = MdRuntime::new(&nop_rom()).expect("runtime should boot a NOP ROM");
        runtime.core.mem.vdp.cram[0..2].copy_from_slice(&[0x0E, 0xEE]);
        let mut rgb = vec![0; 320 * 224 * 3];

        runtime.render_into(&mut rgb);

        assert_eq!(&rgb[0..3], &[0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn runtime_audio_generation_uses_levels_and_requested_sample_count() {
        let mut runtime = MdRuntime::new(&nop_rom()).expect("runtime should boot a NOP ROM");
        runtime.set_audio_levels(AudioLevels {
            master: 0.0,
            ym2612: 1.0,
            psg: 1.0,
        });

        let samples = runtime.generate_audio_samples(44_100, 64);

        assert_eq!(samples.len(), 128);
        assert!(samples.iter().all(|sample| sample.abs() <= f32::EPSILON));
    }

    #[test]
    fn work_ram_is_mirrored_across_upper_address_space() {
        let mut bus = MdBus::with_fresh_z80(&[]);
        bus.write_word(SUPERVISOR_DATA, WORK_RAM_BASE, 0x1234);
        assert_eq!(bus.read_word(SUPERVISOR_DATA, WORK_RAM_BASE), 0x1234);
        assert_eq!(bus.read_word(SUPERVISOR_DATA, 0x00FF_0000), 0x1234);
    }

    #[test]
    fn controller_and_vdp_accesses_are_traced() {
        let mut bus = MdBus::with_fresh_z80(&[]);
        bus.write_word(SUPERVISOR_DATA, 0x00A1_0003, 0x0040);
        bus.write_word(SUPERVISOR_DATA, 0x00C0_0004, 0x8134);
        let accesses = bus.accesses();
        assert!(accesses.iter().any(|access| {
            access.kind == BusAccessKind::WriteWord && access.address == 0x00A1_0003
        }));
        assert!(accesses.iter().any(|access| {
            access.kind == BusAccessKind::WriteWord && access.address == 0x00C0_0004
        }));
        assert_eq!(bus.vdp_control_writes(), vec![0x8134]);
    }

    #[test]
    fn z80_ram_is_mirrored_when_busack_is_asserted() {
        let mut bus = MdBus::with_fresh_z80(&[]);
        bus.write_word(SUPERVISOR_DATA, 0x00A1_1200, 0x0100);
        bus.write_word(SUPERVISOR_DATA, 0x00A1_1100, 0x0100);
        bus.write_byte(SUPERVISOR_DATA, Z80_RAM_BASE, 0x12);
        assert_eq!(bus.read_byte(SUPERVISOR_DATA, Z80_RAM_BASE), 0x12);
        assert_eq!(bus.read_byte(SUPERVISOR_DATA, Z80_RAM_BASE + 0x2000), 0x12);
        assert_eq!(bus.read_word(SUPERVISOR_DATA, Z80_RAM_BASE), 0x1212);
    }

    #[test]
    fn z80_ram_is_hidden_from_68k_without_busack() {
        let mut bus = MdBus::with_fresh_z80(&[]);
        bus.z80_shared.borrow_mut().ram[0] = 0x34;
        assert_eq!(bus.read_byte(SUPERVISOR_DATA, Z80_RAM_BASE), 0xFF);
        assert_eq!(bus.read_word(SUPERVISOR_DATA, Z80_RAM_BASE), 0xFF00);
        bus.write_byte(SUPERVISOR_DATA, Z80_RAM_BASE, 0x12);
        assert_eq!(bus.z80_shared.borrow().ram[0], 0x34);
    }

    #[test]
    fn z80_reset_starts_asserted() {
        let bus = MdBus::with_fresh_z80(&[]);
        assert_eq!(bus.read_word(SUPERVISOR_DATA, 0x00A1_1200), 0x0000);
    }

    #[test]
    fn z80_reset_does_not_reset_sound_chip_state() {
        let mut bus = MdBus::with_fresh_z80(&[]);
        bus.write_word(SUPERVISOR_DATA, 0x00A1_1200, 0x0100);

        {
            let mut shadow = bus.ym2612_shadow.borrow_mut();
            shadow.write_port(0, 0x30);
            shadow.write_port(1, 0xFF);
        }
        {
            let mut shared = bus.z80_shared.borrow_mut();
            let _ = shared.psg_registers.apply_write(0x80);
            let _ = shared.psg_registers.apply_write(0x04);
            let _ = shared.psg_registers.apply_write(0x90);
        }

        bus.write_word(SUPERVISOR_DATA, 0x00A1_1200, 0x0000);

        let ym_status = bus.ym2612_shadow.borrow_mut().read_port(0);
        assert_ne!(
            ym_status & 0x80,
            0,
            "Z80 reset should not clear YM2612 state or its busy flag",
        );
        assert_ne!(
            bus.z80_shared.borrow().psg_registers.tone_periods[0],
            0,
            "Z80 reset should not zero PSG registers",
        );
        assert!(
            bus.z80_shared.borrow().audio_events.iter().next().is_none(),
            "Z80 reset should not leave any synthetic sound-chip reset event queued",
        );
    }

    #[test]
    fn z80_bank_register_shifts_in_bits_msb_first() {
        let shared = Rc::new(RefCell::new(Z80Shared::default()));
        let mut bus = Z80Bus {
            shared: shared.clone(),
            ym2612_shadow: Rc::new(RefCell::new(Ym2612Shadow::new())),
            rom: vec![0; 0x20_0000],
        };

        for bit in [1, 0, 1, 0, 0, 0, 0, 0, 0] {
            bus.write_byte(0x6000, bit);
        }

        assert_eq!(shared.borrow().bank_address, 0x0002_8000);
    }

    #[test]
    fn z80_banked_reads_can_see_68k_work_ram() {
        let mut bus = MdBus::with_fresh_z80(&[]);
        let z80_shared = bus.z80_shared.clone();
        bus.write_byte(SUPERVISOR_DATA, WORK_RAM_BASE, 0x5A);

        z80_shared.borrow_mut().bank_address = WORK_RAM_BASE;
        let z80_bus = Z80Bus {
            shared: z80_shared,
            ym2612_shadow: Rc::new(RefCell::new(Ym2612Shadow::new())),
            rom: vec![0; 0x20_0000],
        };

        assert_eq!(z80_bus.read_byte(0x8000), 0x5A);
    }

    #[test]
    fn z80_io_ports_are_not_wired_to_the_psg() {
        let shared = Rc::new(RefCell::new(Z80Shared::default()));
        let mut bus = Z80Bus {
            shared: shared.clone(),
            ym2612_shadow: Rc::new(RefCell::new(Ym2612Shadow::new())),
            rom: vec![0; 0x20_0000],
        };

        bus.port_out(0x007F, 0x90);

        assert!(shared.borrow().psg_writes.is_empty());
    }

    #[test]
    fn psg_reset_stays_effectively_silent() {
        let mut mixer = AudioMixer::default();
        let samples = mixer.generate(44_100, 2_048);
        let peak = samples
            .iter()
            .copied()
            .fold(0.0f32, |peak: f32, sample: f32| peak.max(sample.abs()));
        assert!(
            peak < 0.02,
            "expected silent PSG reset output, got peak {peak}"
        );
    }

    #[test]
    fn first_order_low_pass_uses_bilinear_transform_coefficients() {
        // The Genesis audio model depends on this low-pass keeping the one-pole
        // bilinear transform stable; arithmetic slips here survive through the
        // mixer as broad tone-colour drift rather than obvious silence.
        let mut filter = FirstOrderFilter::low_pass(3_390.0, 44_100.0);

        assert_close(filter.b0, 0.197_627_22, 0.000_000_02);
        assert_close(filter.b1, 0.197_627_22, 0.000_000_02);
        assert_close(filter.a1, -0.604_745_57, 0.000_000_06);
        assert_close(filter.filter(1.0), 0.197_627_22, 0.000_000_02);
        assert_close(filter.filter(1.0), 0.514_768_6, 0.000_000_1);
    }

    #[test]
    fn first_order_high_pass_preserves_dc_blocker_coefficients() {
        // Keep the DC blocker as a high-pass filter: both feed-forward
        // coefficients must oppose each other, otherwise sustained YM output
        // leaks or amplifies as a bias.
        let mut filter = FirstOrderFilter::high_pass(5.0, 44_100.0);

        assert_close(filter.b0, 0.999_643_9, 0.000_000_06);
        assert_close(filter.b1, -0.999_643_9, 0.000_000_06);
        assert_close(filter.a1, -0.999_287_9, 0.000_000_1);
        assert_close(filter.filter(1.0), 0.999_643_9, 0.000_000_06);
        assert_close(filter.filter(1.0), 0.998_932_06, 0.000_000_1);
    }

    #[test]
    fn second_order_low_pass_uses_biquad_bilinear_coefficients() {
        // The YM path uses this extra low-pass to approximate the console
        // output chain. Coefficients are pinned here because tiny arithmetic
        // mistakes only show up as subtle high-frequency colour drift.
        let mut filter = SecondOrderFilter::low_pass(7_973.0, 44_100.0);

        assert_close(filter.b0, 0.176_304_47, 0.000_000_02);
        assert_close(filter.b1, 0.352_608_95, 0.000_000_03);
        assert_close(filter.b2, 0.176_304_47, 0.000_000_02);
        assert_close(filter.a1, -0.513_326_35, 0.000_000_06);
        assert_close(filter.a2, 0.218_544_25, 0.000_000_03);
        assert_close(filter.filter(1.0), 0.176_304_47, 0.000_000_02);
        assert_close(filter.filter(1.0), 0.619_415_16, 0.000_000_1);
        assert_close(filter.filter(1.0), 0.984_649_7, 0.000_000_1);
    }

    #[test]
    fn audio_events_report_original_z80_cycle() {
        assert_eq!(
            AudioEvent::Ym2612Write {
                z80_cycles: 123,
                port: 0,
                value: 0,
            }
            .z80_cycles(),
            123
        );
        assert_eq!(
            AudioEvent::PsgWrite {
                z80_cycles: 456,
                value: 0,
            }
            .z80_cycles(),
            456
        );
    }

    #[test]
    fn psg_register_writes_decode_latched_tone_volume_and_noise_data() {
        let mut registers = PsgRegisters::default();

        assert!(!registers.apply_write(0xA3));
        assert_eq!(registers.latched_register, 2);
        assert_eq!(registers.tone_periods[1], 0x003);
        assert!(!registers.apply_write(0x15));
        assert_eq!(registers.tone_periods[1], 0x153);

        assert!(!registers.apply_write(0xC7));
        assert_eq!(registers.latched_register, 4);
        assert_eq!(registers.tone_periods[2], 0x007);
        assert!(!registers.apply_write(0x3F));
        assert_eq!(registers.tone_periods[2], 0x3F7);
        assert!(!registers.apply_write(0xC2));
        assert_eq!(registers.tone_periods[2], 0x3F2);

        assert!(!registers.apply_write(0x94));
        assert_eq!(registers.latched_register, 1);
        assert_eq!(registers.volumes[0], 0x04);
        assert!(!registers.apply_write(0xF2));
        assert_eq!(registers.latched_register, 7);
        assert_eq!(registers.volumes[3], 0x02);

        assert!(registers.apply_write(0xE6));
        assert_eq!(registers.latched_register, 6);
        assert_eq!(registers.noise_control, 0x06);
        assert!(registers.apply_write(0x09));
        assert_eq!(registers.noise_control, 0x01);
    }

    #[test]
    fn psg_noise_reset_restores_initial_lfsr_and_output() {
        let mut synth = PsgSynth {
            noise_lfsr: 0x0000,
            noise_output_high: true,
            ..PsgSynth::default()
        };

        synth.reset_noise_lfsr();

        assert_eq!(synth.noise_lfsr, super::PSG_INITIAL_LFSR);
        assert!(!synth.noise_output_high);
    }

    #[test]
    fn ym2612_shadow_reset_clears_status_latch_and_busy_state() {
        let mut shadow = Ym2612Shadow::new();
        shadow.write_port(0, 0x30);
        shadow.write_port(1, 0xFF);
        let status = shadow.read_port(0);
        assert_ne!(status & 0x80, 0, "expected busy flag before reset");

        shadow.reset();

        assert_eq!(shadow.status_decay_clocks_remaining, 0);
        assert_eq!(shadow.last_status_read, 0);
        assert_eq!(shadow.read_port(1), 0);
    }

    #[test]
    fn md_runtime_setters_write_observable_work_ram_and_controller_state() {
        let mut runtime = MdRuntime::new(&nop_rom()).expect("runtime should boot a NOP ROM");

        runtime.set_controller_1(0b0010_0001);
        runtime.set_level_skip_cheat_enabled(true);
        runtime.set_current_level(0x1234);
        runtime.set_game_mode(0x0042);
        runtime.set_current_helmet(0x0007);
        runtime.set_lives(9);
        runtime.set_hitpoints(6);

        assert_eq!(runtime.core.mem.io.p1_port.buttons, 0b0010_0001);
        assert_eq!(
            runtime
                .core
                .mem
                .read_work_ram_byte(super::LEVEL_SKIP_CHEAT_ADDR),
            u8::MAX
        );
        assert_eq!(
            runtime.read_work_ram_word(super::CURRENT_LEVEL_ID_ADDR),
            0x1234
        );
        assert_eq!(
            runtime.read_work_ram_word(super::PLAYER_1_LEVEL_ID_ADDR),
            0x1234
        );
        assert_eq!(runtime.read_work_ram_word(super::GAME_MODE_ADDR), 0x0042);
        assert_eq!(
            runtime.read_work_ram_word(super::CURRENT_HELMET_ID_ADDR),
            0x0007
        );
        assert_eq!(runtime.read_work_ram_word(super::NUMBER_LIVES_ADDR), 9);
        assert_eq!(runtime.read_work_ram_word(super::NUMBER_HITPOINTS_ADDR), 6);
        assert_eq!(
            runtime.read_work_ram_range(super::NUMBER_LIVES_ADDR, 4),
            vec![0x00, 0x09, 0x00, 0x06]
        );

        runtime.set_level_skip_cheat_enabled(false);
        assert_eq!(
            runtime
                .core
                .mem
                .read_work_ram_byte(super::LEVEL_SKIP_CHEAT_ADDR),
            0
        );
    }

    #[test]
    fn scale_channel_matches_megadrive_rgb_rounding() {
        let expanded: Vec<u8> = (0..=7).map(scale_channel).collect();

        assert_eq!(
            expanded,
            vec![0x00, 0x24, 0x49, 0x6D, 0x92, 0xB6, 0xDB, 0xFF]
        );
    }

    #[test]
    fn cram_rgb_decodes_3bit_channels_from_cram_words() {
        let cram = [0x0E, 0xEE];

        assert_eq!(cram_rgb(&cram, 0), [0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn psg_tone_writes_produce_varying_output() {
        let mut mixer = AudioMixer::default();
        let mut events = VecDeque::new();
        events.push_back(AudioEvent::PsgWrite {
            z80_cycles: 0,
            value: 0x80,
        });
        events.push_back(AudioEvent::PsgWrite {
            z80_cycles: 0,
            value: 0x04,
        });
        events.push_back(AudioEvent::PsgWrite {
            z80_cycles: 0,
            value: 0x90,
        });
        events.push_back(AudioEvent::PsgWrite {
            z80_cycles: 0,
            value: 0x00,
        });
        mixer.push_events(events);

        let samples = mixer.generate(44_100, 4_096);
        let mut saw_change = false;
        for pair in samples.windows(2) {
            if (pair[0] - pair[1]).abs() > 1e-4 {
                saw_change = true;
                break;
            }
        }
        assert!(
            saw_change,
            "expected PSG tone writes to vary generated audio"
        );
    }

    #[test]
    fn audio_levels_can_mute_generated_output() {
        let mut mixer = AudioMixer::default();
        let mut events = VecDeque::new();
        events.push_back(AudioEvent::PsgWrite {
            z80_cycles: 0,
            value: 0x80,
        });
        events.push_back(AudioEvent::PsgWrite {
            z80_cycles: 0,
            value: 0x04,
        });
        events.push_back(AudioEvent::PsgWrite {
            z80_cycles: 0,
            value: 0x90,
        });
        events.push_back(AudioEvent::PsgWrite {
            z80_cycles: 0,
            value: 0x00,
        });
        mixer.push_events(events);
        mixer.set_levels(AudioLevels {
            master: 0.0,
            ym2612: 1.0,
            psg: 1.0,
        });

        let samples = mixer.generate(44_100, 512);
        assert!(samples.iter().all(|sample| sample.abs() <= f32::EPSILON));
    }

    #[test]
    fn ym2612_shadow_mirrors_status_reads_across_all_ports() {
        let mut shadow = Ym2612Shadow::new();
        shadow.write_port(0, 0x30);
        shadow.write_port(1, 0xFF);

        let status = shadow.read_port(0);
        assert_ne!(status & 0x80, 0, "expected busy flag after YM write");
        assert_eq!(shadow.read_port(1), status);
        assert_eq!(shadow.read_port(2), status);
        assert_eq!(shadow.read_port(3), status);

        shadow.advance_clocks(YM2612_STATUS_DECAY_CLOCKS);
        assert_eq!(shadow.read_port(1), 0);
        assert_eq!(shadow.read_port(2), 0);
        assert_eq!(shadow.read_port(3), 0);
    }

    #[test]
    fn standalone_gems_bus_mirrors_ym2612_status_reads() {
        let mut bus = GemsBus::new(&[], vec![0; 0x20_0000]);
        bus.record_ym2612_write(0, 0x30);
        bus.record_ym2612_write(1, 0xFF);

        let status = bus.read_byte(0x4000);
        assert_ne!(
            status & 0x80,
            0,
            "standalone GEMS must expose the YM busy flag to the Z80 driver, matching MdRuntime's Z80Bus"
        );
        assert_eq!(bus.read_byte(0x4001), status);

        bus.ym2612_shadow
            .borrow_mut()
            .advance_clocks(YM2612_STATUS_DECAY_CLOCKS);
        assert_eq!(bus.read_byte(0x4001), 0);
    }

    #[test]
    fn standalone_gems_frame_step_advances_ym2612_status_decay() {
        let driver = vec![0u8; 64];
        let patches = vec![0u8; 16];
        let envelopes = vec![0u8; 16];
        let sequences = vec![0u8; 16];
        let samples = vec![0u8; 16];

        let mut engine =
            super::GemsAudioEngine::new(&driver, &patches, &envelopes, &sequences, &samples);
        {
            let mut shadow = engine.z80.io.ym2612_shadow.borrow_mut();
            shadow.write_port(0, 0x30);
            shadow.write_port(1, 0xFF);
            let _ = shadow.read_port(0);
            assert_eq!(
                shadow.status_decay_clocks_remaining,
                YM2612_STATUS_DECAY_CLOCKS
            );
        }

        engine.step_z80_frame();

        assert!(
            engine
                .z80
                .io
                .ym2612_shadow
                .borrow()
                .status_decay_clocks_remaining
                < YM2612_STATUS_DECAY_CLOCKS,
            "standalone GEMS frame stepping must age YM status reads like MdRuntime::step"
        );
    }

    #[test]
    fn hscroll_modes_follow_vdp_layout() {
        let mut vram = vec![0; 0x400];
        vram[0x100..0x110].copy_from_slice(&[
            0x00, 0x08, 0x00, 0x18, 0x00, 0x28, 0x00, 0x38, 0x00, 0x48, 0x00, 0x58, 0x00, 0x68,
            0x00, 0x78,
        ]);
        vram[0x120..0x124].copy_from_slice(&[0x00, 0x48, 0x00, 0x58]);
        vram[0x140..0x144].copy_from_slice(&[0x00, 0x68, 0x00, 0x78]);

        assert_eq!(super::hscroll_for_line(&vram, 0x100, 3, 0, 0x00), 0x008);
        assert_eq!(super::hscroll_for_line(&vram, 0x100, 3, 2, 0x00), 0x018);
        assert_eq!(super::hscroll_for_line(&vram, 0x100, 10, 0, 0x01), 0x048);
        assert_eq!(super::hscroll_for_line(&vram, 0x100, 10, 2, 0x01), 0x058);
        assert_eq!(super::hscroll_for_line(&vram, 0x100, 9, 0, 0x02), 0x048);
        assert_eq!(super::hscroll_for_line(&vram, 0x100, 9, 2, 0x02), 0x058);
        assert_eq!(super::hscroll_for_line(&vram, 0x100, 16, 0, 0x03), 0x068);
        assert_eq!(super::hscroll_for_line(&vram, 0x100, 16, 2, 0x03), 0x078);
    }

    #[test]
    fn gems_engine_constructs_without_panic() {
        // Minimal dummy data — just verifies no crash during construction
        let driver = vec![0u8; 64]; // stub driver (won't produce real audio)
        let patches = vec![0u8; 16];
        let envelopes = vec![0u8; 16];
        let sequences = vec![0u8; 16];
        let samples = vec![0u8; 16];

        let mut engine =
            super::GemsAudioEngine::new(&driver, &patches, &envelopes, &sequences, &samples);
        engine.step_frame();
        let samples = engine.generate_samples(735);
        assert_eq!(
            samples.len(),
            735 * 2,
            "should produce stereo interleaved samples"
        );
    }

    #[test]
    fn gems_engine_init_queues_data_pointers_low_to_high_like_sub_e14ac() {
        let driver = vec![0u8; 64];
        let patches = vec![0u8; 16];
        let envelopes = vec![0u8; 16];
        let sequences = vec![0u8; 16];
        let samples = vec![0u8; 16];

        let mut engine =
            super::GemsAudioEngine::new(&driver, &patches, &envelopes, &sequences, &samples);
        engine.init();

        let queue = &engine.z80.io.ram[usize::from(GEMS_QUEUE_ADDR)..][..14];
        assert_eq!(
            queue,
            [
                0xFF, 0x0B, 0x0E, 0x16, 0x0E, 0xEA, 0x25, 0x0E, 0x66, 0x27, 0x0E, 0x16, 0x87, 0x0E,
            ],
            "ASM sub_E14AC writes the low byte first, then shifts right before queueing the middle and high bytes",
        );
    }

    #[test]
    fn gems_engine_play_sound_does_not_panic() {
        let driver = vec![0u8; 64];
        let patches = vec![0u8; 16];
        let envelopes = vec![0u8; 16];
        let sequences = vec![0u8; 16];
        let samples = vec![0u8; 16];

        let mut engine =
            super::GemsAudioEngine::new(&driver, &patches, &envelopes, &sequences, &samples);
        // play_sound will auto-init, which runs step_z80_frame multiple times
        engine.play_sound(0x17); // sfx_Jump
        engine.step_frame();
        let output = engine.generate_samples(735);
        assert_eq!(output.len(), 735 * 2);
    }

    #[test]
    fn gems_engine_stop_all_sequences_queues_command_16() {
        let driver = vec![0u8; 64];
        let patches = vec![0u8; 16];
        let envelopes = vec![0u8; 16];
        let sequences = vec![0u8; 16];
        let samples = vec![0u8; 16];

        let mut engine =
            super::GemsAudioEngine::new(&driver, &patches, &envelopes, &sequences, &samples);
        engine.init();
        engine.stop_all_sequences();

        let queue = &engine.z80.io.ram[usize::from(GEMS_QUEUE_ADDR)..][..17];
        assert_eq!(
            &queue[14..17],
            [0xFF, 0x16, 0xFF],
            "GEMS driver command $16 reads one more byte as the sequence id; $FF means stop all sequences"
        );
    }

    #[test]
    fn gems_engine_stop_all_then_play_sound_keeps_start_command_intact() {
        let driver = vec![0u8; 64];
        let patches = vec![0u8; 16];
        let envelopes = vec![0u8; 16];
        let sequences = vec![0u8; 16];
        let samples = vec![0u8; 16];

        let mut engine =
            super::GemsAudioEngine::new(&driver, &patches, &envelopes, &sequences, &samples);
        engine.init();
        engine.stop_all_sequences();
        engine.play_sound(0x07);

        let queue = &engine.z80.io.ram[usize::from(GEMS_QUEUE_ADDR)..][..20];
        assert_eq!(
            &queue[14..20],
            [0xFF, 0x16, 0xFF, 0xFF, 0x10, 0x07],
            "ASM/GEMS command $16 consumes a sequence id byte; without the $FF id, the following PlaySound command marker is swallowed and level BGM never starts"
        );
    }

    #[test]
    fn gems_engine_stop_music_reloads_driver_and_requeues_pointer_init_like_rom() {
        let mut driver = vec![0u8; 64];
        driver[0] = 0xC3;
        let patches = vec![0u8; 16];
        let envelopes = vec![0u8; 16];
        let sequences = vec![0u8; 16];
        let samples = vec![0u8; 16];

        let mut engine =
            super::GemsAudioEngine::new(&driver, &patches, &envelopes, &sequences, &samples);
        engine.init();
        engine.play_sound(0x03);
        engine.z80.io.ram[0] = 0x00;
        engine.z80.io.ram[usize::from(GEMS_QUEUE_ADDR) + 30] = 0xAA;

        engine.stop_music();

        let queue = &engine.z80.io.ram[usize::from(GEMS_QUEUE_ADDR)..][..14];
        assert_eq!(
            engine.z80.io.ram[0], 0xC3,
            "ASM StopMusic reloads the GEMS driver before queueing the pointer init"
        );
        assert_eq!(
            queue,
            [
                0xFF, 0x0B, 0x0E, 0x16, 0x0E, 0xEA, 0x25, 0x0E, 0x66, 0x27, 0x0E, 0x16, 0x87, 0x0E,
            ],
            "ASM StopMusic queues the same `$0B` pointer command as initial GEMS setup, not command `$16`"
        );
        assert_eq!(
            engine.z80.io.ram[usize::from(GEMS_QUEUE_ADDR) + 30],
            0x00,
            "ASM StopMusic clears Z80 RAM, so stale title commands cannot survive into gameplay"
        );
    }
}
