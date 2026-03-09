use anyhow::{Result, anyhow, bail};
use font8x8::{BASIC_FONTS, UnicodeFonts};
use kid_core::{
    FrameInput, GameMode, NativeObjectSnapshot, NativeRuntime, NativeTraceFrame, PlayerForm,
    terrain::{BlockGrid, BlockType, Cell, CollisionKind},
};
use kid_data::AssetManifest;
use sdl2::controller::{Button, GameController};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::video::FullscreenType;
use std::thread;
use std::time::{Duration, Instant};

const FRAME_WIDTH: u32 = 320;
const FRAME_HEIGHT: u32 = 224;
const FRAME_TIME: Duration = Duration::from_micros(16_667);
const PLAYER_COLOUR: [u8; 3] = [92, 214, 104];
const ENEMY_COLOUR: [u8; 3] = [220, 90, 88];
const PLATFORM_COLOUR: [u8; 3] = [232, 176, 64];
const GUIDE_COLOUR: [u8; 3] = [62, 84, 108];
const SKY_COLOUR: [u8; 3] = [28, 42, 56];

#[derive(Debug, Clone, Copy)]
pub struct NativePlayOptions {
    pub fullscreen: bool,
    pub scale: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Scene {
    background: [u8; 3],
    commands: Vec<DrawCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DrawCommand {
    Rect {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        colour: [u8; 3],
    },
    Text {
        x: i32,
        y: i32,
        text: String,
        colour: [u8; 3],
        scale: i32,
    },
}

pub fn run_native_player(manifest: &AssetManifest, options: NativePlayOptions) -> Result<()> {
    let sdl = sdl2::init().map_err(|error| anyhow!("failed to init SDL: {error}"))?;
    let video = sdl
        .video()
        .map_err(|error| anyhow!("failed to init SDL video: {error}"))?;
    let game_controller = sdl
        .game_controller()
        .map_err(|error| anyhow!("failed to init SDL game controller subsystem: {error}"))?;

    let usable_bounds = video
        .display_usable_bounds(0)
        .unwrap_or_else(|_| Rect::new(0, 0, 1280, 720));
    let initial_scale = display_scale(options.scale, usable_bounds)?;
    let window = video
        .window(
            "Kid Chameleon Native",
            FRAME_WIDTH * initial_scale,
            FRAME_HEIGHT * initial_scale,
        )
        .position_centered()
        .resizable()
        .build()
        .map_err(|error| anyhow!("failed to create SDL window: {error}"))?;
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|error| anyhow!("failed to create SDL canvas: {error}"))?;
    canvas
        .set_logical_size(FRAME_WIDTH, FRAME_HEIGHT)
        .map_err(|error| anyhow!("failed to set SDL logical size: {error}"))?;
    canvas
        .set_integer_scale(true)
        .map_err(|error| anyhow!("failed to enable SDL integer scaling: {error}"))?;

    let mut fullscreen = options.fullscreen;
    apply_window_mode(&mut canvas, &video, fullscreen, options.scale)?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, FRAME_WIDTH, FRAME_HEIGHT)
        .map_err(|error| anyhow!("failed to create SDL texture: {error}"))?;
    let mut events = sdl
        .event_pump()
        .map_err(|error| anyhow!("failed to create SDL event pump: {error}"))?;
    let mut controller = open_first_controller(&game_controller)?;

