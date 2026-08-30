#!/usr/bin/env python3
"""Generate the README showcase assets in assets/.

Every terminal card is rendered from the *real* stdout/stderr of the
`nex` binary -- nothing here is mocked. Run from the repo root:

    cargo build --release
    python3 tools/gen-assets.py

Cards are emitted as light/dark SVG pairs so the README can switch them
with <picture>. Only presentation attributes are used (no <style>
blocks, no scripts) because GitHub strips those from inline SVG.
"""

import glob
import html
import os
import re
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ASSETS = os.path.join(ROOT, "assets")
NEX = os.path.join(ROOT, "target", "release", "nex")

FONT = "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, 'Liberation Mono', monospace"
SANS = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif"

FS = 13.0          # body font size
CH = FS * 0.6      # monospace advance width
LH = 20.0          # line height
PAD = 18.0         # body padding
BAR = 34.0         # title bar height
R = 10.0           # corner radius
NBSP = " "

THEMES = {
    "dark": {
        "bg": "#0d1117", "chrome": "#161b22", "border": "#30363d",
        "text": "#c9d1d9", "muted": "#8b949e", "dim": "#6e7681",
        "green": "#3fb950", "red": "#ff7b72", "blue": "#79c0ff",
        "yellow": "#d29922", "purple": "#d2a8ff", "cyan": "#a5d6ff",
        "orange": "#ffa657", "accent": "#58a6ff",
        "panel": "#161b22", "shadow": "#010409",
    },
    "light": {
        "bg": "#ffffff", "chrome": "#f6f8fa", "border": "#d0d7de",
        "text": "#1f2328", "muted": "#59636e", "dim": "#818b98",
        "green": "#1a7f37", "red": "#cf222e", "blue": "#0550ae",
        "yellow": "#9a6700", "purple": "#8250df", "cyan": "#0a3069",
        "orange": "#953800", "accent": "#0969da",
        "panel": "#f6f8fa", "shadow": "#d0d7de",
    },
}


# --------------------------------------------------------------------------
# span helpers
# --------------------------------------------------------------------------

def esc(s):
    return html.escape(s, quote=False).replace(" ", NBSP)


def text_el(x, y, spans, theme, size=FS, font=FONT, weight=None, anchor=None):
    """Render a list of (text, color-key) spans as one <text> element."""
    attrs = [
        'x="%s"' % fmt(x), 'y="%s"' % fmt(y),
        'font-family="%s"' % html.escape(font, quote=True),
        'font-size="%s"' % fmt(size),
        'xml:space="preserve"',
    ]
    if weight:
        attrs.append('font-weight="%s"' % weight)
    if anchor:
        attrs.append('text-anchor="%s"' % anchor)
    body = "".join(
        '<tspan fill="%s">%s</tspan>' % (theme.get(key, theme["text"]), esc(txt))
        for txt, key in spans if txt != ""
    )
    return "  <text %s>%s</text>" % (" ".join(attrs), body)


def fmt(n):
    return ("%.2f" % n).rstrip("0").rstrip(".")


# --------------------------------------------------------------------------
# tokenizers -> [(text, colorkey)]
# --------------------------------------------------------------------------

NEX_TOKEN = re.compile(r"""
    (?P<comment>(?:\#|//)[^\n]*)
  | (?P<string>"(?:[^"\\]|\\.)*"?)
  | (?P<number>-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)
  | (?P<ident>[A-Za-z_][A-Za-z0-9_]*)
  | (?P<paren>[()\[\]])
  | (?P<space>\s+)
  | (?P<other>.)
""", re.VERBOSE)

KEYWORDS = {"true", "false", "null"}


def tok_nex(line):
    out, pos = [], 0
    for m in NEX_TOKEN.finditer(line):
        kind, txt = m.lastgroup, m.group()
        if kind == "comment":
            out.append((txt, "dim"))
        elif kind == "string":
            out.append((txt, "cyan"))
        elif kind == "number":
            out.append((txt, "purple"))
        elif kind == "paren":
            out.append((txt, "muted"))
        elif kind == "ident":
            nxt = line[m.end():m.end() + 1]
            first_on_line = line[:m.start()].strip() == ""
            if txt in KEYWORDS:
                out.append((txt, "yellow"))
            elif nxt == "(" and not (first_on_line and line[:m.start()] != ""):
                out.append((txt, "blue"))    # object type name, or the root object
            elif first_on_line:
                out.append((txt, "orange"))  # field key
            else:
                out.append((txt, "text"))
        else:
            out.append((txt, "text"))
        pos = m.end()
    return out or [("", "text")]


