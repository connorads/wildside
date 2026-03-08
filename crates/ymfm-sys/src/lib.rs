use std::ffi::c_void;

const YM2612_INPUT_CLOCK_HZ: u32 = 7_670_454;

unsafe extern "C" {
    fn ymfm_ym2612_create() -> *mut c_void;
    fn ymfm_ym2612_destroy(chip: *mut c_void);
    fn ymfm_ym2612_reset(chip: *mut c_void);
    fn ymfm_ym2612_sample_rate(input_clock_hz: u32) -> u32;
    fn ymfm_ym2612_read(chip: *mut c_void, port: u8) -> u8;
    fn ymfm_ym2612_advance(chip: *mut c_void, clocks: u32);
    fn ymfm_ym2612_irq_asserted(chip: *mut c_void) -> bool;
    fn ymfm_ym2612_write(chip: *mut c_void, port: u8, value: u8);
    fn ymfm_ym2612_generate(
        chip: *mut c_void,
        left: *mut i32,
        right: *mut i32,
        sample_count: usize,
    );
}

pub struct Ym2612 {
    raw: *mut c_void,
    sample_rate_hz: u32,
}

impl Ym2612 {
    pub fn new() -> Self {
        let raw = unsafe { ymfm_ym2612_create() };
        assert!(!raw.is_null(), "ymfm_ym2612_create returned null");
        let sample_rate_hz = unsafe { ymfm_ym2612_sample_rate(YM2612_INPUT_CLOCK_HZ) };
        Self {
            raw,
            sample_rate_hz,
        }
    }

    pub fn reset(&mut self) {
        unsafe { ymfm_ym2612_reset(self.raw) };
    }

    pub fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    pub fn read_port(&self, port: u8) -> u8 {
        assert!(port < 4, "YM2612 port must be in 0..4");
        unsafe { ymfm_ym2612_read(self.raw, port) }
    }

    pub fn advance_clocks(&mut self, clocks: u32) {
        unsafe { ymfm_ym2612_advance(self.raw, clocks) };
    }

    pub fn irq_asserted(&self) -> bool {
        unsafe { ymfm_ym2612_irq_asserted(self.raw) }
    }

    pub fn write_port(&mut self, port: u8, value: u8) {
        assert!(port < 4, "YM2612 port must be in 0..4");
        unsafe { ymfm_ym2612_write(self.raw, port, value) };
    }

    pub fn generate(&mut self, left: &mut [i32], right: &mut [i32]) {
        assert_eq!(left.len(), right.len(), "YM2612 output buffers must match");
        unsafe {
            ymfm_ym2612_generate(self.raw, left.as_mut_ptr(), right.as_mut_ptr(), left.len());
        }
    }
}

impl Default for Ym2612 {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Ym2612 {
    fn drop(&mut self) {
        unsafe { ymfm_ym2612_destroy(self.raw) };
    }
}

#[cfg(test)]
mod tests {
    use super::Ym2612;

    fn write_register(chip: &mut Ym2612, register: u8, value: u8) {
        chip.write_port(0, register);
        chip.write_port(1, value);
    }

    #[test]
    fn produces_varying_output_after_simple_key_on() {
        let mut chip = Ym2612::default();

        write_register(&mut chip, 0x22, 0x00);
        write_register(&mut chip, 0x27, 0x00);
        write_register(&mut chip, 0x2B, 0x00);
        write_register(&mut chip, 0xB0, 0x07);
        write_register(&mut chip, 0xB4, 0xC0);
        for register in [0x30, 0x34, 0x38, 0x3C] {
            write_register(&mut chip, register, 0x01);
        }
        for register in [0x40, 0x44, 0x48, 0x4C] {
            write_register(&mut chip, register, 0x00);
        }
        for register in [0x50, 0x54, 0x58, 0x5C] {
            write_register(&mut chip, register, 0x1F);
        }
        for register in [0x60, 0x64, 0x68, 0x6C] {
            write_register(&mut chip, register, 0x00);
        }
        for register in [0x70, 0x74, 0x78, 0x7C] {
            write_register(&mut chip, register, 0x00);
        }
        for register in [0x80, 0x84, 0x88, 0x8C] {
            write_register(&mut chip, register, 0x0F);
        }
        for register in [0x90, 0x94, 0x98, 0x9C] {
            write_register(&mut chip, register, 0x00);
        }
        write_register(&mut chip, 0xA4, 0x22);
        write_register(&mut chip, 0xA0, 0x69);
        write_register(&mut chip, 0x28, 0xF0);

        let mut left = [0i32; 256];
        let mut right = [0i32; 256];
        chip.generate(&mut left, &mut right);

        assert!(left.iter().any(|sample| *sample != left[0]));
        assert!(right.iter().any(|sample| *sample != right[0]));
    }
}
