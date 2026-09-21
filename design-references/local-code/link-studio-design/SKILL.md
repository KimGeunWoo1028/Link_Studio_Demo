---
name: Link Studio-design
description: Design system skill for Link Studio. Activate when building UI components, pages, or any visual elements. Provides exact color tokens, typography scale, spacing grid, component patterns, and craft rules. Read references/DESIGN.md before writing any CSS or JSX.
---

# Link Studio Design System

You are building UI for **Link Studio**. Light-themed, cool palette, sans-serif typography (sans-serif), compact density on a 4px grid.

## Design Philosophy

- **Layered depth** — use shadow tokens to create a sense of physical layering. Each elevation level has a specific shadow.
- **Solid colors only** — no gradients anywhere. Every surface is a single flat color.
- **Single typeface** — sans-serif carries all text. Hierarchy comes from size, weight, and color — never font mixing.
- **compact density** — 4px base grid. Every dimension is a multiple of 4.
- **cool palette** — the color temperature runs cool, matching the sans-serif typography.
- **Restrained accent** — `#1e9dfe` is the only pop of color. Used exclusively for CTAs, links, focus rings, and active states.
- **Subtle motion** — transitions smooth state changes. Keep durations under 300ms, use ease-out curves.

## Color System

### Core Palette

| Role | Token | Hex | Use |
|------|-------|-----|-----|
| Background | `--background` | `#d7e8ff` | Page/app background |
| Surface | `--surface` | `#ffffff` | Cards, panels, modals |
| Text Primary | `--text-primary` | `#000000` | Headings, body text |
| Text Muted | `--text-muted` | `#b7c3cf` | Captions, placeholders |
| Accent | `--accent` | `#1e9dfe` | CTAs, links, focus rings |

### Status Colors

| Status | Hex | Use |
|--------|-----|-----|
| Success | `#c8f0d8` | Confirmations, positive trends |
| Warning | `#f3ddb5` | Caution states, pending items |
| Danger | `#ffb4b4` | Errors, destructive actions |

### Extended Palette

- `#0a0d11` — Deep background layer or shadow color
- **border:** `#2a3540`
- **muted:** `#5d6b7a` — Secondary text, placeholder text

### CSS Variable Tokens

```css
--blue-accent: #1e9dfe;
--paper-card: #ffffff;
--border: #2a3540;
--text-secondary: #b7c3cf;
--muted: #8a97a6;
--primary: var(--blue-navy);
--secondary: var(--blue-mid);
--accent: var(--blue-accent);
```

## Typography

### Font Stack

- **sans-serif** — Heading 1, Heading 2, Heading 3, Body, Caption

### Type Scale

| Role | Family | Size | Weight |
|------|--------|------|--------|
| Heading 1 | sans-serif | 48px / 3rem | 700 |
| Heading 2 | sans-serif | 32px / 2rem | 600 |
| Heading 3 | sans-serif | 24px / 1.5rem | 600 |
| Body | sans-serif | 16px / 1rem | 400 |
| Caption | sans-serif | 12px / 0.75rem | 400 |

### Typography Rules

- All text uses **sans-serif** — never add another font family
- Max 3-4 font sizes per screen
- Headings: weight 600-700, body: weight 400
- Use color and opacity for text hierarchy, not additional font sizes
- Line height: 1.5 for body, 1.2 for headings

## Spacing & Layout

### Base Grid: 4px

Every dimension (margin, padding, gap, width, height) must be a multiple of **4px**.

### Spacing Scale

`2, 4, 6, 8, 10, 12, 14, 16, 20, 24, 40, 140` px

### Spacing as Meaning

| Spacing | Use |
|---------|-----|
| 4-8px | Tight: related items (icon + label, avatar + name) |
| 12-16px | Medium: between groups within a section |
| 24-32px | Wide: between distinct sections |
| 48px+ | Vast: major page section breaks |

### Border Radius

Scale: `999px`
Default: `999px`

### Container

Max-width: `1100px`, centered with auto margins.

## Component Patterns

### Card