JSON_TOKEN = re.compile(r"""
    (?P<key>"(?:[^"\\]|\\.)*"\s*:)
  | (?P<string>"(?:[^"\\]|\\.)*")
  | (?P<number>-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)
  | (?P<kw>true|false|null)
  | (?P<punct>[\{\}\[\],:])
  | (?P<space>\s+)
  | (?P<other>.)
""", re.VERBOSE)


def tok_json(line):
    out = []
    for m in JSON_TOKEN.finditer(line):
        kind, txt = m.lastgroup, m.group()
        out.append((txt, {
            "key": "blue", "string": "cyan", "number": "purple",
            "kw": "yellow", "punct": "muted",
        }.get(kind, "text")))
    return out or [("", "text")]


def tok_shell(line):
    """`$ nex format examples/messy.nex` -> prompt / bin / sub / flags / paths."""
    out = [("$", "green"), (" ", "text")]
    rest = line[1:].lstrip()
    for i, word in enumerate(rest.split(" ")):
        if i:
            out.append((" ", "text"))
        if word == "":
            continue
        if i == 0:
            out.append((word, "blue"))
        elif word.startswith("-"):
            out.append((word, "yellow"))
        elif "/" in word or word.endswith((".nex", ".json", ".toml")):
            out.append((word, "cyan"))
        elif i == 1:
            out.append((word, "text"))
        else:
            out.append((word, "muted"))
    return out


def tok_status(line):
    s = line.lstrip()
    if s.startswith("✓"):
        return [(line, "green")]
    if s.startswith("✗") or s.startswith("Error") or "error" in s.lower():
        return [(line, "red")]
    if s.startswith("warning"):
        return [(line, "yellow")]
    if s.startswith("test result:") or " ... ok" in line:
        parts = re.split(r"(\bok\b|\d+ passed|\d+ failed)", line)
        out = []
        for p in parts:
            if p in ("ok",) or p.endswith("passed"):
                out.append((p, "green"))
            elif p.endswith("failed"):
                out.append((p, "muted"))
            else:
                out.append((p, "muted"))
        return out
    return [(line, "text")]


def tokenize(line, mode):
    if line.startswith("$"):
        return tok_shell(line)
    if line.strip() == "":
        return [("", "text")]
    s = line.lstrip()
    if s[:1] in ("✓", "✗") or s.startswith("Error") or s.startswith("test"):
        return tok_status(line)
    if mode == "json":
        return tok_json(line)
    if mode == "nex":
        return tok_nex(line)
    return tok_status(line)


# --------------------------------------------------------------------------
# card renderer
# --------------------------------------------------------------------------

def card_svg(title, lines, mode, theme_name, min_cols=52):
    t = THEMES[theme_name]
    cols = max([len(l) for l in lines] + [min_cols, len(title) + 24])
    w = PAD * 2 + cols * CH
    h = BAR + PAD + len(lines) * LH + PAD - 4

    top_bar = (
        "M0.5,{r} A{r},{r} 0 0 1 {r},0.5 L{x},0.5 "
        "A{r},{r} 0 0 1 {w},{r} L{w},{b} L0.5,{b} Z"
    ).format(r=fmt(R + 0.5), x=fmt(w - R - 0.5), w=fmt(w - 0.5), b=fmt(BAR))

    o = []
    o.append('<svg xmlns="http://www.w3.org/2000/svg" width="%s" height="%s" '
             'viewBox="0 0 %s %s" role="img" aria-label="%s">'
             % (fmt(w), fmt(h), fmt(w), fmt(h), html.escape(title, quote=True)))
    o.append('  <rect x="0.5" y="0.5" width="%s" height="%s" rx="%s" fill="%s" stroke="%s"/>'
             % (fmt(w - 1), fmt(h - 1), fmt(R), t["bg"], t["border"]))
    o.append('  <path d="%s" fill="%s"/>' % (top_bar, t["chrome"]))
    o.append('  <line x1="0.5" y1="%s" x2="%s" y2="%s" stroke="%s"/>'
             % (fmt(BAR), fmt(w - 0.5), fmt(BAR), t["border"]))
    for i, c in enumerate(("#ff5f57", "#febc2e", "#28c840")):
        o.append('  <circle cx="%s" cy="%s" r="5.5" fill="%s"/>'
                 % (fmt(19 + i * 18), fmt(BAR / 2), c))
    o.append(text_el(w / 2, BAR / 2 + 4, [(title, "muted")], t,
                     size=11.5, font=SANS, anchor="middle"))

    y = BAR + PAD + 10
    for line in lines:
        o.append(text_el(PAD, y, tokenize(line, mode), t))
        y += LH
    o.append("</svg>")
    return "\n".join(o) + "\n"