    let mut runtime = NativeRuntime::new(manifest.clone());
    try_load_block_grid(&mut runtime, manifest, 0x00);
    let mut trace = runtime.trace_frame();
    let mut hud_visible = true;
    let mut frame = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];

    let scene = build_scene(&trace, hud_visible, runtime.block_grid());
    render_scene(&scene, &mut frame);
    texture
        .update(None, &frame, (FRAME_WIDTH as usize) * 3)
        .map_err(|error| anyhow!("failed to update SDL texture: {error}"))?;
    canvas.clear();
    canvas
        .copy(&texture, None, None)
        .map_err(|error| anyhow!("failed to copy SDL texture: {error}"))?;
    canvas.present();

    'running: loop {
        let frame_started = Instant::now();
        let mut toggle_fullscreen = false;
        let mut reset_runtime = false;

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::ControllerDeviceAdded { .. } => {
                    if controller.is_none() {
                        controller = open_first_controller(&game_controller)?;
                    }
                }
                Event::ControllerDeviceRemoved { .. } => {
                    controller = controller.filter(|gamepad| gamepad.attached());
                    if controller.is_none() {
                        controller = open_first_controller(&game_controller)?;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    repeat: false,
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::F11),
                    repeat: false,
                    ..
                } => toggle_fullscreen = true,
                Event::KeyDown {
                    keycode: Some(Keycode::Tab),
                    repeat: false,
                    ..
                } => hud_visible = !hud_visible,
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    repeat: false,
                    ..
                } => reset_runtime = true,
                _ => {}
            }
        }

        if toggle_fullscreen {
            fullscreen = !fullscreen;
            apply_window_mode(&mut canvas, &video, fullscreen, options.scale)?;
        }

        if reset_runtime {
            runtime = NativeRuntime::new(manifest.clone());
            trace = runtime.trace_frame();
        } else {
            let buttons = keyboard_buttons(&events) | controller_buttons(controller.as_ref());
            trace = runtime.step_frame(FrameInput::from_mask(buttons));
        }

        let scene = build_scene(&trace, hud_visible, runtime.block_grid());
        render_scene(&scene, &mut frame);
        texture
            .update(None, &frame, (FRAME_WIDTH as usize) * 3)
            .map_err(|error| anyhow!("failed to update SDL texture: {error}"))?;
        canvas.clear();
        canvas
            .copy(&texture, None, None)
            .map_err(|error| anyhow!("failed to copy SDL texture: {error}"))?;
        canvas.present();

        let elapsed = frame_started.elapsed();
        if elapsed < FRAME_TIME {
            thread::sleep(FRAME_TIME - elapsed);
        }
    }

    Ok(())
}

fn build_scene(
    trace: &NativeTraceFrame,
    hud_visible: bool,
    block_grid: Option<&BlockGrid>,
) -> Scene {
    match trace.game_mode {
        GameMode::InGame => build_game_scene(trace, hud_visible, block_grid),
        _ => build_status_scene(trace, hud_visible),
    }
}

fn build_status_scene(trace: &NativeTraceFrame, hud_visible: bool) -> Scene {
    let mut commands = Vec::new();
    push_panel(
        &mut commands,
        16,
        16,
        288,
        132,
        [10, 16, 24],
        [205, 196, 173],
    );
    commands.push(text(30, 28, "Kid Chameleon Native", [255, 240, 196], 2));
    commands.push(text(
        30,
        54,
        &format!("State {}", game_mode_label(trace.game_mode)),
        [160, 214, 255],
        1,
    ));
    commands.push(text(
        30,
        68,
        &format!(
            "Frame {}  Start presses {}",
            trace.frame_index, trace.start_press_count
        ),
        [220, 220, 220],
        1,
    ));
    commands.push(text(
        30,
        82,
        &format!(
            "Level {:02X} {}",
            trace.current_level.0,
            trace.current_level_name.as_deref().unwrap_or("")
        ),
        [220, 220, 220],
        1,
    ));
    commands.push(text(
        30,
        96,
        "Native runtime: silent debug host",
        [255, 188, 140],
        1,
    ));
    commands.push(text(
        30,
        118,
        "Enter start  Arrows move  Z jump  X special",
        [190, 190, 190],
        1,
    ));
    commands.push(text(
        30,
        130,
        "C run  R reset  Tab HUD  F11 fullscreen  Esc quit",
        [190, 190, 190],
        1,
    ));
    if hud_visible {
        commands.push(text(
            30,
            160,
            &format!(
                "Camera {:08X},{:08X}",
                trace.camera_x_pos, trace.camera_y_pos
            ),
            [160, 214, 255],
            1,
        ));
    }

    Scene {
        background: [18, 24, 32],
        commands,
    }
}

