// The slash command course (goal item 3): lessons must only teach commands that exist, practice
// only commands the extension can run, quizzes must have one right answer among real commands,
// and every text must exist in every locale.
import { en } from "../src/shared/i18n/en";
import { learnEn } from "../src/shared/i18n/learn.en";
import { LOCALES } from "../src/shared/i18n";
import {
  EMPTY_PROGRESS,
  lessonComplete,
  lessonProgress,
  lessonSets,
  nextLesson,
  recordPracticed,
  registerLessonSet,
  type LessonStep,
} from "../src/shared/lessons";
import { findCommand } from "../src/shared/slashCommands";
import { Controller } from "../src/webview/app/controller";
import { appReducer, initialState, type AppAction, type AppState } from "../src/webview/app/state";
import { BaseBridge } from "../src/webview/host";
import { localeByCode, translate } from "../src/shared/i18n";
import type { InitState, WebviewToHost } from "../src/shared/messages";

const lessons = lessonSets().flatMap((s) => s.lessons);
const steps = lessons.flatMap((l) => l.steps.map((s) => ({ lesson: l.id, step: s })));
const keysOf = (s: LessonStep): string[] =>
  s.kind === "read" ? [s.textKey] : s.kind === "try" ? [s.textKey, s.draftKey] : [s.questionKey, s.explainKey];

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

  test("every lesson text exists in every locale, and no lesson text is unused", () => {
    const used = new Set<string>(["learn.setBasics"]);
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

  test("a lesson is complete when its commands were run and its quizzes answered right", () => {
    let p = EMPTY_PROGRESS;
    expect(lessonProgress(basics, p)).toEqual({ done: 0, total: 3 });
    p = recordPracticed(recordPracticed(p, "status"), "pwd");
    p = { ...p, quizzes: { "basics.folder": "status" } };
    expect(lessonComplete(basics, p)).toBe(false);
    p = { ...p, quizzes: { "basics.folder": "pwd" } };
    expect(lessonComplete(basics, p)).toBe(true);
    expect(nextLesson(p)?.id).toBe("sessions");
  });

  test("recording a command twice keeps the same object", () => {
    const p = recordPracticed(EMPTY_PROGRESS, "status");
    expect(recordPracticed(p, "status")).toBe(p);
  });

  test("a registered lesson set is appended without touching the built-in one", () => {
    const before = lessonSets().length;
    registerLessonSet({ id: "extra", titleKey: "learn.setBasics", lessons: [] });
    expect(lessonSets()).toHaveLength(before + 1);
    expect(lessonSets()[0]!.id).toBe("suffice-basics");
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
    expect(env.bridge.posts).toContainEqual({ type: "setState", key: "learning", value: { practiced: ["pwd"], quizzes: {}, explored: [] } });
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
    expect(env.state.init!.learning).toEqual({ practiced: [], quizzes: { "basics.folder": "pwd" }, explored: ["suffice-tui:plan"] });
    env.ctl.resetLearning();
    expect(env.state.init!.learning).toEqual(EMPTY_PROGRESS);
  });
});
