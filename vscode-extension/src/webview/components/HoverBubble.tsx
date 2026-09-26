// A transcript line whose details open in a speech bubble beside it: after the pointer rests on
// the line for a moment, or on a click (which keeps it open). Used by command and file-change cells.
import { useEffect, useLayoutEffect, useRef, useState, type MouseEvent, type ReactNode } from "react";

const HOVER_OPEN_MS = 300;
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
  /** Pins the bubble open, or closes a pinned one. */
  toggle: (e?: MouseEvent) => void;
}

/**
 * The bubble starts level with the line (the first child, `head`), shifted to the right with its
 * tail on the line; in a view docked on the right of the window it is mirrored and opens to the
 * left. It is up to 90% of the view's height, so most details show without scrolling, and moves up
 * only as far as it must to stay on screen. A webview cannot draw outside its own view, so the
 * bubble stays inside the side bar.
 */
export function HoverBubble({
  className,
  head,
  children,
  bubble,
  label,
  disabled = false,
}: {
  className: string;
  head: (state: BubbleHead) => ReactNode;
  /** Shown under the line while the bubble is closed (a short preview). */
  children?: ReactNode;
  bubble: () => ReactNode;
  label: string;
  disabled?: boolean;
}) {
  const [hover, setHover] = useState(false);
  const [pinned, setPinned] = useState(false);
  const [toLeft, setToLeft] = useState(false);
  const cellRef = useRef<HTMLDivElement>(null);
  const popRef = useRef<HTMLDivElement>(null);
  const hoverTimer = useRef<ReturnType<typeof setTimeout>>();
  const open = !disabled && (hover || pinned);

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
  // Follow the line when the chat scrolls or the view is resized while the bubble is open.
  useEffect(() => {
    if (!open) return;
    const scroller = cellRef.current ? scrollParent(cellRef.current) : null;
    scroller?.addEventListener("scroll", place, { passive: true });
    window.addEventListener("resize", place);
    return () => {
      scroller?.removeEventListener("scroll", place);
      window.removeEventListener("resize", place);
    };
  }, [open, toLeft]);
  useEffect(() => () => clearTimeout(hoverTimer.current), []);

  // Opens after a short dwell, so moving the pointer across a list of lines opens none of them.
  const enter = (e: MouseEvent<HTMLDivElement>) => {
    if (disabled) return;
    const at = { screenX: e.screenX, clientX: e.clientX };
    clearTimeout(hoverTimer.current);
    hoverTimer.current = setTimeout(() => (setToLeft(dockedRight(at)), setHover(true)), HOVER_OPEN_MS);
  };
  const leave = () => {
    clearTimeout(hoverTimer.current);
    setHover(false);
  };
  const toggle = (e?: MouseEvent) => {
    if (disabled) return;
    if (e && e.detail > 0) setToLeft(dockedRight(e));
    setPinned(!pinned);
  };

  return (
    <div ref={cellRef} className={`${className} ${open ? "is-open" : ""}`} onMouseEnter={enter} onMouseLeave={leave}>
      {head({ open, toggle })}
      {children}
      {open && (
        <div ref={popRef} className={`sf-bubble ${toLeft ? "is-left" : ""}`} role="region" aria-label={label}>
          <div className="sf-bubble-body">{bubble()}</div>
        </div>
      )}
    </div>
  );
}
