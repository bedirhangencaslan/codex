// A transcript line whose details open in a speech bubble beside it, from an expand button at the
// end of the line (or a click on the line). Used by command and file-change cells.
import { useEffect, useLayoutEffect, useRef, useState, type MouseEvent, type ReactNode } from "react";
import { Icon } from "./icons";

/** Horizontal shift of the bubble from the line, the gap kept to the view's edges, the tail size. */
const BUBBLE_SHIFT = 22;
const BUBBLE_MARGIN = 6;
const BUBBLE_TAIL = 7;

/** The nearest scrolling ancestor; the bubble follows the line when it scrolls. */
function scrollParent(el: HTMLElement): HTMLElement | null {
  for (let p = el.parentElement; p; p = p.parentElement) {
    if (/(auto|scroll)/.test(getComputedStyle(p).overflowY)) return p;
  }
  return null;
}

/**
 * Whether the view sits in the right half of the VS Code window (the secondary side bar, or the
 * side bar moved to the right). A webview is told nothing about where it is docked, so this
 * compares the pointer's screen and view coordinates with the window's own position.
 */
function dockedRight(e: { screenX: number; clientX: number }): boolean {
  if (!window.outerWidth) return false;
  const viewCenter = e.screenX - e.clientX + window.innerWidth / 2;
  return viewCenter > window.screenX + window.outerWidth / 2;
}

export interface BubbleHead {
  open: boolean;
  /** Opens or closes the bubble. */
  toggle: (e?: MouseEvent) => void;
  /** The expand button, to be placed at the end of the line (null when there is nothing to expand). */
  expand: ReactNode;
}

/**
 * The bubble starts level with the line (the first child, `head`), shifted to the right with its
 * tail on the line; in a view docked on the right of the window it is mirrored and opens to the
 * left. It is up to 90% of the view's height, so most details show without scrolling, and moves up
 * only as far as it must to stay on screen. It closes from the button again, a click outside, or
 * Escape. A webview cannot draw outside its own view, so the bubble stays inside the side bar.
 */
export function ExpandBubble({
  className,
  head,
  children,
  bubble,
  label,
  expandLabel,
  collapseLabel,
  disabled = false,
}: {
  className: string;
  head: (state: BubbleHead) => ReactNode;
  /** Shown under the line (a short preview). */
  children?: ReactNode;
  bubble: () => ReactNode;
  label: string;
  expandLabel: string;
  collapseLabel: string;
  disabled?: boolean;
}) {
  const [opened, setOpened] = useState(false);
  const [toLeft, setToLeft] = useState(false);
  const cellRef = useRef<HTMLDivElement>(null);
  const popRef = useRef<HTMLDivElement>(null);
  const open = !disabled && opened;

  const place = () => {
    const cell = cellRef.current;
    const pop = popRef.current;
    const line = cell?.firstElementChild?.getBoundingClientRect();
    if (!cell || !pop || !line) return;
    const box = cell.getBoundingClientRect();
    const height = pop.offsetHeight;
    const top = Math.max(BUBBLE_MARGIN, Math.min(line.top + 2, window.innerHeight - BUBBLE_MARGIN - height));
    const tail = Math.max(6, Math.min(line.top + line.height / 2 - top - BUBBLE_TAIL, height - 2 * BUBBLE_TAIL - 6));
    const right = window.innerWidth - box.right;
    pop.style.top = `${top}px`;
    pop.style.left = `${toLeft ? box.left : box.left + BUBBLE_SHIFT}px`;
    pop.style.right = `${toLeft ? right + BUBBLE_SHIFT : right}px`;
    pop.style.setProperty("--tail-top", `${tail}px`);
  };
  useLayoutEffect(() => {
    if (open) place();
  });
  // While open: follow the line when the chat scrolls or the view is resized; close on a click
  // outside the line and its bubble, or on Escape.
  useEffect(() => {
    if (!open) return;
    const scroller = cellRef.current ? scrollParent(cellRef.current) : null;
    const outside = (e: globalThis.MouseEvent) => {
      if (cellRef.current && !cellRef.current.contains(e.target as Node)) setOpened(false);
    };
    const escape = (e: KeyboardEvent) => e.key === "Escape" && setOpened(false);
    scroller?.addEventListener("scroll", place, { passive: true });
    window.addEventListener("resize", place);
    document.addEventListener("mousedown", outside);
    document.addEventListener("keydown", escape);
    return () => {
      scroller?.removeEventListener("scroll", place);
      window.removeEventListener("resize", place);
      document.removeEventListener("mousedown", outside);
      document.removeEventListener("keydown", escape);
    };
  }, [open, toLeft]);

  const toggle = (e?: MouseEvent) => {
    if (disabled) return;
    if (e && e.detail > 0) setToLeft(dockedRight(e));
    setOpened(!opened);
  };
  const expand = disabled ? null : (
    <button
      type="button"
      className="sf-expand"
      onClick={(e) => (e.stopPropagation(), toggle(e))}
      aria-expanded={open}
      aria-label={open ? collapseLabel : expandLabel}
      title={open ? collapseLabel : expandLabel}
    >
      <Icon name={open ? "collapse" : "expand"} size={13} />
    </button>
  );

  return (
    <div ref={cellRef} className={`${className} ${open ? "is-open" : ""}`}>
      {head({ open, toggle, expand })}
      {children}
      {open && (
        <div ref={popRef} className={`sf-bubble ${toLeft ? "is-left" : ""}`} role="region" aria-label={label}>
          <div className="sf-bubble-body">{bubble()}</div>
          {/* The bubble covers the line and its button, so it carries its own. */}
          <button type="button" className="sf-expand sf-bubble-close" onClick={() => setOpened(false)} aria-label={collapseLabel} title={collapseLabel}>
            <Icon name="collapse" size={13} />
          </button>
        </div>
      )}
    </div>
  );
}
