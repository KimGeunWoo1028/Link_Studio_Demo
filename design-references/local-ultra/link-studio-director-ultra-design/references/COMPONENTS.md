# Component Reference

> Repeated DOM patterns detected by structural analysis. Each component appeared 3+ times.

## Detected Components

| Component | Category | Instances | Key Classes |
|-----------|----------|-----------|-------------|
| **Status Chip** | badge | 5× | `.status-chip`, `.status-chip--offline` |
| **Mono** | unknown | 4× | `.mono` |
| **Btn** | button | 3× | `.btn`, `.btn-ghost` |
| **Status Unit** | unknown | 3× | `.status-unit` |

## Buttons

### Btn

**Instances found:** 3

**CSS classes:** `.btn` `.btn-ghost`

**HTML structure:**

```html
<button type="submit" class="btn btn-ghost">Create</button>
```

**Base styles (from design tokens):**

```css
.btn {
  background: #b7c3cf;
  color: #eef2f6;
  border-radius: 8px;
  padding: 4px 8px;
  cursor: pointer;
}```

## Badges & Chips

### Status Chip

**Instances found:** 5

**CSS classes:** `.status-chip` `.status-chip--offline`

**HTML structure:**

```html
<span class="status-chip status-chip--offline"><span class="status-chip__dot" aria-hidden="true"></span>Idle</span>
```

**Base styles (from design tokens):**

```css
.status-chip {
  background: #1f2932;
  border-radius: 8px;
  padding: 2px 4px;
  font-size: 12px;
}```

## Other Components

### Mono

**Instances found:** 4

**CSS classes:** `.mono`

**HTML structure:**

```html
<code class="mono" data-testid="camera-url">Start a session to generate a URL</code>
```

**Base styles (from design tokens):**

```css
.mono {
  background: #1f2932;
  padding: 4px;
}```

### Status Unit

**Instances found:** 3

**CSS classes:** `.status-unit`

**HTML structure:**

```html
<div class="status-unit"><span class="k">Session</span><span class="v"><span class="status-chip status-chip--offline"><span class="status-chip__dot" aria-hidden="true"></span>Idle</span></span></div>
```

**Base styles (from design tokens):**

```css
.status-unit {
  background: #1f2932;
  padding: 4px;
}```

## Component Rules

- Match class names exactly from the patterns above
- Each component instance must be visually identical to others of its type
- Do not add extra wrappers or change the DOM structure
- Use `#b7c3cf` for all interactive/active states

