use kid_md::{GameState, MdRuntime};

pub(crate) enum CheatEffect {
    Instant,
    Adjustable { min: u16, max: u16 },
}

pub(crate) struct CheatEntry {
    pub label: &'static str,
    pub effect: CheatEffect,
    pub read: fn(&GameState) -> u16,
    pub apply: fn(&mut MdRuntime, u16),
}

pub(crate) const CHEATS: &[CheatEntry] = &[
    CheatEntry {
        label: "Level",
        effect: CheatEffect::Adjustable { min: 0, max: 0x69 },
        read: |state| state.current_level_id,
        apply: |rt, val| rt.set_current_level(val),
    },
    CheatEntry {
        label: "Helmet",
        effect: CheatEffect::Adjustable { min: 0, max: 9 },
        read: |state| state.current_helmet_id,
        apply: |rt, val| rt.set_current_helmet(val),
    },
    CheatEntry {
        label: "Lives",
        effect: CheatEffect::Adjustable { min: 0, max: 99 },
        read: |state| state.number_lives,
        apply: |rt, val| rt.set_lives(val),
    },
    CheatEntry {
        label: "Hitpoints",
        effect: CheatEffect::Adjustable { min: 0, max: 5 },
        read: |state| state.number_hitpoints,
        apply: |rt, val| rt.set_hitpoints(val),
    },
    CheatEntry {
        label: "Max Lives",
        effect: CheatEffect::Instant,
        read: |state| state.number_lives,
        apply: |rt, _| rt.set_lives(99),
    },
    CheatEntry {
        label: "Level Skip",
        effect: CheatEffect::Instant,
        read: |_| 0,
        apply: |rt, _| rt.set_level_skip_cheat_enabled(true),
    },
];