```css
.card {
  background: #ffffff;
  border-radius: 999px;
  padding: 16px;
  box-shadow: inset 0 0 0 1px color-mix(in srgb,var(--on-air) 50%,transparent);
}
```

```html
<div class="card">
  <h3>Card Title</h3>
  <p>Card content goes here.</p>
</div>
```

### Button

```css
/* Primary */
.btn-primary {
  background: #1e9dfe;
  color: #000000;
  border-radius: 999px;
  padding: 8px 16px;
  font-weight: 500;
  transition: opacity 150ms ease;
}
.btn-primary:hover { opacity: 0.9; }

/* Ghost */
.btn-ghost {
  background: transparent;
  border: 1px solid #cccccc;
  color: #000000;
  border-radius: 999px;
  padding: 8px 16px;
}
```

```html
<button class="btn-primary">Get Started</button>
<button class="btn-ghost">Learn More</button>
```

### Input

```css
.input {
  background: #d7e8ff;
  border: 1px solid #cccccc;
  border-radius: 999px;
  padding: 8px 12px;
  color: #000000;
  font-size: 14px;
}
.input:focus { border-color: #1e9dfe; outline: none; }
```

```html
<input class="input" type="text" placeholder="Search..." />
```

### Badge / Chip

```css
.badge {
  display: inline-flex;
  align-items: center;
  padding: 4px 8px;
  border-radius: 9999px;
  font-size: 12px;
  font-weight: 500;
  background: #ffffff;
  color: #b7c3cf;
}
```

```html
<span class="badge">New</span>
<span class="badge">Beta</span>
```

### Modal / Dialog

```css
.modal-backdrop { background: rgba(0, 0, 0, 0.6); }
.modal {
  background: #ffffff;
  border-radius: 999px;
  padding: 24px;
  max-width: 480px;
  width: 90vw;
  box-shadow: inset 0 0 0 1px color-mix(in srgb,var(--on-air) 50%,transparent);
}
```

```html
<div class="modal-backdrop">
  <div class="modal">
    <h2>Dialog Title</h2>
    <p>Dialog content.</p>
    <button class="btn-primary">Confirm</button>
    <button class="btn-ghost">Cancel</button>
  </div>
</div>
```

### Table

```css
.table { width: 100%; border-collapse: collapse; }
.table th {
  text-align: left;
  padding: 8px 12px;
  font-weight: 500;
  font-size: 12px;
  color: #b7c3cf;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  border-bottom: 1px solid #cccccc;
}
.table td {
  padding: 12px;
  border-bottom: 1px solid #cccccc;
}
```

```html
<table class="table">
  <thead><tr><th>Name</th><th>Status</th><th>Date</th></tr></thead>
  <tbody>
    <tr><td>Item One</td><td>Active</td><td>Jan 1</td></tr>
    <tr><td>Item Two</td><td>Pending</td><td>Jan 2</td></tr>
  </tbody>
</table>
```

### Navigation

```css
.nav {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
}
.nav-link {
  color: #b7c3cf;
  padding: 8px 12px;
  border-radius: 999px;
  transition: color 150ms;
}
.nav-link:hover { color: #000000; }
.nav-link.active { color: #1e9dfe; }
```

```html
<nav class="nav">
  <a href="/" class="nav-link active">Home</a>
  <a href="/about" class="nav-link">About</a>
  <a href="/pricing" class="nav-link">Pricing</a>
  <button class="btn-primary" style="margin-left: auto">Get Started</button>
</nav>
```

### Extracted Components

These components were found in the codebase:

**BrandMark** (`components/BrandMark.tsx`)
- Variants: `left`, `right`, `up`, `down`

**CameraPreview** (`components/CameraPreview.tsx`)
- Props: `stream`, `slotLabel`, `label`, `live`, `selected`, `connecting`

**ProgramStage** (`components/ProgramStage.tsx`)
- Props: `streams`, `pgm`

**Ui** (`components/ui.tsx`)
- Variants: `live`, `ready`, `offline`, `connecting`, `warning`, `error`, `primary`, `ghost`, `take`, `show`, `neutral`, `hide`, `info`

