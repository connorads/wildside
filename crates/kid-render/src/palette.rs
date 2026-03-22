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
        if !bytes.len().is_multiple_of(2) {
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
        let original = self.colours.clone();
        if self.colours.len() == 15 {
            // Theme foreground palettes store only visible slots 1..15; slot 0
            // is the transparent backdrop and is omitted from the asset bytes.
            // The permutation tables are still expressed in full MD slot space,
            // so visible slot N must read from table[N] and source slot M maps
            // to stored index M-1.
            for (visible_slot, &mapped_slot) in table
                .iter()
                .enumerate()
                .take(self.colours.len() + 1)
                .skip(1)
            {
                let source_slot = usize::from(mapped_slot);
                if source_slot == 0 {
                    continue;
                }
                let source_index = source_slot - 1;
                if source_index < original.len() {
                    self.colours[visible_slot - 1] = original[source_index];
                }
            }
            return;
        }

        // Palettes that include slot 0 can use the table directly.
        for (i, &entry) in table.iter().enumerate().take(16.min(self.colours.len())) {
            let source_index = usize::from(entry);
            if source_index < original.len() {
                self.colours[i] = original[source_index];
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
    ((u16::from(value) * 255 + 3) / 7) as u8
}

#[cfg(test)]
mod tests {
    use super::{MdColour, Palette, expand_channel};

    #[test]
    fn expands_megadrive_channels_using_oracle_rgb_rounding() {
        let expanded: Vec<u8> = (0..=7).map(expand_channel).collect();

        assert_eq!(
            expanded,
            vec![0x00, 0x24, 0x49, 0x6D, 0x92, 0xB6, 0xDB, 0xFF]
        );
    }

    #[test]
    fn decodes_rgb_components_from_packed_md_colour_word() {
        let colour = MdColour::from_word(0x0EEE);

        assert_eq!(colour.red, 0xFF);
        assert_eq!(colour.green, 0xFF);
        assert_eq!(colour.blue, 0xFF);
        assert_eq!(colour.raw_word, 0x0EEE);
    }

    #[test]
    fn applies_permutation_to_visible_only_palette_slots() {
        let make_colour = |raw_word| MdColour {
            red: 0,
            green: 0,
            blue: 0,
            raw_word,
        };
        let mut palette = Palette {
            colours: (1..=15).map(make_colour).collect(),
        };
        let table = [0, 1, 3, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

        palette.apply_permutation(&table);

        let reordered: Vec<u16> = palette
            .colours
            .iter()
            .map(|colour| colour.raw_word)
            .collect();
        assert_eq!(reordered[0], 1);
        assert_eq!(reordered[1], 3);
        assert_eq!(reordered[2], 2);
    }

    #[test]
    fn applies_permutation_directly_when_palette_includes_slot_zero() {
        let make_colour = |raw_word| MdColour {
            red: 0,
            green: 0,
            blue: 0,
            raw_word,
        };
        let mut palette = Palette {
            colours: (0..8).map(make_colour).collect(),
        };
        let table = [0, 2, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

        palette.apply_permutation(&table);

        let reordered: Vec<u16> = palette
            .colours
            .iter()
            .map(|colour| colour.raw_word)
            .collect();
        assert_eq!(reordered[..4], [0, 2, 1, 3]);
    }
}
