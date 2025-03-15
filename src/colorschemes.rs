use egui::Color32;
// https://github.com/Experience-Monks/nice-color-palettes/tree/master
use std::collections::HashMap;
use toml;
/**
 * @brief Sets colorscheme from .toml file.
 *
 * @details Uses serde to deserialize a .toml color scheme,
 * sets egui context's style.visuals accordingly.
 *
 * @param [in] context Egui context to modify.
 * @param [in] path Path of .toml file.
 *
 * @returns
 * 	0 : error
 *  1 : success
 * */

pub fn set_color_scheme(context: &egui::Context, path: &str) -> i32 {
    let colorscheme: HashMap<String, Color32> = parse_color_scheme(path);
    let mut new_style = (*context.style()).clone();

    new_style.visuals.extreme_bg_color = colorscheme["extreme_bg_color"];
    new_style.visuals.faint_bg_color = colorscheme["faint_bg_color"];
    new_style.visuals.code_bg_color = colorscheme["code_bg_color"];
    new_style.visuals.panel_fill = colorscheme["panel_fill"];
    new_style.visuals.window_fill = colorscheme["window_fill"];
    new_style.visuals.hyperlink_color = colorscheme["hyperlink_color"];
    new_style.visuals.window_stroke.color = colorscheme["window_stroke_color"];
    new_style.visuals.warn_fg_color = colorscheme["warn_fg_color"];
    new_style.visuals.error_fg_color = colorscheme["error_fg_color"];

    context.set_style(new_style);
    return 0;
}

/**
 * @brief Reads .toml file into hashmap.
 *
 * @details Given .toml file path,
 * return hashmap of String, Color32
 *
 * @param [in] path Path of .toml file.
 *
 * @returns
 * 	HashMap<String, Color32>
 * */

fn parse_color_scheme(path: &str) -> HashMap<String, Color32> {
    let contents: &str = &std::fs::read_to_string(path).expect("failed to parse file at path");
    let unparsed: HashMap<String, String> = toml::from_str(contents).unwrap();
    let mut parsed: HashMap<String, Color32> = Default::default();
    for (key, value) in unparsed.into_iter() {
        parsed.insert(
            key,
            Color32::from_hex(&value).expect("expected hex rgb values"),
        );
    }
    return parsed;
}
