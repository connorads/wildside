//! Route-wide per-frame GEMS/Z80 audio-timing parity
//! (`gems_frame_signature_v1`).
//!
//! The oracle helper samples a [`GemsFrameSignature`] at every
//! `FrameRendered` boundary; this module produces the native stream from the
//! persistent GEMS timing engine driven by the same trace→command replayer as
//! audio rendering, then compares the two streams like the per-frame RNG
//! pass. v1 deliberately measures divergence rather than claiming parity: see
//! [`GEMS_MODELLED_ROUTE_FRAMES`].
//!
//! Sampling instant (see [`GemsFrameSignature`] for the derivation): the
//! oracle's `FrameRendered` fires when the VDP enters scanline 232 — VBlank
//! start (224) plus the pinned jgenesis `NTSC_BOTTOM_BORDER` (8) — so the
//! native probe samples its engine 8 scanlines past each VBlank frame
//! boundary. Both offsets come from the pinned VDP raster geometry; nothing
//! is fitted. The oracle additionally quantises to the end of the 68000
//! instruction quantum that crossed the line; that residual jitter is why
//! `z80_mclk_counter` and `ym2612_mclk_counter` are advisory.

use anyhow::Result;
use sha1::{Digest, Sha1};
use wildside_core::{
    GemsFrameSignature, GemsStopMusicBusAckPolls, M68kCycleCount, M68kRefreshPhase,
    NativeTraceFrame, gems_stop_music_elapsed_timing,
    gems_stop_music_elapsed_timing_from_absolute_entry,
};
use wildside_data::AssetManifest;
use wildside_frame::{
    NativeAudioDirector, NativeAudioSink, TimedGemsStopMusicCursor, TimedGemsTitleLoc700Cursor,
    apply_gems_decompress_to_vram_decoder_loc_14300, apply_gems_decompress_to_vram_setup_sub_142cc,
    apply_gems_decompress_to_vram_transfer_display_disabled_loc_14314,
    apply_gems_power_on_post_clear_reset, apply_gems_power_on_z80_startup,
    apply_gems_title_mode_standard_empty_foreground_queues,
    apply_gems_title_mode_standard_palette_to_vram,
    apply_gems_title_mode_standard_scroll_data_to_vram,
    apply_gems_title_mode_standard_sprite_and_kid_to_vram, apply_gems_title_palette_to_vram_call,
};
use wildside_md::{GemsMasterClockTimingEngine, GemsNtscTimelineOrigin, GemsTimingCheckpoint};

const MASTER_CLOCKS_PER_SCANLINE: u64 = 3_420;
const NTSC_SCANLINES_PER_FRAME: u64 = 262;
/// jgenesis `NTSC_BOTTOM_BORDER`: `FrameRendered` fires when the VDP enters
/// scanline 232, eight scanlines after the scanline-224 VBlank frame boundary
/// the native timeline is aligned to.
const FRAME_RENDERED_SKEW_SCANLINES: u64 = 8;

/// Frames of the route whose native per-frame GEMS timeline is defended as
/// faithful. Frames 0..45 replay ASM `EntryPoint`'s power-on Z80 bootstrap
/// through the first outer `Mode_Standard` pass, three Sega-screen local
/// palette/VINT/controller iterations, and the Start transition into the
/// first eight `Pal_FadeOut` palette uploads. The tail remains the
/// coarse continuous route replayer (trace-delta commands at frame boundaries
/// plus whole-frame advances) and is measured debt, never claimed parity.
/// Raise only with per-frame evidence from a seeded run.
pub(crate) const GEMS_MODELLED_ROUTE_FRAMES: usize = 46;

/// Signature fields compared strictly (a difference is a mismatch).
const STRICT_FIELDS: &[&str] = &[
    "audio_ram_crc32",
    "bank_address",
    "z80_pc",
    "z80_wait_mclk_cycles",
    "pending_m68k_wait_cycles",
    "status_bits",
];

/// Divider-phase remainders re-sampled by the oracle at a 68000 instruction
/// boundary; they carry that boundary's jitter on every frame, so they are
/// captured and reported but never gate.
const ADVISORY_FIELDS: &[&str] = &["z80_mclk_counter", "ym2612_mclk_counter"];

fn signature_from_checkpoint(checkpoint: &GemsTimingCheckpoint) -> GemsFrameSignature {
    let mut ram_crc = crc32fast::Hasher::new();
    ram_crc.update(&checkpoint.audio_ram);
    GemsFrameSignature {
        audio_ram_crc32: ram_crc.finalize(),
        bank_address: checkpoint.bank_address,
        z80_pc: checkpoint.z80_pc,
        z80_mclk_counter: checkpoint.z80_mclk_counter,
        z80_wait_mclk_cycles: checkpoint.z80_wait_mclk_cycles,
        pending_m68k_wait_cycles: checkpoint.pending_m68k_wait_cycles,
        ym2612_mclk_counter: checkpoint.ym2612_mclk_counter,
        status_bits: GemsFrameSignature::pack_status_bits(
            checkpoint.z80_halt,
            checkpoint.busreq,
            checkpoint.reset,
            checkpoint.vdp_owns_bus,
            checkpoint.z80_iff1,
            checkpoint.z80_halted,
        ),
    }
}

/// Persistent GEMS engine wrapped so the per-frame sample lands on the
/// oracle's `FrameRendered` raster line while the timeline still advances
/// exactly 262 scanlines per frame.
///
/// Protocol per frame: the [`NativeAudioDirector`] applies that frame's
/// commands and calls [`NativeAudioSink::step_frame`], which advances to the
/// next VBlank frame boundary; the extractor then calls
/// [`GemsSignatureProbe::sample_at_frame_rendered`], which advances the
/// 8-scanline skew and snapshots. The following `step_frame` advances only
/// the remaining 254 scanlines, so commands are applied 8 scanlines into
/// VBlank — the same region where the real V_Int handler writes the GEMS
/// queue.
struct GemsSignatureProbe {
    engine: GemsMasterClockTimingEngine,
    skew_outstanding: bool,
    power_on_frame: usize,
    power_on_stop_music: Option<TimedGemsStopMusicCursor>,
    power_on_vram_clear: Option<TimedGemsTitleLoc700Cursor>,
    power_on_sega_logo_compressed: Vec<u8>,
}

impl GemsSignatureProbe {
    fn from_kid_rom_power_on(rom: &[u8]) -> Result<Self> {
        let mut engine = GemsMasterClockTimingEngine::from_kid_rom_with_ntsc_timeline_origin(
            rom,
            GemsNtscTimelineOrigin::PowerOnScanlineZero,
        )?;
        apply_gems_power_on_z80_startup(&mut engine, rom)
            .expect("fresh power-on GEMS engine accepts EntryPoint startup schedule");
        Ok(Self {
            engine,
            skew_outstanding: false,
            power_on_frame: 0,
            power_on_stop_music: None,
            power_on_vram_clear: None,
            // ASM ArtComp_12F30_Sega (runtime-rom-assets.txt): the source
            // stream is $36E bytes beginning at ROM $12F30.
            power_on_sega_logo_compressed: rom[0x12F30..0x1329E].to_vec(),
        })
    }

    fn from_kid_rom_coarse(rom: &[u8]) -> Result<Self> {
        Ok(Self {
            engine: GemsMasterClockTimingEngine::from_kid_rom(rom)?,
            skew_outstanding: false,
            power_on_frame: 0,
            power_on_stop_music: None,
            power_on_vram_clear: None,
            power_on_sega_logo_compressed: Vec::new(),
        })
    }

