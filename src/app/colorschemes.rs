use egui::{ahash::random_state, text_selection::visuals, Color32, Stroke};
// https://github.com/Experience-Monks/nice-color-palettes/tree/master
use rand::{prelude::*, thread_rng};
use std::{collections::HashMap, io::Write};
use std::{
    fs,
    path::{Path, PathBuf},
};
use syntect::highlighting::Color;
use toml;
pub struct colorscheme {
    pub all_names: Vec<String>,
    pub current: HashMap<String, Color32>,
    pub name: String,
}

#[rustfmt::skip]
impl Default for colorscheme {
    fn default() -> Self {
         let current = HashMap::from([
            ("extreme_bg_color".to_string(), Color32::from_hex("#69d2e7").unwrap()),
            ("faint_bg_color".to_string(), Color32::from_hex("#a7dbd8").unwrap()),
            ("code_bg_color".to_string(), Color32::from_hex("#e0e4cc").unwrap()),
            ("panel_fill".to_string(), Color32::from_hex("#f38630").unwrap()),
            ("window_fill".to_string(), Color32::from_hex("#fa6900").unwrap()),
            ("window_stroke_color".to_string(), Color32::from_hex("#586e75").unwrap()),
            ("hyperlink_color".to_string(), Color32::from_hex("#839496").unwrap()),
            ("warn_fg_color".to_string(), Color32::from_hex("#839496").unwrap()),
            ("error_fg_color".to_string(), Color32::from_hex("#839496").unwrap()),
            ]);
        Self {
           all_names: get_colorscheme_filenames(),
           current,
           name: "default".to_string(),
        }
    }
}
pub fn get_colorscheme_filenames() -> Vec<String> {
    let entries = fs::read_dir("./resources/colorschemes").unwrap();
    let files: Vec<String> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| p.extension().map_or(false, |ext| ext == "toml"))
        .map(|p| p.file_name().expect(&format!("failed to get filename for colorscheme {}", p.display())).to_string_lossy().into_owned())
        .collect();
    files
}
pub fn get_random_colorscheme() -> HashMap<String, Color32> {
    let entries = fs::read_dir("./resources/colorschemes").unwrap();
    let files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| p.extension().map_or(false, |ext| ext == "toml"))
        .collect();
    let mut rng = thread_rng();
    let chosen = files.choose(&mut rng).cloned().unwrap();
    let res = fs::read_to_string(chosen).unwrap();
    let color_scheme_unparsed: HashMap<String, String> = toml::from_str(&res).unwrap();
    let mut color_scheme: HashMap<String, Color32> = Default::default();
    for (key, val) in color_scheme_unparsed.into_iter() {
        color_scheme.insert(key, Color32::from_hex(&val).unwrap());
    }
    color_scheme
}

pub fn try_get_colorscheme(filename: &String) -> Option<HashMap<String, Color32>> {
    let target = PathBuf::from("./resources/colorschemes").join(filename);
    if (target.is_file()) {
        let res = fs::read_to_string(target).ok()?;
        let color_scheme_unparsed: HashMap<String, String> = toml::from_str(&res).unwrap();
        let mut color_scheme: HashMap<String, Color32> = Default::default();
        for (key, val) in color_scheme_unparsed.into_iter() {
            color_scheme.insert(key, Color32::from_hex(&val).unwrap());
        }
        Some(color_scheme)
    } else {
        None
    }
}

pub fn create_or_modify_colorscheme(filename: &String, colors: &HashMap<String, Color32>) {
    let target = PathBuf::from("./resources/colorschemes").join(filename);

    let mut color_scheme_serialized: HashMap<String, String> = Default::default();
    for (key, val) in colors.into_iter() {
        color_scheme_serialized.insert(key.clone(), Color32::to_hex(val));
    }
    fs::write(&target, toml::to_string(&color_scheme_serialized).unwrap());
}