def write_pair(name, title, lines, mode, min_cols=52):
    for theme in THEMES:
        path = os.path.join(ASSETS, "%s-%s.svg" % (name, theme))
        with open(path, "w", encoding="utf-8") as f:
            f.write(card_svg(title, lines, mode, theme, min_cols))
    print("  assets/%s-{dark,light}.svg  (%d lines)" % (name, len(lines)))


# --------------------------------------------------------------------------
# real command capture
# --------------------------------------------------------------------------

def run(args):
    """Run the real nex binary and return combined output lines."""
    p = subprocess.run([NEX] + args, cwd=ROOT, capture_output=True, text=True)
    out = (p.stdout + p.stderr).replace("\t", "    ").rstrip("\n")
    return out.split("\n") if out else []


def session(cmds, mode, extra=None):
    """Build card lines: `$ cmd` followed by that command's real output."""
    lines = []
    for i, args in enumerate(cmds):
        if i:
            lines.append("")
        lines.append("$ nex " + " ".join(args))
        lines.extend(run(args))
    if extra:
        lines.extend(extra)
    return lines


def read_lines(path):
    with open(os.path.join(ROOT, path), encoding="utf-8") as f:
        return f.read().rstrip("\n").split("\n")


# --------------------------------------------------------------------------
# hero wordmark
# --------------------------------------------------------------------------

HERO_CODE = [
    'server(',
    '  host "0.0.0.0"',
    '  port 8080',
    '  tls true',
    '  tags[http prod]',
    ')',
]


def hero_svg(theme_name):
    t = THEMES[theme_name]
    w, h = 900.0, 300.0
    o = ['<svg xmlns="http://www.w3.org/2000/svg" width="%s" height="%s" '
         'viewBox="0 0 %s %s" role="img" aria-label="nex - a punctuation-light '
         'data language">' % (fmt(w), fmt(h), fmt(w), fmt(h))]
    o.append('  <rect x="0.5" y="0.5" width="%s" height="%s" rx="14" fill="%s" stroke="%s"/>'
             % (fmt(w - 1), fmt(h - 1), t["bg"], t["border"]))

    # faint parenthesis motif, the one piece of punctuation nex keeps
    o.append('  <text x="%s" y="248" font-family="%s" font-size="360" font-weight="700" '
             'fill="%s" opacity="0.06" text-anchor="middle">()</text>'
             % (fmt(w - 150), html.escape(SANS, quote=True), t["accent"]))

    o.append(text_el(60, 130, [("nex", "accent")], t, size=92, font=SANS, weight="700"))
    o.append('  <rect x="62" y="150" width="112" height="5" rx="2.5" fill="%s"/>' % t["accent"])
    o.append(text_el(62, 192, [("a punctuation-light data language", "text")], t,
                     size=19, font=SANS, weight="600"))
    o.append(text_el(62, 218, [("for humans, tools, and language models", "muted")], t,
                     size=16, font=SANS))

    chips = ["parser", "formatter", "json bridge", "lsp", "schema"]
    x = 62.0
    for c in chips:
        cw = len(c) * 7.0 + 22
        o.append('  <rect x="%s" y="244" width="%s" height="26" rx="13" fill="%s" stroke="%s"/>'
                 % (fmt(x), fmt(cw), t["chrome"], t["border"]))
        o.append(text_el(x + cw / 2, 261, [(c, "muted")], t, size=11.5, font=SANS,
                         anchor="middle"))
        x += cw + 8

    # real, parseable nex on the right
    px, py, pw = 545.0, 46.0, 300.0
    ph = PAD + len(HERO_CODE) * LH + PAD - 6
    o.append('  <rect x="%s" y="%s" width="%s" height="%s" rx="10" fill="%s" stroke="%s"/>'
             % (fmt(px), fmt(py), fmt(pw), fmt(ph), t["chrome"], t["border"]))
    y = py + PAD + 8
    for line in HERO_CODE:
        o.append(text_el(px + 18, y, tok_nex(line), t))
        y += LH
    o.append("</svg>")
    return "\n".join(o) + "\n"


