// The compaction slider's React state, shared by the meters row and the Settings screen. The range
// itself is the pure `compactionRange` (compaction.ts), which the controller also uses.
import { useEffect, useState } from "react";
import { compactionRange } from "./compaction";
import { useApp } from "./context";

/** Slider state: moves locally while dragging, writes the config when released. */
export function useCompactionSlider(contextTokens: number) {
  const { state, ctl } = useApp();
  const range = compactionRange(state, contextTokens);
  const [value, setValue] = useState(range.value);
  useEffect(() => setValue(range.value), [range.value]);
  const commit = async (next: number | null): Promise<boolean> => (next === range.configured ? false : ctl.writeCompactionLimit(next));
  return {
    range,
    value,
    // Dragging below the chat's current context stops at min + 0, never past it.
    setValue: (next: number) => setValue(Math.min(range.max, Math.max(range.min, next))),
    commit,
    disabled: state.server.state !== "ready",
  };
}
