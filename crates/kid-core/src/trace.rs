use super::*;
use crate::combat::enemy_contact_bounds;
use std::collections::BTreeMap;

const FLOATING_DIAMOND_CANDIDATE_LIMIT: usize = 4;
const KID_OBJECT_SUPPORT_VERTICAL_WINDOW_PX: i32 = 48;
const KID_OBJECT_SUPPORT_HORIZONTAL_WINDOW_PX: i32 = 96;
const KID_OBJECT_SUPPORT_CANDIDATE_LIMIT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KidObjectRelation {
    Below,
    Above,
    Left,
    Right,
}

impl KidObjectRelation {
    fn as_debug_label(self) -> &'static str {
        match self {
            Self::Below => "kidbelow",
            Self::Above => "kidabove",
            Self::Left => "kidleft",
            Self::Right => "kidright",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ObjectBoundsPx {
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KidObjectSupportCandidate {
    abs_top_gap_px: i32,
    abs_dx_px: i32,
    summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FloatingDiamondContactCandidate {
    distance_score: i32,
    summary: String,
}

impl NativeRuntime {
    fn dying_hidden_shell_still_visible(&self) -> bool {
        matches!(
            self.player_lifecycle,
            PlayerLifecycle::Dying { remaining_frames }
                if remaining_frames
                    > PLAYER_DEATH_HIDE_FRAMES
                        .saturating_sub(PLAYER_DEATH_HIDDEN_SHELL_VISIBLE_FRAMES)
        )
    }

    fn dying_hidden_camera_still_visible(&self) -> bool {
        matches!(
            self.player_lifecycle,
            PlayerLifecycle::Dying { remaining_frames }
                if remaining_frames
                    > PLAYER_DEATH_HIDE_FRAMES
                        .saturating_sub(PLAYER_DEATH_HIDDEN_CAMERA_VISIBLE_FRAMES)
        )
    }

    fn current_character_movement(&self, player: PlayerState) -> u16 {
        const MOVE_STANDINGSTILL: u16 = 0;
        const MOVE_CRAWLING: u16 = 1;
        const MOVE_WALKING: u16 = 2;
        const MOVE_JUMP: u16 = 3;
        const MOVE_WALL_CLIMBING: u16 = 4;
        const MOVE_UPHILL_DOWNHILL: u16 = 6;

        if self.death_shell_snapshot.is_some()
            && matches!(
                self.player_lifecycle,
                PlayerLifecycle::DeathWindup { .. } | PlayerLifecycle::Dying { .. }
            )
        {
            return MOVE_STANDINGSTILL;
        }
        if matches!(self.player_lifecycle, PlayerLifecycle::Respawning { .. }) {
            return MOVE_STANDINGSTILL;
        }

        if self.helmet.iron_knight_climbing || player.wall_cling_direction.is_some() {
            return MOVE_WALL_CLIMBING;
        }
        if !player.on_ground || player.just_jumped {
            return MOVE_JUMP;
        }
        if self.trace_input.down {
            return MOVE_CRAWLING;
        }
        let on_slope = self.block_grid.as_ref().is_some_and(|grid| {
            let x_px = (player.x_pos >> 16) as i32;
            let y_px = (player.y_pos >> 16) as i32;
            let probe = terrain::probe_floor(grid, x_px, y_px);
            matches!(
                probe.cell.collision,
                terrain::CollisionKind::SlopeUp | terrain::CollisionKind::SlopeDown
            )
        });
        if on_slope && (self.trace_input.left || self.trace_input.right || player.x_velocity != 0) {
            return MOVE_UPHILL_DOWNHILL;
        }
        if self.trace_input.left || self.trace_input.right || player.x_velocity != 0 {
            return MOVE_WALKING;
        }
        MOVE_STANDINGSTILL
    }

    fn title_uses_demo_snapshot(&self) -> bool {
        matches!(self.boot_state, BootState::TitleScreen { .. })
            && self.preserve_demo_title_snapshot
    }

    fn title_uses_full_demo_snapshot(&self) -> bool {
        self.title_uses_demo_snapshot()
            && self.title_screen_elapsed_frames < self.demo_title_full_snapshot_frames()
    }

    fn title_uses_timeout_restart_intro_sequence_3_pose(&self) -> bool {
        matches!(self.boot_state, BootState::TitleScreen { .. })
            && self.timeout_restart_title_intro_sequence_3_pose
            && self.title_screen_elapsed_frames < DEMO_TITLE_FULL_SNAPSHOT_FRAMES
    }

    fn title_uses_timeout_restart_intro_sequence_5_pose(&self) -> bool {
        matches!(self.boot_state, BootState::TitleScreen { .. })
            // ASM sub_1C246 writes `Game_Mode=$2C` and returns immediately, so
            // the loc_1BCD0 slot-2 pose survives into title mode until the
            // later title rebuild overwrites that WRAM slot 28 frames later.
            && self.timeout_restart_title_intro_sequence_5_pose
            && self.title_screen_elapsed_frames
                < TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_5_POSE_CARRY_FRAMES
    }

    fn intro_sequence_4_uses_timeout_restart_intro_sequence_3_pose(&self) -> bool {
        matches!(
            self.boot_state,
            BootState::IntroSequence {
                segment: 3,
                elapsed,
                ..
            } if self.timeout_restart_intro_sequence_4_pose_carry
                && elapsed < TIMEOUT_RESTART_INTRO_SEQUENCE_4_POSE_CARRY_FRAMES
        )
    }

    fn intro_sequence_4_uses_timeout_restart_camera_handoff(&self) -> bool {
        matches!(
            self.boot_state,
            // ASM sub_1B988: `move.w #4,(Camera_Y_pos).w`. The timeout-restart
            // path exposes this only after the carried intro-sequence-3 pose
            // has already cleared. ASM Load_IntroSequence5 only allocates
            // `loc_1BA28` / `loc_1BCD0`; unlike Load_IntroSequence6 and
            // GfxObjects_Collision_BottomBoundary4 it does not call
            // `sub_1B222`, so oracle traces keep the carried camera only for
            // the first 31 intro-sequence-5 frames before the scene drops
            // back to the cleared camera state.
            BootState::IntroSequence {
                segment: 3,
                elapsed,
                ..
            } if self.timeout_restart_intro_sequence_4_pose_carry
                && elapsed >= TIMEOUT_RESTART_INTRO_SEQUENCE_4_CAMERA_HANDOFF_FRAMES
        ) || matches!(
            self.boot_state,
            BootState::IntroSequence {
                segment: 4,
                elapsed,
                ..
            }
                if self.timeout_restart_intro_sequence_4_pose_carry
                    && elapsed < TIMEOUT_RESTART_INTRO_SEQUENCE_5_CAMERA_CARRY_FRAMES
        )
    }

    fn intro_sequence_5_uses_timeout_restart_slot_2_pose(&self) -> bool {
        let visible_elapsed_frames =
            if self.demo_timeout_restart_uses_intro_return_slot_2_early_visibility {
                TIMEOUT_RESTART_INTRO_SEQUENCE_5_SLOT_2_VISIBLE_ELAPSED_FRAMES - 1
            } else {
                TIMEOUT_RESTART_INTRO_SEQUENCE_5_SLOT_2_VISIBLE_ELAPSED_FRAMES
            };
        matches!(
            self.boot_state,
            // ASM loc_1BCD0 allocates five title gfx slots and writes the
            // third one with `x=0`, `y=$E0`. The verifier reads WRAM slot 2
            // directly from $FF24FA/$FF24FE, and oracle probes show that this
            // timeout-restart intro-sequence-5 slot stays zero through frame
            // 10006 before the loc_1BCD0 write becomes visible on 10007.
            // The later Start-skip intro-return lineage reaches the same
            // loc_1BCD0 slot one frame earlier on the timeout-restart revisit,
            // so keep that oracle-backed offset as a separate lineage bit
            // rather than shifting the shared timer globally.
            BootState::IntroSequence {
                segment: 4,
                elapsed,
                ..
            } if self.timeout_restart_intro_sequence_4_pose_carry
                && elapsed >= visible_elapsed_frames
        )
    }

    fn intro_sequence_6_timeout_restart_hidden_camera_y(&self) -> Option<u32> {
        let BootState::IntroSequence6 {
            carry_intro_sequence_5_slot_2_pose: true,
            carry_timeout_restart_hidden_camera_wrap,
            ..
        } = self.boot_state
        else {
            return None;
        };
        let elapsed = self.title_screen_elapsed_frames;
        if !carry_timeout_restart_hidden_camera_wrap {
            // ASM Load_IntroSequence6 still seeds `($FFFFA656).w` with
            // `$FFD00000` and calls `sub_1B222` on the later title-restore
            // revisit, but oracle keeps the carried cleared camera through
            // local mode-$28 frame 34 before the delayed seed appears.
            if elapsed < TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_SEEDED_CAMERA_ELAPSED_FRAMES {
                return None;
            }
            if elapsed < TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_CAMERA_RAMP_ELAPSED_FRAMES {
                return Some(0xFFD0_0000);
            }

            let ramp_frame =
                elapsed - TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_CAMERA_RAMP_ELAPSED_FRAMES;
            if ramp_frame < TIMEOUT_RESTART_INTRO_SEQUENCE_6_CAMERA_RAMP_FRAMES {
                // ASM GfxObjects_Collision_BottomBoundary4:
                // `move.l ($FFFFA656).w,d1; asr.l #4,d1; ... sub.l d1,($FFFFA656).w`
                // With the later `$FFD0_0000` seed, `d1=$FFFD_0000`, so the
                // hidden camera rises by 3 pixels per frame for 16 frames.
                return Some(0xFFD0_0000_u32.wrapping_add(
                    ((ramp_frame as u32).wrapping_add(1)).wrapping_mul(0x0003_0000),
                ));
            }

            return Some(0);
        }
        if elapsed < TIMEOUT_RESTART_INTRO_SEQUENCE_6_CAMERA_WRAP_ELAPSED_FRAMES {
            return None;
        }

        // ASM Load_IntroSequence6 seeds `($FFFFA656).w` with `$FFD00000`,
        // adds `$20` on the timeout-restart lineage, then calls `sub_1B222`
        // so the first exposed hidden row is the wrapped `$FFF0_0000`.
        // Later sub_1B43A allocates GfxObjects_Collision_BottomBoundary4,
        // which delays, derives `d1 = ($FFFFA656).w >> 4`, then repeats
        // `sub.l d1,($FFFFA656).w; bsr.w sub_1B222` for 16 frames. With the
        // `$FFF0_0000` seed, that means `d1=$FFFF0000`, so oracle ramps the
        // hidden camera upward by exactly one pixel per frame from local
        // mode-$28 frame 248 until the worker deletes itself.
        if elapsed < TIMEOUT_RESTART_INTRO_SEQUENCE_6_CAMERA_RAMP_ELAPSED_FRAMES {
            return Some(0xFFF0_0000);
        }

        let ramp_frame = elapsed - TIMEOUT_RESTART_INTRO_SEQUENCE_6_CAMERA_RAMP_ELAPSED_FRAMES;
        if ramp_frame < TIMEOUT_RESTART_INTRO_SEQUENCE_6_CAMERA_RAMP_FRAMES {
            return Some(
                0xFFF0_0000_u32
                    .wrapping_add(((ramp_frame as u32).wrapping_add(1)).wrapping_mul(0x0001_0000)),
            );
        }

        Some(0)
    }

    fn title_hides_demo_return_player(&self) -> bool {
        let hidden_frames = if self.timeout_restart_title_intro_sequence_3_pose {
            TIMEOUT_RESTART_TITLE_PLAYER_HIDDEN_FRAMES
        } else if self.timeout_restart_title_intro_sequence_5_pose {
            let mut hidden_frames = TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_5_HIDDEN_FRAMES;
            if self.timeout_restart_title_intro_sequence_5_uses_intro_return_hidden_delay {
                // ASM sub_1C246 still jumps straight to `Game_Mode=$2C`, but
                // oracle WRAM probes keep the later Start-skip intro-return
                // lineage hidden through frame 16798 before the standard title
                // pose reappears on 16799.
                hidden_frames +=
                    TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_5_START_SKIP_LINEAGE_EXTRA_HIDDEN_FRAMES;
            }
            hidden_frames
        } else {
            let mut hidden_frames = DEMO_TITLE_PLAYER_HIDDEN_FRAMES;
            if self.demo_title_return_uses_intro_sequence_6_early_title_restore {
                // ASM loc_1B5CE has already written the standard title pose on
                // this lineage before the later timeout-restart demo starts.
                // Oracle therefore drops the generic title-hidden rebuild two
                // frames earlier after the later demo-title return and begins
                // exposing that same pose on local title elapsed frames 148
                // and 149.
                hidden_frames = hidden_frames
                    .saturating_sub(DEMO_AUTO_TIMEOUT_INTRO_SEQUENCE_6_EARLY_TITLE_RESTORE_FRAMES);
            }
            hidden_frames
        };
        matches!(self.boot_state, BootState::TitleScreen { .. })
            && self.hide_demo_return_title_player
            && (self.demo_auto_timeout_handoff_active
                || self.title_screen_elapsed_frames < hidden_frames)
    }

    pub fn trace_frame(&self) -> NativeTraceFrame {
        let game_mode = self.game_mode();
        let boot_phase = self.boot_phase_label().map(str::to_owned);
        let boot_phase_elapsed = self.boot_phase_elapsed_frames();
        let cold_boot_frame = self.frame_index == 0
            && !matches!(
                self.boot_state,
                BootState::StartingLevel { .. }
                    | BootState::LevelSkipHold
                    | BootState::InGame
                    | BootState::DemoPlay { .. }
                    | BootState::EndingSequence { .. }
            );
        let current_level_name = self.level_name(self.current_level).map(str::to_owned);
        let level_header = self.level_header(self.current_level);
        let enemy_layout = self.enemy_layout(self.current_level);
        let player_state = self.current_player_state();
        let camera_state = self.current_camera_state();
        let active_objects = self.active_objects();
        let (number_objects, number_gfx_objects) =
            self.object_counts_from_roster(&active_objects, camera_state.x_pos, camera_state.y_pos);
        let active_enemy_object_count = active_objects
            .iter()
            .filter(|object| matches!(object.kind, NativeObjectKind::Enemy { .. }))
            .count();
        let active_platform_object_count = active_objects
            .iter()
            .filter(|object| matches!(object.kind, NativeObjectKind::Platform { .. }))
            .count();
        let mut gameplay_debug_fields = BTreeMap::new();
        if self.progression.form_change_flash_frames > 0 {
            gameplay_debug_fields.insert("currently_transforming".into(), "true".into());
        }
        if self.gameplay_hibernating_for_transform() {
            gameplay_debug_fields.insert("gameplay_hibernated".into(), "true".into());
        }
        let floating_diamond_debug =
            player_state.and_then(|state| self.floating_diamond_contact_candidates(state));
        if !self.pending_prize_diamond_awards.is_empty() || floating_diamond_debug.is_some() {
            gameplay_debug_fields.insert(
                "pending_prize_diamond_awards".into(),
                self.pending_prize_diamond_awards_debug(),
            );
        }
        if let Some(summary) = floating_diamond_debug {
            gameplay_debug_fields.insert("floating_diamond_contact_candidates".into(), summary);
        }
        if let Some(player_state) = player_state
            && let Some(summary) = self.kid_object_support_candidates(player_state)
        {
            gameplay_debug_fields.insert("kid_object_support_candidates".into(), summary);
        }
        if let Some(player_state) = player_state
            && let Some(summary) = self.kid_ground_probe_debug(player_state)
        {
            gameplay_debug_fields.insert("kid_ground_probe".into(), summary);
        }
        if let Some(summary) = self.kid_special_motion_probe_debug() {
            gameplay_debug_fields.insert("kid_special_motion_probe".into(), summary);
        }
        if let Some(player_state) = player_state
            && let Some(summary) = self.kid_goal_probe_debug(player_state)
        {
            gameplay_debug_fields.insert("kid_goal_probe".into(), summary);
        }
        if let Some(summary) = self.kid_gameplay_path_probe_debug() {
            gameplay_debug_fields.insert("kid_gameplay_path_probe".into(), summary);
        }
        if let Some(summary) = self.kid_collision_handoff_probe_debug() {
            gameplay_debug_fields.insert("kid_collision_handoff_probe".into(), summary);
        }
        if let Some(summary) = self.kid_same_frame_stomp_probe_debug() {
            gameplay_debug_fields.insert("kid_same_frame_stomp_probe".into(), summary);
        }
        if let Some(summary) = self.kid_iron_knight_attach_probe_debug() {
            gameplay_debug_fields.insert("kid_iron_knight_attach_probe".into(), summary);
        }
        if let Some(summary) = self.kid_flip_branch_probe_debug() {
            gameplay_debug_fields.insert("kid_flip_branch_probe".into(), summary);
        }
        if let Some(summary) = self.kid_flip_follow_on_probe_debug() {
            gameplay_debug_fields.insert("kid_flip_follow_on_probe".into(), summary);
        }
        if let Some(summary) = self.kid_jump_gravity_probe_debug() {
            gameplay_debug_fields.insert("kid_jump_gravity_probe".into(), summary);
        }
        if let Some(summary) = self.kid_pre_physics_probe_debug() {
            gameplay_debug_fields.insert("kid_pre_physics_probe".into(), summary);
        }
        if let Some(summary) = self.kid_iron_knight_block_breaker_probe_debug() {
            gameplay_debug_fields.insert("kid_iron_knight_block_breaker_probe".into(), summary);
        }
        if let Some(summary) = self.kid_grounded_release_probe_debug() {
            gameplay_debug_fields.insert("kid_grounded_release_probe".into(), summary);
        }
        if let Some(summary) = self.kid_wall_cling_release_probe_debug() {
            gameplay_debug_fields.insert("kid_wall_cling_release_probe".into(), summary);
        }
        NativeTraceFrame {
            frame_index: self.frame_index,
            game_mode,
            game_mode_value: game_mode.as_disassembly_value(),
            start_press_count: self.start_press_count,
            boot_phase,
            boot_phase_elapsed,
            title_screen_ready: self.title_screen_ready(),
            current_player: self.current_player_index,
            current_level: self.current_level,
            player_1_level_id: self.current_level,
            current_level_name,
            player_start_x: level_header.map(|header| header.player_start_x),
            player_start_y: level_header.map(|header| header.player_start_y),
            flag_x: level_header.map(|header| header.flag_x),
            flag_y: level_header.map(|header| header.flag_y),
            level_enemy_object_count: enemy_layout.map(|layout| layout.object_count),
            active_enemy_object_count,
            active_platform_object_count,
            number_lives: if cold_boot_frame {
                0
            } else {
                self.progression.number_lives
            },
            number_continues: self.progression.number_continues,
            number_hitpoints: if cold_boot_frame {
                0
            } else {
                self.progression.number_hitpoints
            },
            current_helmet_id: if cold_boot_frame {
                0
            } else {
                self.progression.current_form.helmet_id()
            },
            paused: self.paused,
            kid_x_pos: player_state.map(|state| state.x_pos),
            kid_y_pos: player_state.map(|state| state.y_pos),
            kid_x_velocity: player_state.map(|state| state.x_velocity),
            kid_y_velocity: player_state.map(|state| state.y_velocity),
            character_movement: player_state.map(|state| self.current_character_movement(state)),
            kid_facing: player_state.map(|state| state.facing),
            kid_on_ground: player_state.map(|state| state.on_ground),
            kid_on_slope: player_state
                .filter(|state| state.on_ground)
                .and_then(|state| {
                    let grid = self.block_grid.as_ref()?;
                    let x_px = (state.x_pos >> 16) as i32;
                    let y_px = (state.y_pos >> 16) as i32;
                    let probe = terrain::probe_floor(grid, x_px, y_px);
                    match probe.cell.collision {
                        terrain::CollisionKind::SlopeUp => Some(1),
                        terrain::CollisionKind::SlopeDown => Some(-1),
                        _ => Some(0),
                    }
                })
                .unwrap_or(0),
            active_objects,
            camera_x_pos: camera_state.x_pos,
            camera_y_pos: camera_state.y_pos,
            number_objects,
            number_gfx_objects,
            level_complete: matches!(self.player_lifecycle, PlayerLifecycle::LevelComplete { .. }),
            game_over: matches!(
                self.player_lifecycle,
                PlayerLifecycle::GameOver { .. } | PlayerLifecycle::ContinueCountdown { .. }
            ),
            continue_countdown: match self.player_lifecycle {
                PlayerLifecycle::ContinueCountdown { remaining_frames } => Some(remaining_frames),
                _ => None,
            },
            game_complete: self.game_complete,
            ending_elapsed: match self.boot_state {
                BootState::EndingSequence { elapsed } => Some(elapsed),
                _ => None,
            },
            options_cursor: match self.boot_state {
                BootState::TitleScreen { cursor } | BootState::OptionsMenu { cursor } => {
                    Some(cursor)
                }
                _ => None,
            },
            sound_test_track: match self.boot_state {
                BootState::SoundTest { track } => Some(track),
                _ => None,
            },
            two_player: self.two_player,
            options_two_player_two_controllers: self.options_two_player_two_controllers,
            options_controls_scheme: self.options_controls_scheme,
            options_speed_button_fast: self.options_speed_button_fast,
            invulnerable_frames: self.progression.invulnerable_frames,
            just_received_damage: self.just_received_damage
                || (self.death_shell_snapshot.is_some()
                    && matches!(
                        self.player_lifecycle,
                        PlayerLifecycle::DeathWindup { .. } | PlayerLifecycle::Dying { .. }
                    )),
            diamond_power_active: self.progression.diamond_power_frames > 0,
            diamond_count: self.progression.diamond_count,
            score: self.progression.score,
            level_timer_seconds: self.level_timer.seconds,
            level_damage_taken: self.level_damage_taken,
            murder_wall_x: self.murder_wall.map(|w| w.x_pos),
            active_projectiles: self
                .projectiles
                .iter()
                .map(|p| (p.x_pos, p.y_pos, p.friendly))
                .collect(),
            active_geysers: self
                .weather
                .lava_geysers
                .iter()
                .filter(|g| g.current_height > 0)
                .map(|g| {
                    (
                        g.x_px,
                        g.y_px.saturating_sub(g.current_height),
                        g.current_height,
                    )
                })
                .collect(),
            block_bounces: self
                .block_bounces
                .iter()
                .map(|b| (b.cell_x, b.cell_y, b.remaining))
                .collect(),
            block_destroy_effects: self
                .block_destroy_effects
                .iter()
                .map(|e| (e.x_px, e.y_px, e.remaining))
                .collect(),
            floating_items: self
                .floating_items
                .iter()
                .map(|f| {
                    let x_px = (f.x_pos >> 16) as u16;
                    let y_px = (f.y_pos >> 16) as u16;
                    (x_px, y_px, f.remaining as u8, f.content.to_id())
                })
                .collect(),
            enemy_death_effects: self
                .enemy_death_effects
                .iter()
                .map(|e| (e.x_pos, e.y_pos, e.remaining))
                .collect(),
            death_windup_elapsed: match self.player_lifecycle {
                PlayerLifecycle::DeathWindup { remaining_frames } => {
                    (PLAYER_DEATH_WINDUP_FRAMES.saturating_sub(remaining_frames)) as u16
                }
                _ => 0,
            },
            form_change_flash_frames: self.progression.form_change_flash_frames,
            stomped_this_frame: self.stomped_this_frame,
            blocks_hit_this_frame: self.blocks_hit_this_frame,
            sound_test_play_pending: self.sound_test_play_pending,
            gameplay_debug_fields,
            physics_trace: self.opening_slice.and_then(|s| s.physics_trace),
        }
    }

    fn kid_object_support_candidates(&self, player: PlayerState) -> Option<String> {
        let player_bounds = self.player_collision_bounds(player);
        let player_x_px = (player.x_pos >> 16) as i32;
        let player_foot_y_px = (player.y_pos >> 16) as i32;
        let mut candidates = Vec::new();

        for enemy in &self.active_enemies {
            let snapshot = enemy.snapshot();
            if let Some(bounds) = enemy_collision_bounds_px(&snapshot)
                && let Some(candidate) = kid_object_support_candidate(
                    format!(
                        "enemy_{:02X}#{:02}",
                        enemy.family.object_type(),
                        enemy.layout_index
                    ),
                    player_x_px,
                    player_foot_y_px,
                    player_bounds,
                    bounds,
                )
            {
                candidates.push(candidate);
            }
        }

        for (platform_index, platform) in self.active_platforms.iter().enumerate() {
            let bounds = platform_collision_bounds_px(*platform);
            if let Some(candidate) = kid_object_support_candidate(
                format!("platform_{platform_index:02}"),
                player_x_px,
                player_foot_y_px,
                player_bounds,
                bounds,
            ) {
                candidates.push(candidate);
            }
        }

        if candidates.is_empty() {
            return None;
        }

        candidates.sort_by_key(|candidate| (candidate.abs_top_gap_px, candidate.abs_dx_px));
        let summaries = candidates
            .into_iter()
            .take(KID_OBJECT_SUPPORT_CANDIDATE_LIMIT)
            .map(|candidate| candidate.summary)
            .collect::<Vec<_>>();
        Some(summaries.join("; "))
    }

    fn kid_ground_probe_debug(&self, player: PlayerState) -> Option<String> {
        let grid = self.block_grid.as_ref()?;
        let x_px = (player.x_pos >> 16) as i32;
        let y_px = (player.y_pos >> 16) as i32;
        let half_width = self.progression.current_form.half_width();
        let left = terrain::probe_floor(grid, x_px - half_width, y_px);
        let center = terrain::probe_floor(grid, x_px, y_px);
        let right = terrain::probe_floor(grid, x_px + half_width, y_px);
        let hitbox = terrain::probe_floor_hitbox(grid, x_px, y_px, half_width);
        let latched_floor_y_px = self
            .opening_slice
            .as_ref()
            .map(|slice| (slice.terrain.floor_y_pos >> 16) as i32)
            .unwrap_or(y_px);

        Some(format!(
            "latched_ground={}/latched_floor_y={}/y_vel={:#010x}/left={}/center={}/right={}/hitbox={}",
            u8::from(player.on_ground),
            latched_floor_y_px,
            player.y_velocity as u32,
            format_kid_floor_probe(left),
            format_kid_floor_probe(center),
            format_kid_floor_probe(right),
            format_kid_floor_probe(hitbox),
        ))
    }

    fn kid_special_motion_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        Some(format!(
            "helmet={:#06x}/skycutter_on_board={}/skycutter_flipboard={}/cyclone_flying={}/cyclone_y_accel={}/iron_knight_climbing={}/iron_knight_climb_accel={}/micromax_stuck={}/pending_stomp_bounce={}/form_flash={}",
            self.progression.current_form.helmet_id(),
            u8::from(self.helmet.skycutter_on_board),
            self.helmet.skycutter_flipboard,
            u8::from(self.helmet.cyclone_flying),
            self.helmet.cyclone_y_accel,
            u8::from(self.helmet.iron_knight_climbing),
            self.helmet.iron_knight_climb_accel,
            u8::from(self.helmet.micromax_stuck),
            u8::from(self.pending_stomp_bounce.is_some()),
            self.progression.form_change_flash_frames,
        ))
    }

    fn kid_goal_probe_debug(&self, player: PlayerState) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let header = self.level_header(self.current_level)?;
        let flag_x_fp = u32::from(header.flag_x) << 16;
        let flag_y_fp = u32::from(header.flag_y) << 16;
        let dx_px = player.x_pos.abs_diff(flag_x_fp) >> 16;
        let dy_px = player.y_pos.abs_diff(flag_y_fp) >> 16;
        let touching_flag = dx_px <= FLAG_HIT_HALF_SIZE && dy_px <= FLAG_HIT_HALF_SIZE;

        Some(format!(
            "lifecycle={}/level_complete={}/touch={}/flag=({}, {})/dx_px={}/dy_px={}",
            player_lifecycle_debug_label(self.player_lifecycle),
            u8::from(matches!(
                self.player_lifecycle,
                PlayerLifecycle::LevelComplete { .. }
            )),
            u8::from(touching_flag),
            header.flag_x,
            header.flag_y,
            dx_px,
            dy_px,
        ))
    }

    fn kid_gameplay_path_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        Some(format!(
            "lifecycle={}/motion={}/post={}/paused={}",
            player_lifecycle_debug_label(self.player_lifecycle),
            self.last_kid_frame_motion_branch.as_debug_label(),
            self.last_kid_frame_post_motion_mutator.as_debug_label(),
            u8::from(self.paused),
        ))
    }

    fn kid_pre_physics_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let start = self.last_kid_frame_pre_physics_start?;
        let before_physics = self
            .last_kid_frame_pre_physics_before_physics
            .unwrap_or(start);
        let first_mutator = self
            .last_kid_frame_pre_physics_mutators
            .first()
            .copied()
            .unwrap_or(KidFramePrePhysicsMutator::None)
            .as_debug_label();
        let mutators = if self.last_kid_frame_pre_physics_mutators.is_empty() {
            String::from("none")
        } else {
            self.last_kid_frame_pre_physics_mutators
                .iter()
                .map(|mutator| mutator.as_debug_label())
                .collect::<Vec<_>>()
                .join(",")
        };

        Some(format!(
            "start[{}/ground={}]/before_physics[{}/ground={}]/first={}/mutators={}",
            kid_pre_physics_state_debug(start),
            u8::from(start.on_ground),
            kid_pre_physics_state_debug(before_physics),
            u8::from(before_physics.on_ground),
            first_mutator,
            mutators,
        ))
    }

