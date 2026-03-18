use anyhow::{Result, bail};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteSet {
    pub foreground: Palette,
    pub background: Palette,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    pub colours: Vec<MdColour>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MdColour {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub raw_word: u16,
}

impl Palette {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() % 2 != 0 {
            bail!("palette data length {} is not even", bytes.len());
        }
        let colours = bytes
            .chunks_exact(2)
            .map(|chunk| MdColour::from_word(u16::from_be_bytes([chunk[0], chunk[1]])))
            .collect();
        Ok(Self { colours })
    }

    /// Apply a palette permutation table. The table maps pixel index → palette
    /// slot, matching the ASM art decompression remapping. Reorders colours so
    /// that un-permuted tile art indices look up the correct colour.
    pub fn apply_permutation(&mut self, table: &[u8; 16]) {
        // The table says: art pixel index i → palette slot table[i].
        // We want: new_palette[i] = original_palette[table[i]].
        let original = self.colours.clone();
        for i in 0..16.min(self.colours.len()) {
            let src = table[i] as usize;
            if src < original.len() {
                self.colours[i] = original[src];
            }
        }
    }
}

impl MdColour {
    pub fn from_word(raw_word: u16) -> Self {
        let blue = ((raw_word >> 9) & 0x7) as u8;
        let green = ((raw_word >> 5) & 0x7) as u8;
        let red = ((raw_word >> 1) & 0x7) as u8;
        Self {
            red: expand_channel(red),
            green: expand_channel(green),
            blue: expand_channel(blue),
            raw_word,
        }
    }
}

fn expand_channel(value: u8) -> u8 {
    ((u16::from(value) * 255) / 7) as u8
}
