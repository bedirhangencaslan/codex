// Chat state for one thread, driven by app-server notifications. This is the webview's port of
// the TUI's event translator (codex-rs/tui/src/chatwidget/protocol.rs handle_server_notification
// and replay.rs handle_thread_item): the same notifications, the same item lifecycle
// (item/started -> deltas -> item/completed), the same token-usage and goal updates. It is pure
// and framework-free so it can be tested against recorded notification sequences.
import type { ServerNotification } from "@protocol/ServerNotification";
import type { ThreadItem } from "@protocol/v2/ThreadItem";
import type { ThreadGoal } from "@protocol/v2/ThreadGoal";
import type { Turn } from "@protocol/v2/Turn";
import type { TurnPlanStep } from "@protocol/v2/TurnPlanStep";
import type { Usage } from "../../shared/meters";

export type TurnStatus = Turn["status"];

export interface Notice {
  kind: "error" | "warning" | "info";
  text: string;
  /** Non-null when the server will retry (transient errors). */
  willRetry?: boolean;
}

export interface TurnView {
  id: string;
  status: TurnStatus;
  items: ThreadItem[];
  errorMessage: string | null;
  plan: TurnPlanStep[] | null;
  planExplanation: string | null;
  notices: Notice[];
  durationMs: number | null;
}

export interface ChatState {
  threadId: string | null;
  threadName: string | null;
  turns: TurnView[];
  activeTurnId: string | null;
  tokenUsage: Usage | null;
  goal: ThreadGoal | null;
  /** Notices that arrived outside any turn (e.g. configuration warnings). */
  notices: Notice[];
}

export const emptyChat: ChatState = {
  threadId: null,
  threadName: null,
  turns: [],
  activeTurnId: null,
  tokenUsage: null,
  goal: null,
  notices: [],
};

export type ChatAction =
  | { type: "threadLoaded"; threadId: string; name: string | null; turns: Turn[] }
  | { type: "threadCleared" }
  | { type: "notification"; notification: ServerNotification }
  | { type: "localNotice"; notice: Notice }
  | { type: "goalLoaded"; goal: ThreadGoal | null };

const newTurn = (id: string, status: TurnStatus = "inProgress"): TurnView => ({
  id,
  status,
  items: [],
  errorMessage: null,
  plan: null,
  planExplanation: null,
  notices: [],
  durationMs: null,
});

function turnFromProtocol(turn: Turn): TurnView {
  return {
    ...newTurn(turn.id, turn.status),
    items: turn.items,
    errorMessage: turn.error?.message ?? null,
    durationMs: turn.durationMs,
  };
}

function updateTurn(state: ChatState, turnId: string, update: (turn: TurnView) => TurnView): ChatState {
  let found = false;
  const turns = state.turns.map((t) => {
    if (t.id !== turnId) return t;
    found = true;
    return update(t);
  });
  // A notification may arrive before turn/started was seen (e.g. after a resume); keep it.
  if (!found) turns.push(update(newTurn(turnId)));
  return { ...state, turns };
}

function upsertItem(turn: TurnView, item: ThreadItem): TurnView {
  const index = turn.items.findIndex((i) => i.id === item.id);
  if (index < 0) return { ...turn, items: [...turn.items, item] };
  const items = turn.items.slice();
  items[index] = item;
  return { ...turn, items };
}

function patchItem(turn: TurnView, itemId: string, patch: (item: ThreadItem) => ThreadItem): TurnView {
  const index = turn.items.findIndex((i) => i.id === itemId);
  if (index < 0) return turn;
  const items = turn.items.slice();
  items[index] = patch(items[index]!);
  return { ...turn, items };
}

function appendAt(list: string[], index: number, delta: string): string[] {
  const next = list.slice();
  while (next.length <= index) next.push("");
  next[index] = next[index] + delta;
  return next;
}

export function chatReducer(state: ChatState, action: ChatAction): ChatState {
  switch (action.type) {
    case "threadLoaded":
      return {
        ...emptyChat,
        threadId: action.threadId,
        threadName: action.name,
        turns: action.turns.map(turnFromProtocol),
        activeTurnId: action.turns.find((t) => t.status === "inProgress")?.id ?? null,
      };
    case "threadCleared":
      return emptyChat;
    case "goalLoaded":
      return { ...state, goal: action.goal };
    case "localNotice":
      return state.activeTurnId
        ? updateTurn(state, state.activeTurnId, (t) => ({ ...t, notices: [...t.notices, action.notice] }))
        : { ...state, notices: [...state.notices, action.notice] };
    case "notification":
      return onNotification(state, action.notification);
  }
}

function belongs(state: ChatState, params: unknown): boolean {
  const threadId = (params as { threadId?: string } | null)?.threadId;
  return threadId === undefined || threadId === state.threadId;
}

