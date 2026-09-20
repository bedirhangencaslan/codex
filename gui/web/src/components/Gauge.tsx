const CIRC = 2 * Math.PI * 15;

/** Circular cached-share gauge; warm ring when the share crosses the TUI's
 * "fast" threshold, cold ring below it. */
export function Gauge({ percent }: { percent: number }) {
  const clamped = Math.max(0, Math.min(100, percent));
  const warm = clamped >= 40;
  return (
    <svg
      className={`gauge ${warm ? "warm" : "cold"}`}
      viewBox="0 0 36 36"
      role="img"
      aria-label={`Önbellekli girdi %${clamped}`}
    >
      <circle className="track" cx="18" cy="18" r="15" fill="none" strokeWidth="3" />
      <circle
        className="val"
        cx="18"
        cy="18"
        r="15"
        fill="none"
        strokeWidth="3"
        strokeDasharray={`${(clamped / 100) * CIRC} ${CIRC}`}
      />
      <text x="18" y="21" textAnchor="middle">
        {clamped}
      </text>
    </svg>
  );
}
