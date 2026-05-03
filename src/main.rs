use std::{env, fs, path::PathBuf};

const A: u8 = 1 << 0;
const B: u8 = 1 << 1;
const C: u8 = 1 << 2;
const D: u8 = 1 << 3;
const E: u8 = 1 << 4;
const F: u8 = 1 << 5;
const G: u8 = 1 << 6;
const ALL: u8 = A | B | C | D | E | F | G;

#[derive(Debug, Clone)]
struct SvgStyle {
    on: String,
    off: String,
    background: String,
    panel: String,
    inactive_opacity: f32,
    glow: bool,
    glass: bool,
}

impl Default for SvgStyle {
    fn default() -> Self {
        Self {
            on: "#1f3328".to_string(),
            off: "#7f9278".to_string(),
            background: "#d8e1cf".to_string(),
            panel: "#c7d2be".to_string(),
            inactive_opacity: 0.24,
            glow: false,
            glass: true,
        }
    }
}

impl SvgStyle {
    fn apply_theme(&mut self, theme: &str) -> Result<(), String> {
        match theme {
            "classic" => *self = Self::default(),
            "green" => {
                self.on = "#15351f".to_string();
                self.off = "#6f846b".to_string();
                self.background = "#dfe8d6".to_string();
                self.panel = "#c8d4bf".to_string();
                self.inactive_opacity = 0.22;
                self.glow = false;
                self.glass = true;
            }
            "amber" => {
                self.on = "#3b2408".to_string();
                self.off = "#9a762e".to_string();
                self.background = "#e8cf8c".to_string();
                self.panel = "#dbb85f".to_string();
                self.inactive_opacity = 0.28;
                self.glow = false;
                self.glass = true;
            }
            "blue" => {
                self.on = "#c9f6ff".to_string();
                self.off = "#426977".to_string();
                self.background = "#10252d".to_string();
                self.panel = "#16333d".to_string();
                self.inactive_opacity = 0.22;
                self.glow = true;
                self.glass = true;
            }
            "negative" => {
                self.on = "#dff2dc".to_string();
                self.off = "#344537".to_string();
                self.background = "#111a14".to_string();
                self.panel = "#1c2a20".to_string();
                self.inactive_opacity = 0.34;
                self.glow = true;
                self.glass = false;
            }
            _ => {
                return Err(format!(
                    "unknown SVG theme: {theme}. Use classic, green, amber, blue, or negative"
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct Style {
    on: char,
    off: char,
    show_labels: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            on: '#',
            off: ' ',
            show_labels: false,
        }
    }
}

fn main() {
    let config = match Config::from_args(env::args().skip(1)) {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            eprintln!();
            print_usage();
            std::process::exit(2);
        }
    };

    if config.help {
        print_usage();
        return;
    }

    let output = render_text(&config.text, config.style);
    println!("{output}");

    if let Some(path) = &config.svg_path {
        if let Err(error) = fs::write(path, render_svg(&config.text, &config.svg_style)) {
            eprintln!("failed to write SVG to {}: {error}", path.display());
            std::process::exit(1);
        }
        println!();
        println!("wrote {}", path.display());
    }

    if config.dump_masks {
        println!();
        print_masks(&config.text);
    }
}

#[derive(Debug, Clone)]
struct Config {
    text: String,
    style: Style,
    dump_masks: bool,
    svg_path: Option<PathBuf>,
    svg_style: SvgStyle,
    help: bool,
}

impl Config {
    fn from_args(args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut style = Style::default();
        let mut dump_masks = false;
        let mut svg_path = None;
        let mut svg_style = SvgStyle::default();
        let mut help = false;
        let mut text_parts = Vec::new();
        let mut args = args.peekable();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => help = true,
                "--labels" => style.show_labels = true,
                "--masks" => dump_masks = true,
                "--svg" => {
                    let path = args
                        .next()
                        .ok_or_else(|| "--svg requires an output path".to_string())?;
                    svg_path = Some(PathBuf::from(path));
                }
                "--theme" => {
                    let theme = args
                        .next()
                        .ok_or_else(|| "--theme requires a theme name".to_string())?;
                    svg_style.apply_theme(&theme)?;
                }
                "--on" => {
                    svg_style.on = parse_hex_color(&next_option_value(&mut args, "--on")?, "--on")?;
                }
                "--off" => {
                    svg_style.off =
                        parse_hex_color(&next_option_value(&mut args, "--off")?, "--off")?;
                }
                "--bg" => {
                    svg_style.background =
                        parse_hex_color(&next_option_value(&mut args, "--bg")?, "--bg")?;
                }
                "--panel" => {
                    svg_style.panel =
                        parse_hex_color(&next_option_value(&mut args, "--panel")?, "--panel")?;
                }
                "--inactive-opacity" => {
                    svg_style.inactive_opacity =
                        parse_opacity(&next_option_value(&mut args, "--inactive-opacity")?)?;
                }
                "--glow" => svg_style.glow = true,
                "--no-glow" => svg_style.glow = false,
                "--no-glass" => svg_style.glass = false,
                "--inverse" => {
                    style.on = ' ';
                    style.off = '#';
                }
                _ if arg.starts_with('-') => return Err(format!("unknown option: {arg}")),
                _ => text_parts.push(arg),
            }
        }

        Ok(Self {
            text: if text_parts.is_empty() {
                "12:34.5".to_string()
            } else {
                text_parts.join(" ")
            },
            style,
            dump_masks,
            svg_path,
            svg_style,
            help,
        })
    }
}

fn next_option_value(
    args: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    option: &str,
) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("{option} requires a value"))
}

