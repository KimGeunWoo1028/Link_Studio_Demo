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
