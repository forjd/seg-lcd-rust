pub const A: u8 = 1 << 0;
pub const B: u8 = 1 << 1;
pub const C: u8 = 1 << 2;
pub const D: u8 = 1 << 3;
pub const E: u8 = 1 << 4;
pub const F: u8 = 1 << 5;
pub const G: u8 = 1 << 6;
pub const ALL: u8 = A | B | C | D | E | F | G;

#[cfg(target_arch = "wasm32")]
mod wasm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Classic,
    Green,
    Amber,
    Blue,
    Negative,
}

impl Theme {
    pub const ALL: [Self; 5] = [
        Self::Classic,
        Self::Green,
        Self::Amber,
        Self::Blue,
        Self::Negative,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Green => "green",
            Self::Amber => "amber",
            Self::Blue => "blue",
            Self::Negative => "negative",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Green => "Green",
            Self::Amber => "Amber",
            Self::Blue => "Blue",
            Self::Negative => "Negative",
        }
    }

    pub fn style(self) -> LcdStyle {
        match self {
            Self::Classic => LcdStyle::default(),
            Self::Green => LcdStyle {
                on: HexColor::new(0x15351f),
                off: HexColor::new(0x6f846b),
                background: HexColor::new(0xdfe8d6),
                panel: HexColor::new(0xc8d4bf),
                inactive_opacity: 0.22,
                glow: false,
                glass: true,
            },
            Self::Amber => LcdStyle {
                on: HexColor::new(0x3b2408),
                off: HexColor::new(0x9a762e),
                background: HexColor::new(0xe8cf8c),
                panel: HexColor::new(0xdbb85f),
                inactive_opacity: 0.28,
                glow: false,
                glass: true,
            },
            Self::Blue => LcdStyle {
                on: HexColor::new(0xc9f6ff),
                off: HexColor::new(0x426977),
                background: HexColor::new(0x10252d),
                panel: HexColor::new(0x16333d),
                inactive_opacity: 0.22,
                glow: true,
                glass: true,
            },
            Self::Negative => LcdStyle {
                on: HexColor::new(0xdff2dc),
                off: HexColor::new(0x344537),
                background: HexColor::new(0x111a14),
                panel: HexColor::new(0x1c2a20),
                inactive_opacity: 0.34,
                glow: true,
                glass: false,
            },
        }
    }
}