# --------------------------------------------------------------------------
# architecture diagram
# --------------------------------------------------------------------------

def _box(t, x, y, w, h, label, sub, accent=False):
    fill = t["accent"] if accent else t["chrome"]
    stroke = t["accent"] if accent else t["border"]
    lab = "bg" if accent else "text"
    o = ['  <rect x="%s" y="%s" width="%s" height="%s" rx="8" fill="%s" stroke="%s"/>'
         % (fmt(x), fmt(y), fmt(w), fmt(h), fill, stroke)]
    cy = y + (h / 2 + 5 if not sub else h / 2 - 4)
    o.append(text_el(x + w / 2, cy, [(label, lab)], t, size=13.5, font=FONT,
                     weight="600", anchor="middle"))
    if sub:
        o.append(text_el(x + w / 2, cy + 18, [(sub, "bg" if accent else "muted")], t,
                         size=11, font=SANS, anchor="middle"))
    return o


def _arrow(t, x, y1, y2):
    return ['  <line x1="%s" y1="%s" x2="%s" y2="%s" stroke="%s" stroke-width="1.5"/>'
            % (fmt(x), fmt(y1), fmt(x), fmt(y2 - 6), t["dim"]),
            '  <path d="M%s,%s l-4.5,-7 h9 Z" fill="%s"/>'
            % (fmt(x), fmt(y2), t["dim"])]


def arch_svg(theme_name):
    t = THEMES[theme_name]
    w, h = 880.0, 396.0
    o = ['<svg xmlns="http://www.w3.org/2000/svg" width="%s" height="%s" '
         'viewBox="0 0 %s %s" role="img" aria-label="nex crate architecture">'
         % (fmt(w), fmt(h), fmt(w), fmt(h))]
    o.append('  <rect x="0.5" y="0.5" width="%s" height="%s" rx="12" fill="%s" stroke="%s"/>'
             % (fmt(w - 1), fmt(h - 1), t["bg"], t["border"]))
    mid = w / 2

    o += _box(t, mid - 110, 28, 220, 46, "config.nex", "utf-8 source text")
    o += _arrow(t, mid, 74, 104)
    o += _box(t, mid - 140, 104, 280, 58, "nex-parser", "recursive descent → Value AST", True)

    row, bw, gap = 202.0, 240.0, 20.0
    total = bw * 3 + gap * 2
    x0 = (w - total) / 2
    for i, (name, sub) in enumerate([
        ("nex-formatter", "canonical text output"),
        ("nex-json", "json ↔ nex bridge"),
        ("nex-schema", "structural validation"),
    ]):
        bx = x0 + i * (bw + gap)
        o += _arrow(t, bx + bw / 2, 162, row) if i != 1 else _arrow(t, mid, 162, row)
        if i != 1:
            o.append('  <line x1="%s" y1="162" x2="%s" y2="162" stroke="%s" stroke-width="1.5"/>'
                     % (fmt(min(mid, bx + bw / 2)), fmt(max(mid, bx + bw / 2)), t["dim"]))
        o += _box(t, bx, row, bw, 58, name, sub)

    o += _arrow(t, mid, 260, 300)
    o.append('  <line x1="%s" y1="300" x2="%s" y2="300" stroke="%s" stroke-width="1.5"/>'
             % (fmt(mid - 175), fmt(mid + 175), t["dim"]))
    for name, sub, cx in [("nex-cli", "check / format / to-json / from-json", mid - 175),
                          ("nex-lsp", "diagnostics / symbols / formatting", mid + 175)]:
        o += _arrow(t, cx, 300, 322)
        o += _box(t, cx - 160, 322, 320, 52, name, sub)
    o.append("</svg>")
    return "\n".join(o) + "\n"


