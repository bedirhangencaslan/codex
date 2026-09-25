import type { ServerNotification } from "@protocol/ServerNotification";
import { chatReducer, emptyChat, lastAgentText, type ChatState } from "../src/webview/state/chatReducer";
import recorded from "./fixtures/turn-notifications.json";

const play = (state: ChatState, notifications: unknown[]) =>
  notifications.reduce<ChatState>(
    (s, n) => chatReducer(s, { type: "notification", notification: n as ServerNotification }),
    state,
  );

describe("chat reducer on a recorded real turn", () => {
  const threadId = (recorded.find((n) => n.method === "thread/started")!.params as { thread: { id: string } }).thread.id;
  const start: ChatState = { ...emptyChat, threadId };
  const end = play(start, recorded);

  test("one completed turn with the user message, the command and the answer", () => {
    expect(end.turns).toHaveLength(1);
    const turn = end.turns[0]!;
    expect(turn.status).toBe("completed");
    expect(turn.items.map((i) => i.type)).toEqual(["userMessage", "commandExecution", "agentMessage"]);
    expect(end.activeTurnId).toBeNull();
  });

  test("streamed deltas add up to the completed text", () => {
    const answer = end.turns[0]!.items.find((i) => i.type === "agentMessage");
    expect(answer && answer.type === "agentMessage" && answer.text).toContain("hello-suffice");
    expect(lastAgentText(end)).toContain("hello-suffice");
  });

  test("the command output is there and token usage was captured", () => {
    const cmd = end.turns[0]!.items.find((i) => i.type === "commandExecution");
    expect(cmd && cmd.type === "commandExecution" && cmd.aggregatedOutput).toContain("hello-suffice");
    expect(end.tokenUsage?.total.inputTokens).toBeGreaterThan(0);
    // The server reports the EFFECTIVE window: 200K x effective_context_window_percent (95).
    // Meters use it (as the TUI does); the compaction clamp uses the raw 200K from the catalog.
    expect(end.tokenUsage?.modelContextWindow).toBe(190_000);
  });

  test("mid-stream the answer grows delta by delta", () => {
    const upTo = recorded.findIndex((n) => n.method === "item/agentMessage/delta");
    const mid = play(start, recorded.slice(0, upTo + 1));
    const answer = mid.turns[0]!.items.find((i) => i.type === "agentMessage");
    expect(answer && answer.type === "agentMessage" && answer.text.length).toBeGreaterThan(0);
    expect(mid.activeTurnId).not.toBeNull();
  });

  test("notifications for another thread are ignored", () => {
    const other = play({ ...emptyChat, threadId: "someone-else" }, recorded);
    expect(other.turns).toHaveLength(0);
  });
});

describe("chat reducer, synthetic cases", () => {
  const base: ChatState = { ...emptyChat, threadId: "t1" };
  const n = (method: string, params: object) => ({ method, params: { threadId: "t1", ...params } });

  test("reasoning summary deltas fill their parts", () => {
    const s = play(base, [
      n("turn/started", { turn: { id: "u1", items: [], status: "inProgress", error: null } }),
      n("item/started", { turnId: "u1", item: { type: "reasoning", id: "r1", summary: [], content: [] } }),
      n("item/reasoning/summaryTextDelta", { turnId: "u1", itemId: "r1", delta: "Look", summaryIndex: 0 }),
      n("item/reasoning/summaryTextDelta", { turnId: "u1", itemId: "r1", delta: "ing", summaryIndex: 0 }),
      n("item/reasoning/summaryPartAdded", { turnId: "u1", itemId: "r1", summaryIndex: 1 }),
      n("item/reasoning/summaryTextDelta", { turnId: "u1", itemId: "r1", delta: "Done", summaryIndex: 1 }),
    ]);
    const r = s.turns[0]!.items[0]!;
    expect(r.type === "reasoning" && r.summary).toEqual(["Looking", "Done"]);
  });

  test("errors become turn notices; failed turns keep their message", () => {
    const s = play(base, [
      n("turn/started", { turn: { id: "u1", items: [], status: "inProgress", error: null } }),
      n("error", { turnId: "u1", willRetry: true, error: { message: "rate limited" } }),
      n("turn/completed", { turn: { id: "u1", items: [], status: "failed", error: { message: "gave up" }, durationMs: 5 } }),
    ]);
    expect(s.turns[0]!.notices).toEqual([{ kind: "error", text: "rate limited", willRetry: true }]);
    expect(s.turns[0]!.errorMessage).toBe("gave up");
    expect(s.turns[0]!.status).toBe("failed");
  });

  test("goal and name updates", () => {
    const goal = { threadId: "t1", objective: "ship", status: "active", tokenBudget: null, tokensUsed: 10, timeUsedSeconds: 1, createdAt: 0, updatedAt: 0 };
    let s = play(base, [n("thread/goal/updated", { turnId: null, goal }), n("thread/name/updated", { threadName: "Refactor" })]);
    expect(s.goal?.objective).toBe("ship");
    expect(s.threadName).toBe("Refactor");
    s = play(s, [n("thread/goal/cleared", {})]);
    expect(s.goal).toBeNull();
  });

  test("a loaded thread replays its stored turns", () => {
    const s = chatReducer(base, {
      type: "threadLoaded",
      threadId: "t2",
      name: "Old",
      turns: [{ id: "a", items: [], itemsView: "full", status: "completed", error: null, startedAt: 1, completedAt: 2, durationMs: 1 }],
    });
    expect(s.threadId).toBe("t2");
    expect(s.turns[0]!.status).toBe("completed");
  });
});
