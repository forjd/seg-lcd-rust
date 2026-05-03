use std::{env, fs, path::PathBuf};

use seg_lcd_rust::{
    Cell, CellKind, HexColor, LcdStyle, TerminalStyle, Theme, parse_opacity, parse_segment_mask,
    print_masks, render_cells_svg, render_cells_text, render_svg, render_text, segment_names,
};

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

    println!("{}", config.render_text());

    if let Some(path) = &config.svg_path {
        if let Err(error) = fs::write(path, config.render_svg()) {
            eprintln!("failed to write SVG to {}: {error}", path.display());
            std::process::exit(1);
        }
        println!();
        println!("wrote {}", path.display());
    }

    if config.dump_masks {
        println!();
        config.print_masks();
    }
}

#[derive(Debug, Clone)]
struct Config {
    display: DisplayInput,
    terminal_style: TerminalStyle,
    dump_masks: bool,
    svg_path: Option<PathBuf>,
    lcd_style: LcdStyle,
    help: bool,
}

#[derive(Debug, Clone)]
enum DisplayInput {
    Text(String),
    Cells(Vec<Cell>),
}

impl Config {
    fn from_args(args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut terminal_style = TerminalStyle::default();
        let mut dump_masks = false;
        let mut svg_path = None;
        let mut lcd_style = LcdStyle::default();
        let mut help = false;
        let mut text_parts = Vec::new();
        let mut mask_cells = Vec::new();
        let mut args = args.peekable();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => help = true,
                "--labels" => terminal_style.show_labels = true,
                "--masks" => dump_masks = true,
                "--svg" => {
                    let path = args
                        .next()
                        .ok_or_else(|| "--svg requires an output path".to_string())?;
                    svg_path = Some(PathBuf::from(path));
                }
                "--mask" => {
                    let mask = parse_segment_mask(&next_option_value(&mut args, "--mask")?)?;
                    mask_cells.push(Cell {
                        kind: CellKind::Segments(mask),
                        decimal: false,
                    });
                }
                "--theme" => {
                    let theme = args
                        .next()
                        .ok_or_else(|| "--theme requires a theme name".to_string())?
                        .parse::<Theme>()?;
                    lcd_style = theme.style();
                }
                "--on" => {
                    lcd_style.on = HexColor::parse(&next_option_value(&mut args, "--on")?, "--on")?;
                }
                "--off" => {
                    lcd_style.off =
                        HexColor::parse(&next_option_value(&mut args, "--off")?, "--off")?;
                }
                "--bg" => {
                    lcd_style.background =
                        HexColor::parse(&next_option_value(&mut args, "--bg")?, "--bg")?;
                }
                "--panel" => {
                    lcd_style.panel =
                        HexColor::parse(&next_option_value(&mut args, "--panel")?, "--panel")?;
                }
                "--inactive-opacity" => {
                    lcd_style.inactive_opacity =
                        parse_opacity(&next_option_value(&mut args, "--inactive-opacity")?)?;
                }
                "--glow" => lcd_style.glow = true,
                "--no-glow" => lcd_style.glow = false,
                "--no-glass" => lcd_style.glass = false,
                "--inverse" => {
                    terminal_style.on = ' ';
                    terminal_style.off = '#';
                }
                _ if arg.starts_with('-') => return Err(format!("unknown option: {arg}")),
                _ => text_parts.push(arg),
            }
        }

        let display = if mask_cells.is_empty() {
            DisplayInput::Text(if text_parts.is_empty() {
                "12:34.5".to_string()
            } else {
                text_parts.join(" ")
            })
        } else if text_parts.is_empty() {
            DisplayInput::Cells(mask_cells)
        } else {
            return Err("--mask cannot be combined with display text".to_string());
        };

        Ok(Self {
            display,
            terminal_style,
            dump_masks,
            svg_path,
            lcd_style,
            help,
        })
    }

    fn render_text(&self) -> String {
        match &self.display {
            DisplayInput::Text(text) => render_text(text, self.terminal_style),
            DisplayInput::Cells(cells) => render_cells_text(cells, self.terminal_style),
        }
    }

    fn render_svg(&self) -> String {
        match &self.display {
            DisplayInput::Text(text) => render_svg(text, self.lcd_style),
            DisplayInput::Cells(cells) => render_cells_svg(cells, self.lcd_style),
        }
    }

    fn print_masks(&self) {
        match &self.display {
            DisplayInput::Text(text) => print_masks(text),
            DisplayInput::Cells(cells) => {
                for cell in cells {
                    match cell.kind {
                        CellKind::Segments(mask) => {
                            println!("{mask:07b}  {}", segment_names(mask).join(","));
                        }
                        CellKind::Colon => println!(":  colon"),
                        CellKind::Blank => println!("0000000"),
                    }
                }
            }
        }
    }
}

fn next_option_value(
    args: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    option: &str,
) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("{option} requires a value"))
}

fn print_usage() {
    println!(
        "seg-lcd-rust - seven-segment LCD simulator\n\n\
         Usage:\n  cargo run -- [OPTIONS] [TEXT]\n\n\
         Options:\n  --inverse   render dark inactive LCD blocks with clear active segments\n  \
         --labels    render segment letters instead of filled segments\n  \
         --masks     print each character's seven-bit segment mask\n  \
         --mask MASK  render one custom digit from segments A-G, 0b bits, or 0x hex\n  \
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
         cargo run -- --mask ABDEG --mask BCG\n  \
         cargo run -- --svg display.svg --theme amber 10:58.42\n  \
         cargo run -- --svg display.svg --theme blue --glow 88:88.88"
    );
}
