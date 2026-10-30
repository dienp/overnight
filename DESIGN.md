---
name: Overnight
description: A macOS menu bar app that keeps a MacBook awake while Claude works, then lets it sleep.
colors:
  night-a: "#23264f"
  night-b: "#3a316f"
  dusk-a: "#2a2d3b"
  dusk-b: "#3f4459"
  rose: "#6b4a7a"
  dawn: "#c9875a"
  moon: "#f7d680"
  accent: "#5856d6"
  accent-dark: "#7d7aff"
  accent-fill: "#5856d6"
  accent-fill-dark: "#5e5ce6"
  accent-text: "#5856d6"
  accent-text-dark: "#a5a3ff"
  green: "#34c759"
  green-dark: "#30d158"
  orange: "#ff9500"
  orange-dark: "#ff9f0a"
  red: "#ff3b30"
  red-dark: "#ff453a"
  green-text: "#1d7a35"
  green-text-dark: "#30d158"
  orange-text: "#a35200"
  orange-text-dark: "#ff9f0a"
  red-text: "#c41e14"
  red-text-dark: "#ff7a72"
  popover-bg: "#f2f2f7"
  popover-bg-dark: "#1c1c1e"
  popover-card: "#ffffff"
  popover-card-dark: "#2c2c2e"
  popover-text: "#1c1c1e"
  popover-text-dark: "#f2f2f7"
  popover-muted: "#6e6e73"
  popover-muted-dark: "#a1a1a6"
  popover-faint: "#aeaeb2"
  popover-faint-dark: "#636366"
  docs-ground: "#f7f7fa"
  docs-ground-dark: "#141418"
  docs-card: "#ffffff"
  docs-card-dark: "#1e1e24"
  docs-ink: "#1c1c1e"
  docs-ink-dark: "#ececf1"
  docs-muted: "#5d5d66"
  docs-muted-dark: "#a3a3ab"
  docs-day: "#e8ebf4"
  docs-day-dark: "#22242e"
  code-ground: "#1d2040"
  code-ground-dark: "#1a1c34"
  code-ink: "#e9e9f6"
typography:
  popover-title:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, Helvetica Neue, sans-serif"
    fontSize: "15px"
    fontWeight: 600
  popover-body:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, Helvetica Neue, sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.4
  popover-small:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, Helvetica Neue, sans-serif"
    fontSize: "12px"
    fontWeight: 400
  popover-caption:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, Helvetica Neue, sans-serif"
    fontSize: "11px"
    fontWeight: 400
  popover-group-label:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, Helvetica Neue, sans-serif"
    fontSize: "11px"
    fontWeight: 600
    letterSpacing: "0.04em"
  popover-figure:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, Helvetica Neue, sans-serif"
    fontSize: "15px"
    fontWeight: 600
    fontFeature: "tnum"
  docs-display:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, Helvetica Neue, Segoe UI, sans-serif"
    fontSize: "clamp(44px, 5.4vw, 72px)"
    fontWeight: 700
    lineHeight: 1
    letterSpacing: "-0.03em"
  docs-chapter:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, Helvetica Neue, Segoe UI, sans-serif"
    fontSize: "clamp(28px, 3.4vw, 38px)"
    fontWeight: 700
    lineHeight: 1.15
    letterSpacing: "-0.02em"
  docs-clock:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, Helvetica Neue, Segoe UI, sans-serif"
    fontSize: "clamp(28px, 3.4vw, 40px)"
    fontWeight: 600
    fontFeature: "tnum"
  docs-section:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, Helvetica Neue, Segoe UI, sans-serif"
    fontSize: "22px"
    fontWeight: 700
    lineHeight: 1.3
    letterSpacing: "-0.01em"
  docs-lede:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, Helvetica Neue, Segoe UI, sans-serif"
    fontSize: "20px"
    fontWeight: 400
    lineHeight: 1.5
  docs-body:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, Helvetica Neue, Segoe UI, sans-serif"
    fontSize: "17px"
    fontWeight: 400
    lineHeight: 1.6
  docs-small:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, Helvetica Neue, Segoe UI, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.45
  code:
    fontFamily: "ui-monospace, SF Mono, Menlo, monospace"
    fontSize: "14px"
    lineHeight: 1.55
