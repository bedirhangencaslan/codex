// Interface styles the user can hand to the model as skills: a catalogue grouped into families
// (styles that look alike share a group), each with a live CSS preview in the gallery and a
// SKILL.md the extension installs under `<SUFFICE_HOME>/skills/ui-styles/<id>/`. Codex lists an
// installed skill by name and one line; the body below is only read when the model uses it.
// The skill texts are model-facing, so they are English on purpose; the interface texts are in
// the i18n dictionaries (`uiStyle.<id>.*`, `uiGroup.<id>.*`).
import type { MessageKey } from "./i18n";

/** Folder under `<SUFFICE_HOME>/skills` that holds the style skills. */
export const UI_STYLE_ROOT = "ui-styles";

export interface UiStyleGroup {
  id: string;
  titleKey: MessageKey;
  descKey: MessageKey;
}

interface SkillBody {
  /** One sentence: what the style looks like. Also the skill's description. */
  summary: string;
  /** When the style fits (and when it does not). */
  fit: string;
  traits: string[];
  /** Design tokens as CSS custom properties. */
  tokens: Record<string, string>;
  /** A core CSS recipe the model can start from. */
  css: string;
  components: string[];
  accessibility: string[];
  avoid: string[];
}

export interface UiStyle {
  id: string;
  group: string;
  skill: SkillBody;
}

const key = (k: string) => k as MessageKey;

export const UI_STYLE_GROUPS: UiStyleGroup[] = ["depth", "flat", "bold", "light", "editorial"].map((id) => ({
  id,
  titleKey: key(`uiGroup.${id}.title`),
  descKey: key(`uiGroup.${id}.desc`),
}));