**CameraPage** (`pages/CameraPage.tsx`)
- Variants: `loading`, `unsupported`, `insecure`, `ready`, `localhost`, `connecting`, `offline`, `NotAllowedError`, `NotFoundError`, `warning`
- Props: `type`, `sdp`

**DirectorPage** (`pages/DirectorPage.tsx`)
- Variants: `ready`, `connecting`, `offline`, `warning`
- Props: `pip_enabled`

## Animation & Motion

This project uses **subtle motion**. Transitions smooth state changes without calling attention.

### Motion Tokens

- **Easing functions:** `ease`
- **Animated properties:** `background`, `border-color`, `opacity`

### Motion Guidelines

- **Duration:** 150-300ms for micro-interactions, 300-500ms for page transitions
- **Easing:** Use `ease` as the default easing curve
- **Direction:** Elements enter from bottom/right, exit to top/left
- **Reduced motion:** Always respect `prefers-reduced-motion` — disable animations when set

## Depth & Elevation

### Shadow Tokens

- Subtle: `inset 0 0 0 1px color-mix(in srgb,var(--on-air) 50%,transparent)`

## Anti-Patterns (Never Do)

- **No gradients** — solid colors only, everywhere
- **No blur effects** — no backdrop-blur, no filter: blur()
- **No zebra striping** — tables and lists use borders for separation
- **No invented colors** — every hex value must come from the palette above
- **No arbitrary spacing** — every dimension is a multiple of 4px
- **No extra fonts** — only sans-serif are allowed
- **No arbitrary border-radius** — use the scale: 999px
- **No opacity for disabled states** — use muted colors instead

## Workflow

1. **Read** `references/DESIGN.md` before writing any UI code
2. **Pick colors** from the Color System section — never invent new ones
3. **Set typography** — sans-serif only, using the type scale
4. **Build layout** on the 4px grid — check every margin, padding, gap
5. **Match components** to patterns above before creating new ones
6. **Apply elevation** — use shadow tokens
7. **Validate** — every value traces back to a design token. No magic numbers.

## Brand Spec

- **Brand color:** `#1e9dfe`
- **Brand typeface:** sans-serif

## Quick Reference

```
Background:     #d7e8ff
Surface:        #ffffff
Text:           #000000 / #b7c3cf
Accent:         #1e9dfe
Border:         (not extracted)
Font:           sans-serif
Spacing:        4px grid
Radius:         999px
Components:     9 detected
```

## When to Trigger

Activate this skill when:
- Creating new components, pages, or visual elements for Link Studio
- Writing CSS, Tailwind classes, styled-components, or inline styles
- Building page layouts, templates, or responsive designs
- Reviewing UI code for design consistency
- The user mentions "Link Studio" design, style, UI, or theme
- Generating mockups, wireframes, or visual prototypes

---

# Full Reference Files

> Every output file is embedded below. Claude has full design system context from /skills alone.

## Design System Tokens (DESIGN.md)

# Link Studio DESIGN.md

> Auto-generated design system — reverse-engineered via static analysis by skillui.
> Frameworks: None detected
> Colors: 14 · Fonts: 1 · Components: 9
> Icon library: not detected · State: not detected
> Primary theme: light · Dark mode toggle: no · Motion: subtle

---

## 1. Visual Theme & Atmosphere

This is a **light-themed** interface with a cool, approachable feel. The light background emphasizes content clarity. Typography uses **sans-serif** throughout — a clean, modern choice that maintains consistency. Spacing follows a **4px base grid** (compact density), with scale: 2, 4, 6, 8, 10, 12, 14, 16px. The accent color **#1e9dfe** anchors interactive elements (buttons, links, focus rings). Motion is subtle — smooth transitions (150-300ms) ease state changes without drawing attention.

---

## 2. Color Palette & Roles