rounded:
  hairline: "4px"
  control: "6px"
  button: "8px"
  card: "10px"
  panel: "12px"
  round: "50%"
spacing:
  popover-edge: "10px"
  popover-group: "12px"
  popover-hero: "14px"
  docs-gutter: "24px"
  docs-measure: "68ch"
  docs-bar: "52px"
components:
  button-primary:
    backgroundColor: "{colors.accent-fill}"
    textColor: "#ffffff"
    rounded: "{rounded.button}"
    padding: "8px 12px"
  button-quiet:
    backgroundColor: "rgba(255, 255, 255, 0.14)"
    textColor: "#ffffff"
    rounded: "{rounded.button}"
    padding: "8px 12px"
  button-secondary:
    backgroundColor: "{colors.popover-card}"
    textColor: "{colors.accent-text}"
    rounded: "{rounded.button}"
    padding: "8px 12px"
  button-small:
    rounded: "{rounded.button}"
    padding: "4px 10px"
  hero-note:
    backgroundColor: "rgba(255, 255, 255, 0.14)"
    textColor: "#ffffff"
    rounded: "{rounded.button}"
    padding: "7px 10px"
  hero-note-off:
    textColor: "{colors.orange-text}"
    rounded: "{rounded.button}"
    padding: "7px 10px"
  group:
    backgroundColor: "{colors.popover-card}"
    rounded: "{rounded.card}"
    padding: "10px 12px"
  pill:
    textColor: "{colors.popover-muted}"
    rounded: "{rounded.control}"
    padding: "1px 6px"
  pill-working:
    textColor: "{colors.green-text}"
    rounded: "{rounded.control}"
    padding: "1px 6px"
  pill-paused:
    textColor: "{colors.orange-text}"
    rounded: "{rounded.control}"
    padding: "1px 6px"
  pill-stopped:
    textColor: "{colors.red-text}"
    rounded: "{rounded.control}"
    padding: "1px 6px"
  docs-button:
    backgroundColor: "{colors.moon}"
    textColor: "{colors.night-a}"
    rounded: "{rounded.card}"
    padding: "11px 20px"
  docs-button-ghost:
    backgroundColor: "transparent"
    textColor: "#ffffff"
    rounded: "{rounded.card}"
    padding: "11px 20px"
  docs-marker:
    backgroundColor: "{colors.moon}"
    textColor: "{colors.night-a}"
    rounded: "{rounded.round}"
    size: "28px"
  docs-code-block:
    backgroundColor: "{colors.code-ground}"
    textColor: "{colors.code-ink}"
    typography: "{typography.code}"
    rounded: "{rounded.card}"
    padding: "16px 18px"
---

# Design system: Overnight

## Overview

**Creative North Star: "One night, told by its sky"**

Overnight has one visual world across two surfaces: the menu bar popover (380 by 620) and the documentation site. Both are plain macOS at rest: system type, system grays, grouped white cards on a light gray ground, system status colours. The world lives in one place on each surface, a sky that carries the time of night. In the popover that sky is the hero at the top, which turns to full night while the Mac is held awake, to an unlit dusk while armed but letting it sleep, and to night warming into dawn the morning after. In the docs the same sky owns the frame: the first viewport, the chapter openers and the sticky running head move from dusk through night and rose to dawn and daylight as the reader moves through the night.

The moon is the only ornament. The crescent and star from the menu bar icon appear in the popover hero, the docs wordmark and the running head, and its gold is the colour of the docs' markers and primary button. Everything outside the sky stays calm: a reading column of 68ch on a near-white ground, tables and code blocks without decoration, screenshots of the real popover with sample data.

