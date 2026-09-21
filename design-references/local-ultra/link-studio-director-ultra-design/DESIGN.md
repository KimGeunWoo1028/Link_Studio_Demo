# Link Studio Director Ultra DESIGN.md

> Auto-generated design system — reverse-engineered via static analysis by skillui.
> Frameworks: None detected
> Colors: 11 · Fonts: 1 · Components: 0
> Icon library: not detected · State: not detected
> Primary theme: dark · Dark mode toggle: no · Motion: none

---

## 1. Visual Theme & Atmosphere

This is a **dark-themed** interface with a flat, neutral visual language. Elevation is achieved through color and border shifts rather than shadows — a clean, industrial aesthetic. Typography uses **Segoe UI Variable** throughout — a clean, modern choice that maintains consistency. Spacing follows a **4px base grid** (compact density), with scale: 2, 4, 6, 8, 10, 12, 16, 20px. The palette is predominantly monochromatic with **#b7c3cf** as the single accent color — used sparingly for interactive elements and emphasis.

---

## 2. Color Palette & Roles

| Token | Hex | Role | Use |
|---|---|---|---|
| background | `#2a3540` | background | Page background, darkest surface |
| surface | `#1f2932` | surface | Card and panel backgrounds |
| text-primary | `#eef2f6` | text-primary | Headings and body text |
| text-muted | `#8a97a6` | text-muted | Captions, placeholders, secondary info |
| accent | `#b7c3cf` | accent | CTAs, links, focus rings, active states |
| info | `#004490` | info | Informational highlights |
| unknown | `#121820` | unknown | Palette color |
| unknown | `#6b7784` | unknown | Palette color |
| unknown | `#ffffff` | unknown | Palette color |
| unknown | `#07090c` | unknown | Palette color |
| unknown | `#000000` | unknown | Palette color |

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
- **Segoe UI Variable** — Heading 1, Heading 2, Heading 3, Body, Caption

| Role | Font | Size | Weight |
|---|---|---|---|
| Heading 1 | Segoe UI Variable | 48px / 3rem | 700 |
| Heading 2 | Segoe UI Variable | 32px / 2rem | 600 |
| Heading 3 | Segoe UI Variable | 24px / 1.5rem | 600 |
| Body | Segoe UI Variable | 16px / 1rem | 400 |
| Caption | Segoe UI Variable | 12px / 0.75rem | 400 |

**Typographic Rules:**
- Use **Segoe UI Variable** for all text — do not mix font families
- Maintain consistent hierarchy: no more than 3-4 font sizes per screen
- Headings use bold (600-700), body uses regular (400)
- Line height: 1.5 for body text, 1.2 for headings
- Use color and opacity for secondary hierarchy, not additional font sizes


---

## 4. Component Stylings

No components detected. Scan `src/components/` or `components/` to populate this section.

---

## 5. Layout Principles

- **Base spacing unit:** 4px
- **Spacing scale:** 2, 4, 6, 8, 10, 12, 16, 20, 24
- **Border radius:** 4px, 8px, 999px

**Spacing as Meaning:**
| Spacing | Use |
|---|---|
| 4-8px | Tight: related items within a group |
| 12-16px | Medium: between groups |
| 24-32px | Wide: between sections |
| 48px+ | Vast: major section breaks |


---

## 6. Depth & Elevation

No box-shadow values detected. The design uses a **flat visual style** — elevation is conveyed through background color shifts and borders rather than shadows.

**Elevation Strategy:**
| Level | Technique | Use |
|---|---|---|
| 0 — Base | Background color | Page background |
| 1 — Raised | Lighter surface + subtle border | Cards, panels |
| 2 — Floating | Even lighter surface + stronger border | Dropdowns, popovers |
| 3 — Overlay | Backdrop + modal surface | Modals, dialogs |


---

## 8. Do's and Don'ts

### Do's

- Use `#b7c3cf` for interactive elements (buttons, links, focus rings)
- Use `#2a3540` as the primary page background
- Use **Segoe UI Variable** for all UI text
- Follow the **4px** spacing grid for all margins, padding, and gaps
- Use border and background shifts for elevation — not shadows
- Use border-radius from the scale: 4px, 8px, 999px

### Don'ts

- Don't introduce colors outside this palette — extend the design tokens first
- Don't mix font families — use Segoe UI Variable consistently
- Don't use arbitrary spacing values — stick to multiples of 4px
- Don't add box-shadow — this design system uses flat elevation
- Don't use gradients — the design uses solid colors only
- Don't use arbitrary border-radius values — pick from the defined scale
- Don't use backdrop-blur or blur effects

### Anti-Patterns (detected from codebase)

- No box-shadow on any element
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
Background: #1f2932
Border: 1px solid var(--border)
Radius: 8px
Padding: 16px
Font: Segoe UI Variable
No shadows — use borders and surface colors for depth.
```

### Build a Button

```
Primary: bg #b7c3cf, text white
Ghost: bg transparent, border var(--border)
Padding: 8px 16px
Radius: 8px
Hover: opacity 0.9 or lighter shade
Focus: ring with #b7c3cf
```

### Build a Page Layout

```
Background: #2a3540
Max-width: 1280px, centered
Grid: 4px base
Responsive: mobile-first, breakpoints from Section 9
```

### Build a Stats Card

```
Surface: #1f2932
Label: #8a97a6 (muted, 12px, uppercase)
Value: #eef2f6 (primary, 24-32px, bold)
Status: use success/warning/danger from Section 2
```

### Build a Form

```
Input bg: #2a3540
Input border: 1px solid var(--border)
Focus: border-color #b7c3cf
Label: #8a97a6 12px
Spacing: 16px between fields
Radius: 8px
```

### General Component

```
1. Read DESIGN.md Sections 2-6 for tokens
2. Colors: only from palette
3. Font: Segoe UI Variable, type scale from Section 3
4. Spacing: 4px grid
5. Components: match patterns from Section 4
6. Elevation: flat, surface shifts
```