/// Map a block grid cell to a display colour based on its collision kind and block type.
fn cell_colour(cell: &Cell) -> Option<[u8; 3]> {
    if cell.collision == CollisionKind::Empty {
        return None;
    }
    if cell.has_block {
        return Some(match cell.block_type {
            BlockType::Prize => [232, 196, 56],    // yellow
            BlockType::Smash => [220, 148, 52],    // orange
            BlockType::Ice => [120, 210, 230],     // cyan
            BlockType::Rubber => [220, 130, 180],  // pink
            BlockType::Metal => [160, 170, 180],   // steel grey
            _ => [140, 140, 140],                  // generic block grey
        });
    }
    Some(match cell.collision {
        CollisionKind::Solid => [90, 100, 90],     // grey
        CollisionKind::Spikes => [200, 60, 60],    // red
        CollisionKind::SlopeUp | CollisionKind::SlopeDown => [140, 100, 60], // brown
        CollisionKind::HiddenBlock => [80, 80, 100], // dim blue-grey
        CollisionKind::Empty => unreachable!(),
    })
}

/// Push draw commands for all block grid cells visible within the camera viewport.
fn push_block_grid_cells(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    block_grid: Option<&BlockGrid>,
) {
    let Some(grid) = block_grid else { return };

    let camera_x_px = (trace.camera_x_pos >> 16) as i32;
    let camera_y_px = (trace.camera_y_pos >> 16) as i32;

    // Determine visible cell range (with 1-cell margin for partially visible cells)
    let col_start = (camera_x_px / 16).max(0) as u16;
    let col_end = ((camera_x_px + FRAME_WIDTH as i32 + 15) / 16)
        .min(i32::from(grid.width_cells)) as u16;
    let row_start = (camera_y_px / 16).max(0) as u16;
    let row_end = ((camera_y_px + FRAME_HEIGHT as i32 + 15) / 16)
        .min(i32::from(grid.height_cells)) as u16;

    for row in row_start..row_end {
        for col in col_start..col_end {
            let cell = grid.cell_at(col, row);
            if let Some(colour) = cell_colour(&cell) {
                let world_x = i32::from(col) * 16;
                let world_y = i32::from(row) * 16;
                commands.push(DrawCommand::Rect {
                    x: world_x - camera_x_px,
                    y: world_y - camera_y_px,
                    width: 16,
                    height: 16,
                    colour,
                });
            }
        }
    }
}