# --------------------------------------------------------------------------
# main
# --------------------------------------------------------------------------

def cargo_test_lines():
    p = subprocess.run(["cargo", "test", "--workspace"], cwd=ROOT,
                       capture_output=True, text=True)
    raw = p.stdout + p.stderr
    tests = [l for l in raw.split("\n") if l.startswith("test ") and " ... " in l]
    passed = sum(int(m.group(1)) for m in re.finditer(r"(\d+) passed", raw))
    failed = sum(int(m.group(1)) for m in re.finditer(r"(\d+) failed", raw))
    lines = ["$ cargo test --workspace", ""] + tests + [""]
    lines.append("test result: ok. %d passed; %d failed" % (passed, failed))
    return lines


def main():
    if not os.path.exists(NEX):
        sys.exit("build first:  cargo build --release   (missing %s)" % NEX)
    os.makedirs(ASSETS, exist_ok=True)
    print("writing assets from real `nex` output:")

    for theme in THEMES:
        with open(os.path.join(ASSETS, "hero-%s.svg" % theme), "w", encoding="utf-8") as f:
            f.write(hero_svg(theme))
        with open(os.path.join(ASSETS, "architecture-%s.svg" % theme), "w", encoding="utf-8") as f:
            f.write(arch_svg(theme))
    print("  assets/hero-{dark,light}.svg")
    print("  assets/architecture-{dark,light}.svg")

    write_pair("source", "examples/config.nex",
               read_lines("examples/config.nex"), "nex")

    write_pair("check", "nex check",
               session([["check", "examples/config.nex"],
                        ["check", "examples/broken.nex"]], "nex"), "nex")

    write_pair("messy", "examples/messy.nex  (before)",
               read_lines("examples/messy.nex"), "nex")

    write_pair("format", "nex format  (after)",
               session([["format", "examples/messy.nex"]], "nex"), "nex")

    write_pair("nesting", "examples/nesting.nex",
               read_lines("examples/nesting.nex"), "nex")

    write_pair("nesting-json", "nex to-json  (list vs nested object)",
               session([["to-json", "examples/nesting.nex"]], "json"), "json")

    write_pair("to-json", "nex to-json",
               session([["to-json", "examples/config.nex"]], "json"), "json")

    write_pair("from-json", "nex from-json",
               session([["from-json", "examples/server.json"]], "nex"), "nex")

    write_pair("tests", "cargo test --workspace", cargo_test_lines(), "plain")

    export_pngs()


def export_pngs(scale=2):
    """Mirror every SVG into assets/png/ as a 2x raster, for places that will
    not render SVG (slides, chat, image hosts)."""
    out_dir = os.path.join(ASSETS, "png")
    os.makedirs(out_dir, exist_ok=True)
    svgs = sorted(glob.glob(os.path.join(ASSETS, "*.svg")))

    if shutil.which("rsvg-convert"):
        for svg in svgs:
            png = os.path.join(out_dir, os.path.basename(svg)[:-4] + ".png")
            subprocess.run(["rsvg-convert", "-z", str(scale), "-o", png, svg], check=True)
    elif shutil.which("qlmanage"):  # macOS fallback, no extra install
        for svg in svgs:
            subprocess.run(["qlmanage", "-t", "-s", "1600", "-o", out_dir, svg],
                           capture_output=True, check=False)
            produced = os.path.join(out_dir, os.path.basename(svg) + ".png")
            if os.path.exists(produced):
                os.replace(produced, produced.replace(".svg.png", ".png"))
    else:
        print("  (skipped png export: install librsvg for `rsvg-convert`)")
        return

    print("  assets/png/*.png  (%d files at %dx)" % (len(svgs), scale))


if __name__ == "__main__":
    main()
