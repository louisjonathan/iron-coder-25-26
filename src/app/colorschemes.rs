use egui::{ahash::random_state, text_selection::visuals, Color32, Stroke};
// https://github.com/Experience-Monks/nice-color-palettes/tree/master
use rand::prelude::*;
use std::collections::HashMap;
use toml;

pub struct colorschemes {
    configs: Vec<&'static str>,
}
impl Default for colorschemes {
    fn default() -> Self {
        Self {
            configs: vec![
                include_str!("../../resources/colorschemes/0.toml"),
                include_str!("../../resources/colorschemes/1.toml"),
                include_str!("../../resources/colorschemes/2.toml"),
                include_str!("../../resources/colorschemes/3.toml"),
                include_str!("../../resources/colorschemes/4.toml"),
                include_str!("../../resources/colorschemes/5.toml"),
                include_str!("../../resources/colorschemes/6.toml"),
                include_str!("../../resources/colorschemes/7.toml"),
                include_str!("../../resources/colorschemes/8.toml"),
                include_str!("../../resources/colorschemes/9.toml"),
                include_str!("../../resources/colorschemes/10.toml"),
                include_str!("../../resources/colorschemes/11.toml"),
                include_str!("../../resources/colorschemes/12.toml"),
                include_str!("../../resources/colorschemes/13.toml"),
                include_str!("../../resources/colorschemes/14.toml"),
                include_str!("../../resources/colorschemes/15.toml"),
                include_str!("../../resources/colorschemes/16.toml"),
                include_str!("../../resources/colorschemes/17.toml"),
                include_str!("../../resources/colorschemes/18.toml"),
                include_str!("../../resources/colorschemes/19.toml"),
                include_str!("../../resources/colorschemes/20.toml"),
                include_str!("../../resources/colorschemes/21.toml"),
                include_str!("../../resources/colorschemes/22.toml"),
                include_str!("../../resources/colorschemes/23.toml"),
                include_str!("../../resources/colorschemes/24.toml"),
                include_str!("../../resources/colorschemes/25.toml"),
                include_str!("../../resources/colorschemes/26.toml"),
                include_str!("../../resources/colorschemes/27.toml"),
                include_str!("../../resources/colorschemes/28.toml"),
                include_str!("../../resources/colorschemes/29.toml"),
                include_str!("../../resources/colorschemes/30.toml"),
                include_str!("../../resources/colorschemes/31.toml"),
                include_str!("../../resources/colorschemes/32.toml"),
                include_str!("../../resources/colorschemes/33.toml"),
                include_str!("../../resources/colorschemes/34.toml"),
                include_str!("../../resources/colorschemes/35.toml"),
                include_str!("../../resources/colorschemes/36.toml"),
                include_str!("../../resources/colorschemes/37.toml"),
                include_str!("../../resources/colorschemes/38.toml"),
                include_str!("../../resources/colorschemes/39.toml"),
                include_str!("../../resources/colorschemes/40.toml"),
                include_str!("../../resources/colorschemes/41.toml"),
                include_str!("../../resources/colorschemes/42.toml"),
                include_str!("../../resources/colorschemes/43.toml"),
                include_str!("../../resources/colorschemes/44.toml"),
                include_str!("../../resources/colorschemes/45.toml"),
                include_str!("../../resources/colorschemes/46.toml"),
                include_str!("../../resources/colorschemes/47.toml"),
                include_str!("../../resources/colorschemes/48.toml"),
                include_str!("../../resources/colorschemes/49.toml"),
                include_str!("../../resources/colorschemes/50.toml"),
                include_str!("../../resources/colorschemes/51.toml"),
                include_str!("../../resources/colorschemes/52.toml"),
                include_str!("../../resources/colorschemes/53.toml"),
                include_str!("../../resources/colorschemes/54.toml"),
                include_str!("../../resources/colorschemes/55.toml"),
                include_str!("../../resources/colorschemes/56.toml"),
                include_str!("../../resources/colorschemes/57.toml"),
                include_str!("../../resources/colorschemes/58.toml"),
                include_str!("../../resources/colorschemes/59.toml"),
                include_str!("../../resources/colorschemes/60.toml"),
                include_str!("../../resources/colorschemes/61.toml"),
                include_str!("../../resources/colorschemes/62.toml"),
                include_str!("../../resources/colorschemes/63.toml"),
                include_str!("../../resources/colorschemes/64.toml"),
                include_str!("../../resources/colorschemes/65.toml"),
                include_str!("../../resources/colorschemes/66.toml"),
                include_str!("../../resources/colorschemes/67.toml"),
                include_str!("../../resources/colorschemes/68.toml"),
                include_str!("../../resources/colorschemes/69.toml"),
                include_str!("../../resources/colorschemes/70.toml"),
                include_str!("../../resources/colorschemes/71.toml"),
                include_str!("../../resources/colorschemes/72.toml"),
                include_str!("../../resources/colorschemes/73.toml"),
                include_str!("../../resources/colorschemes/74.toml"),
                include_str!("../../resources/colorschemes/75.toml"),
                include_str!("../../resources/colorschemes/76.toml"),
                include_str!("../../resources/colorschemes/77.toml"),
                include_str!("../../resources/colorschemes/78.toml"),
                include_str!("../../resources/colorschemes/79.toml"),
                include_str!("../../resources/colorschemes/80.toml"),
                include_str!("../../resources/colorschemes/81.toml"),
                include_str!("../../resources/colorschemes/82.toml"),
                include_str!("../../resources/colorschemes/83.toml"),
                include_str!("../../resources/colorschemes/84.toml"),
                include_str!("../../resources/colorschemes/85.toml"),
                include_str!("../../resources/colorschemes/86.toml"),
                include_str!("../../resources/colorschemes/87.toml"),
                include_str!("../../resources/colorschemes/88.toml"),
                include_str!("../../resources/colorschemes/89.toml"),
                include_str!("../../resources/colorschemes/90.toml"),
                include_str!("../../resources/colorschemes/91.toml"),
                include_str!("../../resources/colorschemes/92.toml"),
                include_str!("../../resources/colorschemes/93.toml"),
                include_str!("../../resources/colorschemes/94.toml"),
                include_str!("../../resources/colorschemes/95.toml"),
                include_str!("../../resources/colorschemes/96.toml"),
                include_str!("../../resources/colorschemes/97.toml"),
                include_str!("../../resources/colorschemes/98.toml"),
                include_str!("../../resources/colorschemes/99.toml"),
                include_str!("../../resources/colorschemes/example_colorscheme.toml"),
            ],
        }
    }
}

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
impl colorschemes {
    pub fn get_random_color_scheme() -> u8 {
        let num: u8 = rand::random::<u8>();
        return num % 99;
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
     *  HashMap<String, Color32>
     * */

    pub fn get_color_scheme(&mut self, idx: &u8) -> HashMap<String, Color32> {
        let contents = self.configs[*idx as usize];
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
    pub fn set_color_scheme(&mut self, context: &egui::Context, idx: &u8) -> i32 {
        let colorscheme: HashMap<String, Color32> = self.get_color_scheme(idx);

        let mut new_style = (*context.style()).clone();
        //new_style.visuals.widgets.style()
        //set text override color (requires ui.visuals_mut().widgets.*.fg_stroke_color to be set for some reason)
        new_style.visuals.override_text_color = Some(colorscheme["extreme_bg_color"]);
        new_style.visuals.extreme_bg_color = colorscheme["faint_bg_color"];
        new_style.visuals.faint_bg_color = colorscheme["extreme_bg_color"];
        new_style.visuals.code_bg_color = colorscheme["code_bg_color"];
        new_style.visuals.panel_fill = colorscheme["panel_fill"];
        new_style.visuals.window_fill = colorscheme["window_fill"];
        new_style.visuals.hyperlink_color = colorscheme["hyperlink_color"];
        new_style.visuals.window_stroke.color = colorscheme["window_stroke_color"];
        new_style.visuals.warn_fg_color = colorscheme["warn_fg_color"];
        new_style.visuals.error_fg_color = colorscheme["error_fg_color"];

        context.set_style(new_style);
        return 1;
    }
}
