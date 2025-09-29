use crate::components::input_system::InputKey;

use super::*;

pub fn update(gm: &mut GameManager) {
    // Update recipe discovery based on current inventory
    if let Some(player) = &gm.game_state.player {
        gm.game_state.crafting_system.discover_recipes(&player.inventory);
    }

    // Mouse-based inventory interactions: select/drag/drop; context menu for Use/Destroy
    if let Some(player) = &mut gm.game_state.player {
        let inv = &mut player.inventory;
        let mouse = gm.input_system.get_screen_mouse_position();
        let left_click = gm.input_system.is_mouse_left_just_pressed();
        let left_held = gm.input_system.is_mouse_left_held();
        let right_click = gm.input_system.is_mouse_right_just_pressed();

        // Recreate panel layout to match full-screen 10-column UI
        let (w, h) = turbo::resolution();
        let panel_margin = 8.0_f32;
        let panel_x = panel_margin;
        let panel_y = panel_margin;
        let panel_w = w as f32 - panel_margin * 2.0;
        let panel_h = h as f32 - panel_margin * 2.0;

        // Grid metrics
        let cols = 10usize; // full-screen bag grid columns
        let bag_count = inv.max_slots.saturating_sub(10);
        let rows = (bag_count + cols - 1) / cols;
        let desired_slot = 32.0_f32;
        let slot_margin = 4.0_f32;
        let available_w = panel_w - 40.0 - (cols as f32 - 1.0) * slot_margin;
        let slot_size_w = (available_w / cols as f32).floor();
        let mut slot_size = desired_slot.min(slot_size_w).max(22.0_f32);
        // Hotbar metrics
        let hotbar_slot_size = slot_size.min(32.0);
        let hotbar_start_y = panel_y + 40.0;
        let grid_start_x = panel_x + 20.0;
        let grid_start_y = hotbar_start_y + hotbar_slot_size + 16.0;

        // Hit-test inventory slots: hotbar row 0..9 at top, bag grid 10..39 below
        let mut hovered_slot: Option<usize> = None;
        // Hotbar
        let hotbar_cols = 10usize;
        let hotbar_total_w = hotbar_cols as f32 * (hotbar_slot_size + slot_margin) - slot_margin;
        let hotbar_start_x = panel_x + (panel_w - hotbar_total_w) * 0.5;
        for i in 0..10usize {
            let slot_x = hotbar_start_x + i as f32 * (hotbar_slot_size + slot_margin);
            let slot_y = hotbar_start_y;
            if mouse.x >= slot_x && mouse.x <= slot_x + hotbar_slot_size && mouse.y >= slot_y && mouse.y <= slot_y + hotbar_slot_size {
                hovered_slot = Some(i);
                break;
            }
        }
        if hovered_slot.is_none() {
            // Bag grid 10..max
            for i in 10..inv.max_slots {
                let grid_i = i - 10;
                let col = grid_i % cols;
                let row = grid_i / cols;
                let slot_x = grid_start_x + col as f32 * (slot_size + slot_margin);
                let slot_y = grid_start_y + row as f32 * (slot_size + slot_margin);
                if mouse.x >= slot_x && mouse.x <= slot_x + slot_size && mouse.y >= slot_y && mouse.y <= slot_y + slot_size {
                    hovered_slot = Some(i);
                    break;
                }
            }
        }

        // Handle context menu actions (Use/Destroy) if open and clicked
        if let Some(menu) = &gm.game_state.inventory_context_menu {
            // Very simple hit areas below the cursor: two buttons stacked
            let btn_w = 80.0_f32; let btn_h = 16.0_f32; let pad = 2.0_f32;
            let use_rect = (menu.screen_x, menu.screen_y, btn_w, btn_h);
            let destroy_rect = (menu.screen_x, menu.screen_y + btn_h + pad, btn_w, btn_h);
            let clicked = left_click;
            let mx = mouse.x; let my = mouse.y;
            if clicked {
                if mx >= use_rect.0 && mx <= use_rect.0 + use_rect.2 && my >= use_rect.1 && my <= use_rect.1 + use_rect.3 {
                    // Use one item from the slot if consumable
                    if let Some(slot) = inv.get_slot_mut(menu.slot_index) {
                        if let Some(item_type) = slot.item_type { if item_type.is_consumable() { let _ = slot.remove_items(1); } }
                    }
                    gm.game_state.inventory_context_menu = None;
                } else if mx >= destroy_rect.0 && mx <= destroy_rect.0 + destroy_rect.2 && my >= destroy_rect.1 && my <= destroy_rect.1 + destroy_rect.3 {
                    if let Some(slot) = inv.get_slot_mut(menu.slot_index) { let _ = slot.remove_items(slot.quantity); }
                    gm.game_state.inventory_context_menu = None;
                } else {
                    // Clicked elsewhere closes menu
                    gm.game_state.inventory_context_menu = None;
                }
            }
        }

        // Drag & drop: press to pick, release to drop onto hovered; support merge if same type
        if left_click && gm.game_state.dragging_slot.is_none() {
            gm.game_state.dragging_slot = hovered_slot;
        }
        if !left_held {
            if let Some(src) = gm.game_state.dragging_slot.take() {
                if let Some(dst) = hovered_slot {
                    if src != dst {
                        // Try merge first if same type and room, else swap
                        let (src_type, src_qty, src_max) = if let Some(s) = inv.get_slot(src) { (s.item_type, s.quantity, s.max_stack) } else { (None, 0, 0) };
                        let (dst_type, dst_qty, dst_max) = if let Some(s) = inv.get_slot(dst) { (s.item_type, s.quantity, s.max_stack) } else { (None, 0, 0) };
                        let merged = if let (Some(st), Some(dt)) = (src_type, dst_type) {
                            if st == dt && dst_qty < dst_max {
                                let to_move = (dst_max - dst_qty).min(src_qty);
                                if to_move > 0 {
                                    if let Some(s) = inv.get_slot_mut(src) { let _ = s.remove_items(to_move); }
                                    if let Some(d) = inv.get_slot_mut(dst) { let _ = d.add_items(st, to_move); }
                                    true
                                } else { false }
                            } else { false }
                        } else { false };
                        if !merged {
                            let _ = inv.swap_slots(src, dst);
                        }
                        inv.selected_slot = Some(dst);
                    } else {
                        inv.selected_slot = Some(dst);
                    }
                }
            }
        }

        // Right click: open context menu for Use/Destroy on inventory slot
        if right_click {
            if let Some(slot_idx) = hovered_slot {
                // Open context menu at mouse position
                gm.game_state.inventory_context_menu = Some(super::super::game_manager::InventoryContextMenu { slot_index: slot_idx, screen_x: mouse.x, screen_y: mouse.y });
            }
        }
    }
}