fn build_game_scene(
    trace: &NativeTraceFrame,
    hud_visible: bool,
    block_grid: Option<&BlockGrid>,
) -> Scene {
    let mut commands = vec![
        DrawCommand::Rect {
            x: 0,
            y: 0,
            width: FRAME_WIDTH as i32,
            height: FRAME_HEIGHT as i32,
            colour: SKY_COLOUR,
        },
        DrawCommand::Rect {
            x: 0,
            y: 0,
            width: FRAME_WIDTH as i32,
            height: 1,
            colour: GUIDE_COLOUR,
        },
        DrawCommand::Rect {
            x: 0,
            y: (FRAME_HEIGHT - 1) as i32,
            width: FRAME_WIDTH as i32,
            height: 1,
            colour: GUIDE_COLOUR,
        },
        DrawCommand::Rect {
            x: 0,
            y: 0,
            width: 1,
            height: FRAME_HEIGHT as i32,
            colour: GUIDE_COLOUR,
        },
        DrawCommand::Rect {
            x: (FRAME_WIDTH - 1) as i32,
            y: 0,
            width: 1,
            height: FRAME_HEIGHT as i32,
            colour: GUIDE_COLOUR,
        },
    ];

    push_block_grid_cells(&mut commands, trace, block_grid);

    for object in trace
        .active_objects
        .iter()
        .filter(|object| matches!(object.kind, kid_core::NativeObjectKind::Platform { .. }))
    {
        commands.push(object_rect(object, trace, 32, 8, PLATFORM_COLOUR));
    }
    for object in trace
        .active_objects
        .iter()
        .filter(|object| matches!(object.kind, kid_core::NativeObjectKind::Enemy { .. }))
    {
        commands.push(object_rect(object, trace, 16, 16, ENEMY_COLOUR));
    }
    for object in trace
        .active_objects
        .iter()
        .filter(|object| matches!(object.kind, kid_core::NativeObjectKind::Player))
    {
        commands.push(object_rect(object, trace, 16, 32, PLAYER_COLOUR));
    }

    if hud_visible {
        push_panel(&mut commands, 8, 8, 304, 44, [12, 18, 24], [160, 214, 255]);
        commands.push(text(
            16,
            16,
            &format!(
                "{}  Frame {}  Pause {}",
                game_mode_label(trace.game_mode),
                trace.frame_index,
                on_off(trace.paused)
            ),
            [255, 240, 196],
            1,
        ));
        commands.push(text(
            16,
            28,
            &format!(
                "Level {:02X} {}  Lives {}  HP {}  {}",
                trace.current_level.0,
                trace.current_level_name.as_deref().unwrap_or(""),
                trace.number_lives,
                trace.number_hitpoints,
                form_label(trace.current_helmet_id)
            ),
            [220, 220, 220],
            1,
        ));
        commands.push(text(
            16,
            40,
            &format!(
                "Camera {:08X},{:08X}  Objects {}/{}  Controls Arrows Z X C Enter R Tab Esc",
                trace.camera_x_pos,
                trace.camera_y_pos,
                trace.number_objects,
                trace.number_gfx_objects
            ),
            [190, 190, 190],
            1,
        ));
    }

    Scene {
        background: SKY_COLOUR,
        commands,
    }
}

fn object_rect(
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    width: i32,
    height: i32,
    colour: [u8; 3],
) -> DrawCommand {
    DrawCommand::Rect {
        x: screen_coord(object.x_pos, trace.camera_x_pos),
        y: screen_coord(object.y_pos, trace.camera_y_pos),
        width,
        height,
        colour,
    }
}

fn screen_coord(world: u32, camera: u32) -> i32 {
    let world = i64::from(world);
    let camera = i64::from(camera);
    ((world - camera) >> 16) as i32
}

fn render_scene(scene: &Scene, rgb: &mut [u8]) {
    clear_frame(rgb, scene.background);
    for command in &scene.commands {
        match command {
            DrawCommand::Rect {
                x,
                y,
                width,
                height,
                colour,
            } => fill_rect(rgb, *x, *y, *width, *height, *colour),
            DrawCommand::Text {
                x,
                y,
                text,
                colour,
                scale,
            } => draw_text(rgb, *x, *y, text, *colour, *scale),
        }
    }
}

fn clear_frame(rgb: &mut [u8], colour: [u8; 3]) {
    for chunk in rgb.chunks_exact_mut(3) {
        chunk.copy_from_slice(&colour);
    }
}

fn push_panel(
    commands: &mut Vec<DrawCommand>,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    fill: [u8; 3],
    border: [u8; 3],
) {
    commands.push(DrawCommand::Rect {
        x,
        y,
        width,
        height,
        colour: fill,
    });
    commands.push(DrawCommand::Rect {
        x,
        y,
        width,
        height: 2,
        colour: border,
    });
    commands.push(DrawCommand::Rect {
        x,
        y: y + height - 2,
        width,
        height: 2,
        colour: border,
    });
    commands.push(DrawCommand::Rect {
        x,
        y,
        width: 2,
        height,
        colour: border,
    });
    commands.push(DrawCommand::Rect {
        x: x + width - 2,
        y,
        width: 2,
        height,
        colour: border,
    });
}

fn text(x: i32, y: i32, value: &str, colour: [u8; 3], scale: i32) -> DrawCommand {
    DrawCommand::Text {
        x,
        y,
        text: value.to_string(),
        colour,
        scale,
    }
}

