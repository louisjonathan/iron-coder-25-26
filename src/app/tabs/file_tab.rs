use crate::app::SharedState;
use crate::app::tabs::base_tab::BaseTab;

use egui::ScrollArea;
use log::info;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::{
    Arc, Mutex,
    mpsc::{Receiver, channel},
};
use std::thread;

// notify code inspired by examples at https://github.com/notify-rs/notify/tree/main/examples

pub struct FileTab {
    path: Option<PathBuf>,
    code: String,
    file: Option<File>,
    synced: bool,
    use_syntax_highlighting: bool,
    cached_layout_job: Option<egui::text::LayoutJob>,
    last_highlighted_text: String,

    watcher_rx: Option<Receiver<Event>>,
    watcher_handle: Option<thread::JoinHandle<()>>,
    file_changed_externally: Arc<Mutex<bool>>,

    last_cursor_range: Option<egui::text::CCursorRange>,
    should_request_focus: bool,
}

impl FileTab {
    pub fn default() -> Self {
        Self {
            code: String::new(),
            path: None,
            file: None,
            synced: false,
            use_syntax_highlighting: true,
            cached_layout_job: None,
            last_highlighted_text: String::new(),

            watcher_rx: None,
            watcher_handle: None,
            file_changed_externally: Arc::new(Mutex::new(false)),

            // Cursor restoration logic
            last_cursor_range: None,
            should_request_focus: false,
        }
    }

    pub fn load_from_file(&mut self, file_path: &Path) -> std::io::Result<()> {
        self.code.clear();
        self.path = Some(file_path.canonicalize()?);
        self.file = Some(OpenOptions::new().read(true).write(true).open(file_path)?);
        if let Some(file) = &mut self.file {
            file.read_to_string(&mut self.code)?;
            self.synced = true;
        }

        self.start_watching();

        Ok(())
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(file) = &mut self.file {
            file.rewind()?;
            file.set_len(0)?;
            file.write_all(self.code.as_bytes())?;
            file.sync_all()?;
            self.synced = true;
            // Must request focus after synced changes due to title name change
            self.should_request_focus = true;
        }
        Ok(())
    }

    pub fn is_synced(&self) -> bool {
        self.synced
    }

    fn start_watching(&mut self) {
        if let Some(path) = &self.path {
            let (tx, rx) = channel();
            let path = path.clone();
            let file_changed_flag = Arc::new(Mutex::new(false));
            let flag_clone = Arc::clone(&file_changed_flag);

            let handle = thread::spawn(move || {
                let mut watcher: RecommendedWatcher = notify::recommended_watcher(
                    move |res: Result<notify::Event, notify::Error>| {
                        if let Ok(event) = res {
                            if matches!(event.kind, EventKind::Modify(_)) {
                                tx.send(event).ok();
                            }
                        }
                    },
                )
                .expect("Failed to create file watcher");

                watcher
                    .watch(&path, RecursiveMode::NonRecursive)
                    .expect("Failed to start watching file");

                loop {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            });

            self.watcher_rx = Some(rx);
            self.watcher_handle = Some(handle);
            self.file_changed_externally = flag_clone;
        }
    }

    fn reload_from_disk_if_not_dirty(&mut self) {
        if self.synced {
            if let Some(path) = &self.path {
                if let Ok(mut f) = File::open(path) {
                    let mut new_contents = String::new();
                    if f.read_to_string(&mut new_contents).is_ok() {
                        self.code = new_contents;
                        self.synced = true;
                        self.cached_layout_job = None;
                        self.last_highlighted_text.clear();
                    }
                }
            }
        }
    }
}

impl BaseTab for FileTab {
    fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
        let mut file_changed = false;
        if let Some(rx) = &self.watcher_rx {
            while let Ok(_event) = rx.try_recv() {
                file_changed = true;
            }
        }

        if file_changed {
            self.reload_from_disk_if_not_dirty();
        }

        // Show file path if available
        if let Some(path) = &self.path {
            ui.label(format!("File: {}", path.display()));
        }

        ScrollArea::both().auto_shrink([false; 2]).show(ui, |ui| {
            let former_contents = self.code.clone();

            // did the text change?
            if self.use_syntax_highlighting
                && (self.cached_layout_job.is_none() || self.last_highlighted_text != self.code)
            {
                let language = self
                    .path
                    .as_ref()
                    .and_then(|p| state.syntax_highlighter.detect_language(p));

                self.cached_layout_job = Some(
                    state
                        .syntax_highlighter
                        .highlight_code(&self.code, language),
                );
                self.last_highlighted_text = self.code.clone();
            }

            let response = if self.use_syntax_highlighting && !self.code.is_empty() {
                ui.add(
                    egui::TextEdit::multiline(&mut self.code)
                        .font(egui::TextStyle::Monospace)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .frame(false)
                        .layouter(&mut |ui, string, _wrap_width| {
                            let language = self
                                .path
                                .as_ref()
                                .and_then(|p| state.syntax_highlighter.detect_language(p));
                            let job = state.syntax_highlighter.highlight_code(string, language);
                            ui.fonts(|f| f.layout_job(job))
                        }),
                )
            } else {
                ui.add(
                    egui::TextEdit::multiline(&mut self.code)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .frame(false),
                )
            };

            // Store cursor position whenever the text edit has focus
            if response.has_focus() {
                if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), response.id) {
                    if let Some(cursor_range) = state.cursor.char_range() {
                        self.last_cursor_range = Some(cursor_range);
                    }
                }
            }

            if self.should_request_focus {
                response.request_focus();
                if let Some(cursor_range) = self.last_cursor_range {
                    if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), response.id) {
                        state.cursor.set_char_range(Some(cursor_range));
                        state.store(ui.ctx(), response.id);
                    }
                }
                self.should_request_focus = false;
            }

            if self.synced && self.code != former_contents {
                self.synced = false;
                // Must request focus after synced changes due to title name change
                self.should_request_focus = true;
            }

            // See if a code snippet was released over the editor.
            ui.ctx().memory_mut(|mem| {
                let id = egui::Id::new("released_code_snippet");
                let data: Option<String> = mem.data.get_temp(id);
                if let Some(value) = data {
                    info!("found a released code snippet!");
                    mem.data.remove::<String>(id);
                    self.code += &value;
                }
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