| Token | Hex | Role | Use |
|---|---|---|---|
| background | `#d7e8ff` | background | Page background, darkest surface |
| paper-card | `#ffffff` | surface | Card and panel backgrounds |
| surface | `#ffd4d6` | surface | Card and panel backgrounds |
| text-primary | `#000000` | text-primary | Headings and body text |
| text-secondary | `#b7c3cf` | text-muted | Captions, placeholders, secondary info |
| muted | `#8a97a6` | text-muted | Captions, placeholders, secondary info |
| text-secondary | `#3d4d5e` | text-muted | Captions, placeholders, secondary info |
| blue-accent | `#1e9dfe` | accent | CTAs, links, focus rings, active states |
| danger | `#ffb4b4` | danger | Error states, destructive actions |
| success | `#c8f0d8` | success | Success states, positive indicators |
| warning | `#f3ddb5` | warning | Warning states, caution indicators |
| unknown | `#0a0d11` | unknown | Palette color |
| border | `#2a3540` | unknown | Palette color |
| muted | `#5d6b7a` | unknown | Palette color |

### CSS Variable Tokens

```css
--blue-accent: #1e9dfe;
--paper-card: #ffffff;
--border: #2a3540;
--text-secondary: #b7c3cf;
--muted: #8a97a6;
--primary: var(--blue-navy);
--secondary: var(--blue-mid);
--accent: var(--blue-accent);
```


---

## 3. Typography Rules

**Font Stack:**
- **sans-serif** — Heading 1, Heading 2, Heading 3, Body, Caption

| Role | Font | Size | Weight |
|---|---|---|---|
| Heading 1 | sans-serif | 48px / 3rem | 700 |
| Heading 2 | sans-serif | 32px / 2rem | 600 |
| Heading 3 | sans-serif | 24px / 1.5rem | 600 |
| Body | sans-serif | 16px / 1rem | 400 |
| Caption | sans-serif | 12px / 0.75rem | 400 |

**Typographic Rules:**
- Use **sans-serif** for all text — do not mix font families
- Maintain consistent hierarchy: no more than 3-4 font sizes per screen
- Headings use bold (600-700), body uses regular (400)
- Line height: 1.5 for body text, 1.2 for headings
- Use color and opacity for secondary hierarchy, not additional font sizes


---

## 4. Component Stylings

### Navigation (2)

**CameraPage.test** — `pages/CameraPage.test.tsx`
- Props: `error`

**CameraPage** — `pages/CameraPage.tsx`
- Variants: `loading`, `unsupported`, `insecure`, `ready`, `localhost`, `connecting`, `offline`, `NotAllowedError`, `NotFoundError`, `warning`
- Props: `type`, `sdp`
- State: useState, useRef

### Data Input (1)

**DirectorPage** — `pages/DirectorPage.tsx`
- Variants: `ready`, `connecting`, `offline`, `warning`
- Props: `pip_enabled`
- State: useState, useRef

### Feedback (1)

**ErrorBoundary** — `components/ErrorBoundary.tsx`
- Props: `children`

```tsx
<main className="camera-page" data-testid="camera-state" data-state="init-error">
          <header>
            <BrandMark size={32} />
            <h1>Unexpected initialization error</h1>
          </header>
          <Banner>{this.state.error.message}</Banner>
        </main>
```

### Other (5)

**BrandMark** — `components/BrandMark.tsx`
- Variants: `left`, `right`, `up`, `down`

```tsx
<svg
      width={size}
      height={size}
      viewBox="0 0 32 32"
      aria-hidden="true"
      focusable="false"
      className="brand-mark"
    >
      <path fill="var(--blue-navy
```

**CameraPreview** — `components/CameraPreview.tsx`
- Props: `stream`, `slotLabel`, `label`, `live`, `selected`, `connecting`, `onTake`, `onRename` (+2 more)
- State: useState, useRef

```tsx
<article className={`preview-card ${selected ? "is-pgm" : ""}`}>
      <header>
        <span className="cam-id">{slotLabel}</span>
        {onRename ? (
          <input
            className="cam-name"
            aria-label={`${slotLabel} name`}
            value={name}
            onChange={(event
```

**ProgramStage** — `components/ProgramStage.tsx`
- Props: `streams`, `pgm`
- State: useRef

```tsx
<div className="pgm-stage" data-testid="pgm-stage">
      {main ? (
        <video ref={mainRef} className="pgm-main" autoPlay playsInline muted />
```