fn draw_text(rgb: &mut [u8], x: i32, y: i32, text: &str, colour: [u8; 3], scale: i32) {
    let mut cursor_x = x;
    let mut cursor_y = y;
    for ch in text.chars() {
        if ch == '\n' {
            cursor_x = x;
            cursor_y += 10 * scale;
            continue;
        }
        draw_glyph(rgb, cursor_x, cursor_y, ch, colour, scale);
        cursor_x += 8 * scale;
    }
}

fn draw_glyph(rgb: &mut [u8], x: i32, y: i32, ch: char, colour: [u8; 3], scale: i32) {
    let Some(bitmap) = BASIC_FONTS.get(ch) else {
        return;
    };
    for (row, bits) in bitmap.iter().enumerate() {
        for column in 0..8 {
            if bits & (1 << column) == 0 {
                continue;
            }
            fill_rect(
                rgb,
                x + (column * scale),
                y + ((row as i32) * scale),
                scale,
                scale,
                colour,
            );
        }
    }
}

fn fill_rect(rgb: &mut [u8], x: i32, y: i32, width: i32, height: i32, colour: [u8; 3]) {
    for py in y.max(0)..(y + height).min(FRAME_HEIGHT as i32) {
        for px in x.max(0)..(x + width).min(FRAME_WIDTH as i32) {
            let index = ((py as u32 * FRAME_WIDTH + px as u32) * 3) as usize;
            rgb[index..index + 3].copy_from_slice(&colour);
        }
    }
}

fn display_scale(requested: Option<u32>, usable_bounds: Rect) -> Result<u32> {
    match requested {
        Some(scale @ 1..=4) => Ok(scale),
        Some(scale) => bail!("unsupported native display scale {scale}; use 1-4"),
        None => {
            let width_scale = (usable_bounds.width() / FRAME_WIDTH).max(1);
            let height_scale = (usable_bounds.height() / FRAME_HEIGHT).max(1);
            Ok(width_scale.min(height_scale).clamp(1, 4))
        }
    }
}

fn apply_window_mode(
    canvas: &mut WindowCanvas,
    video: &sdl2::VideoSubsystem,
    fullscreen: bool,
    scale: Option<u32>,
) -> Result<()> {
    if fullscreen {
        canvas
            .window_mut()
            .set_fullscreen(FullscreenType::Desktop)
            .map_err(|error| anyhow!("failed to enable fullscreen: {error}"))?;
        return Ok(());
    }

    canvas
        .window_mut()
        .set_fullscreen(FullscreenType::Off)
        .map_err(|error| anyhow!("failed to disable fullscreen: {error}"))?;
    let usable_bounds = video
        .display_usable_bounds(0)
        .unwrap_or_else(|_| Rect::new(0, 0, 1280, 720));
    let scale = display_scale(scale, usable_bounds)?;
    canvas
        .window_mut()
        .set_size(FRAME_WIDTH * scale, FRAME_HEIGHT * scale)
        .map_err(|error| anyhow!("failed to resize SDL window: {error}"))?;
    canvas.window_mut().set_position(
        sdl2::video::WindowPos::Centered,
        sdl2::video::WindowPos::Centered,
    );
    Ok(())
}

fn keyboard_buttons(events: &sdl2::EventPump) -> u8 {
    let keyboard = events.keyboard_state();
    let bindings = [
        (Keycode::Up, 1 << 0),
        (Keycode::Down, 1 << 1),
        (Keycode::Left, 1 << 2),
        (Keycode::Right, 1 << 3),
        (Keycode::Z, 1 << 4),
        (Keycode::X, 1 << 5),
        (Keycode::C, 1 << 6),
        (Keycode::Return, 1 << 7),
    ];
    let mut buttons = 0u8;
    for (keycode, mask) in bindings {
        let Some(scancode) = Scancode::from_keycode(keycode) else {
            continue;
        };
        if keyboard.is_scancode_pressed(scancode) {
            buttons |= mask;
        }
    }
    buttons
}