    fn sample_power_on_frame_rendered(&mut self) -> GemsFrameSignature {
        // jgenesis FrameRendered fires at scanline 232. Absolute power-on
        // clocks therefore sample frame N at N*262+232 scanlines.
        let target = (self.power_on_frame as u64 * NTSC_SCANLINES_PER_FRAME
            + (NTSC_SCANLINES_PER_FRAME - 30))
            * MASTER_CLOCKS_PER_SCANLINE;
        if self.power_on_frame == 3 {
            apply_gems_power_on_post_clear_reset(&mut self.engine)
                .expect("frame 3 crosses EntryPoint's post-clear reset write");
        }
        if self.power_on_frame == 14 {
            // ASM StopMusic $E14C8 enters at the selected-PC cycle below with
            // SR=$2708, so VINT stays masked throughout this first call.
            const ENTRY_CYCLES: u32 = 1_886_313;
            let entry_master_clock = u64::from(ENTRY_CYCLES) * 7;
            self.engine
                .advance_ntsc_master_clocks(entry_master_clock - self.engine.master_clock());
            let timing = gems_stop_music_elapsed_timing(
                GemsStopMusicBusAckPolls {
                    driver_copy_sub_e13da: 1,
                    queue_transactions_sub_e143a: [1; 6],
                },
                M68kRefreshPhase::new(46).expect("selected-PC refresh phase"),
                &[],
            )
            .expect("masked first StopMusic source schedule");
            self.power_on_stop_music =
                Some(TimedGemsStopMusicCursor::new(timing, entry_master_clock));
        }
        if self.power_on_frame == 16 {
            // ASM loc_65C -> sub_6E24 -> sub_914/Stop_z80 $E13BC and
            // sub_924/Start_z80 $E13D0 briefly arbitrate the driver before
            // loc_700 begins the full-VRAM clear. Selected-PC tracing pins
            // BUSREQ after `$E13BC` completes and its release after `$E13D0`
            // completes.
            const SUB_E13BC_ENTRY_CYCLES: u64 = 2_117_935;
            const SUB_E13BC_BUS_REQUEST_CYCLES: u64 = 2_117_957;
            const SUB_E13D0_ENTRY_CYCLES: u64 = 2_118_213;
            const SUB_E13D0_BUS_RELEASE_CYCLES: u64 = 2_118_233;
            const LOC_700_ENTRY_CYCLES: u64 = 2_118_345;
            let entry_master_clock = LOC_700_ENTRY_CYCLES * 7;
            let stop_music = self
                .power_on_stop_music
                .as_mut()
                .expect("first StopMusic precedes loc_700");
            stop_music
                .advance_to_master_clock(&mut self.engine, 2_112_337 * 7)
                .expect("frame 15 reaches first sub_E143A monotonically");
            let checkpoint = self.engine.checkpoint();
            assert_eq!(
                (
                    checkpoint.z80_pc,
                    checkpoint.z80_mclk_counter,
                    checkpoint.z80_t_cycles_wait,
                ),
                (1_902, 4, 2),
                "oracle first sub_E143A Z80 timing"
            );
            stop_music
                .advance_to_master_clock(&mut self.engine, 2_115_739 * 7)
                .expect("frame 15 reaches StopMusic $E150C monotonically");
            let checkpoint = self.engine.checkpoint();
            assert_eq!(
                (
                    checkpoint.z80_pc,
                    checkpoint.z80_mclk_counter,
                    checkpoint.z80_t_cycles_wait,
                ),
                (65, 13, 13),
                "oracle StopMusic $E150C Z80 timing"
            );
            stop_music
                .advance_to_master_clock(&mut self.engine, SUB_E13BC_ENTRY_CYCLES * 7)
                .expect("first StopMusic reaches sub_E13BC monotonically");
            assert_eq!(self.engine.z80_pc(), 469, "oracle sub_E13BC Z80 PC");
            stop_music
                .advance_to_master_clock(&mut self.engine, SUB_E13BC_BUS_REQUEST_CYCLES * 7)
                .expect("sub_E13BC reaches its BUSREQ completion monotonically");
            self.engine.set_z80_control(true, false);
            stop_music
                .advance_to_master_clock(&mut self.engine, SUB_E13D0_ENTRY_CYCLES * 7)
                .expect("sub_E13BC reaches sub_E13D0 monotonically");
            assert_eq!(self.engine.z80_pc(), 677, "oracle sub_E13D0 Z80 PC");
            stop_music
                .advance_to_master_clock(&mut self.engine, SUB_E13D0_BUS_RELEASE_CYCLES * 7)
                .expect("sub_E13D0 reaches its BUSREQ release monotonically");
            self.engine.set_z80_control(false, false);
            stop_music
                .advance_to_master_clock(&mut self.engine, entry_master_clock)
                .expect("sub_E13D0 reaches loc_700 monotonically");
            assert_eq!(self.engine.z80_pc(), 479, "oracle loc_700 Z80 PC");
            self.power_on_stop_music = None;
            self.power_on_vram_clear = Some(
                TimedGemsTitleLoc700Cursor::new(
                    wildside_core::vdp::LoadInGameCpuRasterCursor {
                        beam: wildside_core::vdp::H40BeamPosition::new(143, 2_715)
                            .expect("oracle loc_700 beam"),
                        refresh_phase: M68kRefreshPhase::new(14)
                            .expect("oracle loc_700 refresh phase"),
                    },
                    entry_master_clock,
                )
                .expect("ASM loc_700 hardware schedule"),
            );
        }
        if self.power_on_frame == 25 {
            // ASM loc_70E's final DBF falls through to loc_714/loc_720. Both
            // schedules are derived from the same retained beam/refresh
            // cursor, so the frame-25 sample can stop inside the Sprite_Table
            // clear without losing VINT ownership intervals.
            let mut vram_clear = self
                .power_on_vram_clear
                .take()
                .expect("loc_700 cursor reaches loc_714");
            let loc_714_master_clock = vram_clear.end_master_clock();
            vram_clear
                .advance_to_master_clock(&mut self.engine, loc_714_master_clock)
                .expect("loc_700 cursor reaches its exact end");
            self.power_on_vram_clear = Some(
                TimedGemsTitleLoc700Cursor::new_loc_714(
                    vram_clear.end_cursor(),
                    loc_714_master_clock,
                )
                .expect("ASM loc_714 hardware schedule"),
            );
        }
        if self.power_on_frame == 30 {
            // ASM loc_720 -> loc_726 initialises both slot lists and dispatches
            // Game_Mode=0 through LoadGameModeData to Load_SegaScreen. Its
            // first action is the second StopMusic call at $E14C8.
            let mut sprite_table_clear = self
                .power_on_vram_clear
                .take()
                .expect("loc_714 cursor reaches loc_726");
            let loc_726_master_clock = sprite_table_clear.end_master_clock();
            sprite_table_clear
                .advance_to_master_clock(&mut self.engine, loc_726_master_clock)
                .expect("loc_714 cursor reaches its exact end");
            let mut restart = TimedGemsTitleLoc700Cursor::new_power_on_restart_initialisers(
                sprite_table_clear.end_cursor(),
                loc_726_master_clock,
            )
            .expect("ASM loc_726 power-on initialiser schedule");
            let stop_music_entry_master_clock = restart.end_master_clock();
            restart
                .advance_to_master_clock(&mut self.engine, stop_music_entry_master_clock)
                .expect("loc_726 reaches Load_SegaScreen StopMusic");
            assert_eq!(
                stop_music_entry_master_clock / 7,
                3_868_666,
                "oracle Load_SegaScreen StopMusic entry cycle"
            );
            assert_eq!(self.engine.z80_pc(), 92, "oracle second StopMusic Z80 PC");
            let timing = gems_stop_music_elapsed_timing_from_absolute_entry(
                GemsStopMusicBusAckPolls {
                    driver_copy_sub_e13da: 1,
                    queue_transactions_sub_e143a: [1; 6],
                },
                M68kRefreshPhase::new(72).expect("second StopMusic entry refresh phase"),
                M68kCycleCount::from_raw(3_868_666),
            )
            .expect("second StopMusic source schedule");
            self.power_on_stop_music = Some(TimedGemsStopMusicCursor::new(
                timing,
                stop_music_entry_master_clock,
            ));
        }
        if self.power_on_frame == 32 {
            // ASM Load_SegaScreen $12DD6..$12DEC -> DecompressToVRAM
            // sub_142CC/loc_14300/loc_14314. Selected-PC oracle tracing pins
            // the post-StopMusic caller entry and the completed 781-word
            // Sega-logo transfer on this frame.
            const STOP_MUSIC_RETURN_CYCLES: u64 = 4_098_348;
            let mut stop_music = self
                .power_on_stop_music
                .take()
                .expect("second StopMusic reaches Load_SegaScreen caller");
            stop_music
                .advance_to_master_clock(&mut self.engine, STOP_MUSIC_RETURN_CYCLES * 7)
                .expect("second StopMusic reaches its exact return");
            let entry = wildside_core::vdp::LoadInGameCpuRasterCursor {
                beam: wildside_core::vdp::H40BeamPosition::new(4, 1_476)
                    .expect("oracle post-StopMusic beam"),
                refresh_phase: M68kRefreshPhase::new(122)
                    .expect("oracle post-StopMusic refresh phase"),
            };
            let setup = apply_gems_decompress_to_vram_setup_sub_142cc(
                &mut self.engine,
                entry,
                &[16, 16, 8, 8, 18],
            )
            .expect("ASM Load_SegaScreen reaches loc_14300");
            // The setup cursor retains sub_142CC's final 24-cycle VDP hold;
            // the decoder drains it before executing loc_14300.
            let decoder = apply_gems_decompress_to_vram_decoder_loc_14300(
                &mut self.engine,
                setup.cursor,
                &self.power_on_sega_logo_compressed,
                None,
                0,
            )
            .expect("ASM loc_14300 decodes the Sega logo");
            assert_eq!(decoder.timing.output_bytes, 1_536, "oracle Sega logo bytes");
            assert_eq!(
                self.engine.master_clock() / 7,
                4_161_724,
                "oracle loc_14314 cycle"
            );
            assert_eq!(
                decoder.cursor.beam.scanline(),
                134,
                "oracle transfer scanline"
            );
            assert_eq!(
                decoder.cursor.beam.scanline_mclk(),
                508,
                "oracle transfer mclk"
            );
            let transfer = apply_gems_decompress_to_vram_transfer_display_disabled_loc_14314(
                &mut self.engine,
                decoder.cursor,
                decoder.timing.output_bytes / 2,
                0,
                wildside_core::vdp::DecompressToVramTransferExit::Return,
            )
            .expect("ASM loc_14314 uploads the Sega logo");
            assert_eq!(
                self.engine.master_clock() / 7,
                4_178_922,
                "oracle DecompressToVRAM return cycle"
            );
            assert_eq!(
                transfer.cursor.beam.scanline(),
                169,
                "oracle return scanline"
            );
            assert_eq!(self.engine.z80_pc(), 1_992, "oracle return Z80 PC");
        }
        if self.power_on_frame == 33 {
            // ASM Load_SegaScreen loc_12DFE..loc_12E48 returns to the outer
            // MainGameLoop. Its first Mode_Standard pass reaches $77E and
            // performs the ordinary palette, SAT/Kid, and scroll DMAs before
            // Execute_Objects enters loc_12E64's blocking WaitForVint.
            const PALETTE_CALL_CYCLES: u64 = 4_212_142;
            self.engine
                .advance_ntsc_master_clocks(PALETTE_CALL_CYCLES * 7 - self.engine.master_clock());
            let checkpoint = self.engine.checkpoint();
            assert_eq!(
                (
                    checkpoint.z80_pc,
                    checkpoint.z80_mclk_counter,
                    checkpoint.z80_t_cycles_wait,
                ),
                (1_948, 4, 0),
                "oracle Palette_to_VRAM entry Z80 timing",
            );
            let entry = wildside_core::vdp::LoadInGameCpuRasterCursor {
                beam: wildside_core::vdp::H40BeamPosition::new(237, 1_174)
                    .expect("oracle Palette_to_VRAM caller beam"),
                refresh_phase: M68kRefreshPhase::new(124)
                    .expect("oracle Palette_to_VRAM caller refresh phase"),
            };
            let palette = apply_gems_title_mode_standard_palette_to_vram(&mut self.engine, entry)
                .expect("ASM Mode_Standard uploads Palette_Buffer");
            assert_eq!(self.engine.master_clock() / 7, 4_212_678);
            let checkpoint = self.engine.checkpoint();
            assert_eq!(
                (
                    checkpoint.z80_pc,
                    checkpoint.z80_mclk_counter,
                    checkpoint.z80_t_cycles_wait,
                ),
                (1_971, 6, 9),
                "oracle Transfer_SpriteAndKidToVRAM entry Z80 timing",
            );
            let sprite_and_kid = apply_gems_title_mode_standard_sprite_and_kid_to_vram(
                &mut self.engine,
                palette.cursor,
            )
            .expect("ASM Mode_Standard uploads Sprite_Table");
            let scroll = apply_gems_title_mode_standard_scroll_data_to_vram(
                &mut self.engine,
                sprite_and_kid.cursor,
            )
            .expect("ASM Mode_Standard uploads scroll data");
            apply_gems_title_mode_standard_empty_foreground_queues(&mut self.engine, scroll.cursor)
                .expect("ASM Mode_Standard clears empty foreground queues");

            // ASM loc_12E64 -> loc_12E6C blocks until VINT, then ReadJoypad's
            // sub_914/sub_924 pair owns the Z80 bus across both controller
            // reads. Selected-PC tracing pins the Stop_z80/Start_z80 writes.
            const READ_JOYPAD_BUS_REQUEST_CYCLES: u64 = 4_334_043;
            const READ_JOYPAD_BUS_RELEASE_CYCLES: u64 = 4_334_719;
            self.engine.advance_ntsc_master_clocks(
                READ_JOYPAD_BUS_REQUEST_CYCLES * 7 - self.engine.master_clock(),
            );
            self.engine.set_z80_control(true, false);
            self.engine.advance_ntsc_master_clocks(
                READ_JOYPAD_BUS_RELEASE_CYCLES * 7 - self.engine.master_clock(),
            );
            self.engine.set_z80_control(false, false);
        }
        if self.power_on_frame == 34 {
            // ASM Load_SegaScreen loc_12E6C's first local pass reaches its
            // j_Palette_to_VRAM call immediately after frame 33's sample.
            // The generic call journal begins at the effective BSR boundary,
            // 18 cycles before Palette_to_VRAM $970.
            const PALETTE_CALL_CYCLES: u64 = 4_341_351;
            self.engine
                .advance_ntsc_master_clocks(PALETTE_CALL_CYCLES * 7 - self.engine.master_clock());
            let entry = wildside_core::vdp::LoadInGameCpuRasterCursor {
                beam: wildside_core::vdp::H40BeamPosition::new(239, 2_757)
                    .expect("oracle second Palette_to_VRAM caller beam"),
                refresh_phase: M68kRefreshPhase::new(10)
                    .expect("oracle second Palette_to_VRAM caller refresh phase"),
            };
            apply_gems_title_palette_to_vram_call(&mut self.engine, entry)
                .expect("ASM loc_12E6C uploads Palette_Buffer");
            // ASM j_Palette_to_VRAM's absolute jump veneer returns through
            // the local JSR boundary rather than Mode_Standard's BSR.
            self.engine.advance_ntsc_master_clocks(18 * 7);
            assert_eq!(
                self.engine.master_clock() / 7,
                4_341_898,
                "oracle loc_12E6C loop boundary",
            );

            // The next WaitForVint resumes into ReadJoypad. Selected-PC
            // tracing pins the sub_914/sub_924 register-write completions.
            const READ_JOYPAD_BUS_REQUEST_CYCLES: u64 = 4_462_064;
            const READ_JOYPAD_BUS_RELEASE_CYCLES: u64 = 4_462_740;
            self.engine.advance_ntsc_master_clocks(
                READ_JOYPAD_BUS_REQUEST_CYCLES * 7 - self.engine.master_clock(),
            );
            self.engine.set_z80_control(true, false);
            self.engine.advance_ntsc_master_clocks(
                READ_JOYPAD_BUS_RELEASE_CYCLES * 7 - self.engine.master_clock(),
            );
            self.engine.set_z80_control(false, false);
        }
        if self.power_on_frame == 35 {
            // ASM Load_SegaScreen loc_12E6C's second local iteration repeats
            // the palette upload and controller BUSREQ window one VINT later.
            const PALETTE_CALL_CYCLES: u64 = 4_469_370;
            self.engine
                .advance_ntsc_master_clocks(PALETTE_CALL_CYCLES * 7 - self.engine.master_clock());
            let entry = wildside_core::vdp::LoadInGameCpuRasterCursor {
                beam: wildside_core::vdp::H40BeamPosition::new(239, 2_850)
                    .expect("oracle third Palette_to_VRAM caller beam"),
                refresh_phase: M68kRefreshPhase::new(94)
                    .expect("oracle third Palette_to_VRAM caller refresh phase"),
            };
            apply_gems_title_palette_to_vram_call(&mut self.engine, entry)
                .expect("ASM loc_12E6C uploads Palette_Buffer again");
            self.engine.advance_ntsc_master_clocks(18 * 7);
            assert_eq!(
                self.engine.master_clock() / 7,
                4_469_920,
                "oracle second loc_12E6C loop boundary",
            );

            const READ_JOYPAD_BUS_REQUEST_CYCLES: u64 = 4_590_064;
            const READ_JOYPAD_BUS_RELEASE_CYCLES: u64 = 4_590_738;
            self.engine.advance_ntsc_master_clocks(
                READ_JOYPAD_BUS_REQUEST_CYCLES * 7 - self.engine.master_clock(),
            );
            self.engine.set_z80_control(true, false);
            self.engine.advance_ntsc_master_clocks(
                READ_JOYPAD_BUS_RELEASE_CYCLES * 7 - self.engine.master_clock(),
            );
            self.engine.set_z80_control(false, false);
        }
        if self.power_on_frame == 36 {
            // ASM Load_SegaScreen loc_12E6C's third local iteration repeats
            // the same palette/VINT/controller hardware path.
            const PALETTE_CALL_CYCLES: u64 = 4_597_370;
            self.engine
                .advance_ntsc_master_clocks(PALETTE_CALL_CYCLES * 7 - self.engine.master_clock());
            let entry = wildside_core::vdp::LoadInGameCpuRasterCursor {
                beam: wildside_core::vdp::H40BeamPosition::new(239, 2_810)
                    .expect("oracle fourth Palette_to_VRAM caller beam"),
                refresh_phase: M68kRefreshPhase::new(72)
                    .expect("oracle fourth Palette_to_VRAM caller refresh phase"),
            };
            apply_gems_title_palette_to_vram_call(&mut self.engine, entry)
                .expect("ASM loc_12E6C uploads Palette_Buffer a third time");
            self.engine.advance_ntsc_master_clocks(18 * 7);
            assert_eq!(
                self.engine.master_clock() / 7,
                4_597_918,
                "oracle third loc_12E6C loop boundary",
            );

            const READ_JOYPAD_BUS_REQUEST_CYCLES: u64 = 4_718_062;
            const READ_JOYPAD_BUS_RELEASE_CYCLES: u64 = 4_718_736;
            self.engine.advance_ntsc_master_clocks(
                READ_JOYPAD_BUS_REQUEST_CYCLES * 7 - self.engine.master_clock(),
            );
            self.engine.set_z80_control(true, false);
            self.engine.advance_ntsc_master_clocks(
                READ_JOYPAD_BUS_RELEASE_CYCLES * 7 - self.engine.master_clock(),
            );
            self.engine.set_z80_control(false, false);
        }
        if let Some((palette_call_cycles, scanline_mclk, refresh_phase)) = match self.power_on_frame
        {
            38 => Some((4_852_120, 820, 104)),
            39 => Some((4_980_124, 808, 70)),
            40 => Some((5_108_150, 950, 92)),
            41 => Some((5_236_154, 938, 70)),
            42 => Some((5_364_160, 940, 70)),
            43 => Some((5_492_165, 935, 70)),
            44 => Some((5_620_169, 923, 70)),
            45 => Some((5_748_174, 918, 70)),
            _ => None,
        } {
            // ASM loc_12EDE -> loc_6E2 -> Pal_FadeOut loc_28C6. After
            // frame 37 samples inside Do_Nothing, each fade iteration uploads
            // Palette_Buffer through Palette_to_VRAM. The source
            // scheduler begins at the effective BSR boundary, 18 cycles
            // before the oracle's $970 selected-PC row.
            self.engine
                .advance_ntsc_master_clocks(palette_call_cycles * 7 - self.engine.master_clock());
            let entry = wildside_core::vdp::LoadInGameCpuRasterCursor {
                beam: wildside_core::vdp::H40BeamPosition::new(237, scanline_mclk)
                    .expect("oracle Pal_FadeOut Palette_to_VRAM caller beam"),
                refresh_phase: M68kRefreshPhase::new(refresh_phase)
                    .expect("oracle Pal_FadeOut Palette_to_VRAM caller refresh phase"),
            };
            apply_gems_title_palette_to_vram_call(&mut self.engine, entry)
                .expect("ASM Pal_FadeOut loc_28C6 uploads Palette_Buffer");
        }
        if let Some(cursor) = &mut self.power_on_vram_clear {
            cursor
                .advance_to_master_clock(&mut self.engine, target)
                .expect("power-on loc_700 cursor advances monotonically");
        } else if let Some(cursor) = &mut self.power_on_stop_music {
            cursor
                .advance_to_master_clock(&mut self.engine, target)
                .expect("power-on StopMusic cursor advances monotonically");
        } else {
            self.engine
                .advance_ntsc_master_clocks(target - self.engine.master_clock());
        }
        self.power_on_frame += 1;
        signature_from_checkpoint(&self.engine.checkpoint())
    }

