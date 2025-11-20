use egui::{Color32, Stroke, ahash::random_state, text_selection::visuals};
// https://github.com/Experience-Monks/nice-color-palettes/tree/master
use rand::{prelude::*, thread_rng};
use std::{collections::{HashMap, HashSet}, io::Write, sync::Mutex};
use std::{
    fs,
    path::{Path, PathBuf},
};
use syntect::highlighting::Color;
use toml;

// ===== WCAG Contrast Calculation =====

/// Use WCAG formula to calculate relative luminance of a color
fn relative_luminance(color: &Color32) -> f32 {
    fn channel_luminance(channel: u8) -> f32 {
        let c = (channel as f32) / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    0.2126 * channel_luminance(color.r()) +
    0.7152 * channel_luminance(color.g()) +
    0.0722 * channel_luminance(color.b())
}

/// Calculate contrast ratio between two colors using WCAG formula
fn contrast_ratio(color1: &Color32, color2: &Color32) -> f32 {
    let l1 = relative_luminance(color1);
    let l2 = relative_luminance(color2);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

/// Select black or white text based on background luminance for optimal contrast
pub fn text_color_for_background(background: &Color32) -> Color32 {
    let luminance = relative_luminance(background);
    if luminance > 0.5 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

// ===== Latching Debug Output =====
// Only prints when values change 

static DEBUG_LATCHES: Mutex<Option<HashSet<String>>> = Mutex::new(None);

/// Print a debug message only once per unique key for debug statements that will otherwise print once per frame
pub fn debug_once(key: &str, message: String) {
    let mut latches = DEBUG_LATCHES.lock().unwrap();
    if latches.is_none() {
        *latches = Some(HashSet::new());
    }

    if let Some(set) = latches.as_mut() {
        if set.insert(key.to_string()) {
            println!("{}", message);
        }
    }
}

/// Calculate RGB distance between two colors (Euclidean distance in RGB space)
fn rgb_distance(c1: &Color32, c2: &Color32) -> f32 {
    let dr = (c1.r() as f32 - c2.r() as f32).abs();
    let dg = (c1.g() as f32 - c2.g() as f32).abs();
    let db = (c1.b() as f32 - c2.b() as f32).abs();
    (dr.powi(2) + dg.powi(2) + db.powi(2)).sqrt()
}

/// Check if a color is too close to monochrome (black/white/gray)
/// Returns true if R, G, B values are too similar (low saturation)
fn is_near_monochrome(color: &Color32) -> bool {
    let r = color.r() as f32;
    let g = color.g() as f32;
    let b = color.b() as f32;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);

    // If the difference between max and min is less than 30, it's too gray
    let saturation = max - min;
    saturation < 30.0
}

// ===== HSV and RGB Conversion =====
/// Convert RGB to HSV 
fn rgb_to_hsv(color: &Color32) -> (f32, f32, f32) {
    let r = color.r() as f32 / 255.0;
    let g = color.g() as f32 / 255.0;
    let b = color.b() as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let s = if max == 0.0 { 0.0 } else { delta / max };
    let v = max;

    (h.rem_euclid(360.0), s, v)
}

/// Convert HSV back to RGB
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color32 {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Color32::from_rgb(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}


// ===== Utility Helpers =====

/// Brighten a color by boosting its value/luminance by 50%
fn brighten(color: &Color32) -> Color32 {
    let (h, s, v) = rgb_to_hsv(color);
    // Boost luminance by 50% towards maximum (1.0)
    let new_v = v + (1.0 - v) * 0.5;
    hsv_to_rgb(h, s, new_v)
}

/// Generate a triadic color (120° hue rotation) with guaranteed saturation and brightness
fn generate_triadic(base: &Color32, offset: f32) -> Color32 {
    let (h, s, v) = rgb_to_hsv(base);
    let new_h = (h + offset).rem_euclid(360.0);

    // Ensure saturation is at least 70% for vibrant colors
    let new_s = s.max(0.7);

    // Ensure value (brightness) is at least 60%
    let new_v = v.max(0.6);

    hsv_to_rgb(new_h, new_s, new_v)
}

/// Generate a complementary color (180° hue rotation) with guaranteed saturation and brightness
fn generate_complementary(base: &Color32) -> Color32 {
    let (h, s, v) = rgb_to_hsv(base);
    let new_h = (h + 180.0).rem_euclid(360.0);

    // Ensure saturation is at least 70% for vibrant colors
    let new_s = s.max(0.7);

    // Ensure value (brightness) is at least 60%
    let new_v = v.max(0.6);

    hsv_to_rgb(new_h, new_s, new_v)
}


// ===== Color Finding Algorithm =====
/// Calculate 3 contrasting colors for canvas use
/// Never returns monochrome colors! Instead uses color theory to generate missing colors if needed
/// Returns exactly 3 colors: [0] Primary (group borders), [1] Secondary (wires), [2] Tertiary (pins)
pub fn calculate_contrast_colors(
    background: &Color32,
    colorscheme: &HashMap<String, Color32>,
) -> Vec<Color32> {

    let mut selected = Vec::new();
    let mut remaining: Vec<_> = colorscheme.iter()
        .filter(|(_, color)| !is_near_monochrome(color))
        .collect();

    // Find up to 3 non-monochrome colors from the scheme
    for _ in 0..3 {
        if remaining.is_empty() {
            break;
        }

        let mut best_score = 0.0;
        let mut best_idx = 0;

        for (idx, (name, color)) in remaining.iter().enumerate() {
            // Distance from background
            let bg_distance = rgb_distance(background, color);

            // Minimum distance from already-selected colors
            let min_selected_distance = if selected.is_empty() {
                f32::MAX
            } else {
                selected.iter()
                    .map(|selected_color| rgb_distance(color, selected_color))
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0)
            };

            // Score is the minimum of the two distances (we want BOTH to be large)
            let score = bg_distance.min(min_selected_distance);

            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }

        let (_, color) = remaining.remove(best_idx);
        // Brighten the selected color by 50%
        selected.push(brighten(color));
    }

    // If we found fewer than 3 colors, use color theory to generate the rest
    let mut result = match selected.len() {
        0 => {
            // No colored options at all - generate from background
            // Use complementary and triadic colors
            let base = generate_complementary(background);
            let second = generate_triadic(&base, 120.0);
            let third = generate_triadic(&base, 240.0);
            vec![base, second, third]
        }
        1 => {
            // Found 1 color - generate 2 more using triadic harmony
            let base = selected[0];
            let second = generate_triadic(&base, 120.0);
            let third = generate_triadic(&base, 240.0);
            vec![base, second, third]
        }
        2 => {
            // Found 2 colors - generate 1 more based on the first
            let base = selected[0];
            let second = selected[1];
            let third = generate_complementary(&base);
            vec![base, second, third]
        }
        _ => {
            // Found 3 or more - use first 3
            selected.into_iter().take(3).collect()
        }
    };

    // Swap indices 0 and 2 so pins get the most contrasting color and I don't have to refactor everything <3
    result.swap(0, 2);
    result
}

pub struct colorscheme {
    pub all_names: Vec<String>,
    pub current: HashMap<String, Color32>,
    pub name: String,
    pub contrast_colors: Vec<Color32>,
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

        // Calculate contrast colors for canvas (using canvas background)

        ///NOTE: If you change what key is used for canvas background, update here too
        let background = current.get("window_fill").copied().unwrap_or(Color32::GRAY);
        let contrast_colors = calculate_contrast_colors(&background, &current);

        Self {
           all_names: get_colorscheme_filenames(),
           current,
           name: "default".to_string(),
           contrast_colors,
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
        .map(|p| {
            p.file_name()
                .expect(&format!(
                    "failed to get filename for colorscheme {}",
                    p.display()
                ))
                .to_string_lossy()
                .into_owned()
        })
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
            let background = colors.get("window_fill").copied().unwrap_or(Color32::GRAY);
            self.contrast_colors = calculate_contrast_colors(&background, &colors);

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
        self.current = color_scheme.clone();
        self.name = chosen.file_name().unwrap().to_str().unwrap().to_string();

        let background = color_scheme.get("window_fill").copied().unwrap_or(Color32::GRAY);
        self.contrast_colors = calculate_contrast_colors(&background, &color_scheme);
    }
}