fn controller_buttons(controller: Option<&GameController>) -> u8 {
    let Some(controller) = controller else {
        return 0;
    };

    let mut buttons = 0u8;
    if controller.button(Button::DPadUp) {
        buttons |= 1 << 0;
    }
    if controller.button(Button::DPadDown) {
        buttons |= 1 << 1;
    }
    if controller.button(Button::DPadLeft) {
        buttons |= 1 << 2;
    }
    if controller.button(Button::DPadRight) {
        buttons |= 1 << 3;
    }
    if controller.button(Button::A) {
        buttons |= 1 << 4;
    }
    if controller.button(Button::B) {
        buttons |= 1 << 5;
    }
    if controller.button(Button::X) {
        buttons |= 1 << 6;
    }
    if controller.button(Button::Start) {
        buttons |= 1 << 7;
    }
    buttons
}

fn open_first_controller(
    game_controller: &sdl2::GameControllerSubsystem,
) -> Result<Option<GameController>> {
    let joystick_count = game_controller
        .num_joysticks()
        .map_err(|error| anyhow!("failed to enumerate SDL controllers: {error}"))?;
    for index in 0..joystick_count {
        if !game_controller.is_game_controller(index) {
            continue;
        }
        let controller = game_controller
            .open(index)
            .map_err(|error| anyhow!("failed to open SDL controller {index}: {error}"))?;
        return Ok(Some(controller));
    }
    Ok(None)
}

fn game_mode_label(mode: GameMode) -> &'static str {
    match mode {
        GameMode::SegaScreen => "SegaScreen",
        GameMode::IntroSequence1 => "IntroSequence1",
        GameMode::IntroSequence2 => "IntroSequence2",
        GameMode::IntroSequence3 => "IntroSequence3",
        GameMode::TitleScreen => "TitleScreen",
        GameMode::InGame => "InGame",
    }
}