    fn kid_collision_handoff_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let probe = self.last_kid_frame_collision_handoff_probe?;
        let pending_form = probe
            .pending_form
            .map(|form| format!("{:#06x}", form.helmet_id()))
            .unwrap_or_else(|| String::from("none"));
        let pending_level_skip = probe
            .pending_level_skip
            .map(|level| format!("{:#04x}", level.0))
            .unwrap_or_else(|| String::from("none"));
        let stomp_enemy = match (
            probe.stomp_enemy_object_type,
            probe.stomp_enemy_x_pos,
            probe.stomp_enemy_y_pos,
        ) {
            (Some(object_type), Some(x_pos), Some(y_pos)) => {
                format!("{object_type:#04x}@{x_pos:#010x},{y_pos:#010x}")
            }
            _ => String::from("none"),
        };
        let wall_cling = match probe.wall_cling_direction {
            Some(Direction::Left) => "left",
            Some(Direction::Right) => "right",
            None => "none",
        };

        Some(format!(
            "lifecycle={}/prev_post={}/pending_stomp={}/pending_form={}/pending_level_skip={}/form_flash={}/ground={}/jump_buffer={}/wall_cling={}/stomp_candidate={}/stomp_enemy={}/anchor={:#010x},{:#010x}",
            player_lifecycle_debug_label(probe.player_lifecycle),
            probe.previous_post_motion_mutator.as_debug_label(),
            u8::from(probe.pending_stomp_bounce),
            pending_form,
            pending_level_skip,
            probe.form_change_flash_frames,
            u8::from(probe.on_ground_before),
            probe.jump_buffer_frames,
            wall_cling,
            u8::from(probe.stomp_candidate),
            stomp_enemy,
            probe.anchor_x_pos,
            probe.anchor_y_pos,
        ))
    }

    fn kid_same_frame_stomp_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let probe = self.last_kid_frame_same_frame_stomp_probe?;
        let enemy = match (
            probe.enemy_object_type,
            probe.enemy_x_pos,
            probe.enemy_y_pos,
        ) {
            (Some(object_type), Some(x_pos), Some(y_pos)) => {
                format!("{object_type:#04x}@{x_pos:#010x},{y_pos:#010x}")
            }
            _ => String::from("none"),
        };

        Some(format!(
            "candidate={}/enemy={}/pre_y_vel={:#010x}/post_y_vel={:#010x}/anchor={:#010x},{:#010x}",
            u8::from(probe.stomp_candidate),
            enemy,
            probe.pre_physics_y_velocity as u32,
            probe.post_physics_y_velocity as u32,
            probe.anchor_x_pos,
            probe.anchor_y_pos,
        ))
    }

    fn kid_iron_knight_attach_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let probe = self.last_kid_frame_iron_knight_attach_probe?;

        Some(format!(
            "c_press={}/ground={}/climbing={}=>{}/facing={}/y_vel={:#010x}->{:#010x}/wall={}/candidate={}/attach={}",
            u8::from(probe.c_pressed),
            u8::from(probe.on_ground_before),
            u8::from(probe.climbing_before),
            u8::from(probe.climbing_after),
            direction_debug_label(probe.facing),
            probe.y_velocity_before as u32,
            probe.y_velocity_after as u32,
            wall_probe_debug_summary(probe.wall_probe),
            u8::from(probe.candidate),
            u8::from(probe.attach_triggered),
        ))
    }

    fn kid_flip_branch_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let probe = self.last_kid_frame_kid_flip_branch_probe?;

        Some(format!(
            "wall_scan={}/b_press={}/c_press={}/helmet={:#06x}/facing={}/ground={}=>{}/branch={}/wall={}",
            u8::from(probe.wall_scan_hit),
            u8::from(probe.b_pressed),
            u8::from(probe.c_pressed),
            probe.helmet_id,
            direction_debug_label(probe.facing),
            u8::from(probe.on_ground_before),
            u8::from(probe.on_ground_after),
            probe.branch.as_debug_label(),
            wall_probe_debug_summary(probe.wall_probe),
        ))
    }

    fn kid_flip_follow_on_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let probe = self.last_kid_frame_kid_flip_follow_on_probe?;

        Some(format!(
            "branch={}/motion={}/post={}/entry[{}/ground={}/wall_cling={}]/press_away[{}/ground={}/wall_cling={}]/jump_gravity[{}/ground={}/wall_cling={}]/floor[{}/ground={}/wall_cling={}]/wall[{}/ground={}/wall_cling={}]",
            probe.branch.as_debug_label(),
            probe.motion_branch.as_debug_label(),
            probe.post_motion_mutator.as_debug_label(),
            kid_flip_follow_on_stage_debug(probe.stages.entry),
            u8::from(probe.stages.entry.on_ground),
            direction_debug_label(probe.stages.entry.wall_cling_direction),
            kid_flip_follow_on_stage_debug(probe.stages.after_press_away),
            u8::from(probe.stages.after_press_away.on_ground),
            direction_debug_label(probe.stages.after_press_away.wall_cling_direction),
            kid_flip_follow_on_stage_debug(probe.stages.after_jump_gravity),
            u8::from(probe.stages.after_jump_gravity.on_ground),
            direction_debug_label(probe.stages.after_jump_gravity.wall_cling_direction),
            kid_flip_follow_on_stage_debug(probe.stages.after_floor),
            u8::from(probe.stages.after_floor.on_ground),
            direction_debug_label(probe.stages.after_floor.wall_cling_direction),
            kid_flip_follow_on_stage_debug(probe.stages.after_wall),
            u8::from(probe.stages.after_wall.on_ground),
            direction_debug_label(probe.stages.after_wall.wall_cling_direction),
        ))
    }

    fn kid_jump_gravity_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let probe = self.last_kid_frame_jump_gravity_probe?;

        Some(format!(
            "branch={}/jump={}/jump_buffer={}=>{}/just_jumped={}=>{}/b_held={}/custom_gravity={}/y_vel={:#010x}->{:#010x}/ground={}=>{}",
            probe.branch.as_debug_label(),
            u8::from(probe.jump_pressed),
            probe.jump_buffer_frames_before,
            probe.jump_buffer_frames_after,
            u8::from(probe.just_jumped_before),
            u8::from(probe.just_jumped_after),
            u8::from(probe.input_b_held),
            u8::from(probe.custom_gravity),
            probe.y_velocity_before as u32,
            probe.y_velocity_after as u32,
            u8::from(probe.on_ground_before),
            u8::from(probe.on_ground_after),
        ))
    }

    fn kid_grounded_release_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let probe = self.last_kid_frame_grounded_release_probe?;
        let held = match probe.held_direction {
            Some(Direction::Left) => "left",
            Some(Direction::Right) => "right",
            None => "none",
        };
        let wall_cling = match probe.wall_cling_direction {
            Some(Direction::Left) => "left",
            Some(Direction::Right) => "right",
            None => "none",
        };
        let scan = match (probe.probe_row, probe.left_col, probe.right_col) {
            (Some(row), Some(left), Some(right)) => format!("row={row}/cols={left}..{right}"),
            _ => String::from("row=none/cols=none"),
        };
        let support = match (probe.support_col, probe.support_collision, probe.probe_row) {
            (Some(col), Some(collision), Some(row)) => format!(
                "{}@{col},{row} block={}",
                collision_kind_debug_label(collision),
                block_type_debug_label(
                    probe.support_block_type.unwrap_or(terrain::BlockType::None)
                ),
            ),
            _ => String::from("none"),
        };
        let terrain_provenance =
            match (probe.support_col, probe.probe_row, self.block_grid.as_ref()) {
                (Some(col), Some(row), Some(current_grid)) => {
                    let current_cell = current_grid.cell_at(col, row);
                    let baseline = if self.block_grid_source == BlockGridSource::Generated {
                        self.load_block_grid_for_level(self.current_level)
                            .map(|grid| grid.cell_at(col, row))
                    } else {
                        None
                    };
                    match baseline {
                        Some(baseline_cell) => format!(
                            "/terrain=grid={}/current[{}]/baseline[{}]/same={}",
                            block_grid_source_debug_label(self.block_grid_source),
                            terrain_cell_debug_summary(current_cell),
                            terrain_cell_debug_summary(baseline_cell),
                            u8::from(current_cell == baseline_cell),
                        ),
                        None => format!(
                            "/terrain=grid={}/current[{}]",
                            block_grid_source_debug_label(self.block_grid_source),
                            terrain_cell_debug_summary(current_cell),
                        ),
                    }
                }
                _ => String::new(),
            };

        Some(format!(
            "ground={}/held={}/jump={}/jump_buffer={}/velocity={}/custom_gravity={}/wall_cling={}/scan={}/support={}/release={}/post={}{}",
            u8::from(probe.on_ground_before),
            held,
            u8::from(probe.jump_pressed),
            probe.jump_buffer_frames,
            probe.velocity_branch.as_debug_label(),
            u8::from(probe.custom_gravity),
            wall_cling,
            scan,
            support,
            u8::from(probe.release_triggered),
            self.last_kid_frame_post_motion_mutator.as_debug_label(),
            terrain_provenance,
        ))
    }

    fn kid_iron_knight_block_breaker_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let player = self.last_kid_frame_pre_physics_before_physics?;
        let form = self.progression.current_form;
        let facing = match player.facing {
            Direction::Left => "left",
            Direction::Right => "right",
        };
        let speed_gate =
            form == PlayerForm::IronKnight && player.x_velocity.unsigned_abs() >= 0x48000;

        let (wall_summary, breaker_candidate) = match self.block_grid.as_ref() {
            Some(grid) => {
                let wall_probe = terrain::probe_wall(
                    grid,
                    (player.x_pos >> 16) as i32,
                    (player.y_pos >> 16) as i32,
                    form.half_width(),
                    PLAYER_HEIGHT,
                    player.facing == Direction::Right,
                );
                (
                    wall_probe_debug_summary(Some(wall_probe)),
                    speed_gate && wall_probe.hit && wall_probe.cell.has_block,
                )
            }
            None => (wall_probe_debug_summary(None), false),
        };

        Some(format!(
            "helmet={:#06x}/facing={}/x_vel={:#010x}/speed_gate={}/wall={}/candidate={}",
            form.helmet_id(),
            facing,
            player.x_velocity as u32,
            u8::from(speed_gate),
            wall_summary,
            u8::from(breaker_candidate),
        ))
    }

    fn kid_wall_cling_release_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let probe = self.opening_slice?.wall_cling_release_probe?;
        let held = direction_debug_label(probe.held_direction);
        let wall_cling = direction_debug_label(probe.wall_cling_direction);
        let probe_direction = match probe.probe_direction {
            Some(direction) => format!(
                "{}({})",
                direction_debug_label(Some(direction)),
                probe.probe_direction_source.as_debug_label(),
            ),
            None => String::from("none(none)"),
        };

        Some(format!(
            "held={}/probe_dir={}/wall_cling={}/sticky=R{}L{}/ground={}/y_vel={:#010x}/y={}=>{}/branch={}/sub_ADB8={}/sub_AE6E={}/wall={}/top_edge={}",
            held,
            probe_direction,
            wall_cling,
            u8::from(probe.right_wall_sticky),
            u8::from(probe.left_wall_sticky),
            u8::from(probe.on_ground),
            probe.y_velocity as u32,
            probe.pre_vertical_y_px,
            probe.post_vertical_y_px,
            probe.branch.as_debug_label(),
            probe.sub_adb8_gate.as_debug_label(),
            probe.sub_ae6e_gate.as_debug_label(),
            wall_probe_debug_summary(probe.forward_wall_probe),
            wall_probe_debug_summary(probe.top_edge_probe),
        ))
    }

    fn pending_prize_diamond_awards_debug(&self) -> String {
        let delays = self
            .pending_prize_diamond_awards
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>();
        format!("[{}]", delays.join(","))
    }

    fn floating_diamond_contact_candidates(&self, player: PlayerState) -> Option<String> {
        use crate::constants::{PLAYER_HALF_WIDTH, PLAYER_HEIGHT};

        let player_x_px = (player.x_pos >> 16) as i32;
        let player_y_px = (player.y_pos >> 16) as i32;
        let player_left = player_x_px - PLAYER_HALF_WIDTH;
        let player_right = player_x_px + PLAYER_HALF_WIDTH;
        let player_bottom = player_y_px;
        let mut candidates = Vec::new();

        for item in &self.floating_items {
            if item.content != terrain::PrizeContent::Diamond {
                continue;
            }
            let item_x_px = (item.x_pos >> 16) as i32;
            let item_y_px = (item.y_pos >> 16) as i32;
            let player_top = player_y_px - PLAYER_HEIGHT;
            let item_left = item_x_px - 7;
            let item_right = item_x_px + 7;
            let item_top = item_y_px - PLAYER_HEIGHT;
            let item_bottom = item_y_px;
            let overlap_x = player_left <= item_right && player_right >= item_left;
            let overlap_y = player_top <= item_bottom && player_bottom >= item_top;
            let horizontal_gap_px = if overlap_x {
                0
            } else if player_right <= item_left {
                item_left - player_right
            } else {
                player_left - item_right
            };
            let vertical_gap_px = if overlap_y {
                0
            } else if player_bottom <= item_top {
                item_top - player_bottom
            } else {
                player_top - item_bottom
            };
            let dx_px = item_x_px - player_x_px;
            let dy_px = item_y_px - player_y_px;
            candidates.push(FloatingDiamondContactCandidate {
                distance_score: dx_px.abs() + dy_px.abs(),
                summary: format!(
                    "diamond@{item_x_px},{item_y_px} dx={dx_px} dy={dy_px} gap_x={horizontal_gap_px} gap_y={vertical_gap_px} overlap_x={} overlap_y={} spawn={} rem={} collected={}",
                    u8::from(overlap_x),
                    u8::from(overlap_y),
                    item.spawn_delay,
                    item.remaining,
                    u8::from(item.collected),
                ),
            });
        }

        if candidates.is_empty() {
            return None;
        }

        candidates.sort_unstable_by_key(|candidate| candidate.distance_score);
        let summaries = candidates
            .into_iter()
            .take(FLOATING_DIAMOND_CANDIDATE_LIMIT)
            .map(|candidate| candidate.summary)
            .collect::<Vec<_>>();
        Some(summaries.join("; "))
    }

    fn player_collision_bounds(&self, player: PlayerState) -> ObjectBoundsPx {
        let half_width = self.progression.current_form.half_width();
        let x_px = (player.x_pos >> 16) as i32;
        let foot_y_px = (player.y_pos >> 16) as i32;
        ObjectBoundsPx {
            left: x_px - half_width,
            right: x_px + half_width,
            top: foot_y_px - PLAYER_HEIGHT,
            bottom: foot_y_px,
        }
    }

    pub fn game_mode(&self) -> GameMode {
        if matches!(
            self.player_lifecycle,
            PlayerLifecycle::Dying { .. } | PlayerLifecycle::Respawning { .. }
        ) {
            return GameMode::LevelTransition;
        }
        if matches!(self.boot_state, BootState::TitleScreen { .. })
            && self.demo_auto_timeout_handoff_active
        {
            return GameMode::SegaScreen;
        }
        match self.boot_state {
            BootState::SegaScreen { .. } => GameMode::SegaScreen,
            BootState::IntroSequence { segment: 0, .. } => GameMode::IntroSequence1,
            BootState::IntroSequence { segment: 1, .. } => GameMode::IntroSequence2,
            BootState::IntroSequence { segment: 2, .. } => GameMode::IntroSequence3,
            BootState::IntroSequence { segment: 3, .. } => GameMode::IntroSequence4,
            BootState::IntroSequence { .. } => GameMode::IntroSequence5,
            BootState::IntroSequence6 { .. } => GameMode::IntroSequence6,
            BootState::TitleScreen { .. } => GameMode::TitleScreen,
            BootState::OptionsMenu { .. } | BootState::SoundTest { .. } => GameMode::OptionsMenu,
            BootState::LevelSkipHold => GameMode::LevelTransition,
            BootState::StartingLevel { .. } | BootState::InGame => GameMode::InGame,
            BootState::DemoPlay { .. } => GameMode::DemoPlay,
            BootState::EndingSequence { .. } => GameMode::EndingSequence,
        }
    }

    pub(crate) fn current_player_state(&self) -> Option<PlayerState> {
        if self.level_skip_resume_hold {
            return self.level_skip_resume_player_state();
        }
        let demo_use_demo_2 = self.demo_uses_second_recording();
        match self.boot_state {
            BootState::SegaScreen { .. } => None,
            BootState::IntroSequence {
                segment: 2,
                elapsed,
                ..
            } if elapsed >= INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES => {
                Some(PlayerState::intro_sequence_3_pose_for_roster(
                    self.active_intro_sequence_3_roster_index,
                ))
            }
            // ASM loc_1B96A writes `Game_Mode=$20`, but Load_IntroSequence4 /
            // sub_1B988 do not clear the old intro-sequence-3 roster
            // immediately. Oracle timeout-restart traces keep the previous
            // roster pose visible through the early intro-sequence-4 frames.
            BootState::IntroSequence { segment: 3, .. }
                if self.intro_sequence_4_uses_timeout_restart_intro_sequence_3_pose() =>
            {
                Some(PlayerState::intro_sequence_3_pose_for_roster(
                    self.active_intro_sequence_3_roster_index,
                ))
            }
            BootState::IntroSequence { segment: 4, .. }
                if self.intro_sequence_5_uses_timeout_restart_slot_2_pose() =>
            {
                Some(PlayerState::intro_sequence_5_slot_2_pose())
            }
            BootState::IntroSequence6 {
                carry_intro_sequence_5_slot_2_pose: true,
                ..
            } if self.title_screen_elapsed_frames
                < TIMEOUT_RESTART_INTRO_SEQUENCE_6_SLOT_2_POSE_CARRY_FRAMES =>
            {
                // ASM Load_IntroSequence6 swaps straight to `loc_1BAB0`,
                // `loc_1B37C`, and `sub_1B43A`; the old loc_1BCD0 slot-2 pose
                // only survives as stale WRAM for the brief oracle-backed
                // hand-off window before the rebuilt title shell clears it.
                Some(PlayerState::intro_sequence_5_slot_2_pose())
            }
            BootState::IntroSequence6 {
                carry_timeout_restart_hidden_camera_wrap: false,
                ..
            } if self.title_screen_elapsed_frames
                >= TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_TITLE_POSE_VISIBLE_ELAPSED_FRAMES =>
            {
                // ASM loc_1B5CE still owns the standard title-pose write on
                // the later mode-$28 revisit, but oracle keeps Kid hidden for
                // three extra local frames after the delayed `$FFD00000`
                // camera-ramp lineage before exposing `x=$2E`, `y=$8C`.
                Some(PlayerState::title_pose())
            }
            BootState::IntroSequence6 {
                carry_timeout_restart_hidden_camera_wrap: true,
                ..
            } if self.title_screen_elapsed_frames
                >= TIMEOUT_RESTART_INTRO_SEQUENCE_6_TITLE_POSE_VISIBLE_ELAPSED_FRAMES =>
            {
                // ASM loc_1B5CE: once the mode-$28 title-shell rebuild reaches
                // `move.w #$2E,x_pos(a3); move.w #$8C,y_pos(a3)`, oracle starts
                // exposing the standard title pose even though Game_Mode stays
                // at `$28` until Obj_TitleMenu later reaches loc_1BB02.
                Some(PlayerState::title_pose())
            }
            BootState::IntroSequence6 { .. } => None,
            BootState::IntroSequence { .. } => None,
            BootState::TitleScreen { .. } if self.title_uses_full_demo_snapshot() => {
                self.opening_slice.map(|slice| slice.player)
            }
            BootState::TitleScreen { .. }
                if self.title_uses_timeout_restart_intro_sequence_3_pose() =>
            {
                Some(PlayerState::intro_sequence_3_pose_for_roster(
                    self.active_intro_sequence_3_roster_index,
                ))
            }
            BootState::TitleScreen { .. }
                if self.title_uses_timeout_restart_intro_sequence_5_pose() =>
            {
                Some(PlayerState::intro_sequence_5_slot_2_pose())
            }
            BootState::TitleScreen { .. } if self.title_hides_demo_return_player() => None,
            BootState::TitleScreen { .. }
            | BootState::OptionsMenu { .. }
            | BootState::SoundTest { .. } => {
                self.title_screen_ready().then(PlayerState::title_pose)
            }
            BootState::EndingSequence { .. } => Some(PlayerState::title_pose()),
            BootState::DemoPlay { frame_counter, .. }
                if frame_counter < demo::DEMO_TITLE_POSE_FRAMES =>
            {
                Some(PlayerState::title_pose())
            }
            BootState::DemoPlay { frame_counter, .. }
                if demo_use_demo_2.is_some_and(|use_demo_2| {
                    frame_counter < self.demo_gameplay_start_frame(use_demo_2)
                }) =>
            {
                None
            }
            BootState::LevelSkipHold => self.level_skip_player_state(),
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. } => {
                if let Some(snapshot) = self.death_shell_snapshot.as_ref()
                    && self.dying_hidden_shell_still_visible()
                {
                    Some(snapshot.frozen_player)
                } else if let Some(player_state) = self.death_respawn_bootstrap_player_state() {
                    Some(player_state)
                } else if matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. }) {
                    None
                } else {
                    self.opening_slice.map(|slice| slice.player)
                }
            }
        }
    }

    pub(crate) fn current_camera_state(&self) -> CameraState {
        if self.level_skip_resume_hold {
            return self.level_skip_resume_camera_state();
        }
        if matches!(self.boot_state, BootState::LevelSkipHold) {
            return self.level_skip_camera_state();
        }
        if self.intro_sequence_4_uses_timeout_restart_camera_handoff() {
            return CameraState {
                x_pos: 0,
                y_pos: 0x0004_0000,
            };
        }
        if matches!(self.boot_state, BootState::TitleScreen { .. })
            && self.timeout_restart_title_intro_sequence_4_camera_carry
            && self.title_screen_elapsed_frames
                < TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_4_CAMERA_CARRY_FRAMES
        {
            // ASM sub_1C246 jumps straight to `Game_Mode=$2C`; oracle keeps
            // sub_1B988's `move.w #4,(Camera_Y_pos).w` result only for the
            // short title rebuild hand-off before the cleared title camera
            // takes over again on frame 32039.
            return CameraState {
                x_pos: 0,
                y_pos: 0x0004_0000,
            };
        }
        if let Some(camera_y_pos) = self.intro_sequence_6_timeout_restart_hidden_camera_y() {
            return CameraState {
                x_pos: 0,
                y_pos: camera_y_pos,
            };
        }
        if self.title_uses_demo_snapshot() {
            return self
                .opening_slice
                .map(|slice| slice.camera)
                .unwrap_or_default();
        }
        if let Some((frame_counter, use_demo_2)) = self
            .demo_startup_frame()
            .zip(self.demo_uses_second_recording())
            && frame_counter < self.demo_gameplay_start_frame(use_demo_2)
        {
            let camera_y_pos = if !use_demo_2 {
                if frame_counter == 60 && self.demo_uses_intro_sequence_6_hidden_camera_wrap {
                    demo::DEMO_1_INTRO_SEQUENCE_6_CAMERA_WRAP_Y_FP
                } else if frame_counter == 60 && self.demo_uses_demo_title_return_camera_wrap {
                    // ASM `loc_119D8 -> loc_6E2 -> Obj_TitleMenu loc_1BB02 ->
                    // Load_DemoPlay -> Load_InGame` keeps the later demo-1
                    // startup on the same wrapped preload row that oracle
                    // shows after the generic demo-title return.
                    0xFF20_0000
                } else if frame_counter == demo::DEMO_1_GAMEPLAY_START_FRAME
                    && self.demo_uses_intro_sequence_6_hidden_camera_wrap
                {
                    // ASM `loc_1BB02 -> Load_DemoPlay -> Load_InGame` has
                    // already advanced the hidden Knights Isle bootstrap, but
                    // oracle keeps the timeout-restart `IntroSequence6`
                    // lineage on the last `$00E0_0000` hidden row for one
                    // extra trace frame before the visible bootstrap starts.
                    0x00E0_0000
                } else if frame_counter == 60 && self.demo_uses_start_skip_intro_return_camera_wrap
                {
                    // ASM sub_1B8F4 / loc_1B920 reach title via sub_1C246's
                    // deferred Start skip, and the later Obj_TitleMenu
                    // loc_1BB02 -> Load_DemoPlay -> Load_InGame hand-off keeps
                    // a different wrapped hidden camera row than cold boot.
                    0xFF20_0000
                } else if let Some(camera_y_pos) = demo::demo_1_hidden_camera_y(frame_counter) {
                    camera_y_pos
                } else {
                    self.opening_slice
                        .map(|slice| slice.camera.y_pos)
                        .unwrap_or_default()
                }
            } else if frame_counter == demo::DEMO_CAMERA_PRELOAD_FRAME {
                // ASM Obj_TitleMenu loc_1BB02 flips into `Game_Mode=$10`, and
                // the following `Load_DemoPlay -> Load_InGame` hand-off
                // exposes one hidden camera preload row before Load_InGame's
                // `$02A0_0000` respawn camera snap takes over on the next
                // frame. Oracle WRAM keeps the later Start-skip intro-return
                // lineage and the later IntroSequence6 revisit that still
                // carries `loc_1BCD0`'s slot-2 pose eight pixels lower than
                // the plain timeout-restart preload row here.
                if self.timeout_restart_title_intro_sequence_5_uses_intro_return_hidden_delay
                    || self.demo_uses_intro_sequence_6_later_demo_2_preload_row
                {
                    demo::DEMO_2_INTRO_RETURN_CAMERA_PRELOAD_Y_FP
                } else {
                    demo::DEMO_2_CAMERA_PRELOAD_Y_FP
                }
            } else if frame_counter >= demo::DEMO_CAMERA_HOLD_START_FRAME {
                0x02A0_0000
            } else {
                0
            };
            return CameraState {
                x_pos: 0,
                y_pos: camera_y_pos,
            };
        }
        if let Some(snapshot) = self.death_shell_snapshot.as_ref() {
            if matches!(self.player_lifecycle, PlayerLifecycle::DeathWindup { .. }) {
                return snapshot.frozen_camera;
            }
            if matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. })
                && self.dying_hidden_camera_still_visible()
            {
                // ASM `lose_life -> Teleport -> loc_6E2`: after the hidden
                // shell camera window ends, the trace must expose the cleared
                // transition camera until the respawn bootstrap takes over.
                return snapshot.frozen_camera;
            }
        }
        if matches!(self.player_lifecycle, PlayerLifecycle::Respawning { .. }) {
            // ASM `lose_life -> Teleport -> loc_6E2`: mode 8 clears the
            // transition camera before the hidden respawn bootstrap later
            // rebuilds gameplay state.
            return CameraState::default();
        }
        if matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. }) {
            CameraState::default()
        } else {
            self.opening_slice
                .map(|slice| slice.camera)
                .unwrap_or_default()
        }
    }

    pub(crate) fn active_objects(&self) -> Vec<NativeObjectSnapshot> {
        if self.level_skip_resume_hold {
            return self.level_skip_resume_active_objects();
        }
        if matches!(self.boot_state, BootState::LevelSkipHold) {
            return self.level_skip_active_objects();
        }
        if self.title_uses_full_demo_snapshot() {
            let mut objects = Vec::new();
            if let Some(player_state) = self.current_player_state() {
                objects.push(NativeObjectSnapshot {
                    kind: NativeObjectKind::Player,
                    x_pos: player_state.x_pos,
                    y_pos: player_state.y_pos,
                    layout_index: None,
                    x_velocity: None,
                    y_velocity: None,
                    facing: None,
                    on_ground: None,
                    hit_points: None,
                    damage_flash: false,
                    palette_variant: 0,
                    stationary: false,
                    shared_walker_state: None,
                    shared_walker_patrol_timer: None,
                    shared_walker_patrol_short_next: None,
                    walk_phase: None,
                    wait_timer: None,
                    debug_fields: BTreeMap::new(),
                });
            }
            objects.extend(self.active_enemy_roster.iter().cloned());
            objects.extend(self.active_platform_roster.iter().cloned());
            return objects;
        }
        if let Some((frame_counter, use_demo_2)) = self
            .demo_startup_frame()
            .zip(self.demo_uses_second_recording())
            && frame_counter < self.demo_gameplay_start_frame(use_demo_2)
        {
            return Vec::new();
        }
        if let Some(snapshot) = self.death_shell_snapshot.as_ref() {
            if matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. }) {
                if self.dying_hidden_shell_still_visible() {
                    return snapshot.frozen_active_objects.clone();
                }
                return snapshot
                    .frozen_active_objects
                    .iter()
                    .filter(|object| !matches!(object.kind, NativeObjectKind::Player))
                    .cloned()
                    .collect();
            }
            if matches!(self.player_lifecycle, PlayerLifecycle::DeathWindup { .. }) {
                let mut objects = snapshot.frozen_active_objects.clone();
                if let Some(player_state) = self.opening_slice.map(|slice| slice.player)
                    && let Some(player_object) = objects
                        .iter_mut()
                        .find(|object| matches!(object.kind, NativeObjectKind::Player))
                {
                    player_object.x_pos = player_state.x_pos;
                    player_object.y_pos = player_state.y_pos;
                }
                return objects;
            }
        }
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) || matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. })
        {
            return Vec::new();
        }

        let mut objects = Vec::new();
        if let Some(player_state) = self.current_player_state() {
            objects.push(NativeObjectSnapshot {
                kind: NativeObjectKind::Player,
                x_pos: player_state.x_pos,
                y_pos: player_state.y_pos,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                wait_timer: None,
                debug_fields: BTreeMap::new(),
            });
        }
        objects.extend(self.active_enemy_roster.iter().cloned());
        objects.extend(self.active_platform_roster.iter().cloned());
        objects
    }

    pub(crate) fn object_counts_from_roster(
        &self,
        active_objects: &[NativeObjectSnapshot],
        camera_x_pos: u32,
        camera_y_pos: u32,
    ) -> (u16, u16) {
        if self.level_skip_resume_hold {
            return self.level_skip_resume_object_counts();
        }
        if matches!(self.boot_state, BootState::LevelSkipHold) {
            return self.level_skip_object_counts();
        }
        if let Some((frame_counter, use_demo_2)) = self
            .demo_startup_frame()
            .zip(self.demo_uses_second_recording())
            && frame_counter <= demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME
        {
            return if frame_counter <= demo::DEMO_LEVEL_SWITCH_FRAME {
                (4, 4)
            } else if frame_counter >= self.demo_gameplay_start_frame(use_demo_2) {
                (5, 4)
            } else if frame_counter >= demo::DEMO_OBJECT_WARMUP_FRAME {
                (5, 2)
            } else {
                (1, 0)
            };
        }
        if self.title_uses_full_demo_snapshot() {
            let roster_count = active_objects.len() as u16;
            // ORIGINAL_QUIRK: the timeout/abort title rebuild keeps four fixed
            // title objects live alongside the preserved gameplay roster while
            // the demo snapshot is still being shown.
            return (roster_count + 4, roster_count + 4);
        }
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return if matches!(
                self.boot_state,
                BootState::TitleScreen { .. }
                    | BootState::OptionsMenu { .. }
                    | BootState::SoundTest { .. }
            ) && self.title_screen_ready()
            {
                (4, 4)
            } else {
                (0, 0)
            };
        }
        if let Some(snapshot) = self.death_shell_snapshot.as_ref()
            && (matches!(self.player_lifecycle, PlayerLifecycle::DeathWindup { .. })
                || matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. }))
        {
            return (
                snapshot.frozen_number_objects,
                snapshot.frozen_number_gfx_objects,
            );
        }
        if matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. }) {
            return (0, 0);
        }
        if let Some(counts) = self.death_respawn_bootstrap_object_counts() {
            return counts;
        }

        let roster_count = active_objects.len() as u16;
        if let Some(counts) = opening_level_one_object_counts(
            self.current_level,
            roster_count,
            camera_x_pos,
            camera_y_pos,
        ) {
            return counts;
        }
        // The original game keeps 2 fixed system objects (game manager
        // and HUD/state). An additional palette-rotation object is
        // allocated for Mountain (fg=7), Cave (fg=8), or Island (bg=4)
        // themes. The native roster already includes the player.
        let extra = self
            .level_header(self.current_level)
            .map(|h| {
                if h.level_theme == 7 || h.level_theme == 8 || h.background_theme == 4 {
                    1u16
                } else {
                    0
                }
            })
            .unwrap_or(0);
        (roster_count + 2 + extra, roster_count + 2 + extra)
    }

    fn death_respawn_bootstrap_elapsed_frames(&self) -> Option<usize> {
        match self.player_lifecycle {
            PlayerLifecycle::Respawning { remaining_frames } => {
                Some(PLAYER_DEATH_RESPAWN_SETTLE_FRAMES.saturating_sub(remaining_frames))
            }
            _ => None,
        }
    }

    fn death_respawn_bootstrap_player_state(&self) -> Option<PlayerState> {
        let elapsed = self.death_respawn_bootstrap_elapsed_frames()?;
        if self.current_level != LevelId(0) {
            return None;
        }
        if elapsed < PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_ANIMATION_FRAMES {
            let elapsed_i64 = i64::try_from(elapsed).unwrap_or(i64::MAX);
            let x_pos = i64::from(PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_INITIAL_X_FP)
                + i64::from(PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_X_STEP_FP) * elapsed_i64;
            let y_pos = i64::from(PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_INITIAL_Y_FP)
                + i64::from(PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_Y_STEP_FP) * elapsed_i64;
            return Some(PlayerState {
                x_pos: x_pos as u32,
                y_pos: y_pos as u32,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            });
        }

        Some(PlayerState {
            x_pos: PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_HOLD_X_FP,
            y_pos: PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_HOLD_Y_FP,
            x_velocity: 0,
            y_velocity: 0,
            facing: Direction::Right,
            on_ground: true,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        })
    }

    fn death_respawn_bootstrap_object_counts(&self) -> Option<(u16, u16)> {
        let elapsed = self.death_respawn_bootstrap_elapsed_frames()?;
        if self.current_level != LevelId(0) {
            return None;
        }
        if elapsed == 0 {
            return Some((15, 13));
        }
        Some((15, 14))
    }
}