Density differs by surface. The popover is a compact settings panel (13px body, 34px rows, 10px outer margins). The docs read like a manual (17px body, 1.6 line height, generous chapter padding).

**Key characteristics:**
- The sky is the only expressive layer; everything else is native macOS.
- Hero and opener backgrounds are state and time, never decoration.
- One filled accent per popover screen: the main button.
- Status colour words, never colour alone; small status text uses the text-safe tokens.
- Light and dark themes for both surfaces, following the system setting.
- Real product screenshots, labelled as sample data.

## Colors

A deep indigo night with a gold moon and an amber dawn, framed by Apple's system neutrals and status colours.

### Primary
- **Night indigo** (night-a) to **night violet** (night-b): the held-awake sky. The popover's on hero is a 160deg gradient between them; the docs' night opener and the intro's lower half use them. Night-a is also the ink on gold and amber (docs buttons, markers, the dawn opener and bar).
- **Moon gold** (moon): the moon and star, the docs' markers and numbered headings, the docs' primary button, the clock in night openers, the skip link and text selection (at 55% alpha). On the popover it appears only inside the hero.
- **System indigo** (accent): the interactive accent for the popover's bars, progress fills, checkboxes and timeline dots, and the docs' links. It stays the same in both docs themes only through the accent-text pair below.

