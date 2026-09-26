// The Suffice course (goal item 3): lessons must only teach commands that exist, practice only
// commands the extension can run, open only panels and screens that exist, quizzes must have one
// right answer, and every text must exist in every locale.
import { en } from "../src/shared/i18n/en";
import { learnEn } from "../src/shared/i18n/learn.en";
import { LOCALES } from "../src/shared/i18n";
import {
  EMPTY_PROGRESS,
  lessonComplete,
  lessonProgress,
  lessonSets,
  nextLesson,
  readMark,
  recordOpened,
  recordPracticed,
  registerLessonSet,
  stepDone,
  type LessonStep,
} from "../src/shared/lessons";
import { findCommand } from "../src/shared/slashCommands";
import { Controller } from "../src/webview/app/controller";
import { appReducer, initialState, SCREENS, type AppAction, type AppState } from "../src/webview/app/state";
import { BaseBridge } from "../src/webview/host";
import { localeByCode, translate } from "../src/shared/i18n";
import type { InitState, WebviewToHost } from "../src/shared/messages";

const lessons = lessonSets().flatMap((s) => s.lessons);
const steps = lessons.flatMap((l) => l.steps.map((s) => ({ lesson: l.id, step: s })));
const keysOf = (s: LessonStep): string[] => {
  switch (s.kind) {
    case "read":
    case "show":
      return [s.textKey];
    case "try":
      return [s.textKey, s.draftKey];
    case "quiz":
      return [s.questionKey, s.explainKey];
    case "choice":
      return [s.questionKey, s.explainKey, ...s.optionKeys];
  }
};

describe("lesson content", () => {
  test("every covered or practiced command exists; practiced ones run in the extension", () => {
    for (const l of lessons) for (const c of l.commands) expect(findCommand(c)).toBeDefined();
    for (const { step } of steps) {
      if (step.kind !== "try") continue;
      expect(findCommand(step.command)?.guiAction).toBeDefined();
    }
  });

  test("a try step's draft runs the command it teaches", () => {
    for (const { step } of steps) {
      if (step.kind !== "try") continue;
      for (const locale of LOCALES) {
        const draft = translate(locale, step.draftKey);
        expect(draft.split(/\s/)[0]).toBe(`/${step.command}`);
      }
    }
  });

  test("quizzes: unique ids, real commands, the answer is one of the options", () => {
    const quizzes = steps.map((s) => s.step).filter((s): s is Extract<LessonStep, { kind: "quiz" }> => s.kind === "quiz");
    expect(new Set(quizzes.map((q) => q.id)).size).toBe(quizzes.length);
    for (const q of quizzes) {
      expect(q.options).toContain(q.answer);
      expect(new Set(q.options).size).toBe(q.options.length);
      for (const o of q.options) expect(findCommand(o)).toBeDefined();
    }
  });

  test("choice quizzes: unique ids, two to four options, the answer is one of them", () => {
    const ids = steps.map((s) => s.step).filter((s) => s.kind === "quiz" || s.kind === "choice").map((s) => (s as { id: string }).id);
    expect(new Set(ids).size).toBe(ids.length);
    for (const { step } of steps) {
      if (step.kind !== "choice") continue;
      expect(step.optionKeys.length).toBeGreaterThanOrEqual(2);
      expect(step.optionKeys.length).toBeLessThanOrEqual(4);
      expect(step.answer).toBeGreaterThanOrEqual(0);
      expect(step.answer).toBeLessThan(step.optionKeys.length);
    }
  });

  test("show steps open panels and screens that exist", () => {
    const panels = ["mode", "skills", "files", "prefs", "model"];
    for (const { step } of steps) {
      if (step.kind !== "show") continue;
      const [kind, name] = step.target.split(":");
      if (kind === "panel") expect(panels).toContain(name);
      else expect(SCREENS).toContain(name);
    }
  });

  test("three levels, every lesson in exactly one", () => {
    expect(lessonSets().slice(0, 3).map((s) => s.id)).toEqual(["suffice-beginner", "suffice-intermediate", "suffice-advanced"]);
    expect(new Set(lessons.map((l) => l.id)).size).toBe(lessons.length);
  });

  test("the built-in course asks no questions", () => {
    for (const { step } of steps) expect(["read", "try", "show"]).toContain(step.kind);
  });

  test("every lesson text exists in every locale, and no lesson text is unused", () => {
    const used = new Set<string>();
    for (const set of lessonSets()) {
      used.add(set.titleKey);
      if (set.descKey) used.add(set.descKey);
    }
    for (const l of lessons) {
      used.add(l.titleKey);
      used.add(l.introKey);
      for (const s of l.steps) keysOf(s).forEach((k) => used.add(k));
    }
    for (const key of used) {
      expect(Object.keys(en)).toContain(key);
      for (const locale of LOCALES) expect((locale.messages as Record<string, string>)[key]).toBeTruthy();
    }
    expect(Object.keys(learnEn).filter((k) => !used.has(k))).toEqual([]);
  });
});