fn kid_pre_physics_state_debug(player: PlayerState) -> String {
    format!(
        "x={:#010x}/y={:#010x}/x_vel={:#010x}/y_vel={:#010x}",
        player.x_pos, player.y_pos, player.x_velocity as u32, player.y_velocity as u32,
    )
}

fn enemy_collision_bounds_px(enemy: &NativeObjectSnapshot) -> Option<ObjectBoundsPx> {
    let NativeObjectKind::Enemy { .. } = enemy.kind else {
        return None;
    };
    let x_px = (enemy.x_pos >> 16) as i32;
    let y_px = (enemy.y_pos >> 16) as i32;
    let bounds = enemy_contact_bounds(enemy);
    Some(ObjectBoundsPx {
        left: x_px + bounds.left_px as i32,
        right: x_px + bounds.right_px as i32,
        top: y_px + bounds.top_px as i32,
        bottom: y_px + bounds.bottom_px as i32,
    })
}

fn platform_collision_bounds_px(platform: ActivePlatform) -> ObjectBoundsPx {
    let x_px = (platform.x_pos >> 16) as i32;
    let y_px = (platform.y_pos >> 16) as i32;
    ObjectBoundsPx {
        left: x_px,
        right: x_px + i32::from(platform.width_px),
        top: y_px,
        bottom: y_px + i32::from(platform.height_px),
    }
}

