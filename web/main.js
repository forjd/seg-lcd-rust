import init, {
  default_display_text,
  render_mask_svg_with_style,
  render_svg_for_theme,
  render_svg_with_style,
  segment_mask_binary,
  segment_mask_hex,
  segment_mask_letters,
  theme_names,
} from "./pkg/seg_lcd_rust.js";

const themes = {
  classic: {
    on: "#1f3328",
    off: "#7f9278",
    background: "#d8e1cf",
    panel: "#c7d2be",
    inactiveOpacity: "0.24",
    glow: false,
    glass: true,
  },
  green: {
    on: "#15351f",
    off: "#6f846b",
    background: "#dfe8d6",
    panel: "#c8d4bf",
    inactiveOpacity: "0.22",
    glow: false,
    glass: true,
  },
  amber: {
    on: "#3b2408",
    off: "#9a762e",
    background: "#e8cf8c",
    panel: "#dbb85f",
    inactiveOpacity: "0.28",
    glow: false,
    glass: true,
  },
  blue: {
    on: "#c9f6ff",
    off: "#426977",
    background: "#10252d",
    panel: "#16333d",
    inactiveOpacity: "0.22",
    glow: true,
    glass: true,
  },
  negative: {
    on: "#dff2dc",
    off: "#344537",
    background: "#111a14",
    panel: "#1c2a20",
    inactiveOpacity: "0.34",
    glow: true,
    glass: false,
  },
};

const preview = document.querySelector("#preview");
const text = document.querySelector("#text");
const customMode = document.querySelector("#custom-mode");
const segments = document.querySelector("#segments");
const maskLetters = document.querySelector("#mask-letters");
const maskBinary = document.querySelector("#mask-binary");
const maskHex = document.querySelector("#mask-hex");
const theme = document.querySelector("#theme");
const on = document.querySelector("#on");
const off = document.querySelector("#off");
const background = document.querySelector("#background");
const panel = document.querySelector("#panel");
const inactiveOpacity = document.querySelector("#inactive-opacity");
const glow = document.querySelector("#glow");
const glass = document.querySelector("#glass");
const download = document.querySelector("#download");
const error = document.querySelector("#error");

let currentSvg = "";
let customMask = 0b1011011;

const segmentDefinitions = [
  ["A", 1 << 0],
  ["B", 1 << 1],
  ["C", 1 << 2],
  ["D", 1 << 3],
  ["E", 1 << 4],
  ["F", 1 << 5],
  ["G", 1 << 6],
];

function titleCase(value) {
  return value.slice(0, 1).toUpperCase() + value.slice(1);
}

function applyTheme(name) {
  const selected = themes[name];
  on.value = selected.on;
  off.value = selected.off;
  background.value = selected.background;
  panel.value = selected.panel;
  inactiveOpacity.value = selected.inactiveOpacity;
  glow.checked = selected.glow;
  glass.checked = selected.glass;
}

function render() {
  try {
    currentSvg = customMode.checked
      ? render_mask_svg_with_style(
          customMask,
          on.value,
          off.value,
          background.value,
          panel.value,
          inactiveOpacity.value,
          glow.checked,
          glass.checked,
        )
      : render_svg_with_style(
          text.value,
          on.value,
          off.value,
          background.value,
          panel.value,
          inactiveOpacity.value,
          glow.checked,
          glass.checked,
        );
    preview.innerHTML = currentSvg;
    error.textContent = "";
    renderMask();
  } catch (err) {
    error.textContent = String(err);
  }
}

function renderMask() {
  for (const button of segments.querySelectorAll("button")) {
    const bit = Number(button.dataset.bit);
    button.classList.toggle("active", (customMask & bit) !== 0);
    button.setAttribute("aria-pressed", String((customMask & bit) !== 0));
  }

  maskLetters.textContent = segment_mask_letters(customMask);
  maskBinary.textContent = segment_mask_binary(customMask);
  maskHex.textContent = segment_mask_hex(customMask);
}

function downloadSvg() {
  const blob = new Blob([currentSvg], { type: "image/svg+xml" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");

  anchor.href = url;
  anchor.download = "seg-lcd-rust.svg";
  anchor.click();
  URL.revokeObjectURL(url);
}

await init();

text.value = default_display_text();
for (const name of JSON.parse(theme_names())) {
  const option = document.createElement("option");
  option.value = name;
  option.textContent = titleCase(name);
  theme.append(option);
}

theme.value = "classic";
applyTheme(theme.value);
render_svg_for_theme(text.value, theme.value);
render();

theme.addEventListener("change", () => {
  applyTheme(theme.value);
  render();
});

for (const control of [
  text,
  customMode,
  on,
  off,
  background,
  panel,
  inactiveOpacity,
  glow,
  glass,
]) {
  control.addEventListener("input", render);
}

download.addEventListener("click", downloadSvg);

for (const [name, bit] of segmentDefinitions) {
  const button = document.createElement("button");
  button.type = "button";
  button.textContent = name;
  button.dataset.bit = String(bit);
  button.setAttribute("aria-pressed", "false");
  button.addEventListener("click", () => {
    customMask ^= bit;
    customMode.checked = true;
    render();
  });
  segments.append(button);
}
renderMask();

for (const button of [maskLetters, maskBinary, maskHex]) {
  button.addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(button.textContent);
    } catch {
      error.textContent = "Clipboard access is unavailable in this browser.";
    }
  });
}
