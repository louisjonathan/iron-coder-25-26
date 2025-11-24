use crate::app::canvas_board::CanvasBoard;
use crate::app::canvas_element::CanvasSelection;
use crate::app::colorschemes::debug_once;
use crate::app::tabs::base_tab::BaseTab;
use crate::app::{AddProtocolConnectionCommand, CanvasProtocol};
use crate::app::{SharedState, connection_wizard};
use crate::app::{canvas_board, canvas_connection::CanvasConnection};
use crate::board;
use eframe::egui::{Align2, Color32, FontId, Key, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use egui::PointerButton;
use egui::color_picker::color_picker_color32;
use egui::debug_text::print;
use emath::RectTransform;
use syntect::highlighting::Color;

use std::cell::RefCell;
use std::rc::Rc;

use crate::app::connection_wizard::{ConnectionWizard, WizardResult, WizardState, WizardType};
use egui_extras::RetainedImage;
use std::collections::HashMap;

pub struct CanvasTab {
    canvas_zoom: f32,
    canvas_offset: Vec2,
    connection_in_progress: Option<Rc<RefCell<CanvasConnection>>>,
    selection: Option<CanvasSelection>,
    pin_tooltip: Option<(Rc<RefCell<CanvasBoard>>, u32)>,
}

impl CanvasTab {
    pub fn new() -> Self {
        Self {
            canvas_zoom: 5.0,
            canvas_offset: Vec2::new(0.0, 0.0),
            connection_in_progress: None,
            selection: None,
            pin_tooltip: None,
        }
    }
}

impl BaseTab for CanvasTab {
    fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
        if state.reset_canvas {
            self.reset_canvas();
            state.reset_canvas = false;
        }

        // grab mouse location
        let mouse_screen = ui.input(|i| i.pointer.hover_pos()).unwrap_or_default();

        let response = ui.allocate_response(ui.available_size(), Sense::click_and_drag());

        let rect = response.rect;

        if response.hovered() {
            let scroll_delta = ui.ctx().input(|i| i.smooth_scroll_delta.y);
            let zoom_factor = 1.01;

            // handle scrolling to zoom on mouse location using transformations
            if scroll_delta != 0.0 {
                let zoom = if scroll_delta > 0.0 {
                    zoom_factor
                } else {
                    1.0 / zoom_factor
                };

                let to_screen = emath::RectTransform::from_to(
                    Rect::from_min_size(Pos2::ZERO, rect.size() / self.canvas_zoom),
                    rect.translate(self.canvas_offset),
                );

                let mouse_canvas_before = to_screen.inverse().transform_pos(mouse_screen);

                self.canvas_zoom *= zoom;

                let new_to_screen = emath::RectTransform::from_to(
                    Rect::from_min_size(Pos2::ZERO, rect.size() / self.canvas_zoom),
                    rect.translate(self.canvas_offset),
                );

                let mouse_screen_after = new_to_screen.transform_pos(mouse_canvas_before);

                // change offset based on where we zoom
                self.canvas_offset += mouse_screen - mouse_screen_after;
            }
        }

        let rect: Rect = response.rect;
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, rect.size() / self.canvas_zoom),
            rect.translate(self.canvas_offset),
        );

        let mouse_pos = mouse_screen - rect.min;
        let mouse_canvas = to_screen
            .inverse()
            .transform_pos(Pos2::new(mouse_screen.x, mouse_screen.y));

        self.draw_grid(ui, &to_screen);

        let draw_all_pins = ui.input(|i| i.modifiers.alt);

        // Handle Delete key via keybindings system
        if state.keybindings.is_pressed(ui.ctx(), "delete") {
            if let Some(s) = self.selection.take() {
                match s {
                    CanvasSelection::Board(board) => {
                        self.selection = None;
                        state.project.remove_board(&board);
                        println!("Delete: Removed board from canvas");
                    }
                    CanvasSelection::Connection(connection) => {
                        self.selection = None;

                        // Check if connection is part of a protocol group
                        let is_part_of_group = connection.borrow().protocol_group_id.is_some();
                        let group_id = connection.borrow().protocol_group_id;
                        let role = connection.borrow().role.clone();

                        // If wizard is active, check if this connection is part of the wizard
                        if let Some(wizard) = state.connection_wizard.as_mut() {
                            let created_connections = wizard.created_connections();
                            let is_wizard_connection = created_connections
                                .iter()
                                .any(|c| Rc::ptr_eq(c, &connection));

                            if is_wizard_connection {
                                // This connection is part of the wizard group
                                // Undo the wizard state to remove both selections
                                if wizard.undo_full_connection() {
                                    println!(
                                        "Delete: Removed wizard connection (undoing wizard state)"
                                    );
                                }
                            }
                        }

                        state.project.remove_connection(&connection);

                        if is_part_of_group {
                            println!(
                                "Delete: Removed individual connection ({}) from protocol group",
                                role.unwrap_or_else(|| "unknown".to_string())
                            );
                        } else {
                            println!("Delete: Removed connection from canvas");
                        }
                    }
                    // Handle protocol group deletion
                    CanvasSelection::ProtocolGroup {
                        group_id,
                        connections,
                    } => {
                        self.selection = None;
                        println!(
                            "Delete: Removing {} connections from protocol group",
                            connections.len()
                        );

                        // Remove all connections in the group
                        for conn in connections {
                            state.project.remove_connection(&conn);
                        }

                        // Remove the protocol group itself
                        state.project.remove_protocol_group(&group_id);
                    }
                    // Select one from group: Delete only the selected individual connection
                    CanvasSelection::WithinProtocolGroup {
                        group_id,
                        selected_connection,
                        ..
                    } => {
                        let role = selected_connection.borrow().role.clone();

                        state.project.remove_connection(&selected_connection);
                        // Update selection with fresh connection list
                        let updated_connections = state.project.get_group_connections(&group_id);
                        if !updated_connections.is_empty() {
                            // Stay in select-one mode with updated list
                            self.selection = Some(CanvasSelection::WithinProtocolGroup {
                                group_id,
                                all_connections: updated_connections.clone(),
                                selected_connection: updated_connections[0].clone(),
                            });
                        } else {
                            self.selection = None;
                        }
                        // Update wizard state
                        if let Some(wizard) = state.connection_wizard.as_mut() {
                            wizard.update_missing_roles_after_deletion(&selected_connection);
                        }

                        println!(
                            "Delete: Removed individual connection ({}) from select one from group",
                            role.unwrap_or_else(|| "unknown".to_string())
                        );
                    }
                }
            }
        }

        // Handle Backspace to remove last waypoint during connection drawing
        if state.keybindings.is_pressed(ui.ctx(), "backspace") {
            if let Some(conn) = &self.connection_in_progress {
                conn.borrow_mut().remove_last_point();
            }
        }

        let quit_connection = ui.input(|i| i.key_pressed(Key::Escape));
        if quit_connection {
            self.connection_in_progress = None;
            self.pin_tooltip = None;

            // If wizard is active, clean up partial selections
            if let Some(wizard) = state.connection_wizard.as_mut() {
                let selected = wizard.selected_pins();
                if selected.len() % 2 == 1 {
                    // We started a connection but didn't finish it (odd number)
                    // Just undo one selection to clean up
                    if let Some(main_board) = &state.project.main_board {
                        wizard.handle_undo(&main_board.borrow().board);
                    }
                }
            }

            // Clear selection to remove any visual artifacts
            self.selection = None;
        }

        // BOARDS
        for b in state.project.boards_iter() {
            b.borrow_mut().draw(ui, &to_screen, &mouse_screen);
        }

        // Get cached contrast colors from colorscheme
        // [0] = Primary (group borders), [1] = Secondary (wires), [2] = Tertiary (pins)
        let contrast_colors = &state.colorschemes.contrast_colors;

        let pin_color = contrast_colors.get(2).copied().unwrap_or(Color32::WHITE);
        let wire_color = contrast_colors.get(1).copied().unwrap_or(Color32::WHITE);
        let group_border_color = contrast_colors.get(0).copied().unwrap_or(Color32::RED);

        // Draw group selection highlighting
        match &self.selection {
            Some(CanvasSelection::ProtocolGroup { connections, .. }) => {
                // Draw group border behind all connections using group border color
                for conn in connections {
                    conn.borrow()
                        .highlight_as_group(ui, &to_screen, group_border_color);
                }
            }
            Some(CanvasSelection::WithinProtocolGroup {
                all_connections, ..
            }) => {
                // Draw group border behind all connections using group border color
                for conn in all_connections {
                    conn.borrow()
                        .highlight_as_group(ui, &to_screen, group_border_color);
                }
                // Selected wire will use group border color for select one from group highlight (drawn later)
            }
            _ => {}
        }

        // CONNECTIONS
        for c in state.project.connections_iter() {
            if let Some(CanvasSelection::WithinProtocolGroup {
                selected_connection,
                ..
            }) = &self.selection
            {
                if Rc::ptr_eq(&c, selected_connection) {
                    continue;
                }
            }
            c.borrow_mut()
                .draw(ui, &to_screen, mouse_canvas, &state.colorschemes.current);
        }

        // Draw select one from group connection with group border highlight color
        if let Some(CanvasSelection::WithinProtocolGroup {
            selected_connection,
            ..
        }) = &self.selection
        {
            // Temporarily override the connection's color to group border color (same as border)
            let original_color = selected_connection.borrow().color;
            selected_connection.borrow_mut().color = group_border_color;
            selected_connection.borrow_mut().draw(
                ui,
                &to_screen,
                mouse_canvas,
                &state.colorschemes.current,
            );
            selected_connection.borrow_mut().color = original_color; // Restore
        }
        if let Some(c) = &self.connection_in_progress {
            c.borrow_mut()
                .draw(ui, &to_screen, mouse_canvas, &state.colorschemes.current);
        }

        // PINS
        for b in state.project.boards_iter() {
            b.borrow_mut().draw_pins(
                ui,
                &to_screen,
                &mouse_screen,
                draw_all_pins,
                state.connection_wizard.as_mut(),
                &state.colorschemes.current,
                pin_color,
            );
        }

        // Show pin hints during connection
        if let Some(conn_rc) = &self.connection_in_progress {
            let conn = conn_rc.borrow();
            let sp = conn.get_start_pin();
            if let Some(set) = conn.get_connection_roles() {
                let sb = conn.get_start_board().borrow();

                // Determine which roles to highlight based on wizard state
                let roles_to_show: Vec<String> = if let Some(wizard) = &state.connection_wizard {
                    // Wizard active: only show current required role
                    if let Some(current_role) = wizard.current_role() {
                        vec![current_role]
                    } else {
                        vec![]
                    }
                } else {
                    // No wizard: show all roles from connection (old behavior)
                    set.iter().cloned().collect()
                };

                // Draw the pins for the appropriate roles
                if sb.board.is_main_board() {
                    for b in state.project.peripheral_boards.iter() {
                        let cb = b.borrow();
                        for role in &roles_to_show {
                            cb.draw_pins_from_role(ui, &to_screen, role);
                        }
                    }
                } else {
                    if let Some(b) = &state.project.main_board {
                        for role in &roles_to_show {
                            let cb = b.borrow();
                            cb.draw_pins_from_role(ui, &to_screen, role);
                        }
                    }
                }
            }
        }

        // Keybind tips
        let mut longest_text = 0.0;
        let mut offset = 0.0;
        let text_rect = ui.painter().text(
            rect.min + Vec2 { x: 0.0, y: offset },
            Align2::LEFT_TOP,
            "Alt: Show all pins",
            FontId::monospace(12.0),
            state.colorschemes.current["code_bg_color"],
        );

        longest_text = text_rect.width();
        offset += 12.0;

        if self.selection.is_some() {
            let delete_keybinding_text =
                if let Some(delete_binding) = state.keybindings.get_keybinding("delete") {
                    let mut parts = Vec::new();
                    if delete_binding.ctrl {
                        parts.push("Ctrl");
                    }
                    if delete_binding.alt {
                        parts.push("Alt");
                    }
                    parts.push(&delete_binding.key);
                    format!(
                        "{}: Remove current selection from canvas",
                        parts.join(" + ")
                    )
                } else {
                    "Delete: Remove current selection from canvas".to_string()
                };

            let temp = ui.painter().text(
                rect.min + Vec2 { x: 0.0, y: offset },
                Align2::LEFT_TOP,
                &delete_keybinding_text,
                FontId::monospace(12.0),
                state.colorschemes.current["code_bg_color"],
            );
            offset += 12.0;
            if temp.width() > longest_text {
                longest_text = temp.width();
            }
        }

        if self.connection_in_progress.is_some() {
            let temp = ui.painter().text(
                rect.min + Vec2 { x: 0.0, y: offset },
                Align2::LEFT_TOP,
                "Escape: Quit current connection",
                FontId::monospace(12.0),
                state.colorschemes.current["code_bg_color"],
            );
            offset += 12.0;
            if temp.width() > longest_text {
                longest_text = temp.width();
            }
        }

        //   Wizard UI - scaled and styled
        const UI_SCALE: f32 = 1.3;
        let base_bar_height = 40.0;
        let base_spacing = 10.0;

        let bar_height = base_bar_height * UI_SCALE;
        let spacing = base_spacing * UI_SCALE;

        // Calculate wizard UI position: offset by the keybind text width + padding
        let text_width = longest_text;
        let padding_after_text = spacing * 2.0; // Extra padding
        let wizard_x_offset = text_width + padding_after_text;

        let bar_rect = Rect::from_min_size(
            rect.min + Vec2::new(wizard_x_offset, 0.0),
            Vec2::new(rect.width() - wizard_x_offset, bar_height),
        );

        egui::Area::new("canvas_top_bar".into())
            .fixed_pos(bar_rect.min)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                ui.set_max_width(bar_rect.width());
                ui.set_max_height(bar_height);

                // Apply scaling and styling in a clean scope
                ui.scope(|ui| {
                    // Clone and modify style for this UI only
                    let mut style = (*ui.ctx().style()).clone();

                    // Scale all text
                    for (_text_style, font_id) in style.text_styles.iter_mut() {
                        font_id.size *= UI_SCALE;
                    }

                    // Calculate best text color based on background luminance
                    let ui_background = state
                        .colorschemes
                        .current
                        .get("panel_fill")
                        .copied()
                        .unwrap_or(Color32::DARK_GRAY);
                    let text_color =
                        crate::app::colorschemes::text_color_for_background(&ui_background);
                    style.visuals.override_text_color = Some(text_color);

                    // Apply scaled spacing
                    style.spacing.item_spacing.x = spacing;
                    style.spacing.button_padding = egui::vec2(spacing, spacing * 0.5);

                    ui.set_style(style);

                    ui.horizontal(|ui| {
                        // Wizard area is already offset by text width, no need for extra space
                        // Determine current selection based on wizard state
                        let selected = if let Some(ref wizard) = state.connection_wizard {
                            match wizard.wizard_type {
                                WizardType::None => 0,
                                WizardType::I2C => 1,
                                WizardType::SPI => 2,
                                WizardType::UART => 3,
                            }
                        } else {
                            0
                        };

                        let options: [WizardType; 4] = [
                            WizardType::None,
                            WizardType::I2C,
                            WizardType::SPI,
                            WizardType::UART,
                        ];

                        ui.label("Connection Type:");
                        egui::ComboBox::from_id_source("connection_type_combo")
                            .selected_text(options[selected].display_name())
                            .show_ui(ui, |cb_ui| {
                                for (i, wizard_type) in options.iter().enumerate() {
                                    if cb_ui
                                        .selectable_value(
                                            &mut selected.clone(),
                                            i,
                                            wizard_type.display_name(),
                                        )
                                        .clicked()
                                    {
                                        match wizard_type {
                                            WizardType::None => {
                                                state.connection_wizard = None;
                                            }
                                            WizardType::I2C
                                            | WizardType::SPI
                                            | WizardType::UART => {
                                                if let Some(main_board_rc) =
                                                    state.project.main_board.as_mut()
                                                {
                                                    let main_board = main_board_rc.borrow();
                                                    // Starts the wizard
                                                    state.connection_wizard =
                                                        Some(ConnectionWizard::new(
                                                            *wizard_type,
                                                            &main_board.board,
                                                        ));
                                                }
                                            }
                                        }
                                    }
                                }
                            });

                        ui.separator();

                        // Flags to defer wizard mutation
                        let mut should_start_completing = false;
                        let mut should_exit_review = false;

                        // Show wizard status and controls
                        if let Some(wizard) = &state.connection_wizard {
                            // Status message
                            let status = wizard.status_message();
                            let color = if wizard.error_message.is_some() {
                                state.colorschemes.current["error_fg_color"]
                            } else {
                                state.colorschemes.current["warn_fg_color"]
                            };
                            ui.colored_label(color, status);

                            // Review UI - Show connection status for each role
                            use crate::app::connection_wizard::WizardState;
                            if let WizardState::Review {
                                missing_roles,
                                existing_connections,
                                ..
                            } = &wizard.state
                            {
                                ui.separator();
                                ui.label("Connection Status:");
                                ui.vertical(|ui| {
                                    // Show all required roles with status
                                    for (main_role, peripheral_role) in
                                        &wizard.wizard_type.required_roles()
                                    {
                                        let is_connected =
                                            existing_connections.contains_key(main_role);
                                        let status = if is_connected { "●" } else { "○" };
                                        let role_color = if is_connected {
                                            state.colorschemes.current["code_bg_color"]
                                        } else {
                                            state.colorschemes.current["error_fg_color"]
                                        };

                                        ui.horizontal(|ui| {
                                            ui.colored_label(
                                                role_color,
                                                format!(
                                                    "{} {}: {}",
                                                    status,
                                                    main_role,
                                                    if is_connected {
                                                        "Connected"
                                                    } else {
                                                        "Missing!"
                                                    }
                                                ),
                                            );
                                        });
                                    }
                                });
                                ui.separator();

                                // Button: Either "Complete Missing" or "Exit" depending on state
                                let has_missing = !missing_roles.is_empty();
                                if has_missing {
                                    if ui.button("Complete Missing Connections").clicked() {
                                        should_start_completing = true;
                                    }
                                } else {
                                    if ui.button("Exit Review (Esc)").clicked() {
                                        should_exit_review = true;
                                    }
                                }
                            } else {
                                // Show selected pins so far (for normal wizard mode)
                                let selected = wizard.selected_pins();
                                if !selected.is_empty() {
                                    ui.separator();
                                    ui.label(format!("Connections:"));
                                    ui.vertical(|ui| {
                                        for (role, _, name, _) in selected {
                                            ui.label(format!("{} : {}", role, name));
                                        }
                                    });
                                }
                            }
                            ui.separator();

                            // Undo button for wizard
                            let can_undo = !wizard.created_connections().is_empty()
                                || !wizard.selected_pins().is_empty();
                            if ui
                                .add_enabled(can_undo, egui::Button::new("Undo (Ctrl+Z)"))
                                .clicked()
                            {
                                // Get the last connection BEFORE calling undo
                                let conn_to_remove = wizard.created_connections().last().cloned();

                                // Undo FULL connection (both pins in the pair)
                                let wizard_mut = state.connection_wizard.as_mut().unwrap();
                                if wizard_mut.undo_full_connection() {
                                    println!("Full connection undone (2 selections removed)");
                                }

                                // Remove the connection from the project
                                if let Some(conn) = conn_to_remove {
                                    println!("Removing connection from project");
                                    state.project.remove_connection(&conn);

                                    // Clear selection to remove visual remnants
                                    self.selection = None;
                                }
                            }

                            if ui.button("Cancel").clicked() {
                                state.connection_wizard = None;
                            }
                        }

                        // Handle deferred wizard mutations (after borrow ends)
                        if should_exit_review {
                            state.connection_wizard = None;
                            self.selection = None;
                        }

                        if should_start_completing {
                            if let Some(main_board_rc) = state.project.main_board.as_mut() {
                                let main_board = main_board_rc.borrow();
                                if let Some(wiz) = state.connection_wizard.as_mut() {
                                    wiz.start_completing_missing(&main_board.board);
                                }
                            }
                        }

                        ui.add_space(ui.available_width());
                    });
                }); // Close scope
            });

        /* interaction flow
            1. check for current connection
            2. check pins for click
            3. check connections for click
            4. check boards for click
            5. drag canvas
        */

        // 1
        if let Some(mut conn) = self.connection_in_progress.take() {
            let mut clicked_pin: Option<u32> = None;
            let boards: Vec<_> = state.project.boards_iter().cloned().collect();
            for canvas_board_rc in &boards {
                let pin_opt = {
                    let mut canvas_board = canvas_board_rc.borrow_mut();
                    if !canvas_board.contains(&to_screen, &mouse_screen) {
                        None
                    } else {
                        canvas_board.pin_click(&to_screen, &response, &mouse_screen, &ui)
                    }
                };

                if let Some((pin, button)) = pin_opt {
                    clicked_pin = Some(pin);

                    let board_id = canvas_board_rc.borrow().id;

                    // If wizard is active, validate the pin selection first
                    if let Some(mut cw) = state.connection_wizard.take() {
                        if !cw.handle_pin_selected(
                            pin,
                            &canvas_board_rc.borrow().board,
                            board_id,
                            &mut state.project,
                        ) {
                            // Pin was invalid, wizard shows error, don't create connection
                            // Reset clicked_pin so connection_in_progress gets restored below
                            clicked_pin = None;
                            break;
                        }
                        // Pin was valid, wizard advances, now create the connection normally
                        state.connection_wizard = Some(cw);
                    }

                    // Create the connection (same as before, wizard or not)
                    let conn_clone = conn.clone();
                    {
                        let mut connection = conn_clone.borrow_mut();

                        let pin_location_opt = {
                            let canvas_board = canvas_board_rc.borrow();
                            canvas_board
                                .get_pin_location(&pin)
                                .map(|loc| loc + canvas_board.get_canvas_position())
                        };

                        if let Some(pin_location) = pin_location_opt {
                            connection.add_end_point(&mouse_canvas, pin_location);
                            connection.show_popup = true;
                        }

                        connection
                            .get_start_board()
                            .borrow_mut()
                            .connections
                            .push(conn_clone.clone());

                        connection.end(canvas_board_rc.clone(), pin.clone());
                    }

                    {
                        let mut canvas_board = canvas_board_rc.borrow_mut();
                        canvas_board.add_connection(&conn_clone);
                    }

                    state.project.add_connection(&conn_clone);

                    // If wizard is active, track this connection
                    if let Some(cw) = state.connection_wizard.as_mut() {
                        // Use the number of already-created connections to determine which role this is
                        // Connection 0 → role 0 (SDA), Connection 1 → role 1 (SCL), etc.
                        let created_count = cw.created_connections().len();
                        let required_roles = cw.wizard_type.required_roles();

                        if let Some((main_role, _)) = required_roles.get(created_count) {
                            conn_clone.borrow_mut().role = Some(main_role.clone());
                        }

                        cw.add_created_connection(conn_clone.clone());
                    }

                    break;
                }
            }

            if clicked_pin.is_none() {
                self.connection_in_progress = Some(conn);

                if response.clicked() {
                    if let Some(conn) = &self.connection_in_progress {
                        conn.borrow_mut().add_point(mouse_canvas);
                    }
                }
            }
        } else {
            let mut clicked_pin: Option<u32> = None;
            let mut ignore_canvas = false;

            // 2
            let boards: Vec<_> = state.project.boards_iter().cloned().collect();
            for canvas_board_rc in &boards {
                if clicked_pin.is_none() {
                    let pin_opt = {
                        let mut canvas_board = canvas_board_rc.borrow_mut();
                        canvas_board.pin_click(&to_screen, &response, &mouse_screen, &ui)
                    };

                    if let Some((pin, button)) = pin_opt {
                        if button == PointerButton::Secondary {
                            let canvas_board = canvas_board_rc.borrow();
                            self.pin_tooltip = Some((canvas_board_rc.clone(), pin));
                        } else if button == PointerButton::Primary {
                            clicked_pin = Some(pin);
                            if self.check_pin_use(canvas_board_rc, &pin, &state.project.connections)
                            {
                                break;
                            }

                            let board_id = canvas_board_rc.borrow().id;

                            // If wizard is active, validate the pin selection first
                            if let Some(mut cw) = state.connection_wizard.take() {
                                if !cw.handle_pin_selected(
                                    pin,
                                    &canvas_board_rc.borrow().board,
                                    board_id,
                                    &mut state.project,
                                ) {
                                    // Pin was invalid, wizard shows error, don't start connection

                                    break;
                                }

                                // Pin was valid, wizard advances, now start the connection normally
                                state.connection_wizard = Some(cw);
                            }

                            // Use cached secondary color for wires
                            let wire_color = state
                                .colorschemes
                                .contrast_colors
                                .get(1)
                                .copied()
                                .unwrap_or(Color32::WHITE);

                            // Start the connection with contrast-based color
                            let mut conn = Rc::new(RefCell::new(CanvasConnection::new(
                                canvas_board_rc.clone(),
                                pin.clone(),
                                wire_color,
                            )));
                            {
                                let mut connection = conn.borrow_mut();
                                let canvas_board = canvas_board_rc.borrow();
                                if let Some(pin_location) = canvas_board.get_pin_location(&pin) {
                                    connection.add_point(
                                        pin_location + canvas_board.get_canvas_position(),
                                    );
                                }
                            }

                            {
                                let mut canvas_board = canvas_board_rc.borrow_mut();
                                canvas_board.add_connection(&conn);
                            }

                            self.connection_in_progress = Some(conn.clone());
                            break;
                        }
                    }
                }
            }

            if response.clicked_by(egui::PointerButton::Primary) {
                // Two-level click system for protocol groups
                let mut connection_clicked = false;
                for c in state.project.connections_iter() {
                    let connection = c.borrow();
                    if connection.contains(&to_screen, &mouse_screen) {
                        connection_clicked = true;

                        // Check if this connection belongs to a protocol group
                        if let Some(group_id) = connection.protocol_group_id {
                            // Connection is part of a group

                            // Check if we already have this group selected
                            let group_already_selected = matches!(
                                &self.selection,
                                Some(CanvasSelection::ProtocolGroup { group_id: id, .. }) if *id == group_id
                            );

                            if group_already_selected {
                                // Second click on group → Drill down to individual connection + Enter Review
                                let all_connections =
                                    state.project.get_group_connections(&group_id);

                                self.selection = Some(CanvasSelection::WithinProtocolGroup {
                                    group_id,
                                    all_connections: all_connections.clone(),
                                    selected_connection: c.clone(),
                                });

                                //   Auto-enter Review mode on select one from group (2nd click)
                                if let Some(protocol_group) =
                                    state.project.get_protocol_group(&group_id)
                                {
                                    let wizard_type = protocol_group.protocol_type;

                                    state.connection_wizard =
                                        Some(ConnectionWizard::enter_review_mode(
                                            wizard_type,
                                            group_id,
                                            all_connections,
                                        ));
                                }
                            } else {
                                // First click → Select entire group (no wizard)
                                let group_connections =
                                    state.project.get_group_connections(&group_id);

                                self.selection = Some(CanvasSelection::ProtocolGroup {
                                    group_id,
                                    connections: group_connections,
                                });
                            }
                        } else {
                            // Not part of a group, select normally
                            self.selection = Some(CanvasSelection::Connection(c.clone()));
                        }

                        ignore_canvas = true;
                        break;
                    }
                }

                // 4 only check boards if we didnt click connection
                if !connection_clicked {
                    for b in state.project.boards_iter_rev() {
                        let board = b.borrow();
                        if board.contains(&to_screen, &mouse_screen) {
                            self.selection = Some(CanvasSelection::Board(b.clone()));
                            ignore_canvas = true;
                            break;
                        }
                    }
                }
            }

            if let Some(s) = self.selection.as_mut() {
                if s.interact(&to_screen, &self.canvas_zoom, &response, &mouse_screen) {
                    ignore_canvas = true;
                }
                s.highlight(ui, &to_screen, &state.colorschemes.current);
            }

            // 5
            if clicked_pin == None && !ignore_canvas {
                // Exit wizard when clicking canvas during select one from group
                if response.clicked_by(egui::PointerButton::Primary) {
                    if matches!(
                        &self.selection,
                        Some(CanvasSelection::WithinProtocolGroup { .. })
                    ) {
                        state.connection_wizard = None;
                        self.selection = None;
                        println!(
                            "Clicked canvas during select one from group - exiting Review mode"
                        );
                    }
                }

                if response.dragged() {
                    self.canvas_offset += response.drag_delta();
                    for b_ref in state.project.boards_iter() {
                        let mut b = b_ref.borrow_mut();
                        b.canvas_update(&to_screen);
                    }
                }

                if response.clicked() {
                    self.selection = None;
                }
            }
        }

        if response.clicked() {
            self.pin_tooltip = None;
        }

        // Handle Ctrl+Z: Global undo system
        if state.keybindings.is_pressed(ui.ctx(), "undo") {
            if let Some(wizard) = state.connection_wizard.as_mut() {
                // Wizard is active: use wizard undo
                let conn_to_remove = wizard.created_connections().last().cloned();

                if wizard.undo_full_connection() {
                    println!("Ctrl+Z (Wizard): Full connection undone (2 selections removed)");
                }

                if let Some(conn) = conn_to_remove {
                    println!("Ctrl+Z (Wizard): Removing connection from project");
                    state.project.remove_connection(&conn);
                    self.selection = None;
                }
            } else {
                // No wizard: use global command history
                if state.command_history.undo(&mut state.project) {
                    println!("Ctrl+Z (Global): Command undone");
                    self.selection = None;
                } else {
                    println!("Ctrl+Z (Global): Nothing to undo");
                }
            }
        }

        // Handle Ctrl+Y: Global redo system
        if state.keybindings.is_pressed(ui.ctx(), "redo") {
            // Redo only works with global command history (not wizard)
            if state.connection_wizard.is_none() {
                if state.command_history.redo(&mut state.project) {
                    println!("Ctrl+Y (Global): Command redone");
                    self.selection = None;
                } else {
                    println!("Ctrl+Y (Global): Nothing to redo");
                }
            } else {
                println!("Ctrl+Y: Redo not available during wizard");
            }
        }

        // Handle wizard completion - group all connections for undo/redo when wizard finishes
        if let Some(wizard) = &state.connection_wizard {
            if matches!(wizard.state, WizardState::Complete { .. }) {
                self.handle_wizard_completion(state);
            }
        }

        // pin info tooltip
        if let Some((board, pin)) = &self.pin_tooltip {
            let canvas_board = board.borrow();
            canvas_board.draw_pin_from_number(pin, ui, &to_screen);
            let pos = canvas_board.get_pin_location(pin).unwrap() + canvas_board.canvas_pos;
            let screen_pos = to_screen.transform_pos(pos);
            egui::show_tooltip_at(
                ui.ctx(),
                ui.layer_id(),
                egui::Id::new("pin_info"),
                screen_pos,
                |ui| {
                    ui.set_min_width(40.0);
                    ui.set_max_width(rect.width() / 4.0);
                    canvas_board.board.pinout.ui_show_pin_info(pin, ui);
                },
            );
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CanvasTab {
    fn draw_grid(&self, ui: &mut egui::Ui, to_screen: &RectTransform) {
        let grid_color = Color32::from_rgba_unmultiplied(42, 42, 42, 255);
        let spacing = 10.0;
        let major_spacing = 50.0;

        let screen_rect = ui.clip_rect();
        let canvas_rect = to_screen.inverse().transform_rect(screen_rect);

        let x_start = (canvas_rect.min.x / spacing).floor() as i32;
        let x_end = (canvas_rect.max.x / spacing).ceil() as i32;

        let y_start = (canvas_rect.min.y / spacing).floor() as i32;
        let y_end = (canvas_rect.max.y / spacing).ceil() as i32;

        for i in x_start..=x_end {
            let x = i as f32 * spacing;
            let p1 = to_screen.transform_pos(Pos2::new(x, canvas_rect.min.y));
            let p2 = to_screen.transform_pos(Pos2::new(x, canvas_rect.max.y));
            ui.painter()
                .line_segment([p1, p2], Stroke::new(1.0, grid_color));
        }

        for j in y_start..=y_end {
            let y = j as f32 * spacing;
            let p1 = to_screen.transform_pos(Pos2::new(canvas_rect.min.x, y));
            let p2 = to_screen.transform_pos(Pos2::new(canvas_rect.max.x, y));
            ui.painter()
                .line_segment([p1, p2], Stroke::new(1.0, grid_color));
        }

        let x_major_start = (canvas_rect.min.x / major_spacing).floor() as i32;
        let x_major_end = (canvas_rect.max.x / major_spacing).ceil() as i32;

        let y_major_start = (canvas_rect.min.y / major_spacing).floor() as i32;
        let y_major_end = (canvas_rect.max.y / major_spacing).ceil() as i32;

        for i in x_major_start..=x_major_end {
            let x = i as f32 * major_spacing;
            let p1 = to_screen.transform_pos(Pos2::new(x, canvas_rect.min.y));
            let p2 = to_screen.transform_pos(Pos2::new(x, canvas_rect.max.y));
            ui.painter()
                .line_segment([p1, p2], Stroke::new(3.0, grid_color));
        }

        for j in y_major_start..=y_major_end {
            let y = j as f32 * major_spacing;
            let p1 = to_screen.transform_pos(Pos2::new(canvas_rect.min.x, y));
            let p2 = to_screen.transform_pos(Pos2::new(canvas_rect.max.x, y));
            ui.painter()
                .line_segment([p1, p2], Stroke::new(3.0, grid_color));
        }
    }

    fn check_pin_use(
        &self,
        board: &Rc<RefCell<CanvasBoard>>,
        pin_num: &u32,
        connections: &Vec<Rc<RefCell<CanvasConnection>>>,
    ) -> bool {
        for c in connections {
            let connection = c.borrow();
            if Rc::ptr_eq(&connection.get_start_board(), board)
                && pin_num.eq(&connection.get_start_pin())
            {
                return true;
            }
            if let Some(eb) = connection.get_end_board() {
                if Rc::ptr_eq(&eb, board) && pin_num.eq(&connection.get_end_pin().unwrap()) {
                    return true;
                }
            }
        }
        return false;
    }

    pub fn fit_to_screen(&mut self, state: &mut SharedState, screen_rect: &Rect) {
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);

        for b_ref in state.project.boards_iter() {
            let b = b_ref.borrow();
            let p = b.get_canvas_position();

            min = min.min(p);
            max = max.max(p);
        }
    }

    pub fn reset_canvas(&mut self) {
        self.canvas_zoom = 5.0;
        self.canvas_offset = Vec2::new(0.0, 0.0);
        self.connection_in_progress = None;
        self.selection = None;
        self.pin_tooltip = None;
    }

    pub fn zoom_in(&mut self, viewport_size: Vec2) {
        let zoom_factor = 1.1;
        let viewport_center = viewport_size / 2.0;

        let to_screen_before = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, viewport_size / self.canvas_zoom),
            Rect::from_min_size(Pos2::ZERO, viewport_size).translate(self.canvas_offset),
        );
        let center_canvas_before = to_screen_before
            .inverse()
            .transform_pos(viewport_center.to_pos2());

        self.canvas_zoom *= zoom_factor;
        self.canvas_zoom = self.canvas_zoom.min(50.0);

        let to_screen_after = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, viewport_size / self.canvas_zoom),
            Rect::from_min_size(Pos2::ZERO, viewport_size).translate(self.canvas_offset),
        );
        let center_screen_after = to_screen_after.transform_pos(center_canvas_before);

        self.canvas_offset += viewport_center - center_screen_after.to_vec2();
    }

    pub fn zoom_out(&mut self, viewport_size: Vec2) {
        let zoom_factor = 0.9;
        let viewport_center = viewport_size / 2.0;

        let to_screen_before = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, viewport_size / self.canvas_zoom),
            Rect::from_min_size(Pos2::ZERO, viewport_size).translate(self.canvas_offset),
        );
        let center_canvas_before = to_screen_before
            .inverse()
            .transform_pos(viewport_center.to_pos2());

        self.canvas_zoom *= zoom_factor;
        self.canvas_zoom = self.canvas_zoom.max(0.1);

        let to_screen_after = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, viewport_size / self.canvas_zoom),
            Rect::from_min_size(Pos2::ZERO, viewport_size).translate(self.canvas_offset),
        );
        let center_screen_after = to_screen_after.transform_pos(center_canvas_before);

        self.canvas_offset += viewport_center - center_screen_after.to_vec2();
    }

    pub fn handle_delete_key(&mut self, state: &mut SharedState) {
        if let Some(s) = self.selection.take() {
            match s {
                CanvasSelection::Board(board) => {
                    self.selection = None;
                    state.project.remove_board(&board);
                    println!("Delete: Removed board from canvas");
                }
                CanvasSelection::Connection(connection) => {
                    self.selection = None;
                    state.project.remove_connection(&connection);
                    println!("Delete: Removed connection from canvas");
                }
                _ => {
                    println!("Delete: Cannot delete this selection type");
                }
            }
        }
    }

    pub fn handle_backspace_key(&mut self) {
        if let Some(conn) = &self.connection_in_progress {
            conn.borrow_mut().remove_last_point();
            println!("Backspace: Removed last waypoint");
        }
    }

    pub fn select_all_elements(&mut self, state: &mut SharedState) {
        self.selection = None;
        println!("Select All: Canvas operation");
    }

    /// Handle wizard completion by grouping all created connections for undo/redo
    /// This is called when the wizard state becomes Complete
    /// Connections are already created and added to the project during the wizard flow
    fn handle_wizard_completion(&mut self, state: &mut SharedState) {
        // Take the wizard out of state so we can process it
        if let Some(wizard) = state.connection_wizard.take() {
            if let WizardState::Complete {
                created_connections,
            } = wizard.state
            {
                // Create a ProtocolConnection to group all these connections
                let mut protocol_conn = CanvasProtocol::new(wizard.wizard_type);

                // Add all the connections that were created during the wizard
                for conn in created_connections {
                    protocol_conn.add_connection(conn);
                }

                //   Assign group ID to all connections
                protocol_conn.assign_to_connections();

                //   Store the protocol group in the project
                state.project.add_protocol_group(protocol_conn.clone());

                // Add the grouped protocol connection to command history for undo/redo
                // Note: The connections are already in the project, so we don't execute the command
                // We just add it to history so undo will remove them all as a group
                let command = Box::new(AddProtocolConnectionCommand::new(protocol_conn));
                state.command_history.add_to_history(command);
            }
        }
    }
}
