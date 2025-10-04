use crate::app::tabs::base_tab::BaseTab;
use crate::app::SharedState;

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use egui::ScrollArea;
use std::path::{PathBuf, Path};
use log::info;

use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use std::sync::{mpsc::{channel, Receiver}, Arc, Mutex};
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
		}
	}

	pub fn load_from_file(&mut self, file_path: &Path) -> std::io::Result<()> {
		self.code.clear();
		self.path = Some(file_path.canonicalize()?);
		self.file = Some(
			OpenOptions::new()
				.read(true)
				.write(true)
				.open(file_path)?,
		);
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
			file.write(self.code.as_bytes())?;
			file.sync_all()?;
			self.synced = true;
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
				let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error> | {
					if let Ok(event) = res {
						if matches!(event.kind, EventKind::Modify(_)) {
							tx.send(event).ok();
						}
					}
				}).expect("Failed to create file watcher");

				watcher.watch(&path, RecursiveMode::NonRecursive)
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
			if self.use_syntax_highlighting && 
			(self.cached_layout_job.is_none() || self.last_highlighted_text != self.code) {
				
				let language = self.path.as_ref()
					.and_then(|p| state.syntax_highlighter.detect_language(p));
				
				self.cached_layout_job = Some(state.syntax_highlighter.highlight_code(&self.code, language));
				self.last_highlighted_text = self.code.clone();
			}
			
			let response = if self.use_syntax_highlighting && !self.code.is_empty() {
				// apply syntax highlighting
				let cached_job = self.cached_layout_job.clone();
				ui.add(
					egui::TextEdit::multiline(&mut self.code)
						.font(egui::TextStyle::Monospace)
						.lock_focus(true)
						.desired_width(f32::INFINITY)
						.frame(false)
						.layouter(&mut |ui, string, _wrap_width| {
							let language = self.path.as_ref()
								.and_then(|p| state.syntax_highlighter.detect_language(p));
							let job = state.syntax_highlighter.highlight_code(string, language);
							ui.fonts(|f| f.layout_job(job))
						}),
				)
			} else {
				// just draw it nromally if above doesnt work
				ui.add(
					egui::TextEdit::multiline(&mut self.code)
						.font(egui::TextStyle::Monospace)
						.code_editor()
						.lock_focus(true)
						.desired_width(f32::INFINITY)
						.frame(false),
				)
			};
			
			// check if the code has changed, so we can set the synced flag
			if self.synced && self.code != former_contents {
				self.synced = false;
			}
			
			// See if a code snippet was released over the editor.
			// TODO -- if so, insert it on the proper line
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