fn format_kid_floor_probe(probe: terrain::FloorProbe) -> String {
    format!(
        "ground={}/y={}/cell={},{} collision={:?} block={:?}",
        u8::from(probe.on_ground),
        probe.floor_y_px,
        probe.cell_x,
        probe.cell_y,
        probe.cell.collision,
        probe.cell.block_type,
    )
}

fn direction_debug_label(direction: Option<Direction>) -> &'static str {
    match direction {
        Some(Direction::Left) => "left",
        Some(Direction::Right) => "right",
        None => "none",
    }
}

fn kid_flip_follow_on_stage_debug(stage: KidFlipFollowOnStageState) -> String {
    format!(
        "x_vel={:#010x}/y_vel={:#010x}",
        stage.x_velocity as u32, stage.y_velocity as u32
    )
}

fn wall_probe_debug_summary(probe: Option<terrain::WallProbe>) -> String {
    match probe {
        Some(probe) => format!(
            "hit={}/x={}/cell={},{} collision={} block={} has_block={}",
            u8::from(probe.hit),
            probe.wall_x_px,
            probe.cell_x,
            probe.cell_y,
            collision_kind_debug_label(probe.cell.collision),
            block_type_debug_label(probe.cell.block_type),
            u8::from(probe.cell.has_block),
        ),
        None => String::from("hit=0/x=0/cell=0,0 collision=empty block=None has_block=0"),
    }
}

