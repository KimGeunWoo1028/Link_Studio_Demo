# Link Studio Design System

Brand source: Link Studio 1차/2차 발표 PDF (slides + director mockups). Not a copy of slide layout.

## Principles

- Professional live-production software, low learning curve
- Brand navy and the geometric L/play mark, not orange SaaS chrome
- Director is a dark monitor workspace; Camera is a light, outdoor-readable phone surface
- Status always has a label, never color alone
- Motion 150–160ms, no decorative animation

## Color

| Token | Hex | Use |
|---|---|---|
| `--blue-navy` | `#004490` | Logo, primary actions (extracted from PDF header mark) |
| `--blue-mid` | `#4478b4` | Secondary / chevron mid |
| `--blue-accent` | `#1E9DFE` | Logo highlight, focus on dark |
| `--paper` | `#F7F7F7` | Camera page / presentation ground |
| `--bg` | `#121820` | Director canvas (from PDF director mock `#161d25`–`#1f2932`) |
| `--on-air` | `#D61F2C` | LIVE / PGM tally (from ON AIR mock badges) |

Semantic aliases live in `src/styles/tokens.css`.

## Typography

Segoe UI / Malgun Gothic. Hierarchy by size, weight, and muted color — one family.

## Components

Button, StatusChip, Panel, Field, Banner, EmptyState in `src/components/ui.tsx`.
Logo mark in `src/components/BrandMark.tsx` and `public/logo.svg`.
