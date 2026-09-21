import type { ButtonHTMLAttributes, HTMLAttributes, LabelHTMLAttributes, ReactNode } from "react";

type Tone = "live" | "ready" | "offline" | "connecting" | "warning" | "error" | "neutral";

export function Button({
  variant = "primary",
  className = "",
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: "primary" | "ghost" | "take" | "show" | "hide";
}) {
  return <button type="button" className={`btn btn-${variant} ${className}`.trim()} {...props} />;
}

export function StatusChip({ tone, children }: { tone: Tone; children: ReactNode }) {
  return (
    <span className={`status-chip status-chip--${tone}`}>
      <span className="status-chip__dot" aria-hidden="true" />
      {children}
    </span>
  );
}

export function Panel({
  title,
  children,
  className = "",
  actions,
}: {
  title?: ReactNode;
  children: ReactNode;
  className?: string;
  actions?: ReactNode;
}) {
  return (
    <section className={`panel ${className}`.trim()}>
      {title ? (
        <header className="panel__head">
          <h2>{title}</h2>
          {actions}
        </header>
      ) : null}
      {children}
    </section>
  );
}

export function Field({
  label,
  children,
  className = "",
  ...props
}: LabelHTMLAttributes<HTMLLabelElement> & { label: string }) {
  return (
    <label className={`field ${className}`.trim()} {...props}>
      <span className="field__label">{label}</span>
      {children}
    </label>
  );
}

export function Banner({
  tone = "error",
  children,
  ...props
}: HTMLAttributes<HTMLDivElement> & { tone?: "error" | "warning" | "info" }) {
  return (
    <div className={`banner banner--${tone}`} role={tone === "error" ? "alert" : "status"} {...props}>
      {children}
    </div>
  );
}

export function EmptyState({ title, body }: { title: string; body: string }) {
  return (
    <div className="empty-state">
      <strong>{title}</strong>
      <p>{body}</p>
    </div>
  );
}