### Secondary
- **Dusk slate** (dusk-a to dusk-b): the night without its glow. Popover standby hero, the docs' first opener and the running head's default.
- **Rose** (rose): the usage-limit hour. Ends the rose opener gradient (night-b, then #4f3d70 at 60%, then rose). The running head uses #4a3a68 in this phase.
- **Dawn amber** (dawn): the morning. The popover's morning hero runs night-a 0%, night-b 55%, dawn 140% at 172deg, so only the lower edge warms. The docs' dawn opener and running head turn amber with night-a type.

### Tertiary
- **Accent fill** (accent-fill, accent-fill-dark): the accent behind white text, used only on the primary button. White on it measures 5.65:1 light and 5.06:1 dark.
- **Accent text** (accent-text, accent-text-dark): the accent as text, link, secondary button label and focus ring. In dark it lifts to a pale indigo so it reads on dark cards (6.16:1 on the dark card). The docs' single accent token uses the same two values.
- **Status fills** (green, orange, red and their dark values): Apple's system colours, for switches, dots, bars, tints and the tab badge. Never for small text on light grounds.
- **Status text** (green-text, orange-text, red-text and their dark values): darker light-theme values that pass AA on white and on their own 14 to 18% tints (4.66 to 4.98:1). In dark, green and orange text equal the fills and red lifts to a lighter coral.

### Neutral
- **Popover ground and cards** (popover-bg, popover-card): grouped-list gray behind white cards, Apple's system values. Dark uses #1c1c1e behind #2c2c2e.
- **Popover text, muted, faint**: primary text, secondary values and sublabels, and disabled or idle marks (idle dots, the off-state moon outline).
- **Hairlines**: rgba(0, 0, 0, 0.08) in the popover, rgba(28, 28, 30, 0.12) in the docs; white at the same alphas in dark.
- **Controls**: rgba(118, 118, 128, 0.12), 0.24 in dark. The track behind tabs, segmented controls, inputs and switches.
- **Docs ground, card, ink, muted** (docs-ground and family): a calm reading column, slightly cooler than the popover's ground. Dark ground is near-black #141418.
- **Daylight** (docs-day): the last opener, the build chapter.
- **Code** (code-ground, code-ink): a dark night-blue block in both themes. Inline code uses the accent at 9% (14% dark).

### Named rules
**The sky carries the time rule.** Night, dusk, rose, dawn and day colours appear only on surfaces that show the time or state of the night: the popover hero, the docs intro, chapter openers and running head. Content cards never take them.

**The one filled accent rule.** In the popover the only filled accent is the main button. Selected tabs and segments are raised white segments, not accent fills.

**The text-safe status rule.** Status words and small status text use green-text, orange-text and red-text. The bright fills are for shapes. Tinted surfaces mix the fill into the card (12 to 18%) and keep text in the text token; secondary text on a callout mixes the text colour 78% with the callout's tone instead of gray.

**The two accents rule.** Text and rings use accent-text; white-on-accent uses accent-fill. They are the same in light and differ in dark. Never put white text on accent-text-dark or use accent-fill-dark as link colour.

**The dark duplicate rule.** The popover's dark theme is declared twice in src/styles.css: once under prefers-color-scheme for the app, once on [data-theme="dark"] for the preview's theme=dark. The two blocks must hold identical values; change both in the same edit. The docs have a single dark block.

## Typography

**Display font:** the system stack (-apple-system, BlinkMacSystemFont, SF Pro Text, Helvetica Neue; the docs add Segoe UI)
**Body font:** the same system stack
**Label/mono font:** ui-monospace, SF Mono, Menlo, monospace, for code and the ntfy topic field only

**Character:** San Francisco everywhere, because the product is a macOS utility and its docs describe one. Hierarchy comes from size and weight, not from a second face.

### Hierarchy
- **Display** (docs-display): the docs wordmark heading, with the moon at 0.82em beside it. Used once.
- **Headline** (docs-chapter, with docs-clock beside it): chapter openers. The clock (8 PM, 11 PM, 2 AM, 8 AM, Day) sits right, in moon gold on night grounds, night ink on dawn, and dark gold #a87a14 on daylight (moon gold in dark).
- **Title** (docs-section, popover-title): docs section headings, which carry a numbered gold disc when they explain a popover marker; the popover hero headline.
- **Body** (docs-body, popover-body): docs paragraphs in a 68ch column; the docs lede at 20px (18px under 700px) with a 34ch measure. The popover's base text at 13px.
- **Label** (popover-group-label, popover-small, popover-caption): the popover's group headers in uppercase muted gray, hint and note text at 12px, sublabels, timestamps and project names at 11px.
- **Figures** (popover-figure): hero stats, log stats, times, sizes and the docs clock and markers use tabular figures. The free-space figure is 20px semibold.

### Named rules
**The one face rule.** No web fonts and no display face. Monospace appears only for code, paths and the ntfy topic.

**The tabular time rule.** Every time, duration, percentage and size uses tabular-nums so it does not jitter as it updates.

**The named control rule.** In the docs, the name of a button, switch or setting is set in 600 weight wherever the text mentions it, in the same words the popover uses.

## Layout

**Popover.** A fixed 380 by 620 panel, one column. Hero, then tabs (Tonight, Storage, Log), then a scrolling main, then a footer with lid status and Quit. Outer margins are 10px; group cards pad 10px by 12px; the hero pads 14px; rows are at least 34px with hairlines between them. Group headers sit 14px above and 6px below their card, inset 4px.

**Docs.** A sticky 52px running head, then the intro, then chapters. The intro is a three-column grid at 1440: text, the 380px popover with five markers, and a 220 to 280px legend, inside a 1180px frame, 48px gaps. Under 1180px the text moves above the popover and legend; under 700px it is one column and the popover shrinks to fit. Each chapter is an opener band (44px top, 40px bottom; 32 and 28 under 700px) aligned to the 68ch column, then the column (36px top, 72px bottom, 24px gutters). Figures may break out of the column: trios at 960px, pairs at 800px, side-by-sides at 900px with a 300px screenshot. Under 860px they stack and screenshots drop to 280px. Anchors scroll with a 64px offset for the running head.

**Running head.** As the reader scrolls, the bar shows the current chapter's time and name and takes on its phase colour: dusk, night, rose, dawn (amber with night ink), day (frosted ground with a hairline). The section nav collapses to the GitHub link under 860px; the phase name hides under 700px, leaving the time.

## Elevation & Depth

Flat and tonal. Depth comes from the ground and card contrast and from hairlines. Shadows are small and functional in the popover, and in the docs they lift product screenshots and markers off the sky.

### Shadow vocabulary
- **Raised segment** (`box-shadow: 0 1px 2px rgba(0, 0, 0, 0.12)`): the selected tab and selected segment.
- **Switch knob** (`box-shadow: 0 1px 2px rgba(0, 0, 0, 0.25)`): the white knob of a switch.
- **Live dot halo** (`box-shadow: 0 0 0 3px` green at 20%): a working session's dot.
- **Moon glow** (`filter: drop-shadow(0 0 6px rgba(247, 214, 128, 0.55))`): the filled moon in the held-awake hero only.
- **Screenshot lift** (`filter: drop-shadow(0 18px 34px rgba(20, 22, 50, 0.28))`, dark 0.55 black): every popover screenshot in the docs column.
- **Hero screenshot lift** (`filter: drop-shadow(0 30px 60px rgba(0, 0, 0, 0.5))`): the annotated popover on the night sky.
- **Marker ring** (`box-shadow: 0 0 0 5px rgba(247, 214, 128, 0.22), 0 4px 12px rgba(0, 0, 0, 0.35)`): gold markers on the screenshot; the ring widens to 7px at 35% on hover.

### Named rules
**The flat card rule.** Cards, groups, callouts and flow steps carry no shadow; a hairline or a 1px inset line is enough.

## Shapes

Soft, Apple-like rounding on a small set of steps: 12px for the popover panel, 10px for the hero, groups, callouts, docs buttons, code blocks and flow steps, 8px for popover buttons, tab and segment tracks and the hero note, 6px for segments, inputs and pills, 4px for inline code and focus rings in the docs, and full circles for dots, markers and numbered headings. Borders are 1px hairlines. The crescent-and-star mark is the one drawn shape, shared by the app icon, tray icons, popover hero and docs.

## Components

### Buttons
- **Shape:** gently rounded (8px in the popover, 10px in the docs).
- **Primary (popover):** accent-fill with white 600 text, 8px by 12px, full width in the hero. Keep awake tonight is the only one. Pressed darkens to 92% brightness.
- **Quiet (popover):** white at 14% with white text, on the night hero. Let it sleep.
- **Secondary (popover):** card ground, accent-text label, hairline border. Ask Claude to clean up, Copy prompt, Copy topic name. Small variant 4px by 10px at 12px.
- **Link:** accent-text, no underline, in group headers, the hero (Hide this summary) and the footer.
- **Docs primary:** moon gold with night-a text, 11px by 20px. Docs ghost: transparent with white text and a 45% white inset line. Hover brightens to 106%.
- **Disabled:** 50% opacity.

### Hero (signature)
The top of the popover, 14px padding, 10px radius. Four states:
- **Off:** a plain card with a faint outlined moon and dark text.
- **On:** the night gradient, filled glowing moon with its star, white headline and 75% white text.
- **Standby:** the dusk gradient, moon outline in 70% gold, no star.
- **Morning:** night into dawn, filled moon at 85% gold, headline Last night, a stats line, and Hide this summary.
The background changes with a 0.3s transition, off on first draw and under reduced motion.

### Needs-you note
A full-width button under the hero headline when a session needs the user (for example, 2 sessions need you, then Show at the right). On night grounds it is white at 14% with white text; on the off hero it is orange mixed 14% into the card with orange-text. 12px, 600 weight, 7px by 10px, 8px radius. It always names what needs the user and leads to it.

### Tabs and segmented control
A gray control track with 2px padding and 6px segments; the selected segment is the card colour with the raised-segment shadow. A 6px orange dot on a tab flags it (Storage, low on space), with a screen-reader label.

### Groups and rows
White cards with a hairline and 10px radius holding rows separated by hairlines. A row with a description stacks the label over an 11px muted sublabel. Values sit right in muted gray, or in a status text colour.

### Switches and inputs
Switches are 32 by 19 with a white knob, the control track off and green on, 0.2s transitions. Inputs sit on the control fill with no border, 6px radius. Checkboxes use the system control tinted with the accent.

### Pills and status
Session status pills (working, idle, paused, stopped) are 11px semibold with 0.02em tracking on a 15 to 18% tint of their status fill, text in the status text colour. Session dots are 7px, green with a halo when working.

### Usage-limit callout
An orange-tinted card (12% into the card, 35% border) for paused sessions; red (10%, 35%) for stopped ones. Title in 600 weight; secondary text mixed toward the tone; a disclosure (Make it continue by itself next time) in the tone colour with steps and a copyable prompt.

### Focus ring
One ring for every control: 2px solid accent-text, 2px offset (1px for inputs). On any night hero it turns white. In the docs it is 2px accent with a 3px offset and 4px radius, and turns moon gold on the intro, openers and every non-day, non-dawn running head.

### Navigation (docs running head)
A sticky 52px bar: moon wordmark, the phase line (time in 600 tabular, then the chapter name), and section links at 85% opacity, full opacity and underlined on hover or when current. It changes colour with a 0.5s ease-out, none under reduced motion.

### Phase openers (docs)
Full-width bands, chapter title left and clock right, aligned to the reading column. Dusk: dusk-b to dusk-a. Night: night-a to #1b1e41. Rose: night-b to #4f3d70 to rose. Dawn: #c48468 to #d49464 with night-a type. Day: docs-day with ink type.

### Annotated popover and markers (docs)
The first viewport shows the real popover with five 28px gold numbered markers on its right edge, each linking to its section. A legend beside it repeats the numbers with a title and one line. Section headings that explain a marker carry the same number as a 26px gold disc that links back to the top.

### Screenshots (docs)
Captures of the real popover from the preview mock (src/index.html with sample data), at 2x: 760 by 1240 for 380 by 620, cropped shorter only when the state is shorter (the phone section is 760 by 1058). The rounded panel edge is part of the image; no window chrome, no device frame. Shown at 380px in the intro, 300px beside text, and in trios and pairs, always with the screenshot lift, a short caption, and alt text that reads out the state shown. The page says the screenshots show sample sessions. Menu bar icons are black template images shown inverted on dark.

### Flow diagram (docs)
Four cards in a row, 10px radius, inset hairline, joined by drawn chevrons; stacked vertically under 860px.

## Do's and Don'ts

### Do:
- **Do** keep the night, dusk, rose and dawn colours on surfaces that tell the time or state of the night.
- **Do** use green-text, orange-text and red-text for any status text, and the bright fills only for shapes and tints.
- **Do** use accent-fill behind white text and accent-text for links, labels and the focus ring.
- **Do** keep the two dark blocks in src/styles.css identical; change both in one edit.
- **Do** keep one filled accent per popover screen, the main button.
- **Do** use tabular figures for times, durations, sizes and percentages.
- **Do** show the real popover, captured from the preview mock at 2x, and say the data is sample data.
- **Do** keep a visible focus ring on every control, switching to white or moon gold on night grounds.
- **Do** turn off transitions under prefers-reduced-motion and on the popover's first draw.

### Don't:
- **Don't** add a second typeface or a web font; monospace is for code only.
- **Don't** put the night gradient behind content cards, callouts or tables.
- **Don't** fill a selected tab or segment with the accent; raise it on the card colour.
- **Don't** use the system orange, green or red fills as small text on light grounds.
- **Don't** show invented users, numbers or press, or mock up the popover instead of capturing it.
- **Don't** add shadows to cards and groups; use a hairline.