fn parse_hex_color(value: &str, option: &str) -> Result<String, String> {
    let color = value.strip_prefix('#').unwrap_or(value);
    let valid_len = color.len() == 6;
    let valid_digits = color.chars().all(|ch| ch.is_ascii_hexdigit());

    if valid_len && valid_digits {
        Ok(format!("#{color}"))
    } else {
        Err(format!(
            "{option} requires a 6-digit hex color, like #1f3328"
        ))
    }
}

fn parse_opacity(value: &str) -> Result<f32, String> {
    let opacity: f32 = value
        .parse()
        .map_err(|_| "--inactive-opacity requires a number from 0.0 to 1.0".to_string())?;

    if (0.0..=1.0).contains(&opacity) {
        Ok(opacity)
    } else {
        Err("--inactive-opacity requires a number from 0.0 to 1.0".to_string())
    }
}

fn print_usage() {
    println!(
        "seven-seg-lcd - terminal seven-segment LCD simulator\n\n\
         Usage:\n  cargo run -- [OPTIONS] [TEXT]\n\n\
         Options:\n  --inverse   render dark inactive LCD blocks with clear active segments\n  \
         --labels    render segment letters instead of filled segments\n  \
         --masks     print each character's seven-bit segment mask\n  \
         --svg PATH  write a browser-viewable SVG rendering\n  \
         --theme NAME  SVG theme: classic, green, amber, blue, negative\n  \
         --on HEX    SVG active segment color\n  \
         --off HEX   SVG inactive segment color\n  \
         --bg HEX    SVG outer background color\n  \
         --panel HEX SVG display panel color\n  \
         --inactive-opacity N  SVG inactive segment opacity from 0.0 to 1.0\n  \
         --glow      add SVG glow to active segments\n  \
         --no-glow   remove SVG glow\n  \
         --no-glass  remove SVG glass highlight overlay\n  \
         -h, --help  show this help\n\n\
         Examples:\n  cargo run -- 0123456789\n  cargo run -- --labels HELP\n  \
         cargo run -- --svg display.svg --theme amber 10:58.42\n  \
         cargo run -- --svg display.svg --theme blue --glow 88:88.88"
    );
}

