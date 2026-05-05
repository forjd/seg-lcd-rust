use std::{env, net::ToSocketAddrs};

use seg_lcd_rust::{
    Cell, CellKind, HexColor, LcdStyle, Theme, parse_opacity, parse_segment_mask, render_cells_svg,
    render_svg,
};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};
use url::form_urlencoded;

const DEFAULT_ADDR: &str = "127.0.0.1:7878";
const DEFAULT_TEXT: &str = "12:34.5";
const MAX_QUERY_BYTES: usize = 4096;
const MAX_TEXT_CHARS: usize = 256;
const MAX_MASKS: usize = 64;

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

    if let Err(error) = run(config) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[derive(Debug, Clone)]
struct Config {
    addr: String,
    help: bool,
}

impl Config {
    fn from_args(args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut addr = DEFAULT_ADDR.to_string();
        let mut help = false;
        let mut args = args.peekable();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => help = true,
                "--addr" => {
                    addr = args
                        .next()
                        .ok_or_else(|| "--addr requires a socket address".to_string())?;
                    validate_addr(&addr)?;
                }
                _ => return Err(format!("unknown option: {arg}")),
            }
        }

        Ok(Self { addr, help })
    }
}

fn validate_addr(addr: &str) -> Result<(), String> {
    addr.to_socket_addrs()
        .map_err(|error| format!("invalid --addr {addr}: {error}"))?
        .next()
        .ok_or_else(|| format!("invalid --addr {addr}: resolved no socket addresses"))?;
    Ok(())
}

fn run(config: Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server = Server::http(&config.addr)?;
    eprintln!("listening on http://{}", config.addr);

    for request in server.incoming_requests() {
        respond(request);
    }

    Ok(())
}