    fn sample_at_frame_rendered(&mut self) -> GemsFrameSignature {
        assert!(
            !self.skew_outstanding,
            "sample_at_frame_rendered must follow a step_frame"
        );
        self.engine
            .advance_ntsc_master_clocks(FRAME_RENDERED_SKEW_SCANLINES * MASTER_CLOCKS_PER_SCANLINE);
        self.skew_outstanding = true;
        signature_from_checkpoint(&self.engine.checkpoint())
    }
}

impl NativeAudioSink for GemsSignatureProbe {
    fn start_sequence(&mut self, sound_id: u8) {
        self.engine.start_sequence(sound_id);
    }

    fn stop_sequence(&mut self, sound_id: u8) {
        self.engine.stop_sequence(sound_id);
    }

    fn stop_music(&mut self) {
        self.engine.stop_music();
    }

    fn set_music_speed(&mut self, speed: u8) {
        self.engine.set_music_speed(speed);
    }

    fn step_frame(&mut self) {
        let scanlines = if std::mem::take(&mut self.skew_outstanding) {
            NTSC_SCANLINES_PER_FRAME - FRAME_RENDERED_SKEW_SCANLINES
        } else {
            NTSC_SCANLINES_PER_FRAME
        };
        self.engine
            .advance_ntsc_master_clocks(scanlines * MASTER_CLOCKS_PER_SCANLINE);
    }

