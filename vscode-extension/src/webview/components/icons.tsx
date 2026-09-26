// Inline icons (no icon font, no network). 16px, stroke = currentColor.
import type { SVGProps } from "react";

const paths: Record<string, string> = {
  plus: "M8 3v10M3 8h10",
  history: "M2.5 8a5.5 5.5 0 1 0 1.6-3.9M2.5 2.5v2.6h2.6M8 5v3l2 1.5",
  grid: "M2.5 2.5h4.5v4.5H2.5zM9 2.5h4.5v4.5H9zM2.5 9h4.5v4.5H2.5zM9 9h4.5v4.5H9z",
  gear: "M6.39 3.58L6.61 1.45L9.39 1.45L9.61 3.58A4.7 4.7 0 0 1 11.02 4.4L12.98 3.52L14.37 5.93L12.63 7.18A4.7 4.7 0 0 1 12.63 8.82L14.37 10.07L12.98 12.48L11.02 11.6A4.7 4.7 0 0 1 9.61 12.42L9.39 14.55L6.61 14.55L6.39 12.42A4.7 4.7 0 0 1 4.98 11.6L3.02 12.48L1.63 10.07L3.37 8.82A4.7 4.7 0 0 1 3.37 7.18L1.63 5.93L3.02 3.52L4.98 4.4A4.7 4.7 0 0 1 6.39 3.58zM8 6a2 2 0 1 1 0 4 2 2 0 0 1 0-4z",
  send: "M2.5 8h9M8 4.5 11.5 8 8 11.5",
  stop: "M4.5 4.5h7v7h-7z",
  eye: "M1.5 8s2.4-4.5 6.5-4.5S14.5 8 14.5 8s-2.4 4.5-6.5 4.5S1.5 8 1.5 8zM8 6.2a1.8 1.8 0 1 1 0 3.6 1.8 1.8 0 0 1 0-3.6z",
  eyeOff: "M2 2l12 12M6.4 6.5A1.8 1.8 0 0 0 9.5 9.6M4.3 4.5C2.6 5.6 1.5 8 1.5 8s2.4 4.5 6.5 4.5c1.3 0 2.4-.4 3.3-1M7 3.6c.3 0 .7-.1 1-.1 4.1 0 6.5 4.5 6.5 4.5s-.6 1.2-1.7 2.3",
  layers: "M8 2 1.5 5.5 8 9l6.5-3.5zM1.5 8.5 8 12l6.5-3.5M1.5 11 8 14.5l6.5-3.5",
  sparkles: "M6 2.5l1 3 3 1-3 1-1 3-1-3-3-1 3-1zM11.5 9l.6 1.7 1.7.6-1.7.6-.6 1.7-.6-1.7-1.7-.6 1.7-.6z",
  folder: "M1.5 4.5v8h13v-7H8L6.5 4H2z",
  file: "M4 1.5h5l3 3v10H4zM9 1.5v3h3",
  chip: "M4.5 4.5h7v7h-7zM6.5 2v2.5M9.5 2v2.5M6.5 11.5V14M9.5 11.5V14M2 6.5h2.5M2 9.5h2.5M11.5 6.5H14M11.5 9.5H14",
  chevronDown: "M4 6l4 4 4-4",
  chevronRight: "M6 4l4 4-4 4",
  expand: "M9.5 2.5h4v4M13.5 2.5 9 7M6.5 13.5h-4v-4M2.5 13.5 7 9",
  collapse: "M13.5 2.5 9.5 6.5M9.5 3v3.5H13M2.5 13.5l4-4M6.5 13V9.5H3",
  check: "M3 8.5l3 3 7-7",
  copy: "M5.5 5.5h7v8h-7zM3.5 10.5v-8h7",
  terminal: "M2 3h12v10H2zM4.5 6l2 2-2 2M8 10h3.5",
  search: "M7 2.5a4.5 4.5 0 1 1 0 9 4.5 4.5 0 0 1 0-9zM10.3 10.3 14 14",
  key: "M10 2.5a3.5 3.5 0 1 1-3.2 4.9L2 12.2V14h2.5v-1.5H6V11h1.5l.4-.4A3.5 3.5 0 0 1 10 2.5zM11 5h.01",
  book: "M2.5 3c2-.8 3.8-.8 5.5.5 1.7-1.3 3.5-1.3 5.5-.5v10c-2-.8-3.8-.8-5.5.5-1.7-1.3-3.5-1.3-5.5-.5zM8 3.5v10",
  x: "M4 4l8 8M12 4l-8 8",
  warning: "M8 2 14.5 13.5h-13zM8 6.5v3.5M8 11.8v.01",
  info: "M8 1.5a6.5 6.5 0 1 1 0 13 6.5 6.5 0 0 1 0-13zM8 7v4M8 4.8v.01",
  refresh: "M13 3v3.5H9.5M3 13V9.5h3.5M12.4 6A5 5 0 0 0 3.6 5M3.6 10a5 5 0 0 0 8.8 1",
  chat: "M2.5 3.5h11v7.5H7l-3 2.5V11H2.5z",
  target: "M8 1.5a6.5 6.5 0 1 1 0 13 6.5 6.5 0 0 1 0-13zM8 4.5a3.5 3.5 0 1 1 0 7 3.5 3.5 0 0 1 0-7zM8 7.3a.7.7 0 1 1 0 1.4.7.7 0 0 1 0-1.4z",
  back: "M10 3.5 5.5 8l4.5 4.5",
  pencil: "M10.5 2.5l3 3L6 13H3v-3zM9 4l3 3",
  palette: "M8 1.5A6.5 6.5 0 0 0 8 14.5c1 0 1.2-.8.8-1.5-.5-.8 0-1.8 1-1.8h1.7a3 3 0 0 0 3-3A6.5 6.5 0 0 0 8 1.5zM4.8 7.5h.01M6.5 4.8h.01M9.8 4.8h.01",
};

export type IconName = keyof typeof paths;

export function Icon({ name, size = 16, ...rest }: { name: IconName; size?: number } & SVGProps<SVGSVGElement>) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth={1.4} strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" {...rest}>
      <path d={paths[name]} />
    </svg>
  );
}