fn respond(request: Request) {
    let api_response = handle_request(request.method(), request.url());
    let mut response = Response::from_string(api_response.body)
        .with_status_code(StatusCode(api_response.status))
        .with_header(content_type(api_response.content_type));

    if let Some(cache_control) = header("Cache-Control", "public, max-age=60") {
        response.add_header(cache_control);
    }

    if let Err(error) = request.respond(response) {
        eprintln!("failed to respond: {error}");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ApiResponse {
    status: u16,
    content_type: &'static str,
    body: String,
}

fn handle_request(method: &Method, url: &str) -> ApiResponse {
    if method != &Method::Get {
        return text_response(405, "method not allowed\n");
    }

    let (path, query) = split_url(url);
    match path {
        "/" | "/svg" => match render_svg_request(query) {
            Ok(svg) => ApiResponse {
                status: 200,
                content_type: "image/svg+xml; charset=utf-8",
                body: svg,
            },
            Err(error) => text_response(error.status(), &format!("{}\n", error.message())),
        },
        "/healthz" => text_response(200, "ok\n"),
        _ => text_response(404, "not found\n"),
    }
}

fn split_url(url: &str) -> (&str, &str) {
    url.split_once('?').unwrap_or((url, ""))
}

fn render_svg_request(query: &str) -> Result<String, ApiError> {
    if query.len() > MAX_QUERY_BYTES {
        return Err(ApiError::PayloadTooLarge(format!(
            "query string is too large; limit is {MAX_QUERY_BYTES} bytes"
        )));
    }

    let mut text = DEFAULT_TEXT.to_string();
    let mut style = LcdStyle::default();
    let mut masks = Vec::new();

    for (key, value) in form_urlencoded::parse(query.as_bytes()) {
        match key.as_ref() {
            "text" => {
                if value.chars().count() > MAX_TEXT_CHARS {
                    return Err(ApiError::PayloadTooLarge(format!(
                        "text is too long; limit is {MAX_TEXT_CHARS} characters"
                    )));
                }
                text = value.into_owned();
            }
            "theme" => style = value.parse::<Theme>()?.style(),
            "mask" => {
                if masks.len() >= MAX_MASKS {
                    return Err(ApiError::PayloadTooLarge(format!(
                        "too many mask parameters; limit is {MAX_MASKS}"
                    )));
                }
                masks.push(Cell {
                    kind: CellKind::Segments(parse_segment_mask(&value)?),
                    decimal: false,
                });
            }
            "on" => style.on = HexColor::parse(&value, "on")?,
            "off" => style.off = HexColor::parse(&value, "off")?,
            "bg" => style.background = HexColor::parse(&value, "bg")?,
            "panel" => style.panel = HexColor::parse(&value, "panel")?,
            "inactive-opacity" => style.inactive_opacity = parse_opacity(&value)?,
            "glow" => style.glow = parse_bool(&value, "glow")?,
            "glass" => style.glass = parse_bool(&value, "glass")?,
            _ => return Err(format!("unknown query parameter: {key}").into()),
        }
    }

    if masks.is_empty() {
        Ok(render_svg(&text, style))
    } else {
        Ok(render_cells_svg(&masks, style))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ApiError {
    BadRequest(String),
    PayloadTooLarge(String),
}

impl ApiError {
    fn status(&self) -> u16 {
        match self {
            Self::BadRequest(_) => 400,
            Self::PayloadTooLarge(_) => 413,
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::BadRequest(message) | Self::PayloadTooLarge(message) => message,
        }
    }
}

impl From<String> for ApiError {
    fn from(message: String) -> Self {
        Self::BadRequest(message)
    }
}

fn parse_bool(value: &str, name: &str) -> Result<bool, String> {
    match value {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(format!("{name} requires true or false")),
    }
}

fn text_response(status: u16, body: &str) -> ApiResponse {
    ApiResponse {
        status,
        content_type: "text/plain; charset=utf-8",
        body: body.to_string(),
    }
}

fn content_type(value: &str) -> Header {
    header("Content-Type", value).expect("static header names and values are valid")
}

fn header(name: &str, value: &str) -> Option<Header> {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).ok()
}

fn print_usage() {
    println!(
        "seg-lcd-rust-api - tiny HTTP SVG renderer\n\n\
         Usage:\n  cargo run --bin seg-lcd-rust-api -- [OPTIONS]\n\n\
         Options:\n  --addr ADDR  socket address to bind, defaults to 127.0.0.1:7878\n  \
         -h, --help   show this help\n\n\
         Endpoints:\n  GET /svg?text=10:58.42&theme=amber\n  \
         GET /svg?mask=ABDEG&mask=BCG&theme=blue&glow=true\n  \
         GET /healthz\n\n\
         Query parameters:\n  text, theme, mask, on, off, bg, panel, inactive-opacity, glow, glass"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_svg_for_text_query() {
        let response = handle_request(&Method::Get, "/svg?text=10%3A58.42&theme=amber");

        assert_eq!(response.status, 200);
        assert_eq!(response.content_type, "image/svg+xml; charset=utf-8");
        assert!(response.body.starts_with("<svg"));
        assert!(response.body.contains("<circle"));
    }

    #[test]
    fn renders_repeated_mask_queries() {
        let response = handle_request(&Method::Get, "/svg?mask=ABDEG&mask=BCG");

        assert_eq!(response.status, 200);
        assert!(response.body.contains("<polygon"));
    }

    #[test]
    fn rejects_unknown_parameters() {
        let response = handle_request(&Method::Get, "/svg?size=large");

        assert_eq!(response.status, 400);
        assert!(response.body.contains("unknown query parameter"));
    }

    #[test]
    fn rejects_non_get_requests() {
        let response = handle_request(&Method::Post, "/svg?text=123");

        assert_eq!(response.status, 405);
    }

    #[test]
    fn rejects_oversized_query_strings() {
        let url = format!("/svg?text={}", "8".repeat(4097));

        let response = handle_request(&Method::Get, &url);

        assert_eq!(response.status, 413);
        assert!(response.body.contains("query string is too large"));
    }

    #[test]
    fn rejects_oversized_decoded_text() {
        let url = format!("/svg?text={}", "8".repeat(257));

        let response = handle_request(&Method::Get, &url);

        assert_eq!(response.status, 413);
        assert!(response.body.contains("text is too long"));
    }

    #[test]
    fn rejects_too_many_masks() {
        let mut url = String::from("/svg");
        for index in 0..65 {
            if index == 0 {
                url.push('?');
            } else {
                url.push('&');
            }
            url.push_str("mask=A");
        }

        let response = handle_request(&Method::Get, &url);

        assert_eq!(response.status, 413);
        assert!(response.body.contains("too many mask parameters"));
    }
}
