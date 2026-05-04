use eframe::egui::{
    self, CentralPanel, Color32, ComboBox, CornerRadius, Pos2, Rect, Sense, Shape, SidePanel,
    Slider, Stroke, StrokeKind, Vec2, vec2,
};
use seg_lcd_rust::{
    A, B, Cell, CellKind, D, DIGIT_ADVANCE, DIGIT_HEIGHT, DIGIT_WIDTH, E, G, HexColor, LcdStyle,
    SEGMENTS, Theme, digit_segment_points, format_segment_mask_binary, format_segment_mask_hex,
    format_segment_mask_letters, parse_cells, render_cells_svg, render_svg,
};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([980.0, 420.0])
            .with_min_inner_size([720.0, 320.0]),
        ..Default::default()
    };

    eframe::run_native(
        "seg-lcd-rust",
        options,
        Box::new(|_cc| Ok(Box::new(LcdApp::default()))),
    )
}

#[derive(Debug)]
struct LcdApp {
    text: String,
    custom_mask: u8,
    show_custom: bool,
    theme: Theme,
    style: LcdStyle,
    last_export: Option<String>,
}

impl Default for LcdApp {
    fn default() -> Self {
        let theme = Theme::Classic;
        Self {
            text: "12:34.5".to_string(),
            custom_mask: A | B | D | E | G,
            show_custom: false,
            theme,
            style: theme.style(),
            last_export: None,
        }
    }
}

impl eframe::App for LcdApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::light());

        SidePanel::right("controls")
            .resizable(false)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading("Display");
                ui.add_space(8.0);
                ui.label("Text");
                ui.text_edit_singleline(&mut self.text);

                ui.add_space(12.0);
                ui.checkbox(&mut self.show_custom, "Preview custom digit");
                segment_editor(ui, &mut self.custom_mask);

                ui.add_space(16.0);
                let mut selected_theme = self.theme;
                ComboBox::from_label("Theme")
                    .selected_text(self.theme.label())
                    .show_ui(ui, |ui| {
                        for theme in Theme::ALL {
                            ui.selectable_value(&mut selected_theme, theme, theme.label());
                        }
                    });
                if selected_theme != self.theme {
                    self.theme = selected_theme;
                    self.style = selected_theme.style();
                }

                ui.add_space(12.0);
                color_row(ui, "Active segment", &mut self.style.on);
                color_row(ui, "Inactive segment", &mut self.style.off);
                color_row(ui, "Background", &mut self.style.background);
                color_row(ui, "Panel", &mut self.style.panel);

                ui.add_space(12.0);
                ui.add(Slider::new(&mut self.style.inactive_opacity, 0.0..=1.0).text("Inactive"));
                ui.checkbox(&mut self.style.glow, "Glow");
                ui.checkbox(&mut self.style.glass, "Glass highlight");

                ui.add_space(16.0);
                if ui.button("Export SVG").clicked() {
                    match std::fs::write("gui-display.svg", self.render_svg()) {
                        Ok(()) => self.last_export = Some("Wrote gui-display.svg".to_string()),
                        Err(error) => self.last_export = Some(format!("Export failed: {error}")),
                    }
                }
                if let Some(message) = &self.last_export {
                    ui.label(message);
                }
            });

        CentralPanel::default()
            .frame(egui::Frame::NONE.fill(to_color32(self.style.background)))
            .show(ctx, |ui| {
                let available = ui.available_size();
                let (rect, _) = ui.allocate_exact_size(available, Sense::hover());
                let cells = self.preview_cells();
                paint_lcd(ui.painter(), rect.shrink(24.0), &cells, self.style);
            });
    }
}

impl LcdApp {
    fn preview_cells(&self) -> Vec<Cell> {
        if self.show_custom {
            vec![Cell {
                kind: CellKind::Segments(self.custom_mask),
                decimal: false,
            }]
        } else {
            parse_cells(&self.text)
        }
    }

    fn render_svg(&self) -> String {
        if self.show_custom {
            render_cells_svg(&self.preview_cells(), self.style)
        } else {
            render_svg(&self.text, self.style)
        }
    }
}

fn segment_editor(ui: &mut egui::Ui, mask: &mut u8) {
    ui.horizontal_wrapped(|ui| {
        for (segment, name) in SEGMENTS {
            let mut enabled = *mask & segment != 0;
            if ui
                .selectable_label(enabled, name)
                .on_hover_text(format!("Toggle segment {name}"))
                .clicked()
            {
                enabled = !enabled;
                if enabled {
                    *mask |= segment;
                } else {
                    *mask &= !segment;
                }
            }
        }
    });

    ui.horizontal_wrapped(|ui| {
        ui.monospace(format_segment_mask_letters(*mask));
        ui.monospace(format_segment_mask_binary(*mask));
        ui.monospace(format_segment_mask_hex(*mask));
    });
}

