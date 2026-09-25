// Small building blocks shared by every screen. Styling lives in styles.css (theme variables).
import type { ButtonHTMLAttributes, ReactNode } from "react";
import { Icon, type IconName } from "./icons";

export function Button({
  variant = "secondary",
  icon,
  children,
  className = "",
  ...rest
}: { variant?: "primary" | "secondary" | "ghost" | "danger"; icon?: IconName } & ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button type="button" className={`sf-btn sf-btn-${variant} ${className}`} {...rest}>
      {icon && <Icon name={icon} />}
      {children && <span>{children}</span>}
    </button>
  );
}

export function IconButton({ icon, label, active, ...rest }: { icon: IconName; label: string; active?: boolean } & ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button type="button" className={`sf-icon-btn ${active ? "is-active" : ""}`} title={label} aria-label={label} {...rest}>
      <Icon name={icon} />
    </button>
  );
}

export function Toggle({ checked, onChange, label, disabled }: { checked: boolean; onChange: (v: boolean) => void; label: string; disabled?: boolean }) {
  return (
    <label className={`sf-toggle ${disabled ? "is-disabled" : ""}`}>
      <input type="checkbox" checked={checked} disabled={disabled} onChange={(e) => onChange(e.target.checked)} />
      <span className="sf-toggle-track" aria-hidden="true">
        <span className="sf-toggle-thumb" />
      </span>
      <span className="sf-toggle-label">{label}</span>
    </label>
  );
}

export function Chip({
  icon,
  children,
  active,
  onClick,
  title,
  badge,
}: {
  icon?: IconName;
  children?: ReactNode;
  active?: boolean;
  onClick?: () => void;
  title?: string;
  badge?: string | number;
}) {
  return (
    <button type="button" className={`sf-chip ${active ? "is-active" : ""}`} onClick={onClick} title={title} aria-pressed={active}>
      {icon && <Icon name={icon} size={14} />}
      {children && <span className="sf-chip-text">{children}</span>}
      {badge !== undefined && badge !== 0 && <span className="sf-chip-badge">{badge}</span>}
    </button>
  );
}

export function Section({ title, description, children, actions }: { title: string; description?: string; children: ReactNode; actions?: ReactNode }) {
  return (
    <section className="sf-section">
      <header className="sf-section-head">
        <div>
          <h2>{title}</h2>
          {description && <p className="sf-muted">{description}</p>}
        </div>
        {actions && <div className="sf-section-actions">{actions}</div>}
      </header>
      {children}
    </section>
  );
}

export function Empty({ children }: { children: ReactNode }) {
  return <div className="sf-empty">{children}</div>;
}

export function Badge({ tone = "neutral", children }: { tone?: "neutral" | "accent" | "success" | "warning" | "danger"; children: ReactNode }) {
  return <span className={`sf-badge sf-badge-${tone}`}>{children}</span>;
}

export function Notice({ tone, children }: { tone: "info" | "warning" | "danger" | "success"; children: ReactNode }) {
  const icon: IconName = tone === "info" ? "info" : tone === "success" ? "check" : "warning";
  return (
    <div className={`sf-notice sf-notice-${tone}`} role={tone === "danger" ? "alert" : "status"}>
      <Icon name={icon} />
      <div>{children}</div>
    </div>
  );
}

export function SearchBox({ value, onChange, placeholder }: { value: string; onChange: (v: string) => void; placeholder: string }) {
  return (
    <label className="sf-search">
      <Icon name="search" />
      <input value={value} onChange={(e) => onChange(e.target.value)} placeholder={placeholder} aria-label={placeholder} />
    </label>
  );
}