describe("progress", () => {
  const basics = lessons.find((l) => l.id === "basics")!;

  test("a lesson is complete when its commands were run", () => {
    let p = EMPTY_PROGRESS;
    expect(lessonProgress(basics, p)).toEqual({ done: 0, total: 2 });
    p = recordPracticed(p, "status");
    expect(lessonComplete(basics, p)).toBe(false);
    p = recordPracticed(p, "pwd");
    expect(lessonComplete(basics, p)).toBe(true);
    expect(nextLesson(p)?.id).toBe("chat");
  });

  test("a lesson with only reading is complete once it has been opened", () => {
    const chat = lessons.find((l) => l.id === "chat")!;
    expect(chat.steps.every((st) => st.kind === "read")).toBe(true);
    expect(lessonComplete(chat, EMPTY_PROGRESS)).toBe(false);
    expect(lessonComplete(chat, recordOpened(EMPTY_PROGRESS, readMark("chat")))).toBe(true);
  });

  test("recording a command twice keeps the same object", () => {
    const p = recordPracticed(EMPTY_PROGRESS, "status");
    expect(recordPracticed(p, "status")).toBe(p);
  });

  test("a registered lesson set is appended without touching the built-in one", () => {
    const before = lessonSets().length;
    registerLessonSet({ id: "extra", titleKey: "learn.levelBeginner", lessons: [] });
    expect(lessonSets()).toHaveLength(before + 1);
    expect(lessonSets()[0]!.id).toBe("suffice-beginner");
  });
});

describe("the controller records practice", () => {
  // The toast clears itself after 4s; do not leave that timer running after the tests.
  beforeEach(() => jest.useFakeTimers({ doNotFake: ["queueMicrotask", "nextTick"] }));
  afterEach(() => jest.useRealTimers());

  class Bridge extends BaseBridge {
    readonly isPreview = false;
    posts: WebviewToHost[] = [];
    post(m: WebviewToHost) {
      this.posts.push(m);
      if (m.type === "rpc") queueMicrotask(() => this.receive({ type: "rpcResult", id: m.id, result: {} }));
    }
    viewState<T>(): T | undefined {
      return undefined;
    }
    setViewState(): void {}
  }
  const init: InitState = {
    locale: "en", languageSetting: "auto", themeId: "vscode", workspaceFolders: [{ name: "w", path: "/w" }], envKeys: [],
    extensionVersion: "0", prices: {}, preferences: "", attachedSkills: [], invisibleTurns: {}, threadStartCompaction: {},
    learning: EMPTY_PROGRESS, threadBlocks: {}, threadPreferences: {}, threadSkills: {},
  };
  function setup() {
    const bridge = new Bridge();
    let state: AppState = { ...initialState, init, server: { state: "ready", codexHome: "/h", userAgent: "x", cwd: "/w" } };
    const ctl = new Controller(bridge, (a: AppAction) => (state = appReducer(state, a)), state, () => (k, p) => translate(localeByCode("en"), k, p));
    return { bridge, ctl, get state() { return state; } };
  }

  test("running a taught command marks it practiced, persists it and says so", async () => {
    const env = setup();
    await env.ctl.send("/pwd");
    expect(env.state.init!.learning.practiced).toEqual(["pwd"]);
    expect(env.bridge.posts).toContainEqual({ type: "setState", key: "learning", value: { practiced: ["pwd"], quizzes: {}, explored: [], opened: [] } });
    expect(env.state.toast).toBe("Lesson step done: /pwd");
  });

  test("a terminal-only command is explained, not counted", async () => {
    const env = setup();
    await env.ctl.send("/vim");
    expect(env.state.init!.learning.practiced).toEqual([]);
  });

  test("quiz answers and reference visits persist; reset clears them", () => {
    const env = setup();
    env.ctl.answerQuiz("basics.folder", "pwd");
    env.ctl.markExplored("suffice-tui:plan");
    expect(env.state.init!.learning).toEqual({ practiced: [], quizzes: { "basics.folder": "pwd" }, explored: ["suffice-tui:plan"], opened: [] });
    env.ctl.resetLearning();
    expect(env.state.init!.learning).toEqual(EMPTY_PROGRESS);
  });

  test("a show step opens its panel on the chat screen, or its screen, and completes", () => {
    const env = setup();
    env.ctl.go("commands");
    env.ctl.showLessonTarget("panel:files");
    expect(env.state.screen).toBe("chat");
    expect(env.state.composer.panel).toBe("files");
    env.ctl.showLessonTarget("screen:api");
    expect(env.state.screen).toBe("api");
    expect(env.state.init!.learning.opened).toEqual(["panel:files", "screen:api"]);
    const files = lessons.find((l) => l.id === "files")!;
    expect(files.steps.filter((st) => st.kind === "show").every((st) => stepDone(st, env.state.init!.learning))).toBe(true);
  });

  test("progress saved before show steps existed still loads", () => {
    const old = { practiced: ["status"], quizzes: {}, explored: [] };
    const screen = lessons.find((l) => l.id === "screen")!;
    expect(lessonProgress(screen, old).done).toBe(0);
    expect(recordOpened(old, "screen:settings").opened).toEqual(["screen:settings"]);
  });
});