fn render_text(text: &str, style: Style) -> String {
    let cells = parse_cells(text);
    let mut rows = [
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
    ];

    for cell in cells {
        let rendered = render_cell(cell, style);
        for (row, rendered_row) in rows.iter_mut().zip(rendered) {
            if !row.is_empty() {
                row.push(' ');
            }
            row.push_str(&rendered_row);
        }
    }

    rows.join("\n")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cell {
    kind: CellKind,
    decimal: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellKind {
    Segments(u8),
    Colon,
    Blank,
}

fn parse_cells(text: &str) -> Vec<Cell> {
    let mut cells: Vec<Cell> = Vec::new();

    for ch in text.chars() {
        match ch {
            '.' => {
                if let Some(last) = cells.last_mut() {
                    last.decimal = true;
                } else {
                    cells.push(Cell {
                        kind: CellKind::Blank,
                        decimal: true,
                    });
                }
            }
            ':' => cells.push(Cell {
                kind: CellKind::Colon,
                decimal: false,
            }),
            _ => cells.push(Cell {
                kind: segment_mask(ch).map_or(CellKind::Blank, CellKind::Segments),
                decimal: false,
            }),
        }
    }

    cells
}

fn render_cell(cell: Cell, style: Style) -> [String; 5] {
    match cell.kind {
        CellKind::Segments(mask) => render_segments(mask, cell.decimal, style),
        CellKind::Colon => [
            "  ".to_string(),
            format!("{} ", style.on),
            "  ".to_string(),
            format!("{} ", style.on),
            "  ".to_string(),
        ],
        CellKind::Blank => render_segments(0, cell.decimal, style),
    }
}

fn render_segments(mask: u8, decimal: bool, style: Style) -> [String; 5] {
    let h = |segment, label| segment_run(mask, segment, label, style);
    let v = |left, right, left_label, right_label| {
        format!(
            "{}   {} ",
            segment_char(mask, left, left_label, style),
            segment_char(mask, right, right_label, style)
        )
    };

    [
        format!(" {}  ", h(A, 'A')),
        v(F, B, 'F', 'B'),
        format!(" {}  ", h(G, 'G')),
        v(E, C, 'E', 'C'),
        format!(" {}{} ", h(D, 'D'), if decimal { style.on } else { ' ' }),
    ]
}

fn segment_run(mask: u8, segment: u8, label: char, style: Style) -> String {
    std::iter::repeat_n(segment_char(mask, segment, label, style), 3).collect()
}

fn segment_char(mask: u8, segment: u8, label: char, style: Style) -> char {
    if mask & segment != 0 {
        if style.show_labels { label } else { style.on }
    } else {
        style.off
    }
}

fn render_svg(text: &str, style: &SvgStyle) -> String {
    let cells = parse_cells(text);
    let width = svg_width(&cells);
    let height = 164;
    let mut svg = String::new();

    svg.push_str(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" role="img">
  <title>Seven-segment LCD display</title>
  <defs>
    <linearGradient id="lcd-panel" x1="0" x2="1" y1="0" y2="1">
      <stop offset="0%" stop-color="{}" stop-opacity="1"/>
      <stop offset="100%" stop-color="{}" stop-opacity="0.78"/>
    </linearGradient>
    <filter id="segment-glow" x="-40%" y="-40%" width="180%" height="180%">
      <feGaussianBlur stdDeviation="2.2" result="blur"/>
      <feMerge>
        <feMergeNode in="blur"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
  </defs>
  <rect width="100%" height="100%" rx="14" fill="{}"/>
  <rect x="8" y="8" width="{}" height="{}" rx="10" fill="url(#lcd-panel)"/>
"##,
        style.panel,
        style.background,
        style.background,
        width - 16,
        height - 16
    ));

    let mut x = 24;
    for cell in cells {
        match cell.kind {
            CellKind::Segments(mask) => {
                push_svg_digit(&mut svg, x, 22, mask, cell.decimal, style);
                x += 86;
            }
            CellKind::Colon => {
                push_svg_circle(&mut svg, x + 8, 62, 5, true, style);
                push_svg_circle(&mut svg, x + 8, 102, 5, true, style);
                x += 28;
            }
            CellKind::Blank => {
                push_svg_digit(&mut svg, x, 22, 0, cell.decimal, style);
                x += 86;
            }
        }
    }

    if style.glass {
        svg.push_str(&format!(
            r##"  <path d="M18 18 H{} Q{} 18 {} 28 V54 C{} 42 198 36 18 48 Z" fill="#ffffff" opacity="0.18"/>
"##,
            width - 18,
            width - 10,
            width - 10,
            width - 80
        ));
    }

    svg.push_str("</svg>\n");
    svg
}

fn svg_width(cells: &[Cell]) -> i32 {
    let content_width: i32 = cells
        .iter()
        .map(|cell| match cell.kind {
            CellKind::Colon => 28,
            CellKind::Segments(_) | CellKind::Blank => 86,
        })
        .sum();

    (content_width + 40).max(120)
}

fn push_svg_digit(svg: &mut String, x: i32, y: i32, mask: u8, decimal: bool, style: &SvgStyle) {
    push_svg_segment(svg, A, mask, horizontal_points(x + 6, y), style);
    push_svg_segment(svg, B, mask, vertical_points(x + 56, y + 10), style);
    push_svg_segment(svg, C, mask, vertical_points(x + 56, y + 66), style);
    push_svg_segment(svg, D, mask, horizontal_points(x + 6, y + 112), style);
    push_svg_segment(svg, E, mask, vertical_points(x, y + 66), style);
    push_svg_segment(svg, F, mask, vertical_points(x, y + 10), style);
    push_svg_segment(svg, G, mask, horizontal_points(x + 6, y + 56), style);
    push_svg_circle(svg, x + 78, y + 120, 4, decimal, style);
}

fn horizontal_points(x: i32, y: i32) -> [(i32, i32); 6] {
    [
        (x + 10, y),
        (x + 50, y),
        (x + 60, y + 8),
        (x + 50, y + 16),
        (x + 10, y + 16),
        (x, y + 8),
    ]
}

fn vertical_points(x: i32, y: i32) -> [(i32, i32); 6] {
    [
        (x + 8, y),
        (x + 16, y + 8),
        (x + 16, y + 42),
        (x + 8, y + 50),
        (x, y + 42),
        (x, y + 8),
    ]
}

fn push_svg_segment(
    svg: &mut String,
    segment: u8,
    mask: u8,
    points: [(i32, i32); 6],
    style: &SvgStyle,
) {
    let is_on = mask & segment != 0;
    let fill = if is_on { &style.on } else { &style.off };
    let opacity = if is_on { 1.0 } else { style.inactive_opacity };
    let filter = if is_on && style.glow {
        r#" filter="url(#segment-glow)""#
    } else {
        ""
    };
    let points = points
        .iter()
        .map(|(x, y)| format!("{x},{y}"))
        .collect::<Vec<_>>()
        .join(" ");

    svg.push_str(&format!(
        r##"  <polygon points="{points}" fill="{fill}" opacity="{opacity:.2}" stroke="#edf4e7" stroke-opacity="0.16" stroke-width="1"{filter}/>
"##
    ));
}

fn push_svg_circle(svg: &mut String, cx: i32, cy: i32, r: i32, is_on: bool, style: &SvgStyle) {
    let fill = if is_on { &style.on } else { &style.off };
    let opacity = if is_on { 1.0 } else { style.inactive_opacity };
    let filter = if is_on && style.glow {
        r#" filter="url(#segment-glow)""#
    } else {
        ""
    };
    svg.push_str(&format!(
        r##"  <circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}" opacity="{opacity:.2}"{filter}/>
"##
    ));
}

fn print_masks(text: &str) {
    for ch in text.chars().filter(|ch| *ch != '.') {
        match ch {
            ':' => println!(":  colon"),
            _ => match segment_mask(ch) {
                Some(mask) => println!("{ch}  {mask:07b}  {}", segment_names(mask).join(",")),
                None => println!("{ch}  unsupported"),
            },
        }
    }
}

fn segment_names(mask: u8) -> Vec<&'static str> {
    [
        (A, "A"),
        (B, "B"),
        (C, "C"),
        (D, "D"),
        (E, "E"),
        (F, "F"),
        (G, "G"),
    ]
    .iter()
    .filter_map(|(segment, name)| (mask & segment != 0).then_some(*name))
    .collect()
}

