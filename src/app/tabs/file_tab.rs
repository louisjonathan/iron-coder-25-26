use crate::app::tabs::base_tab::BaseTab;
use crate::app::SharedState;
#[cfg(not (target_arch = "wasm32"))]
use std::fs::{File, OpenOptions};
#[cfg(not (target_arch = "wasm32"))]
use std::io::{Read, Seek, Write};
use egui::ScrollArea;
use std::path::{PathBuf, Path};
use log::info;
#[cfg(target_arch = "wasm32")]
use opfs::{
    persistent::{self, app_specific_dir, DirectoryHandle, FileHandle, WritableFileStream},
    CreateWritableOptions, GetDirectoryHandleOptions, GetFileHandleOptions,
};
#[cfg(target_arch = "wasm32")]
use opfs::{DirectoryHandle as _, FileHandle as _, WritableFileStream as _};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::JsValue;
pub struct FileTab {
    code: String,
    file_handle: Option<FileHandle>,
    synced: bool,
}
#[cfg(not (target_arch = "wasm32"))]
impl FileTab {
    pub fn default() -> Self {
        Self {
            code: String::new(),
            path: None,
            file: None,
            synced: false,
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
}
#[cfg(target_arch = "wasm32")]
impl FileTab {
    pub fn default() -> Self {
        Self {
            code: String::new(),
            file_handle: None,
            synced: false,
        }
    }

    pub async fn load_from_file(&mut self, file_handle: opfs::persistent::FileHandle) -> std::io::Result<()> {
        self.code.clear();
        self.file_handle = Some(file_handle.clone());
        if let Some(file_handle) = &mut self.file_handle {
            let buf = match file_handle.read().await {
                Ok(ret)=>ret,
                Err(e) => return Ok(()),
            };
            self.code = match String::from_utf8(buf){
                Ok(ret) => ret,
                Err(e) => return Ok(()),
            };
            self.synced = true;
        }
        Ok(())
    }

    pub async fn save(&mut self) -> std::io::Result<()> {
        // if let Some(file_handle) = &mut self.file_handle {
            
        //     file.set_len(0)?;
        //     file.write(self.code.as_bytes())?;
        //     file.sync_all()?;
        //     self.synced = true;
        // }
        Ok(())
    }
}
impl BaseTab for FileTab {
    fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
        ScrollArea::both().auto_shrink([false; 2]).show(ui, |ui| {
            let former_contents = self.code.clone();
            let resp = ui.add(
                egui::TextEdit::multiline(&mut self.code)
                    .font(egui::TextStyle::Name("EditorFont".into()))
                    .code_editor()
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .frame(false),
            );
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
                    if resp.hovered() {
                        info!("found a released code snippet!");
                        mem.data.remove::<String>(id);
                        self.code += &value;
                    }
                }
            });
        });
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}