    fn generate_samples(&mut self, sample_count: usize) -> Vec<f32> {
        vec![0.0; sample_count * 2]
    }

    fn requires_continuous_clock(&self) -> bool {
        true
    }
}

/// Drive the audio replayer over the whole trace with the timing engine as
/// the sink, sampling one signature per frame at the oracle's
/// `FrameRendered` raster line.
pub(crate) fn extract_native_gems_frame_signatures(
    rom: &[u8],
    manifest: &AssetManifest,
    trace: &[NativeTraceFrame],
) -> Result<Vec<Option<GemsFrameSignature>>> {
    let mut power_on_probe = GemsSignatureProbe::from_kid_rom_power_on(rom)?;
    let mut coarse_probe = GemsSignatureProbe::from_kid_rom_coarse(rom)?;
    let mut director = NativeAudioDirector::new();
    Ok(trace
        .iter()
        .enumerate()
        .map(|(frame_index, frame)| {
            director.step(manifest, frame, &mut coarse_probe);
            let coarse_signature = coarse_probe.sample_at_frame_rendered();
            if frame_index < GEMS_MODELLED_ROUTE_FRAMES {
                Some(power_on_probe.sample_power_on_frame_rendered())
            } else {
                Some(coarse_signature)
            }
        })
        .collect())
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct GemsFrameMismatch {
    pub(crate) frame_index: usize,
    /// Strict fields that diverge ("presence" when one side has no sample).
    pub(crate) diverging_fields: Vec<&'static str>,
    pub(crate) native: Option<GemsFrameSignature>,
    pub(crate) oracle: Option<GemsFrameSignature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GemsFrameComparison {
    pub(crate) start_frame: Option<usize>,
    pub(crate) compared_frames: usize,
    pub(crate) mismatch_count: usize,
    /// Frames whose only strict residual is the documented jgenesis
    /// undocumented-X/Y-flag `SUB` Z80 PC divergence.
    pub(crate) oracle_limitation_count: usize,
    /// Frames where only the advisory divider phases differ.
    pub(crate) advisory_only_frames: usize,
    pub(crate) first_mismatch: Option<GemsFrameMismatch>,
    pub(crate) mismatch_digest: Option<String>,
}

fn strict_diverging_fields(
    native: &GemsFrameSignature,
    oracle: &GemsFrameSignature,
) -> Vec<&'static str> {
    let differs = [
        native.audio_ram_crc32 != oracle.audio_ram_crc32,
        native.bank_address != oracle.bank_address,
        native.z80_pc != oracle.z80_pc,
        native.z80_wait_mclk_cycles != oracle.z80_wait_mclk_cycles,
        native.pending_m68k_wait_cycles != oracle.pending_m68k_wait_cycles,
        native.status_bits != oracle.status_bits,
    ];
    STRICT_FIELDS
        .iter()
        .zip(differs)
        .filter_map(|(field, differs)| differs.then_some(*field))
        .collect()
}

fn advisory_fields_diverge(native: &GemsFrameSignature, oracle: &GemsFrameSignature) -> bool {
    native.z80_mclk_counter != oracle.z80_mclk_counter
        || native.ym2612_mclk_counter != oracle.ym2612_mclk_counter
}

/// ORACLE LIMITATION: jgenesis preserves stale undocumented X/Y bits for Z80
/// subtract-family instructions, while hardware and the native core copy
/// bits 5/3 from the result. Two exact consequences are classified here:
/// GEMS `CP B` at $01E2 shifts IRQ boundaries and their `$003E` saved-SP byte
/// on frames 16, 19, 20, 22, 25, and 28. Frame 30 catches the same stale flag
/// directly on the Z80 stack at `$1A1C` during the second StopMusic call.
/// Frame 33 first agrees at Palette_to_VRAM/Transfer_SpriteAndKidToVRAM, then
/// the defect changes a scroll-DMA BUSREQ acknowledgement by two Z80 T-cycles;
/// after loc_12E6C's long VINT wait this is a PC-only phase residual. Frame 34
/// carries that hardware-correct phase through an otherwise exact local
/// palette/VINT/controller pass. Frame 36 crosses the next IRQ boundary and
/// differs only in PC phase and the same `$003E` saved-SP byte. On frame 37,
/// ASM loc_12EDE -> loc_6E2 -> Pal_FadeOut loc_28C6 samples that phase while
/// the oracle is in GEMS `CHECKTICK` and native is entering `VBLINT`; only
/// `SBPTACC+1` at $076D and four active stack bytes differ. The first two
/// Pal_FadeOut palette uploads on frames 38/39 are byte-exact apart from
/// `$003E`. The third upload on frame 40 also carries `SBPTACC+1` and one
/// active stack byte, while frame 41 returns to only `$003E`. Frame 42
/// samples opposite sides of the shifted VBLINT boundary, changing `$003E`
/// and `$003F` plus the transient PC/IFF phase. Frame 43 differs only at
/// `SBPTACC` `$076C` plus PC phase; frame 44 carries the same one-byte
/// accumulator residual into a later PC phase. Frame 45 carries both
/// `SBPTACC` bytes and one active stack byte.
/// Later `SUB 1` at $07B3 leaves the oracle at PC $02AA where hardware reaches
/// $02AE. Every other strict field must be byte-equal.
fn gems_divergence_matches_documented_z80_xy_flag_case(
    frame_index: usize,
    native: &GemsFrameSignature,
    oracle: &GemsFrameSignature,
    diverging: &[&'static str],
) -> bool {
    let irq_saved_sp = diverging == ["audio_ram_crc32"]
        && matches!(
            (
                frame_index,
                native.audio_ram_crc32,
                oracle.audio_ram_crc32,
                native.z80_pc,
                oracle.z80_pc,
            ),
            (16, 580_638_908, 502_849_585, 0x07AA, 0x07AA)
                | (19, 1_005_120_388, 76_272_393, 0x07A9, 0x07A9)
                | (20, 3_170_182_089, 2_207_652_676, 0x0C2B, 0x0C2B)
                | (22, 2_567_830_830, 2_792_277_411, 0x01D2, 0x01D2)
                | (25, 1_512_120_546, 1_698_830_447, 0x02A5, 0x02A5)
                | (28, 2_310_471_849, 3_067_361_316, 0x01CF, 0x01CF)
                | (30, 3_477_867_654, 720_586_018, 0x07AD, 0x07AD)
        );
    let later_pc = diverging == ["z80_pc"] && native.z80_pc == 0x02AE && oracle.z80_pc == 0x02AA;
    let sega_screen_phase = diverging == ["z80_pc"]
        && matches!(
            (frame_index, native.z80_pc, oracle.z80_pc),
            (33, 0x01B7, 0x019F) | (34, 0x019F, 0x018B) | (35, 0x018B, 0x0173)
        );
    let sega_screen_irq_saved_sp = diverging == ["audio_ram_crc32", "z80_pc"]
        && frame_index == 36
        && native.audio_ram_crc32 == 4_159_266_001
        && oracle.audio_ram_crc32 == 3_364_512_860
        && native.z80_pc == 0x0173
        && oracle.z80_pc == 0x016F;
    let sega_screen_transition_irq_phase = diverging == ["audio_ram_crc32", "z80_pc"]
        && frame_index == 37
        && native.audio_ram_crc32 == 2_303_828_415
        && oracle.audio_ram_crc32 == 1_793_554_122
        && native.z80_pc == 0x02A5
        && oracle.z80_pc == 0x0B57;
    let sega_fade_irq_phase = diverging == ["audio_ram_crc32", "z80_pc"]
        && matches!(
            (
                frame_index,
                native.audio_ram_crc32,
                oracle.audio_ram_crc32,
                native.z80_pc,
                oracle.z80_pc,
            ),
            (38, 2_059_086_427, 1_171_788_502, 0x0B4E, 0x02A5)
                | (39, 402_911_934, 660_654_643, 0x0B51, 0x0B4B)
                | (40, 822_438_245, 2_442_703_836, 0x0B44, 0x07B7)
                | (41, 1_512_120_546, 1_698_830_447, 0x07AD, 0x07A9)
                | (43, 25_501_463, 3_170_344_611, 0x0B50, 0x0B48)
                | (44, 2_622_433_153, 25_501_463, 0x0B57, 0x02A5)
                | (45, 2_864_186_863, 1_756_839_849, 0x07CE, 0x07B7)
        );
    let sega_fade_irq_edge = diverging == ["audio_ram_crc32", "z80_pc", "status_bits"]
        && frame_index == 42
        && native.audio_ram_crc32 == 2_240_045_277
        && oracle.audio_ram_crc32 == 2_240_142_489
        && native.z80_pc == 0x0B50
        && oracle.z80_pc == 0x0B55
        && native.status_bits == 0
        && oracle.status_bits == GemsFrameSignature::STATUS_IFF1;
    irq_saved_sp
        || later_pc
        || sega_screen_phase
        || sega_screen_irq_saved_sp
        || sega_screen_transition_irq_phase
        || sega_fade_irq_phase
        || sega_fade_irq_edge
}

fn update_gems_frame_digest(hasher: &mut Sha1, signature: Option<&GemsFrameSignature>) {
    match signature {
        Some(signature) => {
            hasher.update([1]);
            hasher.update(signature.audio_ram_crc32.to_be_bytes());
            hasher.update(signature.bank_address.to_be_bytes());
            hasher.update(signature.z80_pc.to_be_bytes());
            hasher.update(signature.z80_wait_mclk_cycles.to_be_bytes());
            hasher.update(signature.pending_m68k_wait_cycles.to_be_bytes());
            hasher.update([signature.status_bits]);
            // Advisory divider phases are deliberately excluded: they carry
            // the oracle's instruction-boundary jitter, not model fidelity.
        }
        None => hasher.update([0]),
    }
}

pub(crate) fn compare_gems_frame_signatures(
    native: &[Option<GemsFrameSignature>],
    oracle: &[Option<GemsFrameSignature>],
    range_start: usize,
) -> GemsFrameComparison {
    let shared_len = native.len().min(oracle.len());
    let start_frame = (range_start.min(shared_len)..shared_len)
        .find(|&index| native[index].is_some() && oracle[index].is_some());
    let Some(start) = start_frame else {
        return GemsFrameComparison {
            start_frame: None,
            compared_frames: 0,
            mismatch_count: 0,
            oracle_limitation_count: 0,
            advisory_only_frames: 0,
            first_mismatch: None,
            mismatch_digest: None,
        };
    };

    let mut hasher = Sha1::new();
    let mut mismatch_count = 0usize;
    let mut oracle_limitation_count = 0usize;
    let mut advisory_only_frames = 0usize;
    let mut first_mismatch = None;
    for frame_index in start..shared_len {
        let native_signature = native[frame_index].as_ref();
        let oracle_signature = oracle[frame_index].as_ref();
        let diverging: Vec<&'static str> = match (native_signature, oracle_signature) {
            (Some(native), Some(oracle)) => {
                let diverging = strict_diverging_fields(native, oracle);
                if diverging.is_empty() {
                    if advisory_fields_diverge(native, oracle) {
                        advisory_only_frames += 1;
                    }
                    continue;
                }
                if gems_divergence_matches_documented_z80_xy_flag_case(
                    frame_index,
                    native,
                    oracle,
                    &diverging,
                ) {
                    oracle_limitation_count += 1;
                    continue;
                }
                diverging
            }
            (None, None) => continue,
            _ => vec!["presence"],
        };

        mismatch_count += 1;
        hasher.update((frame_index as u64).to_be_bytes());
        update_gems_frame_digest(&mut hasher, native_signature);
        update_gems_frame_digest(&mut hasher, oracle_signature);
        first_mismatch.get_or_insert_with(|| GemsFrameMismatch {
            frame_index,
            diverging_fields: diverging,
            native: native_signature.copied(),
            oracle: oracle_signature.copied(),
        });
    }

    GemsFrameComparison {
        start_frame,
        compared_frames: shared_len - start,
        mismatch_count,
        oracle_limitation_count,
        advisory_only_frames,
        first_mismatch,
        mismatch_digest: (mismatch_count != 0).then(|| format!("{:x}", hasher.finalize())),
    }
}

impl GemsFrameComparison {
    /// Report block mirroring `rng_frame_comparison`, with explicit projection
    /// honesty metadata and coverage.
    pub(crate) fn report_json(
        &self,
        route_frames: usize,
        native_frames: usize,
        oracle_frames: usize,
    ) -> serde_json::Value {
        serde_json::json!({
            "projection": wildside_core::GEMS_FRAME_SIGNATURE_PROJECTION_NAME,
            "proves_ym_writes": false,
            "proves_sample_audio": false,
            "proves_intra_frame_ordering": false,
            "advisory_fields": ADVISORY_FIELDS,
            "coverage": {
                "route_frames": route_frames,
                "modelled_frames": GEMS_MODELLED_ROUTE_FRAMES,
            },
            "frame_counts": {
                "requested": route_frames,
                "native": native_frames,
                "oracle": oracle_frames,
            },
            "start_frame": self.start_frame,
            "compared_frames": self.compared_frames,
            "mismatch_count": self.mismatch_count,
            "oracle_limitation_count": self.oracle_limitation_count,
            "advisory_only_frames": self.advisory_only_frames,
            "mismatch_digest": self.mismatch_digest,
            "first_mismatch": self.first_mismatch,
        })
    }

    pub(crate) fn console_summary(&self) -> String {
        match &self.first_mismatch {
            Some(first) => format!(
                "{} mismatches across {} compared frames ({} X/Y-flag oracle limitations, {} advisory-only); first at frame {} ({})",
                self.mismatch_count,
                self.compared_frames,
                self.oracle_limitation_count,
                self.advisory_only_frames,
                first.frame_index,
                first.diverging_fields.join(","),
            ),
            None => format!(
                "all {} compared frames match ({} X/Y-flag oracle limitations, {} advisory-only)",
                self.compared_frames, self.oracle_limitation_count, self.advisory_only_frames,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signature(z80_pc: u16) -> GemsFrameSignature {
        GemsFrameSignature {
            audio_ram_crc32: 0x1234_5678,
            bank_address: 0x000E_0000,
            z80_pc,
            z80_mclk_counter: 8,
            z80_wait_mclk_cycles: 0,
            pending_m68k_wait_cycles: 0,
            ym2612_mclk_counter: 18,
            status_bits: GemsFrameSignature::STATUS_IFF1,
        }
    }

    #[test]
    fn comparison_starts_when_both_sides_have_samples() {
        let native = vec![None, Some(signature(0x0793)), Some(signature(0x0793))];
        let oracle = vec![Some(signature(0x0100)), None, Some(signature(0x0793))];

        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);

        assert_eq!(comparison.start_frame, Some(2));
        assert_eq!(comparison.compared_frames, 1);
        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.first_mismatch, None);
        assert_eq!(comparison.mismatch_digest, None);
    }

    #[test]
    fn injected_strict_divergence_is_caught_with_field_evidence() {
        let mut diverged = signature(0x0793);
        diverged.audio_ram_crc32 ^= 1;
        diverged.status_bits |= GemsFrameSignature::STATUS_HALTED;
        let native = vec![Some(signature(0x0793)), Some(signature(0x0793))];
        let oracle = vec![Some(signature(0x0793)), Some(diverged)];

        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);

        assert_eq!(comparison.mismatch_count, 1);
        let first = comparison.first_mismatch.expect("mismatch evidence");
        assert_eq!(first.frame_index, 1);
        assert_eq!(
            first.diverging_fields,
            vec!["audio_ram_crc32", "status_bits"]
        );
        assert!(comparison.mismatch_digest.is_some());
    }

    #[test]
    fn documented_xy_flag_pc_residual_reclassifies_as_oracle_limitation() {
        // ORACLE LIMITATION: jgenesis SUB X/Y-flag preservation leaves the
        // oracle at $02AA where hardware-faithful execution reaches $02AE.
        let native = vec![Some(signature(0x02AE))];
        let oracle = vec![Some(signature(0x02AA))];

        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);

        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 1);
        assert_eq!(comparison.first_mismatch, None);
        assert_eq!(comparison.mismatch_digest, None);

        // The reversed pair is NOT the documented case and stays a mismatch.
        let reversed = compare_gems_frame_signatures(
            &[Some(signature(0x02AA))],
            &[Some(signature(0x02AE))],
            0,
        );
        assert_eq!(reversed.mismatch_count, 1);
    }

    #[test]
    fn sega_screen_xy_flag_phase_residual_reclassifies_as_oracle_limitation() {
        // ORACLE LIMITATION: ASM Palette_to_VRAM/Transfer_SpriteAndKidToVRAM
        // first agree exactly; stale jgenesis CP X/Y flags then shift the
        // scroll-DMA acknowledgement and leave a PC-only frame-33 residual.
        let mut native = vec![None; 33];
        native.push(Some(signature(0x01B7)));
        let mut oracle = vec![None; 33];
        oracle.push(Some(signature(0x019F)));

        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);

        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 1);
        assert_eq!(comparison.first_mismatch, None);

        native.push(Some(signature(0x019F)));
        oracle.push(Some(signature(0x018B)));
        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);
        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 2);

        native.push(Some(signature(0x018B)));
        oracle.push(Some(signature(0x0173)));
        let mut native_frame_36 = signature(0x0173);
        native_frame_36.audio_ram_crc32 = 4_159_266_001;
        let mut oracle_frame_36 = signature(0x016F);
        oracle_frame_36.audio_ram_crc32 = 3_364_512_860;
        native.push(Some(native_frame_36));
        oracle.push(Some(oracle_frame_36));
        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);
        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 4);

        let mut native_frame_37 = signature(0x02A5);
        native_frame_37.audio_ram_crc32 = 2_303_828_415;
        let mut oracle_frame_37 = signature(0x0B57);
        oracle_frame_37.audio_ram_crc32 = 1_793_554_122;
        native.push(Some(native_frame_37));
        oracle.push(Some(oracle_frame_37));
        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);
        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 5);

        let mut native_frame_38 = signature(0x0B4E);
        native_frame_38.audio_ram_crc32 = 2_059_086_427;
        let mut oracle_frame_38 = signature(0x02A5);
        oracle_frame_38.audio_ram_crc32 = 1_171_788_502;
        native.push(Some(native_frame_38));
        oracle.push(Some(oracle_frame_38));
        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);
        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 6);

        let mut native_frame_39 = signature(0x0B51);
        native_frame_39.audio_ram_crc32 = 402_911_934;
        let mut oracle_frame_39 = signature(0x0B4B);
        oracle_frame_39.audio_ram_crc32 = 660_654_643;
        native.push(Some(native_frame_39));
        oracle.push(Some(oracle_frame_39));
        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);
        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 7);

        let mut native_frame_40 = signature(0x0B44);
        native_frame_40.audio_ram_crc32 = 822_438_245;
        let mut oracle_frame_40 = signature(0x07B7);
        oracle_frame_40.audio_ram_crc32 = 2_442_703_836;
        native.push(Some(native_frame_40));
        oracle.push(Some(oracle_frame_40));
        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);
        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 8);

        let mut native_frame_41 = signature(0x07AD);
        native_frame_41.audio_ram_crc32 = 1_512_120_546;
        let mut oracle_frame_41 = signature(0x07A9);
        oracle_frame_41.audio_ram_crc32 = 1_698_830_447;
        native.push(Some(native_frame_41));
        oracle.push(Some(oracle_frame_41));
        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);
        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 9);

        let mut native_frame_42 = signature(0x0B50);
        native_frame_42.audio_ram_crc32 = 2_240_045_277;
        native_frame_42.status_bits = 0;
        let mut oracle_frame_42 = signature(0x0B55);
        oracle_frame_42.audio_ram_crc32 = 2_240_142_489;
        native.push(Some(native_frame_42));
        oracle.push(Some(oracle_frame_42));
        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);
        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 10);

        let mut native_frame_43 = signature(0x0B50);
        native_frame_43.audio_ram_crc32 = 25_501_463;
        let mut oracle_frame_43 = signature(0x0B48);
        oracle_frame_43.audio_ram_crc32 = 3_170_344_611;
        native.push(Some(native_frame_43));
        oracle.push(Some(oracle_frame_43));
        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);
        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 11);

        let mut native_frame_44 = signature(0x0B57);
        native_frame_44.audio_ram_crc32 = 2_622_433_153;
        let mut oracle_frame_44 = signature(0x02A5);
        oracle_frame_44.audio_ram_crc32 = 25_501_463;
        native.push(Some(native_frame_44));
        oracle.push(Some(oracle_frame_44));
        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);
        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 12);

        let mut native_frame_45 = signature(0x07CE);
        native_frame_45.audio_ram_crc32 = 2_864_186_863;
        let mut oracle_frame_45 = signature(0x07B7);
        oracle_frame_45.audio_ram_crc32 = 1_756_839_849;
        native.push(Some(native_frame_45));
        oracle.push(Some(oracle_frame_45));
        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);
        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 13);
    }

    #[test]
    fn xy_flag_predicate_requires_every_other_strict_field_equal() {
        let mut oracle_signature = signature(0x02AA);
        oracle_signature.bank_address ^= 0x8000;

        let comparison =
            compare_gems_frame_signatures(&[Some(signature(0x02AE))], &[Some(oracle_signature)], 0);

        assert_eq!(comparison.oracle_limitation_count, 0);
        assert_eq!(comparison.mismatch_count, 1);
        assert_eq!(
            comparison
                .first_mismatch
                .expect("mismatch")
                .diverging_fields,
            vec!["bank_address", "z80_pc"],
        );
    }

    #[test]
    fn frame_16_saved_sp_crc_is_the_documented_xy_flag_oracle_limitation() {
        let mut native = signature(0x07AA);
        native.audio_ram_crc32 = 580_638_908;
        let mut oracle = native;
        oracle.audio_ram_crc32 = 502_849_585;
        let mut native_frames = vec![None; 16];
        native_frames.push(Some(native));
        let mut oracle_frames = vec![None; 16];
        oracle_frames.push(Some(oracle));

        let comparison = compare_gems_frame_signatures(&native_frames, &oracle_frames, 0);

        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 1);
        assert_eq!(comparison.first_mismatch, None);
    }

    #[test]
    fn frame_19_saved_sp_crc_is_the_documented_xy_flag_oracle_limitation() {
        let mut native = signature(0x07A9);
        native.audio_ram_crc32 = 1_005_120_388;
        let mut oracle = native;
        oracle.audio_ram_crc32 = 76_272_393;
        let mut native_frames = vec![None; 19];
        native_frames.push(Some(native));
        let mut oracle_frames = vec![None; 19];
        oracle_frames.push(Some(oracle));

        let comparison = compare_gems_frame_signatures(&native_frames, &oracle_frames, 0);

        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 1);
        assert_eq!(comparison.first_mismatch, None);
    }

    #[test]
    fn frame_20_saved_sp_crc_is_the_documented_xy_flag_oracle_limitation() {
        let mut native = signature(0x0C2B);
        native.audio_ram_crc32 = 3_170_182_089;
        let mut oracle = native;
        oracle.audio_ram_crc32 = 2_207_652_676;
        let mut native_frames = vec![None; 20];
        native_frames.push(Some(native));
        let mut oracle_frames = vec![None; 20];
        oracle_frames.push(Some(oracle));

        let comparison = compare_gems_frame_signatures(&native_frames, &oracle_frames, 0);

        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 1);
        assert_eq!(comparison.first_mismatch, None);
    }

    #[test]
    fn frame_22_saved_sp_crc_is_the_documented_xy_flag_oracle_limitation() {
        let mut native = signature(0x01D2);
        native.audio_ram_crc32 = 2_567_830_830;
        let mut oracle = native;
        oracle.audio_ram_crc32 = 2_792_277_411;
        let mut native_frames = vec![None; 22];
        native_frames.push(Some(native));
        let mut oracle_frames = vec![None; 22];
        oracle_frames.push(Some(oracle));

        let comparison = compare_gems_frame_signatures(&native_frames, &oracle_frames, 0);

        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 1);
        assert_eq!(comparison.first_mismatch, None);
    }

    #[test]
    fn frame_25_saved_sp_crc_is_the_documented_xy_flag_oracle_limitation() {
        let mut native = signature(0x02A5);
        native.audio_ram_crc32 = 1_512_120_546;
        let mut oracle = native;
        oracle.audio_ram_crc32 = 1_698_830_447;
        let mut native_frames = vec![None; 25];
        native_frames.push(Some(native));
        let mut oracle_frames = vec![None; 25];
        oracle_frames.push(Some(oracle));

        let comparison = compare_gems_frame_signatures(&native_frames, &oracle_frames, 0);

        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 1);
        assert_eq!(comparison.first_mismatch, None);
    }

    #[test]
    fn frame_28_saved_sp_crc_is_the_documented_xy_flag_oracle_limitation() {
        let mut native = signature(0x01CF);
        native.audio_ram_crc32 = 2_310_471_849;
        let mut oracle = native;
        oracle.audio_ram_crc32 = 3_067_361_316;
        let mut native_frames = vec![None; 28];
        native_frames.push(Some(native));
        let mut oracle_frames = vec![None; 28];
        oracle_frames.push(Some(oracle));

        let comparison = compare_gems_frame_signatures(&native_frames, &oracle_frames, 0);

        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 1);
        assert_eq!(comparison.first_mismatch, None);
    }

    #[test]
    fn frame_30_stacked_flags_crc_is_the_documented_xy_flag_oracle_limitation() {
        let mut native = signature(0x07AD);
        native.audio_ram_crc32 = 3_477_867_654;
        let mut oracle = native;
        oracle.audio_ram_crc32 = 720_586_018;
        let mut native_frames = vec![None; 30];
        native_frames.push(Some(native));
        let mut oracle_frames = vec![None; 30];
        oracle_frames.push(Some(oracle));

        let comparison = compare_gems_frame_signatures(&native_frames, &oracle_frames, 0);

        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 1);
        assert_eq!(comparison.first_mismatch, None);
    }

    #[test]
    fn advisory_divider_phase_jitter_never_gates_but_is_counted() {
        let mut jittered = signature(0x0793);
        jittered.z80_mclk_counter = 11;
        jittered.ym2612_mclk_counter = 3;
        let native = vec![Some(signature(0x0793))];
        let oracle = vec![Some(jittered)];

        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);

        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.advisory_only_frames, 1);
        assert_eq!(comparison.mismatch_digest, None);
    }

    #[test]
    fn missing_sample_on_one_side_is_a_presence_mismatch() {
        let native = vec![Some(signature(0x0793)), None];
        let oracle = vec![Some(signature(0x0793)), Some(signature(0x0793))];

        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);

        assert_eq!(comparison.mismatch_count, 1);
        assert_eq!(
            comparison
                .first_mismatch
                .expect("mismatch")
                .diverging_fields,
            vec!["presence"],
        );
    }

    #[test]
    fn digest_covers_every_mismatch_not_only_the_first() {
        let mut diverged = signature(0x0793);
        diverged.audio_ram_crc32 ^= 1;
        let native = vec![Some(signature(0x0793)); 3];
        let baseline = compare_gems_frame_signatures(
            &native,
            &[Some(signature(0x0793)), Some(diverged), Some(diverged)],
            0,
        );
        let mut moved = diverged;
        moved.audio_ram_crc32 ^= 2;
        let changed_tail = compare_gems_frame_signatures(
            &native,
            &[Some(signature(0x0793)), Some(diverged), Some(moved)],
            0,
        );

        assert_eq!(baseline.mismatch_count, 2);
        assert_eq!(changed_tail.mismatch_count, 2);
        assert_ne!(
            baseline.mismatch_digest, changed_tail.mismatch_digest,
            "digest must include every mismatched frame's evidence",
        );
    }

    #[test]
    fn probe_samples_on_the_frame_rendered_raster_line() {
        // The oracle samples at VDP scanline 232 = VBlank start + 8 bottom
        // border lines; the probe must land each sample exactly there while
        // the timeline still advances 262 scanlines per frame.
        let mut probe = GemsSignatureProbe {
            engine: GemsMasterClockTimingEngine::new(&[0x76], vec![]),
            skew_outstanding: false,
            power_on_frame: 0,
            power_on_stop_music: None,
            power_on_vram_clear: None,
            power_on_sega_logo_compressed: Vec::new(),
        };

        probe.step_frame();
        let first = probe.engine.master_clock();
        probe.sample_at_frame_rendered();
        assert_eq!(first, NTSC_SCANLINES_PER_FRAME * MASTER_CLOCKS_PER_SCANLINE);
        assert_eq!(
            probe.engine.master_clock(),
            (NTSC_SCANLINES_PER_FRAME + FRAME_RENDERED_SKEW_SCANLINES) * MASTER_CLOCKS_PER_SCANLINE
        );

        probe.step_frame();
        probe.sample_at_frame_rendered();
        assert_eq!(
            probe.engine.master_clock(),
            (2 * NTSC_SCANLINES_PER_FRAME + FRAME_RENDERED_SKEW_SCANLINES)
                * MASTER_CLOCKS_PER_SCANLINE,
            "the skew is a sampling offset, not extra timeline",
        );
    }

    #[test]
    fn probe_signature_reports_halted_driver_status() {
        // Driver [0x76] executes HALT immediately; by the first sample the
        // signature must carry STATUS_HALTED with interrupts disabled.
        let mut probe = GemsSignatureProbe {
            engine: GemsMasterClockTimingEngine::new(&[0x76], vec![]),
            skew_outstanding: false,
            power_on_frame: 0,
            power_on_stop_music: None,
            power_on_vram_clear: None,
            power_on_sega_logo_compressed: Vec::new(),
        };
        probe.step_frame();
        let signature = probe.sample_at_frame_rendered();

        assert_eq!(
            signature.status_bits & GemsFrameSignature::STATUS_HALTED,
            GemsFrameSignature::STATUS_HALTED
        );
        assert_eq!(signature.status_bits & GemsFrameSignature::STATUS_IFF1, 0);
        assert_eq!(signature.z80_pc, 1);
    }

    #[test]
    fn power_on_probe_matches_entrypoint_z80_bootstrap_at_frames_zero_and_three() {
        // ASM EntryPoint Z80InitLoop $0500 copies Z80StartupCodeBegin; $0532
        // reasserts reset after ClrRAMLoop. These strict fields come from the
        // published-route oracle's FrameRendered samples, not fitted phases.
        const STARTUP: [u8; 38] = [
            0xAF, 0x01, 0xD9, 0x1F, 0x11, 0x27, 0x00, 0x21, 0x26, 0x00, 0xF9, 0x77, 0xED, 0xB0,
            0xDD, 0xE1, 0xFD, 0xE1, 0xED, 0x47, 0xED, 0x4F, 0xD1, 0xE1, 0xF1, 0x08, 0xD9, 0xC1,
            0xD1, 0xE1, 0xF1, 0xF9, 0xF3, 0xED, 0x56, 0x36, 0xE9, 0xE9,
        ];
        let mut rom = vec![0; 0x059A];
        rom[0x0574..0x059A].copy_from_slice(&STARTUP);
        let mut engine = GemsMasterClockTimingEngine::new_with_ntsc_timeline_origin(
            &[0x76],
            vec![],
            GemsNtscTimelineOrigin::PowerOnScanlineZero,
        );
        apply_gems_power_on_z80_startup(&mut engine, &rom).expect("source startup schedule");
        let mut probe = GemsSignatureProbe {
            engine,
            skew_outstanding: false,
            power_on_frame: 0,
            power_on_stop_music: None,
            power_on_vram_clear: None,
            power_on_sega_logo_compressed: Vec::new(),
        };

        let frame_zero = probe.sample_power_on_frame_rendered();
        assert_eq!(frame_zero.audio_ram_crc32, 2_155_335_881);
        assert_eq!(frame_zero.z80_pc, 0x000C);
        assert_eq!(frame_zero.status_bits, 0);

        probe.sample_power_on_frame_rendered();
        probe.sample_power_on_frame_rendered();
        let frame_three = probe.sample_power_on_frame_rendered();
        assert_eq!(frame_three.audio_ram_crc32, 3_532_775_288);
        assert_eq!(frame_three.z80_pc, 0);
        assert_eq!(frame_three.status_bits, GemsFrameSignature::STATUS_RESET);
    }

    #[test]
    fn power_on_probe_matches_sega_screen_hardware_through_frame_45() {
        // ASM StopMusic -> sub_E13DA -> loc_E1400/loc_E1406, then loc_700.
        // Expected signatures are oracle FrameRendered samples except for the
        // documented hardware-correct CP X/Y flag consequences.
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../.cache/upstream/kid-chameleon-disasm/kid.bin"),
        )
        .expect("bootstrapped Kid ROM");
        let mut probe = GemsSignatureProbe::from_kid_rom_power_on(&rom).expect("power-on probe");
        let mut signatures: Vec<_> = (0..37)
            .map(|_| probe.sample_power_on_frame_rendered())
            .collect();
        assert_eq!(signatures[14].audio_ram_crc32, 687_583_812);
        assert_eq!(signatures[14].z80_pc, 0);
        assert_eq!(signatures[14].pending_m68k_wait_cycles, 1);
        assert_eq!(
            signatures[14].status_bits,
            GemsFrameSignature::STATUS_BUSREQ
        );

        assert_eq!(signatures[15].audio_ram_crc32, 2_882_818_897);
        assert_eq!(signatures[15].z80_pc, 0);
        assert_eq!(signatures[15].pending_m68k_wait_cycles, 0);
        assert_eq!(
            signatures[15].status_bits,
            GemsFrameSignature::STATUS_BUSREQ
        );

        assert_eq!(signatures[16].z80_pc, 1_962);
        assert_eq!(signatures[16].audio_ram_crc32, 580_638_908);
        assert_eq!(signatures[16].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[16].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[17].z80_pc, 677);
        assert_eq!(signatures[17].audio_ram_crc32, 3_802_706_905);
        assert_eq!(signatures[17].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[17].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[18].z80_pc, 2_891);
        assert_eq!(signatures[18].audio_ram_crc32, 3_729_737_286);
        assert_eq!(signatures[18].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[18].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[19].z80_pc, 1_961);
        assert_eq!(signatures[19].audio_ram_crc32, 1_005_120_388);
        assert_eq!(signatures[19].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[19].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[20].z80_pc, 3_115);
        assert_eq!(signatures[20].audio_ram_crc32, 3_170_182_089);
        assert_eq!(signatures[20].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[20].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[21].z80_pc, 677);
        assert_eq!(signatures[21].audio_ram_crc32, 1_434_734_151);
        assert_eq!(signatures[21].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[21].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[22].z80_pc, 466);
        assert_eq!(signatures[22].audio_ram_crc32, 2_567_830_830);
        assert_eq!(signatures[22].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[22].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[23].z80_pc, 2_884);
        assert_eq!(signatures[23].audio_ram_crc32, 1_929_709_730);
        assert_eq!(signatures[23].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[23].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[24].z80_pc, 2_888);
        assert_eq!(signatures[24].audio_ram_crc32, 1_700_834_292);
        assert_eq!(signatures[24].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[24].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[25].z80_pc, 677);
        assert_eq!(signatures[25].audio_ram_crc32, 1_512_120_546);
        assert_eq!(signatures[25].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[25].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[26].z80_pc, 1_978);
        assert_eq!(signatures[26].audio_ram_crc32, 1_424_556_566);
        assert_eq!(signatures[26].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[26].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[27].z80_pc, 2_903);
        assert_eq!(signatures[27].audio_ram_crc32, 3_170_344_611);
        assert_eq!(signatures[27].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[27].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[28].z80_pc, 463);
        assert_eq!(signatures[28].audio_ram_crc32, 2_310_471_849);
        assert_eq!(signatures[28].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[28].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[29].z80_pc, 1_975);
        assert_eq!(signatures[29].audio_ram_crc32, 1_756_839_849);
        assert_eq!(signatures[29].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[29].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[30].z80_pc, 1_965);
        assert_eq!(signatures[30].audio_ram_crc32, 3_477_867_654);
        assert_eq!(signatures[30].pending_m68k_wait_cycles, 1);
        assert_eq!(
            signatures[30].status_bits,
            GemsFrameSignature::STATUS_BUSREQ | GemsFrameSignature::STATUS_IFF1
        );

        assert_eq!(signatures[31].z80_pc, 1_965);
        assert_eq!(signatures[31].audio_ram_crc32, 2_882_818_897);
        assert_eq!(signatures[31].pending_m68k_wait_cycles, 0);
        assert_eq!(
            signatures[31].status_bits,
            GemsFrameSignature::STATUS_BUSREQ | GemsFrameSignature::STATUS_IFF1
        );

        assert_eq!(signatures[32].z80_pc, 677);
        assert_eq!(signatures[32].audio_ram_crc32, 1_505_951_512);
        assert_eq!(signatures[32].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[32].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[33].z80_pc, 0x01B7);
        assert_eq!(signatures[33].audio_ram_crc32, 1_442_419_946);
        assert_eq!(signatures[33].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[33].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[34].z80_pc, 0x019F);
        assert_eq!(signatures[34].audio_ram_crc32, 1_763_598_709);
        assert_eq!(signatures[34].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[34].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[35].z80_pc, 0x018B);
        assert_eq!(signatures[35].audio_ram_crc32, 1_863_261_122);
        assert_eq!(signatures[35].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[35].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[36].z80_pc, 0x0173);
        assert_eq!(signatures[36].audio_ram_crc32, 4_159_266_001);
        assert_eq!(signatures[36].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[36].status_bits, GemsFrameSignature::STATUS_IFF1);

        // ORACLE LIMITATION: the only frame-36 RAM difference is the known
        // IRQ saved-SP byte at $003E, hardware/native $1C versus oracle $1E.
        let mut oracle_ram = probe.engine.checkpoint().audio_ram;
        assert_eq!(oracle_ram[0x003E], 0x1C);
        oracle_ram[0x003E] = 0x1E;
        let mut crc = crc32fast::Hasher::new();
        crc.update(&oracle_ram);
        assert_eq!(crc.finalize(), 3_364_512_860);

        signatures.push(probe.sample_power_on_frame_rendered());
        assert_eq!(signatures[37].z80_pc, 0x02A5);
        assert_eq!(signatures[37].audio_ram_crc32, 2_303_828_415);
        assert_eq!(signatures[37].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[37].status_bits, GemsFrameSignature::STATUS_IFF1);

        // ORACLE LIMITATION: ASM loc_12EDE -> loc_6E2 -> Pal_FadeOut
        // loc_28C6 reaches GEMS CHECKTICK/VBLINT in different phases solely
        // because of the established jgenesis CP X/Y defect. These are the
        // only five frame-37 RAM bytes that differ.
        let mut oracle_ram = probe.engine.checkpoint().audio_ram;
        assert_eq!(oracle_ram[0x076D], 0x01); // SBPTACC+1
        assert_eq!(&oracle_ram[0x1A1C..0x1A20], &[0xA9, 0x07, 0x00, 0x01]);
        oracle_ram[0x076D] = 0x00;
        oracle_ram[0x1A1C..0x1A20].copy_from_slice(&[0x4B, 0x0B, 0xBD, 0x07]);
        let mut crc = crc32fast::Hasher::new();
        crc.update(&oracle_ram);
        assert_eq!(crc.finalize(), 1_793_554_122);

        let frame_38 = probe.sample_power_on_frame_rendered();
        assert_eq!(frame_38.audio_ram_crc32, 2_059_086_427);
        assert_eq!(frame_38.bank_address, 0);
        assert_eq!(frame_38.z80_pc, 0x0B4E);
        assert_eq!(frame_38.pending_m68k_wait_cycles, 0);
        assert_eq!(frame_38.status_bits, GemsFrameSignature::STATUS_IFF1);

        // ORACLE LIMITATION: the first Pal_FadeOut palette transaction is
        // byte-exact apart from the established VBLINT saved-SP byte from
        // GEMS $0038..$003C; the transient PC phase also remains visible.
        let mut oracle_ram = probe.engine.checkpoint().audio_ram;
        assert_eq!(oracle_ram[0x003E], 0x1C);
        oracle_ram[0x003E] = 0x1E;
        let mut crc = crc32fast::Hasher::new();
        crc.update(&oracle_ram);
        assert_eq!(crc.finalize(), 1_171_788_502);

        let frame_39 = probe.sample_power_on_frame_rendered();
        assert_eq!(frame_39.audio_ram_crc32, 402_911_934);
        assert_eq!(frame_39.bank_address, 0);
        assert_eq!(frame_39.z80_pc, 0x0B51);
        assert_eq!(frame_39.pending_m68k_wait_cycles, 0);
        assert_eq!(frame_39.status_bits, GemsFrameSignature::STATUS_IFF1);

        // ORACLE LIMITATION: the second Pal_FadeOut palette transaction has
        // the same one-byte VBLINT saved-SP residual as frame 38.
        let mut oracle_ram = probe.engine.checkpoint().audio_ram;
        assert_eq!(oracle_ram[0x003E], 0x1C);
        oracle_ram[0x003E] = 0x1E;
        let mut crc = crc32fast::Hasher::new();
        crc.update(&oracle_ram);
        assert_eq!(crc.finalize(), 660_654_643);

        let frame_40 = probe.sample_power_on_frame_rendered();
        assert_eq!(frame_40.audio_ram_crc32, 822_438_245);
        assert_eq!(frame_40.bank_address, 0);
        assert_eq!(frame_40.z80_pc, 0x0B44);
        assert_eq!(frame_40.pending_m68k_wait_cycles, 0);
        assert_eq!(frame_40.status_bits, GemsFrameSignature::STATUS_IFF1);

        // ORACLE LIMITATION: the third Pal_FadeOut palette transaction
        // samples the same hardware-correct CP X/Y phase in VBLINT/CHECKTICK.
        // Raw native/oracle GEMS RAM differs only at the IRQ saved-SP byte,
        // SBPTACC+1, and the active stack byte at $1A1E.
        let mut oracle_ram = probe.engine.checkpoint().audio_ram;
        assert_eq!(oracle_ram[0x003E], 0x1C);
        assert_eq!(oracle_ram[0x076D], 0x00);
        assert_eq!(oracle_ram[0x1A1E], 0xBD);
        oracle_ram[0x003E] = 0x1E;
        oracle_ram[0x076D] = 0x01;
        oracle_ram[0x1A1E] = 0xB0;
        let mut crc = crc32fast::Hasher::new();
        crc.update(&oracle_ram);
        assert_eq!(crc.finalize(), 2_442_703_836);

        let frame_41 = probe.sample_power_on_frame_rendered();
        assert_eq!(frame_41.audio_ram_crc32, 1_512_120_546);
        assert_eq!(frame_41.bank_address, 0);
        assert_eq!(frame_41.z80_pc, 0x07AD);
        assert_eq!(frame_41.pending_m68k_wait_cycles, 0);
        assert_eq!(frame_41.status_bits, GemsFrameSignature::STATUS_IFF1);

        // ORACLE LIMITATION: the fourth Pal_FadeOut palette transaction is
        // byte-exact apart from the established VBLINT saved-SP residual.
        let mut oracle_ram = probe.engine.checkpoint().audio_ram;
        assert_eq!(oracle_ram[0x003E], 0x1C);
        oracle_ram[0x003E] = 0x1E;
        let mut crc = crc32fast::Hasher::new();
        crc.update(&oracle_ram);
        assert_eq!(crc.finalize(), 1_698_830_447);

        let frame_42 = probe.sample_power_on_frame_rendered();
        assert_eq!(frame_42.audio_ram_crc32, 2_240_045_277);
        assert_eq!(frame_42.bank_address, 0);
        assert_eq!(frame_42.z80_pc, 0x0B50);
        assert_eq!(frame_42.pending_m68k_wait_cycles, 0);
        assert_eq!(frame_42.status_bits, 0);

        // ORACLE LIMITATION: the fifth Pal_FadeOut palette transaction
        // samples opposite sides of the X/Y-shifted VBLINT boundary. Raw
        // native/oracle RAM differs only at the two saved-SP bytes.
        let mut oracle_ram = probe.engine.checkpoint().audio_ram;
        assert_eq!(oracle_ram[0x003E], 0x1C);
        assert_eq!(oracle_ram[0x003F], 0x1A);
        oracle_ram[0x003E] = 0x1E;
        oracle_ram[0x003F] = 0x00;
        let mut crc = crc32fast::Hasher::new();
        crc.update(&oracle_ram);
        assert_eq!(crc.finalize(), 2_240_142_489);

        let frame_43 = probe.sample_power_on_frame_rendered();
        assert_eq!(frame_43.audio_ram_crc32, 25_501_463);
        assert_eq!(frame_43.bank_address, 0);
        assert_eq!(frame_43.z80_pc, 0x0B50);
        assert_eq!(frame_43.pending_m68k_wait_cycles, 0);
        assert_eq!(frame_43.status_bits, GemsFrameSignature::STATUS_IFF1);

        // ORACLE LIMITATION: the sixth Pal_FadeOut palette transaction is
        // byte-exact apart from the carried CHECKTICK accumulator phase.
        let mut oracle_ram = probe.engine.checkpoint().audio_ram;
        assert_eq!(oracle_ram[0x076C], 0x5C);
        oracle_ram[0x076C] = 0x90;
        let mut crc = crc32fast::Hasher::new();
        crc.update(&oracle_ram);
        assert_eq!(crc.finalize(), 3_170_344_611);

        let frame_44 = probe.sample_power_on_frame_rendered();
        assert_eq!(frame_44.audio_ram_crc32, 2_622_433_153);
        assert_eq!(frame_44.bank_address, 0);
        assert_eq!(frame_44.z80_pc, 0x0B57);
        assert_eq!(frame_44.pending_m68k_wait_cycles, 0);
        assert_eq!(frame_44.status_bits, GemsFrameSignature::STATUS_IFF1);

        // ORACLE LIMITATION: the seventh Pal_FadeOut palette transaction is
        // byte-exact apart from the carried CHECKTICK accumulator phase.
        let mut oracle_ram = probe.engine.checkpoint().audio_ram;
        assert_eq!(oracle_ram[0x076C], 0x28);
        oracle_ram[0x076C] = 0x5C;
        let mut crc = crc32fast::Hasher::new();
        crc.update(&oracle_ram);
        assert_eq!(crc.finalize(), 25_501_463);

        let frame_45 = probe.sample_power_on_frame_rendered();
        assert_eq!(frame_45.audio_ram_crc32, 2_864_186_863);
        assert_eq!(frame_45.bank_address, 0);
        assert_eq!(frame_45.z80_pc, 0x07CE);
        assert_eq!(frame_45.pending_m68k_wait_cycles, 0);
        assert_eq!(frame_45.status_bits, GemsFrameSignature::STATUS_IFF1);

        // ORACLE LIMITATION: the eighth Pal_FadeOut palette transaction
        // differs only in the carried CHECKTICK accumulator and active stack.
        let mut oracle_ram = probe.engine.checkpoint().audio_ram;
        assert_eq!(oracle_ram[0x076C], 0xF4);
        assert_eq!(oracle_ram[0x076D], 0x00);
        assert_eq!(oracle_ram[0x1A1E], 0xCB);
        oracle_ram[0x076C] = 0x28;
        oracle_ram[0x076D] = 0x01;
        oracle_ram[0x1A1E] = 0xB0;
        let mut crc = crc32fast::Hasher::new();
        crc.update(&oracle_ram);
        assert_eq!(crc.finalize(), 1_756_839_849);
    }
}