fn segment_mask(ch: char) -> Option<u8> {
    Some(match ch.to_ascii_uppercase() {
        '0' => A | B | C | D | E | F,
        '1' => B | C,
        '2' => A | B | D | E | G,
        '3' => A | B | C | D | G,
        '4' => B | C | F | G,
        '5' => A | C | D | F | G,
        '6' => A | C | D | E | F | G,
        '7' => A | B | C,
        '8' => ALL,
        '9' => A | B | C | D | F | G,
        'A' => A | B | C | E | F | G,
        'B' => C | D | E | F | G,
        'C' => A | D | E | F,
        'D' => B | C | D | E | G,
        'E' => A | D | E | F | G,
        'F' => A | E | F | G,
        'H' => B | C | E | F | G,
        'L' => D | E | F,
        'P' => A | B | E | F | G,
        'U' => B | C | D | E | F,
        '-' => G,
        '_' => D,
        ' ' => 0,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_digits_to_expected_segments() {
        assert_eq!(segment_mask('0'), Some(A | B | C | D | E | F));
        assert_eq!(segment_mask('8'), Some(ALL));
        assert_eq!(segment_mask('-'), Some(G));
    }

    #[test]
    fn attaches_decimal_point_to_previous_cell() {
        let cells = parse_cells("1.2");

        assert_eq!(
            cells,
            vec![
                Cell {
                    kind: CellKind::Segments(B | C),
                    decimal: true
                },
                Cell {
                    kind: CellKind::Segments(A | B | D | E | G),
                    decimal: false
                }
            ]
        );
    }

    #[test]
    fn renders_the_expected_number_of_rows() {
        let output = render_text("12:34", Style::default());
        assert_eq!(output.lines().count(), 5);
    }

    #[test]
    fn accepts_hex_colors_with_or_without_prefix() {
        assert_eq!(
            parse_hex_color("#102418", "--on"),
            Ok("#102418".to_string())
        );
        assert_eq!(parse_hex_color("102418", "--on"), Ok("#102418".to_string()));
        assert!(parse_hex_color("bad", "--on").is_err());
    }

    #[test]
    fn rejects_invalid_inactive_opacity() {
        assert_eq!(parse_opacity("0.25"), Ok(0.25));
        assert!(parse_opacity("-0.1").is_err());
        assert!(parse_opacity("1.1").is_err());
    }
}
