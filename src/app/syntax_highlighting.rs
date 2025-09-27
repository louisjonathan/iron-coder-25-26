use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::easy::HighlightLines;
use syntect::util::LinesWithEndings;
use eframe::egui::{self, Color32, TextFormat};
use std::path::Path;

// simple implementation of syntax highlighting with syntect ymmv

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_theme: String,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            current_theme: "base16-ocean.dark".to_string(),
        }
    }
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_theme(&mut self, theme_name: &str) {
        if self.theme_set.themes.contains_key(theme_name) {
            self.current_theme = theme_name.to_string();
        }
    }
    pub fn get_current_theme(&self) -> &str {
        &self.current_theme
    }   
    pub fn available_themes(&self) -> Vec<&String> {
        self.theme_set.themes.keys().collect()
    }

    pub fn detect_language(&self, file_path: &Path) -> Option<&str> {
        let extension = file_path.extension()?.to_str()?;
        match extension {
            "rs" => Some("Rust"),
            "toml" => Some("TOML"),
            "json" => Some("JSON"),
            _ => None,
        }
    }

    fn style_to_text_format(&self, style: &Style) -> TextFormat {
        let mut format = TextFormat::default();

        let fg = style.foreground;
        format.color = Color32::from_rgb(fg.r, fg.g, fg.b);
        
        if style.font_style.contains(syntect::highlighting::FontStyle::BOLD) {
            format.background = Color32::from_rgba_premultiplied(255, 255, 255, 20); // bold
        }
        if style.font_style.contains(syntect::highlighting::FontStyle::ITALIC) {
            format.italics = true;
        }
        if style.font_style.contains(syntect::highlighting::FontStyle::UNDERLINE) {
            format.underline = egui::Stroke::new(1.0, format.color);
        }
        
        format
    }

    pub fn highlight_code(&self, code: &str, language: Option<&str>) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();

        let syntax = if let Some(lang) = language {
            self.syntax_set.find_syntax_by_name(lang)
                .or_else(|| self.syntax_set.find_syntax_by_extension(lang))
                .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
        } else {
            self.syntax_set.find_syntax_plain_text()
        };

        let theme = &self.theme_set.themes[&self.current_theme];
        let mut highlighter = HighlightLines::new(syntax, theme);

        for line in LinesWithEndings::from(code) {
            let ranges = highlighter.highlight_line(line, &self.syntax_set)
                .unwrap_or_else(|_| vec![(syntect::highlighting::Style::default(), line)]);

            for (style, text) in ranges {
                let format = self.style_to_text_format(&style);
                job.append(text, 0.0, format);
            }
        }

        job
    }
}