export const UI_STYLES: UiStyle[] = [
  // --- depth & material ---------------------------------------------------------------------
  {
    id: "glassmorphism",
    group: "depth",
    skill: {
      summary: "Frosted, translucent panels floating over a vivid background, with a thin light border and soft depth.",
      fit: "Dashboards, landing pages, media and fintech apps where a colourful backdrop can carry the design. Not for dense data tables or text-heavy pages.",
      traits: [
        "Panels are semi-transparent (white at 10-25% opacity) with a background blur of 12-24px.",
        "A vivid backdrop is required: gradients, blurred colour blobs or a photo. Glass over a flat colour looks like grey plastic.",
        "A 1px light border (white at 25-40%) and an optional top-left highlight give the panel an edge.",
        "Soft, wide shadows; rounded corners of 16-24px; generous padding.",
      ],
      tokens: {
        "--glass-bg": "rgba(255, 255, 255, 0.14)",
        "--glass-border": "rgba(255, 255, 255, 0.35)",
        "--glass-blur": "18px",
        "--glass-shadow": "0 8px 32px rgba(15, 23, 42, 0.25)",
        "--glass-radius": "20px",
        "--text-on-glass": "#ffffff",
      },
      css: `.glass {
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur)) saturate(160%);
  -webkit-backdrop-filter: blur(var(--glass-blur)) saturate(160%);
  border: 1px solid var(--glass-border);
  border-radius: var(--glass-radius);
  box-shadow: var(--glass-shadow);
  color: var(--text-on-glass);
}
body {
  background: radial-gradient(circle at 20% 20%, #7c3aed, transparent 45%),
              radial-gradient(circle at 80% 70%, #06b6d4, transparent 45%), #0f172a;
}`,
      components: [
        "Buttons: a slightly more opaque glass (white 20-30%) or a solid accent for the primary action.",
        "Inputs: glass with a stronger border on focus; keep placeholder text above 4.5:1 contrast.",
        "Navigation: a sticky glass bar; modals: a glass sheet over a dimmed, blurred page.",
      ],
      accessibility: [
        "Check text contrast against the brightest part of the backdrop, not the average; add a darker tint (rgba(15,23,42,0.35)) under text when needed.",
        "Respect prefers-reduced-transparency: fall back to an opaque surface.",
        "backdrop-filter is costly: keep blurred layers few and avoid animating them.",
      ],
      avoid: ["Glass on glass on glass (three or more stacked layers).", "Long paragraphs on glass.", "A plain white or grey backdrop."],
    },
  },
  {
    id: "liquid-glass",
    group: "depth",
    skill: {
      summary: "Highly translucent, lens-like capsules with specular highlights that seem to bend the content behind them.",
      fit: "Floating controls, toolbars, tab bars and media overlays in modern app-like interfaces. Use sparingly on top of content; not for whole pages of panels.",
      traits: [
        "Controls are pill or capsule shaped with large radii, floating above content rather than framing it.",
        "Very light blur (6-10px) with strong saturation so the content behind stays readable and colourful.",
        "Specular highlights: an inner top gradient and a bright 1px inner edge suggest a curved glass lens.",
        "Controls react to interaction: slight scale, brighter highlight and shadow on press.",
      ],
      tokens: {
        "--lg-fill": "rgba(255, 255, 255, 0.10)",
        "--lg-edge": "rgba(255, 255, 255, 0.55)",
        "--lg-blur": "8px",
        "--lg-radius": "999px",
        "--lg-shadow": "0 10px 30px rgba(0, 0, 0, 0.25)",
      },
      css: `.liquid {
  background:
    linear-gradient(180deg, rgba(255,255,255,0.35), rgba(255,255,255,0) 45%),
    var(--lg-fill);
  backdrop-filter: blur(var(--lg-blur)) saturate(190%) brightness(1.05);
  border-radius: var(--lg-radius);
  box-shadow: inset 0 1px 0 var(--lg-edge), inset 0 -1px 0 rgba(255,255,255,0.12), var(--lg-shadow);
  transition: transform .2s ease, box-shadow .2s ease;
}
.liquid:active { transform: scale(0.97); }`,
      components: [
        "Tab bars and toolbars as a single floating capsule; group related icons inside one capsule.",
        "Primary actions keep a tinted fill (accent at 70-85%) so they read as buttons, not as glass.",
      ],
      accessibility: [
        "Icons and labels need a fallback shadow or tint so they stay legible over busy content.",
        "Honour prefers-reduced-transparency and prefers-reduced-motion.",
      ],
      avoid: ["Using it for content cards and long text.", "Heavy blur (it turns into plain glassmorphism).", "Dozens of animated lenses on one screen."],
    },
  },
  {
    id: "neumorphism",
    group: "depth",
    skill: {
      summary: "Soft UI: elements extruded from a same-coloured background with a light shadow on one side and a dark one on the other.",
      fit: "Small, calm interfaces: a music player, smart-home controls, a calculator. Not for complex apps or where clear affordances and contrast matter.",
      traits: [
        "Background and elements share one muted colour (#e0e5ec style).",
        "Raised elements: a light shadow top-left and a dark shadow bottom-right; pressed elements use the same pair inset.",
        "Large radii (12-24px), no borders, low-contrast palette with a single accent colour.",
      ],
      tokens: {
        "--neu-bg": "#e0e5ec",
        "--neu-light": "#ffffff",
        "--neu-dark": "#a3b1c6",
        "--neu-accent": "#6d5dfc",
        "--neu-radius": "18px",
      },
      css: `body { background: var(--neu-bg); }
.neu {
  background: var(--neu-bg);
  border-radius: var(--neu-radius);
  box-shadow: 8px 8px 16px var(--neu-dark), -8px -8px 16px var(--neu-light);
}
.neu:active, .neu.is-pressed {
  box-shadow: inset 6px 6px 12px var(--neu-dark), inset -6px -6px 12px var(--neu-light);
}`,
      components: [
        "Buttons: raised by default, inset when pressed or toggled on; the accent colour marks the active state.",
        "Inputs: always inset so they read as wells.",
        "Sliders and knobs suit the style well; tables do not.",
      ],
      accessibility: [
        "The style is low-contrast by nature: give every interactive element a visible label, icon or accent, not just a shadow.",
        "Focus must be obvious: add an accent outline, shadows alone are not enough.",
      ],
      avoid: ["Text-heavy pages.", "Many nested raised levels.", "Using shadow as the only difference between on and off."],
    },
  },
  {
    id: "claymorphism",
    group: "depth",
    skill: {
      summary: "Puffy, inflated 3D shapes in soft pastel colours, like modelling clay, with an outer shadow and inner highlights.",
      fit: "Playful products, onboarding, kids and education apps, illustrations-first landing pages. Not for serious enterprise tools.",
      traits: [
        "Very rounded shapes (24-40px radius), chunky proportions.",
        "Three shadows: a soft outer drop shadow, an inner light highlight top-left and an inner dark shade bottom-right.",
        "Pastel fills on a light background; friendly rounded sans-serif type.",
        "Often paired with 3D clay-style illustrations or emoji-like icons.",
      ],
      tokens: {
        "--clay-bg": "#f4f1fb",
        "--clay-fill": "#c4b5fd",
        "--clay-radius": "32px",
        "--clay-shadow": "0 18px 30px rgba(99, 76, 160, 0.25)",
      },
      css: `.clay {
  background: var(--clay-fill);
  border-radius: var(--clay-radius);
  box-shadow: var(--clay-shadow),
              inset -8px -8px 16px rgba(0,0,0,0.12),
              inset 8px 8px 16px rgba(255,255,255,0.55);
}`,
      components: [
        "Buttons: fully rounded clay pills that squash slightly on press (transform: scale(.97)).",
        "Cards: one pastel colour per card; keep content inside simple.",
      ],
      accessibility: ["Pastel fills often fail contrast with white text: use dark text on pastels.", "Keep motion subtle."],
      avoid: ["Sharp corners and thin lines.", "Dark, saturated palettes.", "Dense forms."],
    },
  },
  {
    id: "skeuomorphism",
    group: "depth",
    skill: {
      summary: "Interfaces that imitate real objects and materials: textures, bevels, stitched leather, brushed metal, realistic lighting.",
      fit: "Instruments, audio plugins, games, nostalgic or luxury themes, and 'modern skeuomorphism' with subtle realism. Not for minimal productivity tools.",
      traits: [
        "Realistic materials: gradients that suggest curvature, subtle textures (noise, paper, wood).",
        "Bevels and embossing: light top edges, dark bottom edges, inner shadows for recesses.",
        "Controls look physical: knobs, toggles with depth, buttons that appear pressed.",
        "Modern variant: keep the realism restrained, one material per surface.",
      ],
      tokens: {
        "--sk-surface": "linear-gradient(180deg, #f7f7f7, #d9d9d9)",
        "--sk-edge-light": "rgba(255,255,255,0.9)",
        "--sk-edge-dark": "rgba(0,0,0,0.25)",
        "--sk-radius": "10px",
      },
      css: `.skeuo-button {
  background: var(--sk-surface);
  border: 1px solid #a8a8a8;
  border-radius: var(--sk-radius);
  box-shadow: inset 0 1px 0 var(--sk-edge-light), 0 2px 3px var(--sk-edge-dark);
  text-shadow: 0 1px 0 rgba(255,255,255,0.8);
}
.skeuo-button:active { background: linear-gradient(180deg, #d0d0d0, #eaeaea); box-shadow: inset 0 2px 4px var(--sk-edge-dark); }`,
      components: ["Toggles as physical switches; sliders as grooves with a metal thumb.", "Panels as materials (paper for notes, metal for controls)."],
      accessibility: ["Textures must not lower text contrast.", "Keep realistic detail out of reading areas."],
      avoid: ["Mixing many materials on one screen.", "Heavy image textures that slow loading."],
    },
  },
  {
    id: "material",
    group: "depth",
    skill: {
      summary: "Material Design 3: tonal surfaces, a colour system from one seed colour, clear elevation and state layers on a 4dp grid.",
      fit: "Android-style apps, admin panels and general-purpose products that need a well-known, consistent system.",
      traits: [
        "Colour roles derived from a seed: primary, secondary, tertiary, surface, surface-container levels, and their 'on-' colours.",
        "Elevation shown with tonal surfaces (lighter containers) plus soft shadows only where needed.",
        "Rounded shapes: small 8px, medium 12px, large 16-28px; pill-shaped buttons.",
        "State layers: hover 8%, focus/pressed 10-12% overlays of the on-colour.",
        "Spacing on a 4/8dp grid; type scale from display to label.",
      ],
      tokens: {
        "--md-primary": "#6750a4",
        "--md-on-primary": "#ffffff",
        "--md-surface": "#fef7ff",
        "--md-surface-container": "#f3edf7",
        "--md-on-surface": "#1d1b20",
        "--md-radius-m": "12px",
      },
      css: `.md-card { background: var(--md-surface-container); color: var(--md-on-surface); border-radius: var(--md-radius-m); padding: 16px; }
.md-button { background: var(--md-primary); color: var(--md-on-primary); border-radius: 999px; padding: 10px 24px; position: relative; overflow: hidden; }
.md-button::after { content: ""; position: absolute; inset: 0; background: currentColor; opacity: 0; }
.md-button:hover::after { opacity: .08; } .md-button:active::after { opacity: .12; }`,
      components: [
        "Buttons: filled, tonal, outlined, text; a FAB for the main action.",
        "Top app bar, navigation bar or rail, cards, chips, dialogs and snackbars from the standard set.",
      ],
      accessibility: ["The on-colour pairs are designed for contrast: always use them.", "Touch targets at least 48x48dp."],
      avoid: ["Inventing extra colours outside the roles.", "Hard black shadows."],
    },
  },

  // --- flat & minimal -----------------------------------------------------------------------
  {
    id: "flat",
    group: "flat",
    skill: {
      summary: "Flat design: solid colours, simple shapes and icons, no gradients, textures or realistic shadows.",
      fit: "Almost anything: apps, marketing sites, icons and illustrations; 'flat 2.0' adds very subtle shadows for affordance.",
      traits: [
        "Solid fills from a bright, limited palette; strong contrast between elements.",
        "Simple geometric icons and illustrations; long shadows are an optional period detail.",
        "Clear sans-serif type, clean edges, small radii (0-8px).",
        "Flat 2.0: allow one subtle shadow level to show what is clickable.",
      ],
      tokens: {
        "--flat-primary": "#2d9cdb",
        "--flat-success": "#27ae60",
        "--flat-warning": "#f2994a",
        "--flat-danger": "#eb5757",
        "--flat-bg": "#ffffff",
        "--flat-radius": "6px",
      },
      css: `.flat-button { background: var(--flat-primary); color: #fff; border: 0; border-radius: var(--flat-radius); padding: 10px 18px; }
.flat-button:hover { filter: brightness(1.08); }
.flat-card { background: var(--flat-bg); border: 1px solid #e5e7eb; border-radius: var(--flat-radius); }`,
      components: ["Buttons read as buttons through colour and shape, not depth.", "Icons: one weight, one style, geometric."],
      accessibility: ["Without depth cues, make interactive elements distinct by colour and label.", "Check contrast of white text on bright fills."],
      avoid: ["Mixing flat icons with realistic ones.", "Rainbow palettes with no hierarchy."],
    },
  },
  {
    id: "minimalism",
    group: "flat",
    skill: {
      summary: "Minimalism: generous whitespace, few elements, a restrained palette and strong typography doing the work.",
      fit: "Portfolios, premium products, documentation, reading-focused apps.",
      traits: [
        "Only what is needed on each screen; one clear primary action.",
        "Large whitespace and margins; content width limited for reading (60-75 characters).",
        "Neutral palette with at most one accent colour.",
        "Hierarchy through type size and weight, not boxes and borders.",
      ],
      tokens: {
        "--min-bg": "#fafafa",
        "--min-text": "#111111",
        "--min-muted": "#6b6b6b",
        "--min-accent": "#111111",
        "--min-space": "clamp(24px, 5vw, 64px)",
      },
      css: `body { background: var(--min-bg); color: var(--min-text); font: 17px/1.6 system-ui, sans-serif; }
main { max-width: 68ch; margin: 0 auto; padding: var(--min-space); }
h1 { font-size: clamp(2rem, 5vw, 3.5rem); font-weight: 600; letter-spacing: -0.02em; }
.min-link { color: inherit; text-underline-offset: 4px; }`,
      components: ["Buttons: text or thin outlined, one solid button for the main action.", "Navigation: a few text links, no icons unless needed."],
      accessibility: ["Muted grey text still needs 4.5:1 contrast.", "Do not hide essential controls for the sake of minimalism."],
      avoid: ["Decoration for its own sake.", "Hiding navigation behind unlabeled icons."],
    },
  },
  {
    id: "swiss",
    group: "flat",
    skill: {
      summary: "Swiss / International Typographic Style: a strict grid, sans-serif type, asymmetric layouts and bold scale contrast.",
      fit: "Editorial sites, posters, portfolios, events, design-conscious brands.",
      traits: [
        "A visible, strict column grid (12 columns); elements align to it precisely.",
        "Neo-grotesque sans-serif (Helvetica, Inter, Arial as fallback), flush-left and ragged-right text.",
        "Strong contrast of scale: very large headlines next to small body text.",
        "Limited colours: black, white and one strong colour (often red).",
        "Asymmetric compositions and generous negative space.",
      ],
      tokens: {
        "--sw-text": "#000000",
        "--sw-bg": "#ffffff",
        "--sw-accent": "#e30613",
        "--sw-gutter": "24px",
      },
      css: `.swiss { display: grid; grid-template-columns: repeat(12, 1fr); gap: var(--sw-gutter); font-family: "Helvetica Neue", Inter, Arial, sans-serif; }
.swiss h1 { grid-column: 1 / 9; font-size: clamp(3rem, 9vw, 8rem); line-height: .9; font-weight: 700; letter-spacing: -0.04em; }
.swiss .meta { grid-column: 10 / 13; font-size: .8rem; text-transform: uppercase; }`,
      components: ["Navigation as a typographic list aligned to the grid.", "Images cropped to grid cells; no rounded corners."],
      accessibility: ["Huge type needs responsive sizing so it does not overflow on phones."],
      avoid: ["Centered, symmetrical layouts.", "Decorative fonts and rounded cards."],
    },
  },
  {
    id: "monochrome",
    group: "flat",
    skill: {
      summary: "Monochrome: one hue (or pure greyscale) in many tints and shades, optionally with a single accent.",
      fit: "Brand-focused sites, developer tools, photography, calm dashboards.",
      traits: [
        "A 9-11 step scale of one hue carries backgrounds, surfaces, borders and text.",
        "Hierarchy through lightness and type, not through extra colours.",
        "At most one accent colour for actions or status.",
      ],
      tokens: {
        "--mono-50": "#f5f7fa",
        "--mono-200": "#d5dbe3",
        "--mono-500": "#6b7a90",
        "--mono-800": "#243044",
        "--mono-950": "#0d1320",
        "--mono-accent": "#ff5a36",
      },
      css: `body { background: var(--mono-50); color: var(--mono-950); }
.card { background: #fff; border: 1px solid var(--mono-200); }
.muted { color: var(--mono-500); }
.button { background: var(--mono-800); color: var(--mono-50); }
.button.primary { background: var(--mono-accent); }`,
      components: ["Status colours (error, success) may break the rule, but keep them muted."],
      accessibility: ["Adjacent steps of one hue can be too close: check contrast for text and borders."],
      avoid: ["Several accent colours.", "Using colour alone to convey state."],
    },
  },
  {
    id: "bento",
    group: "flat",
    skill: {
      summary: "Bento grid: a layout of rounded tiles of different sizes, each presenting one idea, like a bento box.",
      fit: "Product feature overviews, portfolios, dashboards, personal pages.",
      traits: [
        "CSS grid with tiles spanning 1-2 columns or rows; one hero tile, several small ones.",
        "Each tile has one message: a number, an image, a short feature line.",
        "Consistent gaps (12-20px) and radii (16-28px); tiles can use different backgrounds.",
        "Reflows to a single column on phones.",
      ],
      tokens: {
        "--bento-gap": "16px",
        "--bento-radius": "22px",
        "--bento-tile": "#f2f2f7",
      },
      css: `.bento { display: grid; grid-template-columns: repeat(4, 1fr); grid-auto-rows: 160px; gap: var(--bento-gap); }
.bento > * { background: var(--bento-tile); border-radius: var(--bento-radius); padding: 20px; overflow: hidden; }
.bento .wide { grid-column: span 2; } .bento .tall { grid-row: span 2; }
@media (max-width: 640px) { .bento { grid-template-columns: 1fr; } .bento .wide, .bento .tall { grid-column: auto; grid-row: auto; } }`,
      components: ["Tiles as links: the whole tile is clickable with a clear hover state.", "Mix typography tiles, image tiles and stat tiles."],
      accessibility: ["Keep a logical reading order in the DOM, independent of the visual grid."],
      avoid: ["Too many equal tiles (it becomes a plain card grid).", "Cramming paragraphs into tiles."],
    },
  },

  // --- bold & raw ---------------------------------------------------------------------------
  {
    id: "brutalism",
    group: "bold",
    skill: {
      summary: "Brutalism: raw, unpolished web aesthetics with default fonts, visible structure, harsh contrast and no decoration.",
      fit: "Art, fashion, experimental and statement sites. Not for mainstream products that need polish and trust.",
      traits: [
        "System or monospace fonts, default blue underlined links, visible borders and tables.",
        "Stark black/white with occasional clashing colour.",
        "Deliberately 'unfinished' layouts: overlapping, oversized or rotated elements are allowed.",
      ],
      tokens: {
        "--br-bg": "#ffffff",
        "--br-text": "#000000",
        "--br-link": "#0000ee",
        "--br-font": "\"Times New Roman\", ui-monospace, monospace",
      },
      css: `body { background: var(--br-bg); color: var(--br-text); font-family: var(--br-font); }
a { color: var(--br-link); text-decoration: underline; }
.block { border: 2px solid #000; padding: 12px; }
button { font: inherit; border: 2px solid #000; background: #fff; border-radius: 0; }`,
      components: ["Buttons: plain rectangles with a border; hover inverts colours.", "Layout: raw grids and tables, visible dividers."],
      accessibility: ["Raw is not broken: keep semantic HTML, focus styles and readable sizes."],
      avoid: ["Rounded corners, shadows and gradients.", "Making the page unusable on purpose."],
    },
  },
  {
    id: "neo-brutalism",
    group: "bold",
    skill: {
      summary: "Neo-brutalism: thick black outlines, hard offset shadows, flat saturated colours and chunky bold type.",
      fit: "Startups, creative tools, marketing sites, playful dashboards that want personality.",
      traits: [
        "2-4px solid black borders on every component.",
        "Hard shadows with no blur, offset down-right (4-8px), in black.",
        "Flat, saturated fills (yellow, pink, mint, blue) on an off-white background.",
        "Bold grotesque or rounded sans-serif; small or zero radius.",
        "Press state: the element moves into its shadow.",
      ],
      tokens: {
        "--nb-border": "3px solid #000",
        "--nb-shadow": "5px 5px 0 #000",
        "--nb-bg": "#fffdf5",
        "--nb-yellow": "#ffd23f",
        "--nb-pink": "#ff6b9d",
        "--nb-mint": "#3ddc97",
      },
      css: `.nb { background: var(--nb-yellow); border: var(--nb-border); box-shadow: var(--nb-shadow); border-radius: 6px; font-weight: 800; }
.nb:hover { transform: translate(-2px, -2px); box-shadow: 7px 7px 0 #000; }
.nb:active { transform: translate(5px, 5px); box-shadow: 0 0 0 #000; }`,
      components: ["Cards, inputs and buttons all share the border and shadow.", "Tags and badges as small bordered pills with a bright fill."],
      accessibility: ["Bright fills need black text; check yellow and mint contrast.", "Movement on hover should respect reduced motion."],
      avoid: ["Soft blurred shadows.", "Thin borders or gradients."],
    },
  },
  {
    id: "memphis",
    group: "bold",
    skill: {
      summary: "Memphis: 1980s design with scattered geometric shapes, squiggles, confetti patterns and clashing bright colours.",
      fit: "Events, youth brands, playful campaigns, creative landing pages.",
      traits: [
        "Background decorations: triangles, circles, zig-zags, dots and squiggles, often outlined in black.",
        "Clashing bright palette: pink, yellow, teal, purple, with black accents.",
        "Patterns (dots, stripes, grids) as fills; bold geometric type.",
      ],
      tokens: {
        "--mem-pink": "#ff71ce",
        "--mem-yellow": "#fffb96",
        "--mem-teal": "#01cdfe",
        "--mem-purple": "#b967ff",
        "--mem-ink": "#1a1a1a",
      },
      css: `.memphis {
  background:
    radial-gradient(var(--mem-ink) 1.5px, transparent 1.5px) 0 0 / 18px 18px,
    var(--mem-yellow);
}
.memphis .shape { position: absolute; border: 3px solid var(--mem-ink); }
.memphis .triangle { width: 0; height: 0; border: none; border-left: 22px solid transparent; border-right: 22px solid transparent; border-bottom: 38px solid var(--mem-pink); }`,
      components: ["Content sits on solid blocks so text stays readable over the patterns.", "Decorative SVG shapes placed around, not under, text."],
      accessibility: ["Patterns behind text kill readability: always put text on a solid fill.", "Avoid flashing or animated patterns."],
      avoid: ["Using it for dense data.", "Muted, corporate palettes."],
    },
  },
  {
    id: "y2k",
    group: "bold",
    skill: {
      summary: "Y2K: late-90s/early-2000s futurism with chrome and metallic gradients, glossy bubbles, iridescence and bright candy colours.",
      fit: "Fashion, music, nostalgia and youth brands, playful campaigns.",
      traits: [
        "Chrome and liquid-metal gradients, glossy pill buttons with a white highlight.",
        "Iridescent or holographic gradients (pink, lilac, cyan).",
        "Rounded 'bubble' shapes, stars and sparkles, pixel or wide display type.",
      ],
      tokens: {
        "--y2k-chrome": "linear-gradient(180deg, #ffffff 0%, #c0c7d0 45%, #7d8794 55%, #e6ebf1 100%)",
        "--y2k-holo": "linear-gradient(120deg, #ffb3f5, #b3c7ff, #b3fff0, #fff3b3)",
        "--y2k-pink": "#ff4fd8",
        "--y2k-radius": "999px",
      },
      css: `.y2k-button {
  background: var(--y2k-holo);
  border-radius: var(--y2k-radius);
  border: 1px solid rgba(255,255,255,0.8);
  box-shadow: inset 0 6px 8px rgba(255,255,255,0.8), 0 4px 14px rgba(255,79,216,0.35);
}
.chrome-text { background: var(--y2k-chrome); -webkit-background-clip: text; background-clip: text; color: transparent; }`,
      components: ["Glossy capsule buttons; star and sparkle icons; window-like panels with a title bar."],
      accessibility: ["Chrome text is hard to read: use it for short display words only.", "Keep body text on plain backgrounds."],
      avoid: ["Chrome everywhere.", "Tiny text on holographic gradients."],
    },
  },
  {
    id: "pixel",
    group: "bold",
    skill: {
      summary: "Pixel / 8-bit: retro game aesthetics with pixel fonts, hard stepped edges, a limited palette and chunky borders.",
      fit: "Games, game-related tools, playful developer products, retro themes.",
      traits: [
        "A small fixed palette (8-16 colours) such as the NES or PICO-8 palettes.",
        "Pixel fonts for headings (monospace fallback) and crisp, unsmoothed images.",
        "Stepped 'pixel' borders built with box-shadow; no radius, no blur.",
      ],
      tokens: {
        "--px-bg": "#1d2b53",
        "--px-fg": "#fff1e8",
        "--px-accent": "#ff004d",
        "--px-ok": "#00e436",
        "--px-unit": "4px",
      },
      css: `.pixel { image-rendering: pixelated; font-family: "Press Start 2P", ui-monospace, monospace; }
.pixel-box {
  background: var(--px-bg); color: var(--px-fg);
  box-shadow:
    0 calc(-1 * var(--px-unit)) 0 var(--px-fg), 0 var(--px-unit) 0 var(--px-fg),
    calc(-1 * var(--px-unit)) 0 0 var(--px-fg), var(--px-unit) 0 0 var(--px-fg);
  margin: var(--px-unit);
}`,
      components: ["Buttons as pixel boxes that shift one unit down on press.", "Progress as block bars; icons as sprites."],
      accessibility: ["Pixel fonts are hard to read at body size: keep body text in a clear monospace or sans-serif."],
      avoid: ["Anti-aliased gradients and soft shadows.", "Large paragraphs in pixel fonts."],
    },
  },

  // --- colour & light -----------------------------------------------------------------------
  {
    id: "aurora",
    group: "light",
    skill: {
      summary: "Aurora / gradient mesh: large blurred colour blobs flowing across the background, often with grain and glass or dark cards on top.",
      fit: "SaaS and AI product landing pages, hero sections, onboarding screens.",
      traits: [
        "Two to four radial gradients in harmonious colours, heavily blurred, sometimes slowly animated.",
        "Dark (#0b1020) or very light base; subtle noise texture to avoid banding.",
        "Content on clean cards (glass or solid) above the aurora.",
      ],
      tokens: {
        "--au-base": "#0b1020",
        "--au-1": "#7c3aed",
        "--au-2": "#06b6d4",
        "--au-3": "#f472b6",
      },
      css: `.aurora { position: relative; background: var(--au-base); overflow: hidden; }
.aurora::before {
  content: ""; position: absolute; inset: -20%;
  background:
    radial-gradient(40% 40% at 20% 30%, var(--au-1), transparent 70%),
    radial-gradient(35% 35% at 80% 20%, var(--au-2), transparent 70%),
    radial-gradient(45% 45% at 60% 90%, var(--au-3), transparent 70%);
  filter: blur(60px); opacity: .8;
}
.aurora > * { position: relative; }`,
      components: ["Hero heading in white on the aurora; secondary content on cards.", "Animate slowly (20-40s) or not at all."],
      accessibility: ["Animated backgrounds must stop with prefers-reduced-motion.", "Check text contrast over the brightest blob."],
      avoid: ["Clashing gradient hues.", "Aurora behind long text."],
    },
  },
  {
    id: "neon",
    group: "light",
    skill: {
      summary: "Neon / cyberpunk: dark backgrounds with glowing cyan, magenta and lime accents, grids, scan lines and futuristic type.",
      fit: "Gaming, crypto, music, developer and hacker-themed products, event pages.",
      traits: [
        "Near-black background; neon accents with glow (text-shadow and box-shadow in the same colour).",
        "Perspective grids, scan lines or glitch effects used sparingly.",
        "Monospace or condensed futuristic display type; thin outlined components.",
      ],
      tokens: {
        "--neon-bg": "#07060d",
        "--neon-cyan": "#00f0ff",
        "--neon-magenta": "#ff2bd6",
        "--neon-lime": "#b6ff00",
        "--neon-glow": "0 0 6px currentColor, 0 0 18px currentColor",
      },
      css: `body { background: var(--neon-bg); color: #e6e6f0; }
.neon-text { color: var(--neon-cyan); text-shadow: var(--neon-glow); }
.neon-button { color: var(--neon-magenta); border: 1px solid currentColor; box-shadow: var(--neon-glow), inset 0 0 8px currentColor; background: transparent; }
.grid-floor { background: linear-gradient(transparent 95%, rgba(255,43,214,.5) 95%) 0 0 / 40px 40px, linear-gradient(90deg, transparent 95%, rgba(255,43,214,.5) 95%) 0 0 / 40px 40px; transform: perspective(400px) rotateX(60deg); }`,
      components: ["Outlined glowing buttons; terminal-like panels; data shown with monospace numbers."],
      accessibility: ["Glow reduces legibility: body text stays plain light grey without glow.", "Flicker and glitch effects must respect reduced motion."],
      avoid: ["Glow on every element.", "Neon on light backgrounds."],
    },
  },
  {
    id: "pastel",
    group: "light",
    skill: {
      summary: "Soft pastel: light, low-saturation colours, rounded shapes and gentle shadows for a calm, friendly feel.",
      fit: "Wellness, lifestyle, kids, planners, note-taking and consumer apps.",
      traits: [
        "Pastel fills (mint, peach, lilac, sky, butter) on a warm off-white background.",
        "Rounded corners (12-20px), soft low-opacity shadows.",
        "Rounded sans-serif type; dark but soft text colour instead of pure black.",
      ],
      tokens: {
        "--pa-bg": "#fffaf5",
        "--pa-mint": "#c8f2e0",
        "--pa-peach": "#ffd9c7",
        "--pa-lilac": "#e3d7ff",
        "--pa-sky": "#cfe8ff",
        "--pa-text": "#3b3552",
      },
      css: `body { background: var(--pa-bg); color: var(--pa-text); }
.pa-card { background: var(--pa-lilac); border-radius: 18px; box-shadow: 0 6px 18px rgba(59,53,82,.08); }
.pa-button { background: var(--pa-mint); color: var(--pa-text); border-radius: 999px; }`,
      components: ["Each section or category gets its own pastel; accents slightly more saturated for actions."],
      accessibility: ["White text on pastels fails contrast: use the dark text colour.", "Borders may be needed where two pastels meet."],
      avoid: ["Neon or fully saturated accents.", "Grey-on-pastel text."],
    },
  },
  {
    id: "duotone",
    group: "light",
    skill: {
      summary: "Duotone: a palette and imagery reduced to two strong colours, with photos mapped to a dark and a light tone.",
      fit: "Music, events, campaigns, bold editorial and brand pages.",
      traits: [
        "Two colours define everything: a dark tone (shadows) and a light tone (highlights).",
        "Photos converted to duotone with CSS blend modes or SVG filters.",
        "Large type in one of the two colours on the other.",
      ],
      tokens: {
        "--duo-dark": "#1e1b4b",
        "--duo-light": "#fb7185",
      },
      css: `.duotone { position: relative; background: var(--duo-light); }
.duotone img { filter: grayscale(1) contrast(1.1); mix-blend-mode: multiply; display: block; width: 100%; }
.duotone::after { content: ""; position: absolute; inset: 0; background: var(--duo-dark); mix-blend-mode: lighten; pointer-events: none; }`,
      components: ["Buttons in the light colour on dark sections and the reverse; no third colour except neutrals."],
      accessibility: ["Check that the two colours contrast enough for text (4.5:1)."],
      avoid: ["Adding a third brand colour.", "Low-contrast pairs such as two mid tones."],
    },
  },

  // --- editorial & organic ------------------------------------------------------------------
  {
    id: "editorial",
    group: "editorial",
    skill: {
      summary: "Editorial / magazine: serif headlines, large type, multi-column text, pull quotes, rules and generous margins.",
      fit: "Blogs, publications, long-form stories, portfolios and premium brands.",
      traits: [
        "Serif display type for headlines (a serif fallback stack), a readable serif or sans for body.",
        "Strong typographic hierarchy: kicker, headline, deck, byline, body.",
        "Columns, drop caps, pull quotes and thin rules; images with captions.",
        "Warm paper-like background and near-black text.",
      ],
      tokens: {
        "--ed-bg": "#f7f3ec",
        "--ed-text": "#1b1b1b",
        "--ed-accent": "#b3261e",
        "--ed-serif": "\"Playfair Display\", Georgia, \"Times New Roman\", serif",
        "--ed-measure": "66ch",
      },
      css: `body { background: var(--ed-bg); color: var(--ed-text); }
h1 { font-family: var(--ed-serif); font-size: clamp(2.5rem, 7vw, 5.5rem); line-height: 1; font-weight: 700; }
.kicker { font: 600 .75rem/1 system-ui, sans-serif; letter-spacing: .12em; text-transform: uppercase; color: var(--ed-accent); }
.article { max-width: var(--ed-measure); font: 1.15rem/1.7 Georgia, serif; }
.article > p:first-of-type::first-letter { float: left; font: 700 4.2em/0.8 var(--ed-serif); padding: 6px 8px 0 0; }
blockquote { font-family: var(--ed-serif); font-size: 1.6rem; border-top: 1px solid; border-bottom: 1px solid; padding: 16px 0; }`,
      components: ["Navigation as a masthead with sections; article cards with image, kicker and headline."],
      accessibility: ["Columns collapse to one on phones; line length stays within 45-75 characters."],
      avoid: ["Rounded app-like cards everywhere.", "Tiny body text."],
    },
  },
  {
    id: "organic",
    group: "editorial",
    skill: {
      summary: "Organic: natural, flowing shapes (blobs, waves), earthy colours and soft textures instead of hard geometry.",
      fit: "Wellness, food, sustainability, outdoor and craft brands.",
      traits: [
        "Blob shapes made with uneven border-radius values or SVG paths; wavy section dividers.",
        "Earthy palette: sage, clay, sand, moss, terracotta.",
        "Soft textures (paper, grain) and hand-picked photography of natural materials.",
      ],
      tokens: {
        "--org-sand": "#f3ebdd",
        "--org-sage": "#a3b18a",
        "--org-moss": "#588157",
        "--org-clay": "#c97c5d",
        "--org-ink": "#2f3e2e",
      },
      css: `.blob { background: var(--org-sage); border-radius: 62% 38% 46% 54% / 60% 44% 56% 40%; }
body { background: var(--org-sand); color: var(--org-ink); }
.wave { height: 60px; background: var(--org-moss); clip-path: path("M0 30 Q 150 0 300 30 T 600 30 V60 H0 Z"); }`,
      components: ["Buttons with soft, slightly irregular radii; cards shaped like pebbles for highlights only."],
      accessibility: ["Earthy tones can be close in lightness: check contrast.", "Irregular shapes must not crop content."],
      avoid: ["Neon and chrome.", "Strict rectangular grids everywhere."],
    },
  },
  {
    id: "hand-drawn",
    group: "editorial",
    skill: {
      summary: "Hand-drawn / sketch: wobbly outlines, handwritten type, doodle icons and paper textures for a personal, informal feel.",
      fit: "Education, creative tools, personal sites, wireframe-like prototypes and friendly onboarding.",
      traits: [
        "Irregular borders drawn with uneven border-radius, or SVG strokes that look hand-made.",
        "Handwritten display font (with a cursive fallback) and a clear sans for body text.",
        "Doodle icons, arrows and underlines; paper or notebook backgrounds.",
      ],
      tokens: {
        "--hd-paper": "#fffdf7",
        "--hd-ink": "#2b2b2b",
        "--hd-marker": "#ffe066",
        "--hd-font": "\"Patrick Hand\", \"Comic Neue\", \"Segoe Print\", cursive",
      },
      css: `.sketch {
  border: 2px solid var(--hd-ink);
  border-radius: 255px 15px 225px 15px / 15px 225px 15px 255px;
  background: var(--hd-paper);
}
.highlight { background: linear-gradient(transparent 55%, var(--hd-marker) 55%); }
h1, h2 { font-family: var(--hd-font); }`,
      components: ["Buttons as sketched boxes that wiggle slightly on hover.", "Arrows and underlines as SVG strokes pointing at key actions."],
      accessibility: ["Keep handwritten fonts for headings and short labels only.", "Wiggle animations respect reduced motion."],
      avoid: ["Handwritten body text.", "Mixing with glossy or realistic styles."],
    },
  },
];