fn collision_kind_debug_label(collision: terrain::CollisionKind) -> &'static str {
    match collision {
        terrain::CollisionKind::Empty => "empty",
        terrain::CollisionKind::HiddenBlock => "hidden_block",
        terrain::CollisionKind::SlopeDown => "slope_down",
        terrain::CollisionKind::SlopeUp => "slope_up",
        terrain::CollisionKind::Solid => "solid",
        terrain::CollisionKind::Spikes => "spikes",
    }
}

fn block_type_debug_label(block_type: terrain::BlockType) -> &'static str {
    match block_type {
        terrain::BlockType::Smash => "Smash",
        terrain::BlockType::Prize => "Prize",
        terrain::BlockType::Ice => "Ice",
        terrain::BlockType::SolidGhost => "SolidGhost",
        terrain::BlockType::Teleporter => "Teleporter",
        terrain::BlockType::Metal => "Metal",
        terrain::BlockType::Rubber => "Rubber",
        terrain::BlockType::Shifting => "Shifting",
        terrain::BlockType::Shooter => "Shooter",
        terrain::BlockType::Mushroom => "Mushroom",
        terrain::BlockType::Evanescent => "Evanescent",
        terrain::BlockType::Lift => "Lift",
        terrain::BlockType::Drill => "Drill",
        terrain::BlockType::Deleted => "Deleted",
        terrain::BlockType::None => "None",
    }
}

