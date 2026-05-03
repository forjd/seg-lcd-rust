use wasm_bindgen::prelude::*;

use crate::{HexColor, LcdStyle, Theme, parse_opacity, render_svg};

#[wasm_bindgen]
pub fn default_display_text() -> String {
    "12:34.5".to_string()
}

#[wasm_bindgen]
pub fn theme_names() -> String {
    let names = Theme::ALL.map(Theme::name);
    format!(
        "[{}]",
        names
            .iter()
            .map(|name| format!("\"{name}\""))
            .collect::<Vec<_>>()
            .join(",")
    )
}

#[wasm_bindgen]
pub fn render_svg_for_theme(text: &str, theme: &str) -> Result<String, JsValue> {
    let theme = theme
        .parse::<Theme>()
        .map_err(|error| JsValue::from_str(&error))?;
    Ok(render_svg(text, theme.style()))
}

#[wasm_bindgen]
#[allow(clippy::too_many_arguments)]
pub fn render_svg_with_style(
    text: &str,
    on: &str,
    off: &str,
    background: &str,
    panel: &str,
    inactive_opacity: &str,
    glow: bool,
    glass: bool,
) -> Result<String, JsValue> {
    let style = LcdStyle {
        on: parse_color(on, "on")?,
        off: parse_color(off, "off")?,
        background: parse_color(background, "background")?,
        panel: parse_color(panel, "panel")?,
        inactive_opacity: parse_opacity(inactive_opacity).map_err(|error| {
            JsValue::from_str(&error.replace("--inactive-opacity", "inactive opacity"))
        })?,
        glow,
        glass,
    };

    Ok(render_svg(text, style))
}

fn parse_color(value: &str, name: &str) -> Result<HexColor, JsValue> {
    HexColor::parse(value, name).map_err(|error| JsValue::from_str(&error))
}
