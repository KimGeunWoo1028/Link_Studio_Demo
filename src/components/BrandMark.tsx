export function BrandMark({ size = 28 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 32 32"
      aria-hidden="true"
      focusable="false"
      className="brand-mark"
    >
      <path fill="var(--blue-navy)" d="M4 5h16.2L13.4 16.5H4V5Z" />
      <path fill="var(--blue-accent)" d="M16.2 16.2H28L18.4 27H4v-5.2h9.2L16.2 16.2Z" />
    </svg>
  );
}

export function IconWifi() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M5 12.5a9 9 0 0 1 14 0" />
      <path d="M8.5 16a5 5 0 0 1 7 0" />
      <circle cx="12" cy="19.5" r="1.2" fill="currentColor" stroke="none" />
    </svg>
  );
}

export function IconEthernet() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" strokeWidth="2">
      <rect x="8" y="3" width="8" height="7" rx="1" />
      <path d="M10 10v3H7v4h3v3h4v-3h3v-4h-3v-3" />
    </svg>
  );
}

export function IconCopy() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" strokeWidth="2">
      <rect x="9" y="9" width="11" height="11" rx="1.5" />
      <path d="M5 15V5h10" />
    </svg>
  );
}

export function IconChevron({ dir }: { dir: "left" | "right" | "up" | "down" }) {
  const rotate = { left: 90, right: -90, up: 180, down: 0 }[dir];
  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      aria-hidden="true"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      style={{ transform: `rotate(${rotate}deg)` }}
    >
      <path d="M6 9l6 6 6-6" />
    </svg>
  );
}