fn form_label(helmet_id: u16) -> &'static str {
    match PlayerForm::from_helmet_id(helmet_id) {
        Some(PlayerForm::TheKid) => "TheKid",
        Some(PlayerForm::Skycutter) => "Skycutter",
        Some(PlayerForm::Cyclone) => "Cyclone",
        Some(PlayerForm::RedStealth) => "RedStealth",
        Some(PlayerForm::Eyeclops) => "Eyeclops",
        Some(PlayerForm::Juggernaut) => "Juggernaut",
        Some(PlayerForm::IronKnight) => "IronKnight",
        Some(PlayerForm::Berzerker) => "Berzerker",
        Some(PlayerForm::Maniaxe) => "Maniaxe",
        Some(PlayerForm::Micromax) => "Micromax",
        None => "Unknown",
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "On" } else { "Off" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kid_core::{LevelId, NativeObjectKind};

    fn sample_trace(mode: GameMode) -> NativeTraceFrame {
        NativeTraceFrame {
            frame_index: 720,
            game_mode: mode,
            game_mode_value: mode.as_disassembly_value(),
            start_press_count: 6,
            current_player: 0,
            current_level: LevelId(0),
            player_1_level_id: LevelId(0),
            current_level_name: Some(String::from("Blue Lake Woods 1")),
            player_start_x: Some(0x0750),
            player_start_y: Some(0x00E0),
            flag_x: Some(0x0750),
            flag_y: Some(0x0300),
            level_enemy_object_count: Some(22),
            active_enemy_object_count: 1,
            active_platform_object_count: 1,
            number_lives: 3,
            number_hitpoints: 2,
            current_helmet_id: 0,
            paused: false,
            kid_x_pos: Some(0x0020_0000),
            kid_y_pos: Some(0x010F_0000),
            active_objects: Vec::new(),
            camera_x_pos: 0x0010_0000,
            camera_y_pos: 0x0008_0000,
            number_objects: 3,
            number_gfx_objects: 4,
            level_complete: false,
        }
    }

    fn has_rect(scene: &Scene, x: i32, y: i32, width: i32, height: i32, colour: [u8; 3]) -> bool {
        scene.commands.iter().any(|command| {
            matches!(
                command,
                DrawCommand::Rect {
                    x: cx,
                    y: cy,
                    width: cw,
                    height: ch,
                    colour: cc,
                } if *cx == x && *cy == y && *cw == width && *ch == height && *cc == colour
            )
        })
    }

    fn contains_text(scene: &Scene, needle: &str) -> bool {
        scene.commands.iter().any(
            |command| matches!(command, DrawCommand::Text { text, .. } if text.contains(needle)),
        )
    }

    #[test]
    fn title_scene_stays_text_only() {
        let scene = build_scene(&sample_trace(GameMode::TitleScreen), true, None);
        assert!(contains_text(&scene, "Kid Chameleon Native"));
        assert!(!scene.commands.iter().any(|command| {
            matches!(command, DrawCommand::Rect { colour, .. } if *colour == PLAYER_COLOUR || *colour == ENEMY_COLOUR || *colour == PLATFORM_COLOUR)
        }));
    }

    #[test]
    fn game_scene_places_native_objects_relative_to_camera() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_objects = vec![
            NativeObjectSnapshot {
                kind: NativeObjectKind::Platform {
                    kind: 0,
                    subtype: 0,
                },
                x_pos: 0x0030_0000,
                y_pos: 0x0018_0000,
            },
            NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy { object_type: 0x01 },
                x_pos: 0x0028_0000,
                y_pos: 0x0014_0000,
            },
            NativeObjectSnapshot {
                kind: NativeObjectKind::Player,
                x_pos: 0x0018_0000,
                y_pos: 0x0010_0000,
            },
        ];
        let scene = build_scene(&trace, true, None);
        assert!(has_rect(&scene, 32, 16, 32, 8, PLATFORM_COLOUR));
        assert!(has_rect(&scene, 24, 12, 16, 16, ENEMY_COLOUR));
        assert!(has_rect(&scene, 8, 8, 16, 32, PLAYER_COLOUR));
    }

    #[test]
    fn hud_toggle_removes_control_overlay() {
        let trace = sample_trace(GameMode::InGame);
        assert!(contains_text(&build_scene(&trace, true, None), "Controls Arrows"));
        assert!(!contains_text(
            &build_scene(&trace, false, None),
            "Controls Arrows"
        ));
    }

    #[test]
    fn offscreen_objects_clip_without_drawing() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_objects = vec![NativeObjectSnapshot {
            kind: NativeObjectKind::Player,
            x_pos: 0,
            y_pos: 0,
        }];
        trace.camera_x_pos = 0x0040_0000;
        trace.camera_y_pos = 0x0040_0000;
        let scene = build_scene(&trace, false, None);
        let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];
        render_scene(&scene, &mut rgb);
        assert!(
            !rgb.chunks_exact(3)
                .any(|pixel| pixel == PLAYER_COLOUR.as_slice())
        );
    }

    #[test]
    fn block_grid_renders_visible_cells() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;

        // 4x3 grid: top two rows empty, bottom row solid
        let mut data = vec![0u8; 4 * 3 * 2];
        for col in 0..4 {
            let offset = (2 * 4 + col) * 2;
            data[offset] = 0x60; // collision=6 (Solid)
        }
        let grid = BlockGrid::from_level_layout(&data, 4, 3);
        let solid_colour = cell_colour(&grid.cell_at(0, 2)).unwrap();

        let scene = build_scene(&trace, false, Some(&grid));
        // Bottom-row cells (row 2) should produce 16x16 rects at y=32
        for col in 0..4 {
            assert!(
                has_rect(&scene, col * 16, 32, 16, 16, solid_colour),
                "missing solid cell at col {col}"
            );
        }
        // Top-row cells should NOT produce any rects (empty)
        assert!(!scene.commands.iter().any(|command| {
            matches!(command, DrawCommand::Rect { y, height, colour, .. }
                if *y == 0 && *height == 16 && *colour == solid_colour)
        }));
    }

    #[test]
    fn block_grid_only_draws_visible_cells() {
        let mut trace = sample_trace(GameMode::InGame);
        // Camera at (320px, 0) — first 20 columns offscreen
        trace.camera_x_pos = 0x0140_0000;
        trace.camera_y_pos = 0;

        // 40x2 grid: bottom row all solid
        let w = 40u16;
        let h = 2u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        for col in 0..40 {
            let offset = (1 * 40 + col) * 2;
            data[offset] = 0x60;
        }
        let grid = BlockGrid::from_level_layout(&data, w, h);
        let solid_colour = cell_colour(&grid.cell_at(0, 1)).unwrap();

        let scene = build_scene(&trace, false, Some(&grid));
        // Cells at cols 0..19 (world x 0..320) are left of camera — should NOT appear
        let offscreen_rects = scene.commands.iter().filter(|cmd| {
            matches!(cmd, DrawCommand::Rect { x, colour, .. }
                if *colour == solid_colour && *x < -16)
        });
        assert_eq!(offscreen_rects.count(), 0, "offscreen cells should not be drawn");
    }

    #[test]
    fn destroyed_block_renders_as_empty() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;

        // 2x2 grid: one prize block at (0,0), rest empty
        let mut data = vec![0u8; 2 * 2 * 2];
        data[0] = 0xE1; // block=true, collision=Solid, block_type=Prize
        let mut grid = BlockGrid::from_level_layout(&data, 2, 2);
        let prize_colour = cell_colour(&grid.cell_at(0, 0)).unwrap();

        // Before destruction: prize cell visible
        let scene = build_scene(&trace, false, Some(&grid));
        assert!(has_rect(&scene, 0, 0, 16, 16, prize_colour));

        // Destroy the block (set to empty)
        grid.set_cell_at(0, 0, Cell::from_be_bytes(0, 0));
        let scene = build_scene(&trace, false, Some(&grid));
        assert!(!has_rect(&scene, 0, 0, 16, 16, prize_colour));
    }

    #[test]
    fn block_types_have_distinct_colours() {
        let prize = Cell::from_be_bytes(0xE1, 0); // Prize
        let smash = Cell::from_be_bytes(0xE0, 0); // Smash
        let ice = Cell::from_be_bytes(0xE2, 0);   // Ice
        let rubber = Cell::from_be_bytes(0xE6, 0); // Rubber

        let colours: Vec<_> = [prize, smash, ice, rubber]
            .iter()
            .map(|c| cell_colour(c).unwrap())
            .collect();
        // All colours should be unique
        for i in 0..colours.len() {
            for j in (i + 1)..colours.len() {
                assert_ne!(colours[i], colours[j], "block types at indices {i} and {j} share colour");
            }
        }
    }
}

