# Interaction Reference

> Micro-interactions extracted from live DOM. Recreate these exactly for authentic feel.

## Coverage

| Component Type | Count | States Captured |
|----------------|-------|----------------|
| Button | 3 | default, hover, focus |
| Input | 1 | default, hover, focus |

## Transition System

These transition declarations were extracted from interactive elements:

```css
transition: background 0.16s cubic-bezier(0.2, 0.8, 0.2, 1), border-color 0.16s cubic-bezier(0.2, 0.8, 0.2, 1), opacity 0.16s cubic-bezier(0.2, 0.8, 0.2, 1);
transition: all;
```

Apply these to all interactive elements. Never invent new durations or easings.

## Button Interactions

### Button 1 — `Create`

**States:**

- Default: `../screens/states/button-1-default.png`
- Hover: `../screens/states/button-1-hover.png`
- Focus: `../screens/states/button-1-focus.png`

**On hover:**

```css
/* background-color: rgba(0, 0, 0, 0) → */ background-color: rgb(31, 41, 50);
```

**On focus:**

```css
/* outline: rgb(238, 242, 246) none 3px → */ outline: rgb(94, 184, 255) solid 2px;
/* outline-color: rgb(238, 242, 246) → */ outline-color: rgb(94, 184, 255);
```

**Transition:** `background 0.16s cubic-bezier(0.2, 0.8, 0.2, 1), border-color 0.16s cubic-bezier(0.2, 0.8, 0.2, 1), opacity 0.16s cubic-bezier(0.2, 0.8, 0.2, 1)`

### Button 2 — `Start camera session`

**States:**

- Default: `../screens/states/button-2-default.png`
- Hover: `../screens/states/button-2-hover.png`
- Focus: `../screens/states/button-2-focus.png`

**On hover:**

```css
/* background-color: rgb(0, 68, 144) → */ background-color: rgb(0, 51, 109);
```

**On focus:**

```css
/* outline: rgb(255, 255, 255) none 3px → */ outline: rgb(94, 184, 255) solid 2px;
/* outline-color: rgb(255, 255, 255) → */ outline-color: rgb(94, 184, 255);
```

**Transition:** `background 0.16s cubic-bezier(0.2, 0.8, 0.2, 1), border-color 0.16s cubic-bezier(0.2, 0.8, 0.2, 1), opacity 0.16s cubic-bezier(0.2, 0.8, 0.2, 1)`

### Button 3 — `+ Add Camera`

**States:**

- Default: `../screens/states/button-3-default.png`
- Hover: `../screens/states/button-3-hover.png`
- Focus: `../screens/states/button-3-focus.png`

**On hover:**

```css
/* background-color: rgba(0, 0, 0, 0) → */ background-color: rgb(31, 41, 50);
```

**On focus:**

```css
/* outline: rgb(238, 242, 246) none 3px → */ outline: rgb(94, 184, 255) solid 2px;
/* outline-color: rgb(238, 242, 246) → */ outline-color: rgb(94, 184, 255);
```

**Transition:** `background 0.16s cubic-bezier(0.2, 0.8, 0.2, 1), border-color 0.16s cubic-bezier(0.2, 0.8, 0.2, 1), opacity 0.16s cubic-bezier(0.2, 0.8, 0.2, 1)`

## Input Interactions

### Input 1 — `Project name`

**States:**

- Default: `../screens/states/input-1-default.png`
- Hover: `../screens/states/input-1-hover.png`
- Focus: `../screens/states/input-1-focus.png`

**On focus:**

```css
/* outline: rgb(238, 242, 246) none 3px → */ outline: rgb(94, 184, 255) solid 2px;
/* outline-color: rgb(238, 242, 246) → */ outline-color: rgb(94, 184, 255);
```

**Transition:** `all`

## Interaction Rules

- Accent color `#b7c3cf` is used for focus rings, active states, and hover highlights
- Hover effects include **color transitions** — use the extracted values, not approximations
- Focus states use **outline** (not box-shadow) — always match the extracted focus ring
- Transition durations in use: `0.16s`
- Always respect `prefers-reduced-motion` — set all transitions to `0s` when enabled