function onNotification(state: ChatState, n: ServerNotification): ChatState {
  if (!belongs(state, n.params)) return state;
  switch (n.method) {
    case "turn/started":
      return {
        ...updateTurn(state, n.params.turn.id, (t) => ({ ...t, status: "inProgress" })),
        activeTurnId: n.params.turn.id,
      };
    case "turn/completed": {
      const turn = n.params.turn;
      const next = updateTurn(state, turn.id, (t) => ({
        ...t,
        status: turn.status,
        errorMessage: turn.error?.message ?? t.errorMessage,
        durationMs: turn.durationMs,
        // The completed turn carries the final items when the server included them.
        items: turn.items.length > 0 ? mergeItems(t.items, turn.items) : t.items,
      }));
      return { ...next, activeTurnId: state.activeTurnId === turn.id ? null : state.activeTurnId };
    }
    case "item/started":
    case "item/completed":
      return updateTurn(state, n.params.turnId, (t) => upsertItem(t, n.params.item));
    case "item/agentMessage/delta":
      return updateTurn(state, n.params.turnId, (t) => {
        const withItem = t.items.some((i) => i.id === n.params.itemId)
          ? t
          : upsertItem(t, {
              type: "agentMessage",
              id: n.params.itemId,
              text: "",
              phase: null,
              memoryCitation: null,
              delivery: null,
              questions: null,
            });
        return patchItem(withItem, n.params.itemId, (i) =>
          i.type === "agentMessage" ? { ...i, text: i.text + n.params.delta } : i,
        );
      });
    case "item/plan/delta":
      return updateTurn(state, n.params.turnId, (t) =>
        patchItem(t, n.params.itemId, (i) => (i.type === "plan" ? { ...i, text: i.text + n.params.delta } : i)),
      );
    case "item/reasoning/summaryTextDelta":
      return updateTurn(state, n.params.turnId, (t) =>
        patchItem(t, n.params.itemId, (i) =>
          i.type === "reasoning" ? { ...i, summary: appendAt(i.summary, n.params.summaryIndex, n.params.delta) } : i,
        ),
      );
    case "item/reasoning/summaryPartAdded":
      return updateTurn(state, n.params.turnId, (t) =>
        patchItem(t, n.params.itemId, (i) =>
          i.type === "reasoning" ? { ...i, summary: appendAt(i.summary, n.params.summaryIndex, "") } : i,
        ),
      );
    case "item/reasoning/textDelta":
      return updateTurn(state, n.params.turnId, (t) =>
        patchItem(t, n.params.itemId, (i) =>
          i.type === "reasoning" ? { ...i, content: appendAt(i.content, n.params.contentIndex, n.params.delta) } : i,
        ),
      );
    case "item/commandExecution/outputDelta":
      return updateTurn(state, n.params.turnId, (t) =>
        patchItem(t, n.params.itemId, (i) =>
          i.type === "commandExecution" ? { ...i, aggregatedOutput: (i.aggregatedOutput ?? "") + n.params.delta } : i,
        ),
      );
    case "turn/plan/updated":
      return updateTurn(state, n.params.turnId, (t) => ({
        ...t,
        plan: n.params.plan,
        planExplanation: n.params.explanation ?? null,
      }));
    case "error":
      return updateTurn(state, n.params.turnId, (t) => ({
        ...t,
        notices: [...t.notices, { kind: "error", text: n.params.error.message, willRetry: n.params.willRetry }],
      }));
    case "thread/tokenUsage/updated":
      return { ...state, tokenUsage: n.params.tokenUsage };
    case "thread/goal/updated":
      return { ...state, goal: n.params.goal };
    case "thread/goal/cleared":
      return { ...state, goal: null };
    case "thread/name/updated":
      return { ...state, threadName: n.params.threadName ?? null };
    default:
      return state;
  }
}

/**
 * turn/completed may carry only a summary of the turn's items (the recorded fixture has just the
 * final answer), so it must not replace what was streamed: streamed order is kept, items the
 * summary also has are updated in place, and items only the summary has are appended.
 */
function mergeItems(streamed: ThreadItem[], final: ThreadItem[]): ThreadItem[] {
  const byId = new Map(final.map((i) => [i.id, i]));
  const merged = streamed.map((i) => byId.get(i.id) ?? i);
  const streamedIds = new Set(streamed.map((i) => i.id));
  return [...merged, ...final.filter((i) => !streamedIds.has(i.id))];
}

/** Text of the last completed agent message, for /copy. */
export function lastAgentText(state: ChatState): string | undefined {
  for (let t = state.turns.length - 1; t >= 0; t--) {
    const items = state.turns[t]!.items;
    for (let i = items.length - 1; i >= 0; i--) {
      const item = items[i]!;
      if (item.type === "agentMessage" && item.text.trim()) return item.text;
    }
  }
  return undefined;
}
