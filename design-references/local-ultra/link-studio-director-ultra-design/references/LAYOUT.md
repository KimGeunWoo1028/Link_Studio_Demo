# Layout Reference

> Auto-extracted from live DOM. Use this to understand how the site is structured spatially.

## Spacing System

**Base grid:** 4px

**Scale:** `2, 4, 6, 8, 10, 12, 16, 20, 24` px

| Spacing | Semantic Use |
|---------|-------------|
| 4px | Tight — within a component |
| 8px | Medium — between sibling items |
| 16px | Wide — between sections |
| 32px | Vast — major section breaks |

## Flex Layouts

| Element | Direction | Justify | Align | Gap | Children |
|---------|-----------|---------|-------|-----|----------|
| `div.chrome-row` | row | — | center | 12px | 3 |
| `div.url-row` | row | — | start | 8px | 2 |

## Grid Layouts

| Element | Template Columns | Gap | Children |
|---------|-----------------|-----|----------|
| `header.chrome` | `1416px` | 4px | 1 |
| `section#camera-connection.connection-stage` | `1392px` | 16px | 4 |

## Structural Containers

### `<header>` (`header.chrome`)

```
display:          grid
grid-template-columns: 1416px
gap:              4px
padding:          4px 12px
children:         1
```

### `<section>` (`section#camera-connection.connection-stage`)

```
display:          grid
grid-template-columns: 1392px
gap:              16px
padding:          20px 24px 24px
children:         4
```

## Layout Rules

- Primary layout system: **Flexbox**
- Secondary layout system: **CSS Grid** (used for card grids and multi-column layouts)
- Every spacing value must be a multiple of **4px**
- Never use arbitrary margin/padding values outside the spacing scale