pub fn set_colorscheme(ui: &mut egui::Ui, colorscheme: &HashMap<String, Color32>) {
    let mut new_style = (*ui.ctx().style()).clone();
    
    
    new_style.visuals.extreme_bg_color = colorscheme["extreme_bg_color"];
    new_style.visuals.faint_bg_color = colorscheme["faint_bg_color"];
    new_style.visuals.code_bg_color = colorscheme["code_bg_color"];
    new_style.visuals.panel_fill = colorscheme["panel_fill"];
    new_style.visuals.window_fill = colorscheme["window_fill"];

    new_style.visuals.hyperlink_color = colorscheme["panel_fill"];
    new_style.visuals.window_stroke.color = colorscheme["panel_fill"];
    new_style.visuals.warn_fg_color = colorscheme["code_bg_color"];
    new_style.visuals.error_fg_color = colorscheme["faint_bg_color"];

    let widget_states = [
        &mut new_style.visuals.widgets.noninteractive,
        &mut new_style.visuals.widgets.inactive,
        &mut new_style.visuals.widgets.hovered,
        &mut new_style.visuals.widgets.active,
        &mut new_style.visuals.widgets.open,
    ];

    for state in widget_states {
        state.bg_fill = colorscheme["faint_bg_color"];
        state.fg_stroke.color = colorscheme["code_bg_color"];
        state.weak_bg_fill = colorscheme["faint_bg_color"];
        state.bg_stroke.color = colorscheme["window_fill"]; 
    }

    // new_style.visuals.extreme_bg_color = colorscheme["extreme_bg_color"];
    // new_style.visuals.faint_bg_color = colorscheme["faint_bg_color"];
    // new_style.visuals.code_bg_color = colorscheme["faint_bg_color"];
    // new_style.visuals.panel_fill = colorscheme["faint_bg_color"];
    // new_style.visuals.window_fill = colorscheme["faint_bg_color"];

    // new_style.visuals.hyperlink_color = colorscheme["window_fill"];
    // new_style.visuals.window_stroke.color = colorscheme["code_bg_color"];
    // new_style.visuals.warn_fg_color = colorscheme["window_fill"];
    // new_style.visuals.error_fg_color = colorscheme["window_fill"];

    // let widget_states = [
    //     &mut new_style.visuals.widgets.noninteractive,
    //     &mut new_style.visuals.widgets.inactive,
    //     &mut new_style.visuals.widgets.hovered,
    //     &mut new_style.visuals.widgets.active,
    //     &mut new_style.visuals.widgets.open,
    // ];

    // for state in widget_states {
    //     state.bg_fill = colorscheme["faint_bg_color"];
    //     state.fg_stroke.color = colorscheme["code_bg_color"];
    //     state.weak_bg_fill = colorscheme["faint_bg_color"];
    //     state.bg_stroke.color = colorscheme["faint_bg_color"]; 
    // }

    ui.ctx().set_style(new_style);

}
/*
    faint C= code
    window C= code
    panel C= code
    extreme C= code
 */
impl colorscheme {
    pub fn try_use_colorscheme(&mut self, ui: &mut egui::Ui, filename: &String) -> bool {
        if let Some(colors) = try_get_colorscheme(&filename) {
            self.current = colors.clone();
            self.name = filename.clone();
            set_colorscheme(ui, &colors);
            return true;
        }
        return false;
    }
    pub fn use_random_colorscheme(&mut self, ui: &mut egui::Ui) {
        let entries = fs::read_dir("./resources/colorschemes").unwrap();
        let files: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .filter(|p| p.extension().map_or(false, |ext| ext == "toml"))
            .collect();
        let mut rng = thread_rng();
        let chosen = files.choose(&mut rng).cloned().unwrap();
        let res = fs::read_to_string(&chosen).unwrap();
        let color_scheme_unparsed: HashMap<String, String> = toml::from_str(&res).unwrap();
        let mut color_scheme: HashMap<String, Color32> = Default::default();
        for (key, val) in color_scheme_unparsed.into_iter() {
            color_scheme.insert(key, Color32::from_hex(&val).unwrap());
        }
        set_colorscheme(ui, &color_scheme);
        self.current = color_scheme;
        self.name = chosen.file_name().unwrap().to_str().unwrap().to_string();
    }
}
