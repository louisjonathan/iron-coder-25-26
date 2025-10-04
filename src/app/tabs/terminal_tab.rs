use crate::app::tabs::base_tab::BaseTab;
use crate::app::SharedState;

use egui_term::{PtyEvent, TerminalBackend, TerminalView};
use std::sync::mpsc::{Receiver, Sender};

use std::process::Command;

use std::{rc::Rc, cell::RefCell};

pub struct TerminalTab {
    // terminal_output: String,
    // command_input: String,
	terminal_backend: Option<Rc<RefCell<TerminalBackend>>>,
    pty_proxy_receiver: Receiver<(u64, egui_term::PtyEvent)>,
    pty_proxy_sender: Sender<(u64, egui_term::PtyEvent)>,
}

impl TerminalTab {
    pub fn new() -> Self {
        // TerminalTab {
        //     terminal_output: String::new(),
        //     command_input: String::new(),
        // }

		let (pty_proxy_sender, pty_proxy_receiver) = std::sync::mpsc::channel();

        Self {
            terminal_backend: None,
            pty_proxy_receiver,
			pty_proxy_sender,
        }
    }

	fn setup_backend(&mut self, ctx: &egui::Context, state: &mut SharedState) {
		if let Some(term) = &self.terminal_backend {
			if state.output_terminal_backend.is_none() {
				state.output_terminal_backend = Some(term.clone());
			}
		} else {
			if let Some(default_terminal) = &state.default_terminal {
				self.terminal_backend = Some(Rc::new(RefCell::new(TerminalBackend::new(
					0,
					ctx.clone(),
					self.pty_proxy_sender.clone(),
					egui_term::BackendSettings {
						shell: default_terminal.to_string_lossy().to_string(),
						..Default::default()
					},
				)
				.unwrap())));
			}
		}
	}

    // /// Append build/flash output to the terminal
    // pub fn append_build_output(&mut self, output: &str) {
    //     if !output.is_empty() {
    //         self.terminal_output.push_str(output);
    //     }
    // }
}

impl BaseTab for TerminalTab {
    fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
		self.setup_backend(ui.ctx(), state);

		if let Ok((_, PtyEvent::Exit)) = self.pty_proxy_receiver.try_recv() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

		if let Some(term_ref) = self.terminal_backend.as_ref() {
			let mut term = term_ref.borrow_mut();
			let tab_rect = ui.max_rect();

			ui.allocate_ui_at_rect(tab_rect, |ui| {
				let terminal = TerminalView::new(ui, &mut *term)
					.set_focus(true);
				ui.add(terminal);
			});
		}

        // let input = ui.input(|i| i.clone());
        // if input.modifiers.ctrl && input.key_released(egui::Key::C) {
        //     state.stop_board();
        // }

        // if let Some(rx) = &state.rx {
        // for line in rx.try_iter() { // non-blocking
        //         state.terminal_buffer.push_str(&line);
        //     }
        // }

        // let tab_rect = ui.max_rect();

        // let font_height = ui.text_style_height(&egui::TextStyle::Body);
        // let frame_padding = ui.style().spacing.item_spacing.y;
        // let commandline_height = font_height + 2.0 * frame_padding;
        // let commandline_rect = egui::Rect::from_min_size(
        //     egui::pos2(tab_rect.left(), tab_rect.bottom() - commandline_height),
        //     egui::vec2(tab_rect.width(), commandline_height),
        // );
        // ui.allocate_ui_at_rect(commandline_rect, |ui| {
        //     ui.horizontal(|ui| {
        //         let response = ui.add(
        //             egui::TextEdit::singleline(&mut self.command_input)
        //                 .desired_width(ui.available_width()),
        //         );
    
        //         // if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        //         //     let command = self.command_input.trim();
        //         //     if !command.is_empty() {
        //         //         match Command::new("powershell") // TODO fix this for other OSes just demo for now
        //         //             .arg("-c")
        //         //             .arg(command)
        //         //             .output()
        //         //         {
        //         //             Ok(output) => {
        //         //                 let stdout = String::from_utf8_lossy(&output.stdout)
        //         //                     .trim_end()
        //         //                     .to_string();
        //         //                 self.terminal_output
        //         //                     .push_str(&format!("> {}\n{}\n", command, stdout));
        //         //             }
        //         //             Err(err) => {
        //         //                 self.terminal_output
        //         //                     .push_str(&format!("> {}\n{}\n", command, err));
        //         //             }
        //         //         }
        //         //         self.command_input.clear();
        //         //     }
        //         // }
        //     });
        // });

        // let terminalbuffer_rect = egui::Rect::from_min_max(
        //     tab_rect.min,
        //     egui::pos2(tab_rect.right(), commandline_rect.top() - 2.0*frame_padding),
        // );

        // ui.allocate_ui_at_rect(terminalbuffer_rect, |ui| {
        //     egui::ScrollArea::vertical()
        //         .auto_shrink([false; 2])
        //         .stick_to_bottom(true)
        //         .show(ui, |ui| {
        //             ui.add(
        //                 egui::TextEdit::multiline(&mut state.terminal_buffer)
        //                     .font(egui::TextStyle::Monospace)
        //                     .desired_rows(15)
        //                     .lock_focus(true)
        //                     .interactive(false)
        //                     .desired_width(ui.available_width()),
        //             );
        //         });
        // });
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}