fn block_grid_source_debug_label(source: BlockGridSource) -> &'static str {
    match source {
        BlockGridSource::None => "none",
        BlockGridSource::Manual => "manual",
        BlockGridSource::Generated => "generated",
    }
}

fn terrain_cell_debug_summary(cell: terrain::Cell) -> String {
    format!(
        "{},{},skin=0x{:02X}",
        collision_kind_debug_label(cell.collision),
        block_type_debug_label(cell.block_type),
        cell.skin,
    )
}

fn player_lifecycle_debug_label(lifecycle: PlayerLifecycle) -> &'static str {
    match lifecycle {
        PlayerLifecycle::Alive => "alive",
        PlayerLifecycle::DeathWindup { .. } => "death_windup",
        PlayerLifecycle::Dying { .. } => "dying",
        PlayerLifecycle::Respawning { .. } => "respawning",
        PlayerLifecycle::LevelComplete { .. } => "level_complete",
        PlayerLifecycle::GameOver { .. } => "game_over",
        PlayerLifecycle::ContinueCountdown { .. } => "continue_countdown",
    }
}

fn kid_object_support_candidate(
    label: String,
    player_x_px: i32,
    player_foot_y_px: i32,
    player_bounds: ObjectBoundsPx,
    object_bounds: ObjectBoundsPx,
) -> Option<KidObjectSupportCandidate> {
    let object_centre_x_px = (object_bounds.left + object_bounds.right) / 2;
    let dx_px = object_centre_x_px - player_x_px;
    let top_gap_px = player_foot_y_px - object_bounds.top;
    if dx_px.abs() > KID_OBJECT_SUPPORT_HORIZONTAL_WINDOW_PX
        || top_gap_px.abs() > KID_OBJECT_SUPPORT_VERTICAL_WINDOW_PX
    {
        return None;
    }

    let x_overlap_px =
        player_bounds.right.min(object_bounds.right) - player_bounds.left.max(object_bounds.left);
    let y_overlap_px =
        player_bounds.bottom.min(object_bounds.bottom) - player_bounds.top.max(object_bounds.top);
    let relation = classify_kid_object_relation(player_bounds, object_bounds);
    let relation_label = relation
        .map(KidObjectRelation::as_debug_label)
        .unwrap_or("none");
    let support = relation == Some(KidObjectRelation::Below);
    Some(KidObjectSupportCandidate {
        abs_top_gap_px: top_gap_px.abs(),
        abs_dx_px: dx_px.abs(),
        summary: format!(
            "{label} dx={dx_px} top_gap={top_gap_px} x_overlap={x_overlap_px} y_overlap={y_overlap_px} relation={relation_label} support={}",
            u8::from(support)
        ),
    })
}