fn paint_lcd(painter: &egui::Painter, rect: Rect, cells: &[Cell], style: LcdStyle) {
    painter.rect(
        rect,
        CornerRadius::same(14),
        to_color32(style.panel),
        Stroke::new(1.0, to_color32(style.off).linear_multiply(0.2)),
        StrokeKind::Inside,
    );

    let units = cells
        .iter()
        .map(|cell| match cell.kind {
            CellKind::Colon => 0.34,
            CellKind::Segments(_) | CellKind::Blank => 1.0,
        })
        .sum::<f32>()
        .max(1.0);
    let gap_units = (cells.len().saturating_sub(1)) as f32 * 0.14;
    let scale =
        (rect.width() / (units + gap_units) / DIGIT_WIDTH).min(rect.height() / DIGIT_HEIGHT);
    let digit_w = DIGIT_WIDTH * scale;
    let digit_h = DIGIT_HEIGHT * scale;
    let gap = 12.0 * scale;
    let content_w = cells
        .iter()
        .map(|cell| match cell.kind {
            CellKind::Colon => seg_lcd_rust::COLON_ADVANCE * scale,
            CellKind::Segments(_) | CellKind::Blank => digit_w,
        })
        .sum::<f32>()
        + gap * cells.len().saturating_sub(1) as f32;
    let mut x = rect.center().x - content_w / 2.0;
    let y = rect.center().y - digit_h / 2.0;

    for cell in cells.iter().copied() {
        match cell.kind {
            CellKind::Segments(mask) => {
                paint_digit(painter, Pos2::new(x, y), scale, mask, cell.decimal, style);
                x += DIGIT_ADVANCE * scale + gap;
            }
            CellKind::Colon => {
                paint_dot(
                    painter,
                    Pos2::new(x + 8.0 * scale, y + 48.0 * scale),
                    5.0 * scale,
                    true,
                    style,
                );
                paint_dot(
                    painter,
                    Pos2::new(x + 8.0 * scale, y + 88.0 * scale),
                    5.0 * scale,
                    true,
                    style,
                );
                x += seg_lcd_rust::COLON_ADVANCE * scale + gap;
            }
            CellKind::Blank => {
                paint_digit(painter, Pos2::new(x, y), scale, 0, cell.decimal, style);
                x += DIGIT_ADVANCE * scale + gap;
            }
        }
    }

    if style.glass {
        let highlight = Rect::from_min_size(
            rect.min + vec2(12.0, 12.0),
            Vec2::new(rect.width() - 24.0, rect.height() * 0.25),
        );
        painter.rect_filled(
            highlight,
            CornerRadius::same(10),
            Color32::from_white_alpha(30),
        );
    }
}

fn paint_digit(
    painter: &egui::Painter,
    origin: Pos2,
    scale: f32,
    mask: u8,
    decimal: bool,
    style: LcdStyle,
) {
    for (segment, _) in SEGMENTS {
        paint_segment(
            painter,
            mask,
            segment,
            points(origin, scale, digit_segment_points(segment)),
            style,
        );
    }
    paint_dot(
        painter,
        origin + vec2(78.0 * scale, 120.0 * scale),
        4.0 * scale,
        decimal,
        style,
    );
}

fn paint_segment(
    painter: &egui::Painter,
    mask: u8,
    segment: u8,
    points: Vec<Pos2>,
    style: LcdStyle,
) {
    let is_on = mask & segment != 0;
    let fill = if is_on {
        to_color32(style.on)
    } else {
        to_color32(style.off).linear_multiply(style.inactive_opacity)
    };

    if is_on && style.glow {
        painter.add(Shape::convex_polygon(
            points.clone(),
            to_color32(style.on).linear_multiply(0.24),
            Stroke::new(5.0, to_color32(style.on).linear_multiply(0.12)),
        ));
    }

    painter.add(Shape::convex_polygon(
        points,
        fill,
        Stroke::new(1.0, Color32::from_white_alpha(28)),
    ));
}

fn paint_dot(painter: &egui::Painter, center: Pos2, radius: f32, is_on: bool, style: LcdStyle) {
    let fill = if is_on {
        to_color32(style.on)
    } else {
        to_color32(style.off).linear_multiply(style.inactive_opacity)
    };
    if is_on && style.glow {
        painter.circle_filled(
            center,
            radius * 2.2,
            to_color32(style.on).linear_multiply(0.18),
        );
    }
    painter.circle_filled(center, radius, fill);
}

fn points(origin: Pos2, scale: f32, points: [(f32, f32); 6]) -> Vec<Pos2> {
    points
        .into_iter()
        .map(|(x, y)| origin + vec2(x * scale, y * scale))
        .collect()
}

fn color_row(ui: &mut egui::Ui, label: &str, color: &mut HexColor) {
    ui.horizontal(|ui| {
        let mut egui_color = to_color32(*color);
        ui.color_edit_button_srgba(&mut egui_color);
        *color = from_color32(egui_color);
        ui.label(label);
    });
}

fn to_color32(color: HexColor) -> Color32 {
    Color32::from_rgb(color.r, color.g, color.b)
}

fn from_color32(color: Color32) -> HexColor {
    HexColor {
        r: color.r(),
        g: color.g(),
        b: color.b(),
    }
}
