use super::*;

pub fn update(gm: &mut GameManager) {
    let player_pos = if let Some(player) = &gm.game_state.player {
        player.pos.clone()
    } else {
        return;
    };

    let input_state = gm.input_system.get_input_state().clone();
    let movement = gm.input_system.get_movement_vector();
    let water_current = gm.physics_system.get_water_current_at(&player_pos);

    if let (Some(player), Some(raft)) = (&mut gm.game_state.player, &mut gm.game_state.raft) {
        super::super::game_manager::apply_player_input(player, &input_state, &movement);
        super::super::game_manager::apply_physics_update(player, &water_current, gm.delta_time);

        let check_pos = if gm.game_state.game_mode == super::super::game_manager::GameMode::Dive {
            gm.game_state.last_surface_pos.clone()
        } else {
            player.pos.clone()
        };
        player.on_raft = raft.is_on_raft(&check_pos);

        let mut new_mode = gm.game_state.game_mode;
        if input_state.dive && gm.game_state.game_mode != super::super::game_manager::GameMode::Dive {
            new_mode = super::super::game_manager::GameMode::Dive;
            gm.game_state.last_surface_pos = player.pos.clone();
            if let Some(raft_ref) = &gm.game_state.raft {
                let offset = crate::math::Vec3::new(player.pos.x - raft_ref.center.x, player.pos.y - raft_ref.center.y, 0.0);
                gm.render_system.set_dive_offset(offset);
            }
            // Start diving by moving into depth (z axis), keep top-down y at surface
            player.pos.z = -10.0;
            player.depth = -10;
            player.is_diving = true;
            // Camera anchoring handled inside RenderSystem based on world z
        }

        if new_mode == super::super::game_manager::GameMode::Dive {
            // Depth is derived from world z (negative below surface)
            player.depth = (-player.pos.z).max(0.0) as i32;
            player.is_diving = player.pos.z < 0.0;
            if player.pos.z >= 0.0 {
                new_mode = super::super::game_manager::GameMode::Raft;
                player.pos = gm.game_state.last_surface_pos.clone();
                player.pos.z = 0.0;
                player.is_diving = false;
                gm.render_system.clear_dive_offset();
                // Camera anchoring handled inside RenderSystem
            }
        }
        if new_mode != gm.game_state.game_mode {
            gm.render_system.trigger_transition_fade();
            gm.game_state.game_mode = new_mode;
        }
    }

    match gm.game_state.game_mode {
        super::super::game_manager::GameMode::Raft => {
            gm.spawn_system.set_spawn_rate(SpawnType::FloatingItem, 300);
            gm.render_system.set_render_mode(crate::components::renderer::render_system::RenderViewMode::TopDown);
        }
        super::super::game_manager::GameMode::Dive => {
            gm.spawn_system.set_spawn_rate(SpawnType::FloatingItem, u32::MAX);
            gm.render_system.set_render_mode(crate::components::renderer::render_system::RenderViewMode::SideScroll);
        }
    }
    gm.update_spawning_internal(&player_pos);
    gm.update_ai();
    gm.world_system.update(&player_pos);
    gm.render_system.set_camera_target(player_pos);
    gm.render_system.update_camera(gm.delta_time);
    if gm.frame_count < 10 {
        gm.render_system.update_camera(1.0);
    }

    // Entity rendering is centralized via GameManager

    // Rendering is centralized in GameManager.update()
}

