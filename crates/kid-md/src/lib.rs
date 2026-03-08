use anyhow::Result;
use r68k_emu::cpu::{ConfiguredCore, Cycles};
use r68k_emu::interrupts::AutoInterruptController;
use r68k_emu::ram::{AddressBus, AddressSpace};
use std::cell::RefCell;
use thiserror::Error;

const ADDRESS_MASK: u32 = 0x00FF_FFFF;
const WORK_RAM_BASE: u32 = 0x00E0_0000;
const WORK_RAM_END: u32 = 0x00FF_FFFF;
const WORK_RAM_LEN: usize = 0x1_0000;
const Z80_RAM_BASE: u32 = 0x00A0_0000;
const Z80_RAM_END: u32 = 0x00A0_FFFF;
const Z80_RAM_LEN: usize = 0x2000;
const SYNTHETIC_VBLANK_CYCLES: i32 = 120_000;
const V_INT_COUNTER_ADDR: u32 = 0x00FF_F806;
const V_INT_DONE_ADDR: u32 = 0x00FF_F80A;
const CAMERA_X_POS_ADDR: u32 = 0x00FF_F81C;
const CAMERA_Y_POS_ADDR: u32 = 0x00FF_F820;
const NUMBER_OBJECTS_ADDR: u32 = 0x00FF_F84A;
const NUMBER_GFX_OBJECTS_ADDR: u32 = 0x00FF_F85C;
const DEMO_MODE_FLAG_ADDR: u32 = 0x00FF_FBC9;
const GAME_MODE_ADDR: u32 = 0x00FF_FBCA;
const CURRENT_PLAYER_ADDR: u32 = 0x00FF_FC39;
const CURRENT_LEVEL_ID_ADDR: u32 = 0x00FF_FC44;
const PLAYER_1_LEVEL_ID_ADDR: u32 = 0x00FF_FC5A;
const VRAM_LEN: usize = 0x1_0000;
const CRAM_LEN: usize = 0x80;
const VSRAM_LEN: usize = 0x50;
const TRACE_LIMIT: usize = 256;

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
    pub current_level_id: u16,
    pub player_1_level_id: u16,
    pub camera_x_pos: u32,
    pub camera_y_pos: u32,
    pub number_objects: u16,
    pub number_gfx_objects: u16,
    pub vram_nonzero_bytes: usize,
    pub cram_nonzero_bytes: usize,
    pub vsram_nonzero_bytes: usize,
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

pub struct MdRuntime {
    core: ConfiguredCore<AutoInterruptController, MdBus>,
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
        while runtime.synthetic_vblanks as usize > frame_index {
            frame_index += 1;
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
        current_level_id: runtime.core.mem.read_work_ram_word(CURRENT_LEVEL_ID_ADDR),
        player_1_level_id: runtime.core.mem.read_work_ram_word(PLAYER_1_LEVEL_ID_ADDR),
        camera_x_pos: runtime.core.mem.read_work_ram_long(CAMERA_X_POS_ADDR),
        camera_y_pos: runtime.core.mem.read_work_ram_long(CAMERA_Y_POS_ADDR),
        number_objects: runtime.core.mem.read_work_ram_word(NUMBER_OBJECTS_ADDR),
        number_gfx_objects: runtime.core.mem.read_work_ram_word(NUMBER_GFX_OBJECTS_ADDR),
        vram_nonzero_bytes: runtime.core.mem.vram_nonzero_bytes(),
        cram_nonzero_bytes: runtime.core.mem.cram_nonzero_bytes(),
        vsram_nonzero_bytes: runtime.core.mem.vsram_nonzero_bytes(),
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

        let mut core = ConfiguredCore::new_with(0, AutoInterruptController::new(), MdBus::new(rom));
        core.reset();
        Ok(Self {
            core,
            cycles_until_vblank: SYNTHETIC_VBLANK_CYCLES,
            synthetic_vblanks: 0,
        })
    }

    pub fn set_controller_1(&mut self, buttons: u8) {
        self.core.mem.set_controller_1(buttons);
    }