**Ui** — `components/ui.tsx`
- Variants: `live`, `ready`, `offline`, `connecting`, `warning`, `error`, `primary`, `ghost`, `take`, `show`, `neutral`, `hide`, `info`

```tsx
<span className={`status-chip status-chip--${tone}`}>
      <span className="status-chip__dot" aria-hidden="true" />
      {children}
    </span>
```

**ProgramPage** — `pages/ProgramPage.tsx`

```tsx
<main className="program-page">
        <h1>Invalid program URL</h1>
      </main>
```



---

## 5. Layout Principles

- **Base spacing unit:** 4px
- **Spacing scale:** 2, 4, 6, 8, 10, 12, 14, 16, 20, 24, 40, 140
- **Border radius:** 999px
- **Max content width:** 1100px

**Spacing as Meaning:**
| Spacing | Use |
|---|---|
| 4-8px | Tight: related items within a group |
| 12-16px | Medium: between groups |
| 24-32px | Wide: between sections |
| 48px+ | Vast: major section breaks |


---

## 6. Depth & Elevation

### Flat — subtle depth hints

- `inset 0 0 0 1px color-mix(in srgb,var(--on-air) 50%,transparent)`



---

## 7. Animation & Motion

This project uses **subtle motion**. Transitions smooth state changes without demanding attention.

### Motion Guidelines

- Duration: 150-300ms for micro-interactions, 300-500ms for page transitions
- Easing: `ease-out` for enters, `ease-in` for exits
- Always respect `prefers-reduced-motion`


---

## 8. Do's and Don'ts

### Do's

- Use `#1e9dfe` for interactive elements (buttons, links, focus rings)
- Use `#d7e8ff` as the primary page background
- Use **sans-serif** for all UI text
- Follow the **4px** spacing grid for all margins, padding, and gaps
- Use the defined shadow tokens for elevation — see Section 6
- Use border-radius from the scale: 999px
- Reuse existing components from Section 4 before creating new ones

### Don'ts

- Don't introduce colors outside this palette — extend the design tokens first
- Don't mix font families — use sans-serif consistently
- Don't use arbitrary spacing values — stick to multiples of 4px
- Don't create custom box-shadow values outside the system tokens
- Don't use gradients — the design uses solid colors only
- Don't use arbitrary border-radius values — pick from the defined scale
- Don't duplicate component patterns — check Section 4 first
- Don't use backdrop-blur or blur effects

### Anti-Patterns (detected from codebase)

- No gradient backgrounds
- No blur or backdrop-blur effects
- No zebra striping on tables/lists


---

## 9. Responsive Behavior

No breakpoints detected. Consider adding responsive breakpoints to the design system.

---

## 10. Agent Prompt Guide

Use these as starting points when building new UI:

### Build a Card

```
Background: #ffffff
Border: 1px solid var(--border)
Radius: 999px
Padding: 16px
Font: sans-serif
Use shadow tokens from Section 6.
```

### Build a Button

```
Primary: bg #1e9dfe, text white
Ghost: bg transparent, border var(--border)
Padding: 8px 16px
Radius: 999px
Hover: opacity 0.9 or lighter shade
Focus: ring with #1e9dfe
```

### Build a Page Layout

```
Background: #d7e8ff
Max-width: 1100px, centered
Grid: 4px base
Responsive: mobile-first, breakpoints from Section 9
```

### Build a Stats Card

```
Surface: #ffffff
Label: #b7c3cf (muted, 12px, uppercase)
Value: #000000 (primary, 24-32px, bold)
Status: use success/warning/danger from Section 2
```

### Build a Form

```
Input bg: #d7e8ff
Input border: 1px solid var(--border)
Focus: border-color #1e9dfe
Label: #b7c3cf 12px
Spacing: 16px between fields
Radius: 999px
```

### General Component

```
1. Read DESIGN.md Sections 2-6 for tokens
2. Colors: only from palette
3. Font: sans-serif, type scale from Section 3
4. Spacing: 4px grid
5. Components: match patterns from Section 4
6. Elevation: shadow tokens
```

