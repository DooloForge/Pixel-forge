use super::*;

pub fn update(gm: &mut GameManager) {
    let player_pos = if let Some(player) = &gm.game_state.player {
        player.pos.clone()
    } else {
        return;
    };

    let input_state = gm.input_system.get_input_state().clone();
    let movement = gm.input_system.get_movement_vector();

    // Hotbar quick-select 0-9 maps to quick slots 0-9
    if gm.input_system.is_key_just_pressed(crate::components::input::input_system::InputKey::QuickItem1) { if let Some(p) = &mut gm.game_state.player { let _ = p.use_quick_item(0); } }
    if gm.input_system.is_key_just_pressed(crate::components::input::input_system::InputKey::QuickItem2) { if let Some(p) = &mut gm.game_state.player { let _ = p.use_quick_item(1); } }
    if gm.input_system.is_key_just_pressed(crate::components::input::input_system::InputKey::QuickItem3) { if let Some(p) = &mut gm.game_state.player { let _ = p.use_quick_item(2); } }
    if gm.input_system.is_key_just_pressed(crate::components::input::input_system::InputKey::QuickItem4) { if let Some(p) = &mut gm.game_state.player { let _ = p.use_quick_item(3); } }
    if gm.input_system.is_key_just_pressed(crate::components::input::input_system::InputKey::QuickItem5) { if let Some(p) = &mut gm.game_state.player { let _ = p.use_quick_item(4); } }
    if gm.input_system.is_key_just_pressed(crate::components::input::input_system::InputKey::QuickItem6) { if let Some(p) = &mut gm.game_state.player { let _ = p.use_quick_item(5); } }
    if gm.input_system.is_key_just_pressed(crate::components::input::input_system::InputKey::QuickItem7) { if let Some(p) = &mut gm.game_state.player { let _ = p.use_quick_item(6); } }
    if gm.input_system.is_key_just_pressed(crate::components::input::input_system::InputKey::QuickItem8) { if let Some(p) = &mut gm.game_state.player { let _ = p.use_quick_item(7); } }
    if gm.input_system.is_key_just_pressed(crate::components::input::input_system::InputKey::QuickItem9) { if let Some(p) = &mut gm.game_state.player { let _ = p.use_quick_item(8); } }
    if gm.input_system.is_key_just_pressed(crate::components::input::input_system::InputKey::QuickItem0) { if let Some(p) = &mut gm.game_state.player { let _ = p.use_quick_item(9); } }

    // Handle item collection first to avoid borrowing conflicts
    let mut should_collect = false;
    let mut use_hook = false;
    let mut player_pos_for_collection = None;
    
    if let Some(player) = &gm.game_state.player {
        if input_state.collect_item || (input_state.use_tool && player.current_tool == crate::models::player::Tool::Hook) {
            should_collect = true;
            use_hook = player.current_tool == crate::models::player::Tool::Hook;
            player_pos_for_collection = Some(player.pos.clone());
        }
    }
    
    // Perform collection if needed
    if should_collect {
        if let Some(pos) = player_pos_for_collection {
            if use_hook {
                // Convert screen mouse to world coords based on camera centered at player in current view
                // In TopDown, world.y maps to screen.y with camera at player
                let (screen_w, screen_h) = turbo::resolution();
                let mouse = input_state.mouse_pos;
                let world_mouse = crate::math::Vec2::new(
                    mouse.x - screen_w as f32 * 0.5 + pos.x,
                    mouse.y - screen_h as f32 * 0.5 + pos.y,
                );
                let hook_direction = crate::math::Vec2::new(world_mouse.x - pos.x, world_mouse.y - pos.y);
                gm.launch_hook(&pos, hook_direction);
            } else {
                gm.handle_item_collection(&pos, false);
            }
        }
    }

    if let (Some(player), Some(raft)) = (&mut gm.game_state.player, &mut gm.game_state.raft) {
        // Hotbar drag & drop (HUD) when not in inventory scene
        // Geometry mirrors UIRenderer::render_hotbar
        let (screen_w, screen_h) = turbo::resolution();
        let slot_size = 24.0_f32;
        let margin = 4.0_f32;
        let count = 10usize;
        let total_w = count as f32 * slot_size + (count as f32 - 1.0) * margin;
        let start_x = (screen_w as f32 - total_w) * 0.5;
        let y = screen_h as f32 - slot_size - 8.0;
        let mouse = gm.input_system.get_screen_mouse_position();
        let left_click = gm.input_system.is_mouse_left_just_pressed();
        let left_held = gm.input_system.is_mouse_left_held();
        // Only allow when not in inventory scene
        if gm.current_scene == super::super::game_manager::SceneType::Playing {
            // Hit-test hotbar slot under mouse
            let mut hovered_hotbar: Option<usize> = None;
            for i in 0..count {
                let x = start_x + i as f32 * (slot_size + margin);
                if mouse.x >= x && mouse.x <= x + slot_size && mouse.y >= y && mouse.y <= y + slot_size {
                    hovered_hotbar = Some(i);
                    break;
                }
            }
            // Begin drag on press
            if left_click && gm.game_state.dragging_slot.is_none() {
                if let Some(idx) = hovered_hotbar { gm.game_state.dragging_slot = Some(idx); }
            }
            // On release, drop into hovered hotbar slot; swap inventory slots 0..9
            if !left_held {
                if let Some(src) = gm.game_state.dragging_slot.take() {
                    if let Some(dst) = hovered_hotbar {
                        if src != dst {
                            let _ = player.inventory.swap_slots(src, dst);
                        }
                    }
                }
            }
        }
        super::super::game_manager::apply_player_input(player, &input_state, &movement);
        super::super::game_manager::apply_physics_update(player, &gm.game_state.wind, gm.delta_time);

        player.on_raft = raft.is_on_raft(&player.pos);

        let mut new_mode = gm.game_state.game_mode;
        if input_state.dive && gm.game_state.game_mode != super::super::game_manager::GameMode::Dive {
            new_mode = super::super::game_manager::GameMode::Dive;
            if let Some(raft_ref) = &gm.game_state.raft {
                let offset = crate::math::Vec3::new(player.pos.x - raft_ref.center.x, player.pos.y - raft_ref.center.y, 0.0);
                gm.render_system.set_camera_target(player.pos);
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
                player.pos = player.pos.clone();
                player.pos.z = 0.0;
                player.is_diving = false;
                gm.render_system.set_camera_target(player.pos);
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
            gm.spawn_system.set_spawn_rate(SpawnType::FloatingItem, 600); // Reduced spawn rate - every 10 seconds
            gm.spawn_system.set_view_mode(crate::components::systems::spawn_system::ViewMode::TopDown);
            gm.render_system.set_render_mode(crate::components::renderer::render_system::RenderViewMode::TopDown);
        }
        super::super::game_manager::GameMode::Dive => {
            gm.spawn_system.set_spawn_rate(SpawnType::FloatingItem, u32::MAX);
            gm.spawn_system.set_view_mode(crate::components::systems::spawn_system::ViewMode::SideScroll);
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
}

