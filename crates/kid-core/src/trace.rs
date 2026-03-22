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
    fn title_uses_demo_snapshot(&self) -> bool {
        matches!(self.boot_state, BootState::TitleScreen { .. })
            && self.preserve_demo_title_snapshot
    }

    fn title_uses_full_demo_snapshot(&self) -> bool {
        self.title_uses_demo_snapshot()
            && self.title_screen_elapsed_frames < self.demo_title_full_snapshot_frames()
    }

    fn title_hides_demo_return_player(&self) -> bool {
        matches!(self.boot_state, BootState::TitleScreen { .. })
            && self.hide_demo_return_title_player
            && self.title_screen_elapsed_frames < DEMO_TITLE_PLAYER_HIDDEN_FRAMES
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
            kid_facing: player_state.map(|state| state.facing),
            kid_on_ground: player_state.map(|state| state.on_ground),
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
            invulnerable_frames: self.progression.invulnerable_frames,
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
            BootState::SegaScreen { .. } | BootState::IntroSequence { .. } => None,
            BootState::TitleScreen { .. } if self.title_uses_full_demo_snapshot() => {
                self.opening_slice.map(|slice| slice.player)
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
                    frame_counter < demo::gameplay_start_frame(use_demo_2)
                }) =>
            {
                None
            }
            BootState::LevelSkipHold => self.level_skip_player_state(),
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. } => {
                if matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. }) {
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
        if self.title_uses_demo_snapshot() {
            return self
                .opening_slice
                .map(|slice| slice.camera)
                .unwrap_or_default();
        }
        if let Some((frame_counter, use_demo_2)) = self
            .demo_startup_frame()
            .zip(self.demo_uses_second_recording())
            && frame_counter < demo::gameplay_start_frame(use_demo_2)
        {
            let camera_y_pos = if !use_demo_2 {
                if let Some(camera_y_pos) = demo::demo_1_hidden_camera_y(frame_counter) {
                    camera_y_pos
                } else {
                    self.opening_slice
                        .map(|slice| slice.camera.y_pos)
                        .unwrap_or_default()
                }
            } else if frame_counter == demo::DEMO_CAMERA_PRELOAD_FRAME {
                0x0228_0000
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
            && frame_counter < demo::gameplay_start_frame(use_demo_2)
        {
            return Vec::new();
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
            } else if frame_counter >= demo::gameplay_start_frame(use_demo_2) {
                (5, 4)
            } else if frame_counter >= demo::DEMO_OBJECT_WARMUP_FRAME {
                (5, 2)
            } else {
                (1, 0)
            };
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
        if matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. }) {
            return (0, 0);
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
