use crate::app::tabs::base_tab::BaseTab;
use crate::app::SharedState;

use std::process::Command;

pub struct TerminalTab {
    terminal_output: String,
    command_input: String,
}

impl TerminalTab {
    pub fn new() -> Self {
        TerminalTab {
            terminal_output: String::new(),
            command_input: String::new(),
        }
    }
}

impl BaseTab for TerminalTab {
    fn draw(&mut self, ui: &mut egui::Ui, _state: &mut SharedState) {
        let tab_rect = ui.max_rect();

        let font_height = ui.text_style_height(&egui::TextStyle::Body);
        let frame_padding = ui.style().spacing.item_spacing.y;
        let commandline_height = font_height + 2.0 * frame_padding;
        let commandline_rect = egui::Rect::from_min_size(
            egui::pos2(tab_rect.left(), tab_rect.bottom() - commandline_height),
            egui::vec2(tab_rect.width(), commandline_height),
        );
        ui.allocate_ui_at_rect(commandline_rect, |ui| {
            ui.horizontal(|ui| {
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.command_input)
                        .desired_width(ui.available_width()),
                );
    
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let command = self.command_input.trim();
                    if !command.is_empty() {
                        match Command::new("powershell") // TODO fix this for other OSes just demo for now
                            .arg("-c")
                            .arg(command)
                            .output()
                        {
                            Ok(output) => {
                                let stdout = String::from_utf8_lossy(&output.stdout)
                                    .trim_end()
                                    .to_string();
                                self.terminal_output
                                    .push_str(&format!("> {}\n{}\n", command, stdout));
                            }
                            Err(err) => {
                                self.terminal_output
                                    .push_str(&format!("> {}\n{}\n", command, err));
                            }
                        }
                        self.command_input.clear();
                    }
                }
            });
        });

        let terminalbuffer_rect = egui::Rect::from_min_max(
            tab_rect.min,
            egui::pos2(tab_rect.right(), commandline_rect.top() - 2.0*frame_padding),
        );

        ui.allocate_ui_at_rect(terminalbuffer_rect, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.terminal_output)
                            .font(egui::TextStyle::Monospace)
                            .desired_rows(15)
                            .lock_focus(true)
                            .interactive(false)
                            .desired_width(ui.available_width()),
                    );
                });
        });
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}