fn classify_kid_object_relation(
    player_bounds: ObjectBoundsPx,
    object_bounds: ObjectBoundsPx,
) -> Option<KidObjectRelation> {
    let x_overlap_px =
        player_bounds.right.min(object_bounds.right) - player_bounds.left.max(object_bounds.left);
    let y_overlap_px =
        player_bounds.bottom.min(object_bounds.bottom) - player_bounds.top.max(object_bounds.top);
    if x_overlap_px <= 0 || y_overlap_px <= 0 {
        return None;
    }

    let from_top = player_bounds.bottom - object_bounds.top;
    let from_bottom = object_bounds.bottom - player_bounds.top;
    let from_left = player_bounds.right - object_bounds.left;
    let from_right = object_bounds.right - player_bounds.left;
    let min_penetration = from_top.min(from_bottom).min(from_left).min(from_right);

    if min_penetration == from_top {
        Some(KidObjectRelation::Below)
    } else if min_penetration == from_bottom {
        Some(KidObjectRelation::Above)
    } else if min_penetration == from_left {
        Some(KidObjectRelation::Right)
    } else {
        Some(KidObjectRelation::Left)
    }
}

pub(crate) fn opening_level_one_object_counts(
    level_id: LevelId,
    roster_count: u16,
    camera_x_pos: u32,
    camera_y_pos: u32,
) -> Option<(u16, u16)> {
    if level_id != LevelId(0) || camera_x_pos > 0x0200_0000 {
        return None;
    }
    if camera_x_pos == 0 && camera_y_pos > OPENING_GROUNDED_CAMERA_Y_FP + 0x0004_0000 {
        return Some((roster_count + 2, roster_count + 2));
    }
    if camera_x_pos == 0 {
        return Some((roster_count + 1, roster_count + 2));
    }
    Some((roster_count + 2, roster_count + 3))
}