export function skillName(id: string): string {
  return `ui-${id}`;
}

/** Whether a skill name is one of the style skills. */
export function isUiStyleSkill(name: string): boolean {
  return UI_STYLES.some((s) => skillName(s.id) === name);
}

function title(id: string): string {
  return id
    .split("-")
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

/** The SKILL.md installed for a style. */
export function skillMarkdown(style: UiStyle): string {
  const s = style.skill;
  const tokens = Object.entries(s.tokens)
    .map(([k, v]) => `  ${k}: ${v};`)
    .join("\n");
  const list = (items: string[]) => items.map((i) => `- ${i}`).join("\n");
  return `---
name: ${skillName(style.id)}
description: ${JSON.stringify(`${title(style.id)} interface style. ${s.summary} Use when the user asks for this look or has selected it for the project.`)}
---

# ${title(style.id)} interface style

${s.summary}

## When it fits
${s.fit}

## Defining traits
${list(s.traits)}

## Design tokens
\`\`\`css
:root {
${tokens}
}
\`\`\`

## Core CSS recipe
\`\`\`css
${s.css}
\`\`\`

## Components
${list(s.components)}

## Accessibility and performance
${list(s.accessibility)}

## Avoid
${list(s.avoid)}

## How to apply
- Start from the tokens above and adapt the colours to the product's brand; keep the defining traits.
- Apply the style consistently to every component you touch; do not mix it with another style unless the user asked for a combination.
- Keep the project's existing framework (plain CSS, Tailwind, CSS modules, a component library) and express the recipe in it.
`;
}