/// Attempt to load a pre-dumped block grid for the given level.
/// Falls back silently to flat floor if the dump file doesn't exist.
fn try_load_block_grid(runtime: &mut NativeRuntime, manifest: &AssetManifest, level_id: u8) {
    let layout_dir = manifest.output_root.join("level-layout");
    let layout_path = layout_dir.join(format!("{:02X}.bin", level_id));
    let meta_path = layout_dir.join(format!("{:02X}.json", level_id));

    let Ok(data) = std::fs::read(&layout_path) else {
        eprintln!(
            "no block grid at {}, using flat floor (run dump-level-layout to generate)",
            layout_path.display()
        );
        return;
    };
    let Ok(meta_bytes) = std::fs::read(&meta_path) else {
        eprintln!(
            "block grid data found but no metadata at {}",
            meta_path.display()
        );
        return;
    };
    let Ok(meta) = serde_json::from_slice::<serde_json::Value>(&meta_bytes) else {
        eprintln!("failed to parse block grid metadata");
        return;
    };

    let width = meta["width_blocks"].as_u64().unwrap_or(0) as u16;
    let height = meta["height_blocks"].as_u64().unwrap_or(0) as u16;
    if width == 0 || height == 0 {
        eprintln!("invalid block grid dimensions {}x{}", width, height);
        return;
    }

    let grid = BlockGrid::from_level_layout(&data, width, height);
    eprintln!(
        "loaded block grid: {}x{} cells ({}x{} px)",
        width,
        height,
        u32::from(width) * 16,
        u32::from(height) * 16
    );
    runtime.set_block_grid(grid);
}
