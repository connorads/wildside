use super::*;

/// ASM Enemy19_Fireball_Init and loc_32C1A/loc_32C60 preserve allocation HP0
/// and consume floor/wall collisions on the following coroutine callback.
#[test]
fn gameplay_regression_fireball_settled_profile_and_collision_callbacks() {
    let Some(manifest) = cached_generated_manifest() else {
        return;
    };
    #[derive(serde::Deserialize)]
    struct Repro {
        script: Vec<u8>,
        frame_count: usize,
        warp_to_level: u8,
    }
    let repro: Repro = serde_json::from_str(include_str!(
        "../../../../tests/data/dynamic-scenarios/issue-521-fireball-idle-window.json"
    ))
    .expect("checked-in Fireball floor/wall window");
    let trace = ScriptRunner::new(manifest)
        .warp_to_level(repro.warp_to_level)
        .settle_recording_warp()
        .run(&repro.script, repro.frame_count);
    // ROM recording-settled replay witnesses before/after each collision.
    for (frame, x, y, facing) in [
        (0, 0x0091_8000, 0x00D2_D000, Direction::Right),
        (1, 0x0090_0000, 0x00D3_4000, Direction::Right),
        (37, 0x005A_0000, 0x010B_0000, Direction::Right),
        (38, 0x0058_8000, 0x0108_1E00, Direction::Right),
        (95, 0x0004_0000, 0x00CB_1C00, Direction::Right),
        (96, 0x0005_8000, 0x00CB_DA00, Direction::Left),
        (120, 0x0029_8000, 0x00F0_6A00, Direction::Left),
    ] {
        let fireball = trace[frame]
            .active_enemy_details
            .iter()
            .find(|enemy| {
                enemy.kind == NativeObjectKind::Enemy { object_type: 0x19 }
                    && enemy.layout_index == Some(0)
            })
            .expect("ROM Fireball remains active throughout the window");
        assert_eq!(fireball.x_pos, x, "frame {frame}");
        assert_eq!(fireball.y_pos, y, "frame {frame}");
        assert_eq!(fireball.facing, Some(facing), "frame {frame}");
        assert_eq!(fireball.hit_points, Some(0), "frame {frame}");
    }
}