impl std::str::FromStr for Theme {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "classic" => Ok(Self::Classic),
            "green" => Ok(Self::Green),
            "amber" => Ok(Self::Amber),
            "blue" => Ok(Self::Blue),
            "negative" => Ok(Self::Negative),
            _ => Err(format!(
                "unknown SVG theme: {value}. Use classic, green, amber, blue, or negative"
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HexColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl HexColor {
    pub const fn new(rgb: u32) -> Self {
        Self {
            r: ((rgb >> 16) & 0xff) as u8,
            g: ((rgb >> 8) & 0xff) as u8,
            b: (rgb & 0xff) as u8,
        }
    }

    pub fn parse(value: &str, option: &str) -> Result<Self, String> {
        let color = value.strip_prefix('#').unwrap_or(value);
        let valid_len = color.len() == 6;
        let valid_digits = color.chars().all(|ch| ch.is_ascii_hexdigit());

        if !(valid_len && valid_digits) {
            return Err(format!(
                "{option} requires a 6-digit hex color, like #1f3328"
            ));
        }

        let rgb = u32::from_str_radix(color, 16)
            .map_err(|_| format!("{option} requires a 6-digit hex color, like #1f3328"))?;
        Ok(Self::new(rgb))
    }

    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn to_hex_without_prefix(self) -> String {
        format!("{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LcdStyle {
    pub on: HexColor,
    pub off: HexColor,
    pub background: HexColor,
    pub panel: HexColor,
    pub inactive_opacity: f32,
    pub glow: bool,
    pub glass: bool,
}

impl Default for LcdStyle {
    fn default() -> Self {
        Self {
            on: HexColor::new(0x1f3328),
            off: HexColor::new(0x7f9278),
            background: HexColor::new(0xd8e1cf),
            panel: HexColor::new(0xc7d2be),
            inactive_opacity: 0.24,
            glow: false,
            glass: true,
        }
    }
}

pub fn parse_opacity(value: &str) -> Result<f32, String> {
    let opacity: f32 = value
        .parse()
        .map_err(|_| "--inactive-opacity requires a number from 0.0 to 1.0".to_string())?;

    if (0.0..=1.0).contains(&opacity) {
        Ok(opacity)
    } else {
        Err("--inactive-opacity requires a number from 0.0 to 1.0".to_string())
    }
}

pub fn parse_segment_mask(value: &str) -> Result<u8, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("segment mask cannot be empty".to_string());
    }

    if let Some(binary) = trimmed
        .strip_prefix("0b")
        .or_else(|| trimmed.strip_prefix("0B"))
    {
        return parse_numeric_segment_mask(binary, 2, value);
    }

    if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        return parse_numeric_segment_mask(hex, 16, value);
    }

    let mut mask = 0;
    let mut saw_segment = false;

    for ch in trimmed.chars() {
        let segment = match ch.to_ascii_uppercase() {
            'A' => A,
            'B' => B,
            'C' => C,
            'D' => D,
            'E' => E,
            'F' => F,
            'G' => G,
            ',' | '+' | '-' | '_' | ' ' => continue,
            _ => {
                return Err(format!(
                    "invalid segment mask: {value}. Use segment letters A-G, 0b1011011, or 0x5b"
                ));
            }
        };
        mask |= segment;
        saw_segment = true;
    }

    if saw_segment {
        Ok(mask)
    } else {
        Err(format!(
            "invalid segment mask: {value}. Use segment letters A-G, 0b1011011, or 0x5b"
        ))
    }
}

fn parse_numeric_segment_mask(value: &str, radix: u32, original: &str) -> Result<u8, String> {
    let mask = u8::from_str_radix(value, radix).map_err(|_| {
        format!("invalid segment mask: {original}. Use segment letters A-G, 0b1011011, or 0x5b")
    })?;

    if mask <= ALL {
        Ok(mask)
    } else {
        Err(format!(
            "segment mask {original} enables bits outside the seven segments A-G"
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TerminalStyle {
    pub on: char,
    pub off: char,
    pub show_labels: bool,
}

impl Default for TerminalStyle {
    fn default() -> Self {
        Self {
            on: '#',
            off: ' ',
            show_labels: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub kind: CellKind,
    pub decimal: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellKind {
    Segments(u8),
    Colon,
    Blank,
}

pub fn parse_cells(text: &str) -> Vec<Cell> {
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

pub fn render_text(text: &str, style: TerminalStyle) -> String {
    let cells = parse_cells(text);
    render_cells_text(&cells, style)
}

pub fn render_cells_text(cells: &[Cell], style: TerminalStyle) -> String {
    let row_capacity = cells.len().saturating_mul(7);
    let mut rows = [
        String::with_capacity(row_capacity),
        String::with_capacity(row_capacity),
        String::with_capacity(row_capacity),
        String::with_capacity(row_capacity),
        String::with_capacity(row_capacity),
    ];

    for cell in cells.iter().copied() {
        push_text_cell(&mut rows, cell, style);
    }

    let mut output = String::with_capacity(rows.iter().map(String::len).sum::<usize>() + 4);
    for (index, row) in rows.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }
        output.push_str(row);
    }
    output
}

fn push_text_cell(rows: &mut [String; 5], cell: Cell, style: TerminalStyle) {
    if !rows[0].is_empty() {
        for row in rows.iter_mut() {
            row.push(' ');
        }
    }

    match cell.kind {
        CellKind::Segments(mask) => push_text_segments(rows, mask, cell.decimal, style),
        CellKind::Colon => {
            rows[0].push_str("  ");
            rows[1].push(style.on);
            rows[1].push(' ');
            rows[2].push_str("  ");
            rows[3].push(style.on);
            rows[3].push(' ');
            rows[4].push_str("  ");
        }
        CellKind::Blank => push_text_segments(rows, 0, cell.decimal, style),
    }
}

fn push_text_segments(rows: &mut [String; 5], mask: u8, decimal: bool, style: TerminalStyle) {
    push_horizontal_segment(&mut rows[0], mask, A, 'A', style);
    push_vertical_segments(&mut rows[1], mask, F, B, 'F', 'B', style);
    push_horizontal_segment(&mut rows[2], mask, G, 'G', style);
    push_vertical_segments(&mut rows[3], mask, E, C, 'E', 'C', style);
    push_horizontal_segment(&mut rows[4], mask, D, 'D', style);
    rows[4].push(if decimal { style.on } else { ' ' });
    rows[4].push(' ');
}

fn push_horizontal_segment(
    row: &mut String,
    mask: u8,
    segment: u8,
    label: char,
    style: TerminalStyle,
) {
    row.push(' ');
    row.extend(std::iter::repeat_n(
        segment_char(mask, segment, label, style),
        3,
    ));
    row.push_str("  ");
}

fn push_vertical_segments(
    row: &mut String,
    mask: u8,
    left: u8,
    right: u8,
    left_label: char,
    right_label: char,
    style: TerminalStyle,
) {
    row.push(segment_char(mask, left, left_label, style));
    row.push_str("   ");
    row.push(segment_char(mask, right, right_label, style));
    row.push(' ');
}

fn segment_char(mask: u8, segment: u8, label: char, style: TerminalStyle) -> char {
    if mask & segment != 0 {
        if style.show_labels { label } else { style.on }
    } else {
        style.off
    }
}

pub fn render_svg(text: &str, style: LcdStyle) -> String {
    let cells = parse_cells(text);
    render_cells_svg(&cells, style)
}

pub fn render_cells_svg(cells: &[Cell], style: LcdStyle) -> String {
    let width = svg_width(cells);
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
        style.panel.to_hex(),
        style.background.to_hex(),
        style.background.to_hex(),
        width - 16,
        height - 16
    ));

    let mut x = 24.0;
    for cell in cells.iter().copied() {
        match cell.kind {
            CellKind::Segments(mask) => {
                push_svg_digit(&mut svg, x, 22.0, mask, cell.decimal, style);
                x += DIGIT_ADVANCE;
            }
            CellKind::Colon => {
                push_svg_circle(&mut svg, x + 8.0, 62.0, 5.0, true, style);
                push_svg_circle(&mut svg, x + 8.0, 102.0, 5.0, true, style);
                x += COLON_ADVANCE;
            }
            CellKind::Blank => {
                push_svg_digit(&mut svg, x, 22.0, 0, cell.decimal, style);
                x += DIGIT_ADVANCE;
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

pub const DIGIT_WIDTH: f32 = 86.0;
pub const DIGIT_HEIGHT: f32 = 142.0;
pub const DIGIT_ADVANCE: f32 = 86.0;
pub const COLON_ADVANCE: f32 = 28.0;

pub fn svg_width(cells: &[Cell]) -> i32 {
    let content_width: f32 = cells
        .iter()
        .map(|cell| match cell.kind {
            CellKind::Colon => COLON_ADVANCE,
            CellKind::Segments(_) | CellKind::Blank => DIGIT_ADVANCE,
        })
        .sum();

    (content_width as i32 + 40).max(120)
}

pub fn horizontal_points(x: f32, y: f32) -> [(f32, f32); 6] {
    [
        (x + 10.0, y),
        (x + 50.0, y),
        (x + 60.0, y + 8.0),
        (x + 50.0, y + 16.0),
        (x + 10.0, y + 16.0),
        (x, y + 8.0),
    ]
}

pub fn vertical_points(x: f32, y: f32) -> [(f32, f32); 6] {
    [
        (x + 8.0, y),
        (x + 16.0, y + 8.0),
        (x + 16.0, y + 42.0),
        (x + 8.0, y + 50.0),
        (x, y + 42.0),
        (x, y + 8.0),
    ]
}

fn push_svg_digit(svg: &mut String, x: f32, y: f32, mask: u8, decimal: bool, style: LcdStyle) {
    push_svg_segment(svg, A, mask, horizontal_points(x + 6.0, y), style);
    push_svg_segment(svg, B, mask, vertical_points(x + 56.0, y + 10.0), style);
    push_svg_segment(svg, C, mask, vertical_points(x + 56.0, y + 66.0), style);
    push_svg_segment(svg, D, mask, horizontal_points(x + 6.0, y + 112.0), style);
    push_svg_segment(svg, E, mask, vertical_points(x, y + 66.0), style);
    push_svg_segment(svg, F, mask, vertical_points(x, y + 10.0), style);
    push_svg_segment(svg, G, mask, horizontal_points(x + 6.0, y + 56.0), style);
    push_svg_circle(svg, x + 78.0, y + 120.0, 4.0, decimal, style);
}

fn push_svg_segment(
    svg: &mut String,
    segment: u8,
    mask: u8,
    points: [(f32, f32); 6],
    style: LcdStyle,
) {
    let is_on = mask & segment != 0;
    let fill = if is_on { style.on } else { style.off };
    let opacity = if is_on { 1.0 } else { style.inactive_opacity };
    let filter = if is_on && style.glow {
        r#" filter="url(#segment-glow)""#
    } else {
        ""
    };
    let points = points
        .iter()
        .map(|(x, y)| format!("{x:.1},{y:.1}"))
        .collect::<Vec<_>>()
        .join(" ");

    svg.push_str(&format!(
        r##"  <polygon points="{points}" fill="{}" opacity="{opacity:.2}" stroke="#edf4e7" stroke-opacity="0.16" stroke-width="1"{filter}/>
"##,
        fill.to_hex()
    ));
}

fn push_svg_circle(svg: &mut String, cx: f32, cy: f32, r: f32, is_on: bool, style: LcdStyle) {
    let fill = if is_on { style.on } else { style.off };
    let opacity = if is_on { 1.0 } else { style.inactive_opacity };
    let filter = if is_on && style.glow {
        r#" filter="url(#segment-glow)""#
    } else {
        ""
    };
    svg.push_str(&format!(
        r##"  <circle cx="{cx:.1}" cy="{cy:.1}" r="{r:.1}" fill="{}" opacity="{opacity:.2}"{filter}/>
"##,
        fill.to_hex()
    ));
}

pub fn print_masks(text: &str) {
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

pub fn segment_names(mask: u8) -> Vec<&'static str> {
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

pub fn segment_mask(ch: char) -> Option<u8> {
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
        let output = render_text("12:34", TerminalStyle::default());
        assert_eq!(output.lines().count(), 5);
    }

    #[test]
    fn accepts_hex_colors_with_or_without_prefix() {
        assert_eq!(
            HexColor::parse("#102418", "--on"),
            Ok(HexColor::new(0x102418))
        );
        assert_eq!(
            HexColor::parse("102418", "--on"),
            Ok(HexColor::new(0x102418))
        );
        assert!(HexColor::parse("bad", "--on").is_err());
    }

    #[test]
    fn rejects_invalid_inactive_opacity() {
        assert_eq!(parse_opacity("0.25"), Ok(0.25));
        assert!(parse_opacity("-0.1").is_err());
        assert!(parse_opacity("1.1").is_err());
    }

    #[test]
    fn parses_custom_segment_masks() {
        assert_eq!(parse_segment_mask("ABDEG"), Ok(A | B | D | E | G));
        assert_eq!(parse_segment_mask("a,b,d,e,g"), Ok(A | B | D | E | G));
        assert_eq!(parse_segment_mask("0b1011011"), Ok(A | B | D | E | G));
        assert_eq!(parse_segment_mask("0x5b"), Ok(A | B | D | E | G));
    }

    #[test]
    fn rejects_invalid_custom_segment_masks() {
        assert!(parse_segment_mask("").is_err());
        assert!(parse_segment_mask("Q").is_err());
        assert!(parse_segment_mask("0x80").is_err());
    }

    #[test]
    fn renders_custom_mask_cells() {
        let cells = vec![Cell {
            kind: CellKind::Segments(A | B | D | E | G),
            decimal: false,
        }];

        assert_eq!(
            render_cells_text(&cells, TerminalStyle::default()),
            render_text("2", TerminalStyle::default())
        );
        assert!(render_cells_svg(&cells, LcdStyle::default()).contains("<polygon"));
    }
}
