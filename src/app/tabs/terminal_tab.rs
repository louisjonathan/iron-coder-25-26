use crate::app::tabs::base_tab::BaseTab;
use crate::app::SharedState;

use egui_term::{PtyEvent, TerminalBackend, TerminalView};
use std::sync::mpsc::{Receiver, Sender};

use std::process::Command;

use std::{cell::RefCell, rc::Rc};

pub struct TerminalTab {
    // terminal_output: String,
    // command_input: String,
    terminal_backend: Option<Rc<RefCell<TerminalBackend>>>,
    pty_proxy_receiver: Receiver<(u64, egui_term::PtyEvent)>,
    pty_proxy_sender: Sender<(u64, egui_term::PtyEvent)>,
}

impl TerminalTab {
    pub fn new() -> Self {
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
				state.term_open_project_dir();
            }
        } else {
            if let Some(default_terminal) = &state.default_terminal {
                self.terminal_backend = Some(Rc::new(RefCell::new(
                    TerminalBackend::new(
                        0,
                        ctx.clone(),
                        self.pty_proxy_sender.clone(),
                        egui_term::BackendSettings {
                            shell: default_terminal.to_string_lossy().to_string(),
                            ..Default::default()
                        },
                    )
                    .unwrap(),
                )));
				state.term_open_project_dir();
            }
        }
    }
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
                egui::Frame::none()
                    .fill(ui.visuals().panel_fill)
                    .show(ui, |ui| {
                        // Check for focus before adding the terminal using ui.max_rect to recieve input instead
                        let terminal_rect = ui.max_rect();
                        let terminal_id = ui.id().with("terminal");
                        let should_focus = ui.ctx().input(|i| {
                            if let Some(pos) = i.pointer.interact_pos() {
                                terminal_rect.contains(pos)
                            } else {
                                false
                            }
                        });
                        let terminal = TerminalView::new(ui, &mut *term)
                            .set_focus(should_focus);

                        ui.add(terminal);
                    });
            });
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