    pub fn step(&mut self) -> i32 {
        let Cycles(cycles) = self.core.execute1();
        self.cycles_until_vblank -= cycles;
        while self.cycles_until_vblank <= 0 {
            self.cycles_until_vblank += SYNTHETIC_VBLANK_CYCLES;
            if self.core.mem.vint_enabled() {
                self.core.int_ctrl.request_interrupt(6);
                self.synthetic_vblanks += 1;
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
            current_level_id: self.core.mem.read_work_ram_word(CURRENT_LEVEL_ID_ADDR),
            player_1_level_id: self.core.mem.read_work_ram_word(PLAYER_1_LEVEL_ID_ADDR),
            camera_x_pos: self.core.mem.read_work_ram_long(CAMERA_X_POS_ADDR),
            camera_y_pos: self.core.mem.read_work_ram_long(CAMERA_Y_POS_ADDR),
            number_objects: self.core.mem.read_work_ram_word(NUMBER_OBJECTS_ADDR),
            number_gfx_objects: self.core.mem.read_work_ram_word(NUMBER_GFX_OBJECTS_ADDR),
            vram_nonzero_bytes: self.core.mem.vram_nonzero_bytes(),
            cram_nonzero_bytes: self.core.mem.cram_nonzero_bytes(),
            vsram_nonzero_bytes: self.core.mem.vsram_nonzero_bytes(),
            vdp: self.core.mem.vdp_snapshot(),
            steps: Vec::new(),
            accesses: self.core.mem.accesses(),
            vdp_control_writes: self.core.mem.vdp_control_writes(),
            vdp_data_writes: self.core.mem.vdp_data_writes(),
        }
    }
}

pub fn render_frame(probe: &CpuProbe) -> FrameBuffer {
    let width = 320usize;
    let height = 224usize;
    let mut rgb = vec![0; width * height * 3];
    let background = cram_rgb(&probe.vdp.cram, usize::from(probe.vdp.registers[7] & 0x3F));
    for pixel in rgb.chunks_exact_mut(3) {
        pixel.copy_from_slice(&background);
    }

    let plane_a_base = (usize::from(probe.vdp.registers[2] & 0x38)) << 10;
    let plane_b_base = (usize::from(probe.vdp.registers[4] & 0x07)) << 13;
    let window_base = window_plane_base(&probe.vdp.registers);
    let sprite_table_base = sprite_table_base(&probe.vdp.registers);
    let hscroll_base = usize::from(probe.vdp.registers[0x0D]) << 10;
    let vscroll_a = i32::from(read_u16_wrapped(&probe.vdp.vsram, 0) as i16);
    let vscroll_b = i32::from(read_u16_wrapped(&probe.vdp.vsram, 2) as i16);
    let (plane_width_cells, plane_height_cells) = plane_dimensions(probe.vdp.registers[0x10]);
    let window_region = window_region(&probe.vdp.registers, width, height);

    for y in 0..height {
        let plane_a = ScrollPlane {
            base: plane_a_base,
            width_cells: plane_width_cells,
            height_cells: plane_height_cells,
            hscroll: hscroll_for_line(
                &probe.vdp.vram,
                hscroll_base,
                y,
                2,
                probe.vdp.registers[0x0B],
            ),
            vscroll: vscroll_a,
        };
        let plane_b = ScrollPlane {
            base: plane_b_base,
            width_cells: plane_width_cells,
            height_cells: plane_height_cells,
            hscroll: hscroll_for_line(
                &probe.vdp.vram,
                hscroll_base,
                y,
                0,
                probe.vdp.registers[0x0B],
            ),
            vscroll: vscroll_b,
        };
        let window_plane = WindowPlane {
            base: window_base,
            region: window_region,
        };
        for x in 0..width {
            for priority in [false, true] {
                if let Some(colour) = plane_pixel_rgb(&probe.vdp, &plane_b, x, y, priority) {
                    write_rgb(&mut rgb, width, x, y, colour);
                }
                if let Some(colour) =
                    plane_a_or_window_pixel_rgb(&probe.vdp, &plane_a, &window_plane, x, y, priority)
                {
                    write_rgb(&mut rgb, width, x, y, colour);
                }
                if let Some(colour) =
                    sprite_pixel_rgb(&probe.vdp, sprite_table_base, x, y, priority)
                {
                    write_rgb(&mut rgb, width, x, y, colour);
                }
            }
        }
    }

    FrameBuffer { width, height, rgb }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IoState {
    version_register: u8,
    p1_data: u8,
    p2_data: u8,
    p1_ctrl: u8,
    p2_ctrl: u8,
    controller_1: u8,
    controller_2: u8,
    z80_busreq: bool,
    z80_reset: bool,
}

impl Default for IoState {
    fn default() -> Self {
        Self {
            version_register: 0x20,
            p1_data: 0x7F,
            p2_data: 0x7F,
            p1_ctrl: 0x00,
            p2_ctrl: 0x00,
            controller_1: 0,
            controller_2: 0,
            z80_busreq: false,
            z80_reset: false,
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct MdBus {
    rom: Vec<u8>,
    work_ram: Vec<u8>,
    z80_ram: Vec<u8>,
    io: IoState,
    vdp: VdpState,
    accesses: RefCell<Vec<BusAccess>>,
}

impl MdBus {
    fn new(rom: &[u8]) -> Self {
        Self {
            rom: rom.to_vec(),
            work_ram: vec![0; WORK_RAM_LEN],
            z80_ram: vec![0; Z80_RAM_LEN],
            io: IoState::default(),
            vdp: VdpState::default(),
            accesses: RefCell::new(Vec::new()),
        }
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

    fn read_io_register(&self, address: u32) -> u8 {
        match address {
            0x00A1_0000 | 0x00A1_0001 => self.io.version_register,
            0x00A1_0002 | 0x00A1_0003 => {
                self.controller_port_value(self.io.p1_data, self.io.controller_1)
            }
            0x00A1_0004 | 0x00A1_0005 => {
                self.controller_port_value(self.io.p2_data, self.io.controller_2)
            }
            0x00A1_0008 | 0x00A1_0009 => self.io.p1_ctrl,
            0x00A1_000A | 0x00A1_000B => self.io.p2_ctrl,
            0x00A1_000E | 0x00A1_000F | 0x00A1_0014 | 0x00A1_0015 | 0x00A1_001A | 0x00A1_001B => {
                0xFF
            }
            _ => 0x00,
        }
    }

    fn write_io_register(&mut self, address: u32, value: u8) {
        match address {
            0x00A1_0002 | 0x00A1_0003 => self.io.p1_data = value,
            0x00A1_0004 | 0x00A1_0005 => self.io.p2_data = value,
            0x00A1_0008 | 0x00A1_0009 => self.io.p1_ctrl = value,
            0x00A1_000A | 0x00A1_000B => self.io.p2_ctrl = value,
            _ => {}
        }
    }

    fn set_controller_1(&mut self, buttons: u8) {
        self.io.controller_1 = buttons;
    }

    fn controller_port_value(&self, latch: u8, buttons: u8) -> u8 {
        let th_high = latch & 0x40 != 0;
        let mut value = latch & 0x40;
        if th_high {
            value |= 0x3F;
            value &= !controller_low_mask(buttons, 0);
            value &= !controller_low_mask(buttons, 1);
            value &= !controller_low_mask(buttons, 2);
            value &= !controller_low_mask(buttons, 3);
            value &= !controller_low_mask(buttons, 4);
            value &= !controller_low_mask(buttons, 5);
        } else {
            value |= 0x33;
            value &= !controller_low_mask(buttons, 0);
            value &= !controller_low_mask(buttons, 1);
            value &= !controller_low_mask(buttons, 6);
            value &= !controller_low_mask(buttons, 7);
        }
        value
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
                Self::write_vdp_word_slice(&mut self.vdp.vram, self.vdp.data_port_address, value)
            }
            0x03 => {
                Self::write_vdp_word_slice(&mut self.vdp.cram, self.vdp.data_port_address, value)
            }
            0x05 => {
                Self::write_vdp_word_slice(&mut self.vdp.vsram, self.vdp.data_port_address, value)
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
            Z80_RAM_BASE..=Z80_RAM_END => self.z80_ram[Self::z80_ram_index(address)],
            0x00A1_0000..=0x00A1_001F => self.read_io_register(address),
            0x00A1_1100..=0x00A1_1101 => u8::from(!self.io.z80_busreq),
            0x00A1_1200..=0x00A1_1201 => u8::from(!self.io.z80_reset),
            0x00C0_0000..=0x00C0_001F => self.read_vdp_byte(address),
            WORK_RAM_BASE..=WORK_RAM_END => self.work_ram[Self::work_ram_index(address)],
            _ => 0xFF,
        }
    }

    fn write_byte_internal(&mut self, address: u32, value: u8) {
        match address {
            Z80_RAM_BASE..=Z80_RAM_END => {
                let index = Self::z80_ram_index(address);
                self.z80_ram[index] = value;
            }
            0x00A1_0000..=0x00A1_001F => self.write_io_register(address, value),
            0x00A1_1100..=0x00A1_1101 => self.io.z80_busreq = value & 1 != 0,
            0x00A1_1200..=0x00A1_1201 => self.io.z80_reset = value & 1 == 0,
            0x00C0_0000..=0x00C0_001F => self.write_vdp_byte(address, value),
            WORK_RAM_BASE..=WORK_RAM_END => {
                let index = Self::work_ram_index(address);
                self.work_ram[index] = value;
            }
            _ => {}
        }
    }

    fn read_word_internal(&self, address: u32) -> u16 {
        match address {
            0x00A1_0000..=0x00A1_001F => u16::from(self.read_io_register(address)),
            0x00A1_1100..=0x00A1_1101 => {
                let byte = u8::from(!self.io.z80_busreq);
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
            0x00A1_0000..=0x00A1_001F => self.write_io_register(address, value as u8),
            0x00A1_1100..=0x00A1_1101 => self.io.z80_busreq = (value & 0x0100) != 0,
            0x00A1_1200..=0x00A1_1201 => self.io.z80_reset = (value & 0x0100) == 0,
            0x00C0_0000..=0x00C0_0003 => self.write_vdp_data_word(value),
            0x00C0_0004..=0x00C0_0007 => self.apply_vdp_control_write(value),
            WORK_RAM_BASE..=WORK_RAM_END => {
                let index = Self::work_ram_index(address);
                self.work_ram[index] = (value >> 8) as u8;
                self.work_ram[Self::work_ram_index(address.wrapping_add(1))] = value as u8;
            }
            _ => {
                self.write_byte_internal(address, (value >> 8) as u8);
                self.write_byte_internal(address.wrapping_add(1), value as u8);
            }
        }
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
    let mode = mode_register & 0x03;
    let offset = if mode == 0x03 {
        base + line * 4 + plane_offset
    } else {
        base + plane_offset
    };
    i32::from(read_u16_wrapped(vram, offset) as i16)
}

fn plane_pixel_rgb(
    vdp: &VdpSnapshot,
    plane: &ScrollPlane,
    screen_x: usize,
    screen_y: usize,
    priority: bool,
) -> Option<[u8; 3]> {
    let plane_width_pixels = (plane.width_cells * 8) as i32;
    let plane_height_pixels = (plane.height_cells * 8) as i32;
    let world_x = (screen_x as i32 + plane.hscroll).rem_euclid(plane_width_pixels);
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

fn sprite_pixel_rgb(
    vdp: &VdpSnapshot,
    sprite_table_base: usize,
    screen_x: usize,
    screen_y: usize,
    priority: bool,
) -> Option<[u8; 3]> {
    let mut sprite_index = 0usize;
    let mut visited = [false; 80];
    let screen_x = screen_x as i32;
    let screen_y = screen_y as i32;

    for _ in 0..80 {
        if sprite_index >= visited.len() || visited[sprite_index] {
            break;
        }
        visited[sprite_index] = true;

        let entry_base = sprite_table_base + sprite_index * 8;
        let ypos = i32::from(read_u16_wrapped(&vdp.vram, entry_base));
        let size_link = read_u16_wrapped(&vdp.vram, entry_base + 2);
        let pattern = read_u16_wrapped(&vdp.vram, entry_base + 4);
        let xpos = i32::from(read_u16_wrapped(&vdp.vram, entry_base + 6));
        let next_link = usize::from(size_link & 0x7F);
        let width_tiles = usize::from(((size_link >> 10) & 0x03) + 1);
        let height_tiles = usize::from(((size_link >> 8) & 0x03) + 1);
        let left = xpos - 128;
        let top = ypos - 128;
        let width_pixels = (width_tiles * 8) as i32;
        let height_pixels = (height_tiles * 8) as i32;

        if ((pattern & 0x8000) != 0) == priority
            && screen_x >= left
            && screen_x < left + width_pixels
            && screen_y >= top
            && screen_y < top + height_pixels
        {
            let local_x = usize::try_from(screen_x - left).unwrap_or(0);
            let local_y = usize::try_from(screen_y - top).unwrap_or(0);
            if let Some(colour) =
                sprite_tile_pixel_rgb(vdp, pattern, width_tiles, height_tiles, local_x, local_y)
            {
                return Some(colour);
            }
        }

        if next_link == 0 {
            break;
        }
        sprite_index = next_link;
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
    let tile_index = base_tile + tile_row * width_tiles + tile_col;
    let pixel_x = local_x & 7;
    let pixel_y = local_y & 7;
    tile_pixel_rgb_from_parts(vdp, tile_index, palette, pixel_x, pixel_y)
}

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

fn scale_channel(value: u8) -> u8 {
    (u16::from(value) * 255 / 7) as u8
}

fn write_rgb(rgb: &mut [u8], width: usize, x: usize, y: usize, colour: [u8; 3]) {
    let index = (y * width + x) * 3;
    rgb[index..index + 3].copy_from_slice(&colour);
}

fn read_u8_wrapped(slice: &[u8], index: usize) -> u8 {
    if slice.is_empty() {
        0
    } else {
        slice[index % slice.len()]
    }
}

fn read_u16_wrapped(slice: &[u8], index: usize) -> u16 {
    u16::from_be_bytes([
        read_u8_wrapped(slice, index),
        read_u8_wrapped(slice, index + 1),
    ])
}

fn controller_low_mask(buttons: u8, button_bit: u8) -> u8 {
    if buttons & (1 << button_bit) == 0 {
        0
    } else {
        match button_bit {
            0 => 1 << 0,
            1 => 1 << 1,
            2 => 1 << 2,
            3 => 1 << 3,
            4 => 1 << 4,
            5 => 1 << 5,
            6 => 1 << 4,
            7 => 1 << 5,
            _ => 0,
        }
    }
}

impl AddressBus for MdBus {
    fn copy_from(&mut self, other: &Self) {
        self.rom = other.rom.clone();
        self.work_ram = other.work_ram.clone();
        self.z80_ram = other.z80_ram.clone();
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

#[cfg(test)]
mod tests {
    use super::{BusAccessKind, MdBus, WORK_RAM_BASE, Z80_RAM_BASE, probe_rom};
    use r68k_emu::ram::{AddressBus, SUPERVISOR_DATA};

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
    fn work_ram_is_mirrored_across_upper_address_space() {
        let mut bus = MdBus::new(&[]);
        bus.write_word(SUPERVISOR_DATA, WORK_RAM_BASE, 0x1234);
        assert_eq!(bus.read_word(SUPERVISOR_DATA, WORK_RAM_BASE), 0x1234);
        assert_eq!(bus.read_word(SUPERVISOR_DATA, 0x00FF_0000), 0x1234);
    }

    #[test]
    fn controller_and_vdp_accesses_are_traced() {
        let mut bus = MdBus::new(&[]);
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
    fn z80_ram_is_mirrored() {
        let mut bus = MdBus::new(&[]);
        bus.write_byte(SUPERVISOR_DATA, Z80_RAM_BASE, 0x12);
        assert_eq!(bus.read_byte(SUPERVISOR_DATA, Z80_RAM_BASE), 0x12);
        assert_eq!(bus.read_byte(SUPERVISOR_DATA, Z80_RAM_BASE + 0x2000), 0x12);
    }
}
