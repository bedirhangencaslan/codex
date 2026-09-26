// The Suffice course (goal item 3): everything the extension offers, taught in three levels.
// The command reference list is the other half of the screen.
//
// A lesson is a short sequence of steps:
//   read    - one explanation paragraph
//   try     - run a real command in the chat composer; the step completes when the controller
//             actually runs that command (LearningProgress.practiced), not when a button is clicked
//   show    - open the panel or screen the lesson talks about; completes when it was opened from
//             the lesson (LearningProgress.opened)
//   quiz    - pick the command for a situation; exactly one option is right
//   choice  - pick the right statement; exactly one option is right
// The built-in course asks no questions (the owner's choice); quiz and choice remain for
// registered sets. A lesson with nothing to run or open is done once it has been read.
// Lessons are grouped in sets, one per level. registerLessonSet() adds more (a project's own
// commands, a later course) without changing the screen. Texts: `learn.<lesson>.*` in
// learn.en.ts / learn.tr.ts.
import type { MessageKey } from "./i18n";

/** Where a `show` step takes the user: a panel under the chat box, or a screen. */
export type ShowTarget =
  | "panel:mode"
  | "panel:skills"
  | "panel:files"
  | "panel:prefs"
  | "panel:model"
  | "screen:history"
  | "screen:skills"
  | "screen:api"
  | "screen:settings";

export type LessonStep =
  | { kind: "read"; textKey: MessageKey }
  | { kind: "try"; command: string; textKey: MessageKey; draftKey: MessageKey }
  | { kind: "show"; target: ShowTarget; textKey: MessageKey }
  | { kind: "quiz"; id: string; questionKey: MessageKey; options: string[]; answer: string; explainKey: MessageKey }
  | { kind: "choice"; id: string; questionKey: MessageKey; optionKeys: MessageKey[]; answer: number; explainKey: MessageKey };

export interface Lesson {
  id: string;
  titleKey: MessageKey;
  introKey: MessageKey;
  /** Commands the lesson covers, shown as chips and linked to the reference list. */
  commands: string[];
  steps: LessonStep[];
}

export interface LessonSet {
  id: string;
  titleKey: MessageKey;
  /** One line under the level's name. */
  descKey?: MessageKey;
  lessons: Lesson[];
}

/** What the user has done, persisted in globalState (`learning`). */
export interface LearningProgress {
  /** Command names the user has run from the extension's composer. */
  practiced: string[];
  /** quiz id -> the option the user picked last (a command, or a choice's index). */
  quizzes: Record<string, string>;
  /** Reference entries the user has opened (`<set>:<command>`). */
  explored: string[];
  /** Panels and screens opened from a lesson's `show` step, and `lesson:<id>` for lessons read.
   * Missing in progress saved before it existed. */
  opened?: string[];
}

export const EMPTY_PROGRESS: LearningProgress = { practiced: [], quizzes: {}, explored: [], opened: [] };

const k = (lesson: string, part: string) => `learn.${lesson}.${part}` as MessageKey;
const read = (lesson: string, part: string): LessonStep => ({ kind: "read", textKey: k(lesson, part) });
const tryIt = (lesson: string, command: string): LessonStep => ({
  kind: "try",
  command,
  textKey: k(lesson, `try.${command}`),
  draftKey: k(lesson, `try.${command}.draft`),
});
const show = (lesson: string, target: ShowTarget): LessonStep => ({
  kind: "show",
  target,
  textKey: k(lesson, `show.${target.replace(":", "_")}`),
});
const lesson = (id: string, commands: string[], steps: LessonStep[]): Lesson => ({
  id,
  titleKey: k(id, "title"),
  introKey: k(id, "intro"),
  commands,
  steps,
});

const beginner: LessonSet = {
  id: "suffice-beginner",
  titleKey: "learn.levelBeginner",
  descKey: "learn.levelBeginnerDesc",
  lessons: [
    lesson("basics", ["status", "pwd"], [
      read("basics", "what"),
      read("basics", "popup"),
      tryIt("basics", "status"),
      tryIt("basics", "pwd"),
    ]),
    lesson("chat", [], [
      read("chat", "send"),
      read("chat", "context"),
      read("chat", "stop"),
      read("chat", "approvals"),
    ]),
    lesson("screen", [], [
      read("screen", "meters"),
      read("screen", "cells"),
      read("screen", "diffs"),
      read("screen", "reasoning"),
      show("screen", "screen:settings"),
    ]),
    lesson("sessions", ["new", "resume", "rename", "compact"], [
      read("sessions", "fresh"),
      tryIt("sessions", "rename"),
      read("sessions", "compact"),
      tryIt("sessions", "resume"),
    ]),
  ],
};

const intermediate: LessonSet = {
  id: "suffice-intermediate",
  titleKey: "learn.levelIntermediate",
  descKey: "learn.levelIntermediateDesc",
  lessons: [
    lesson("modes", ["plan", "goal", "review"], [
      read("modes", "plan"),
      tryIt("modes", "plan"),
      read("modes", "goal"),
      tryIt("modes", "goal"),
      read("modes", "review"),
    ]),
    lesson("control", ["model", "permissions"], [
      read("control", "model"),
      tryIt("control", "model"),
      read("control", "permissions"),
      tryIt("control", "permissions"),
    ]),
    lesson("files", ["mention"], [
      read("files", "why"),
      read("files", "block"),
      read("files", "select"),
      read("files", "locked"),
      show("files", "panel:files"),
    ]),
    lesson("skills", ["skills"], [
      read("skills", "what"),
      read("skills", "select"),
      read("skills", "global"),
      show("skills", "panel:skills"),
      show("skills", "screen:skills"),
    ]),
    lesson("prefs", [], [
      read("prefs", "what"),
      read("prefs", "once"),
      show("prefs", "panel:prefs"),
    ]),
  ],
};

const advanced: LessonSet = {
  id: "suffice-advanced",
  titleKey: "learn.levelAdvanced",
  descKey: "learn.levelAdvancedDesc",
  lessons: [
    lesson("cost", ["invisible", "status"], [
      read("cost", "context"),
      read("cost", "cache"),
      read("cost", "breaks"),
      read("cost", "invisible"),
      tryIt("cost", "invisible"),
    ]),
    lesson("engine", [], [
      read("engine", "intro"),
      read("engine", "reasoning"),
      read("engine", "tools"),
      read("engine", "output"),
      read("engine", "patch"),
      read("engine", "parallel"),
      read("engine", "keepalive"),
    ]),
    lesson("compaction", ["compact"], [
      read("compaction", "what"),
      read("compaction", "slider"),
      read("compaction", "frozen"),
      read("compaction", "sync"),
      show("compaction", "screen:settings"),
    ]),
    lesson("keys", [], [
      read("keys", "keys"),
      read("keys", "prices"),
      show("keys", "screen:api"),
    ]),
    lesson("terminal", ["vim", "theme", "mcp", "diff"], [
      read("terminal", "why"),
      read("terminal", "where"),
    ]),
  ],
};

const sets: LessonSet[] = [beginner, intermediate, advanced];

export function lessonSets(): readonly LessonSet[] {
  return sets;
}

export function registerLessonSet(set: LessonSet): void {
  const existing = sets.findIndex((s) => s.id === set.id);
  if (existing >= 0) sets[existing] = set;
  else sets.push(set);
}

/** A step is done when its command was run, its target opened, or its quiz answered correctly. Reading is done by being shown. */
export function stepDone(step: LessonStep, progress: LearningProgress): boolean {
  if (step.kind === "try") return progress.practiced.includes(step.command);
  if (step.kind === "show") return (progress.opened ?? []).includes(step.target);
  if (step.kind === "quiz") return progress.quizzes[step.id] === step.answer;
  if (step.kind === "choice") return progress.quizzes[step.id] === String(step.answer);
  return true;
}

export function lessonProgress(lesson: Lesson, progress: LearningProgress): { done: number; total: number } {
  const graded = lesson.steps.filter((s) => s.kind !== "read");
  if (graded.length === 0) return { done: (progress.opened ?? []).includes(readMark(lesson.id)) ? 1 : 0, total: 1 };
  return { done: graded.filter((s) => stepDone(s, progress)).length, total: graded.length };
}

/** The mark a lesson without anything to do gets once it has been opened. */
export function readMark(lessonId: string): string {
  return `lesson:${lessonId}`;
}

export function lessonComplete(lesson: Lesson, progress: LearningProgress): boolean {
  const { done, total } = lessonProgress(lesson, progress);
  return done === total;
}

/** The first lesson not yet complete, across all sets; undefined when everything is done. */
export function nextLesson(progress: LearningProgress): Lesson | undefined {
  return sets.flatMap((s) => s.lessons).find((l) => !lessonComplete(l, progress));
}

/** The set a lesson belongs to. */
export function setOf(lessonId: string): LessonSet | undefined {
  return sets.find((s) => s.lessons.some((l) => l.id === lessonId));
}

/** Records a command run from the composer; returns the new progress (unchanged object if already known). */
export function recordPracticed(progress: LearningProgress, command: string): LearningProgress {
  return progress.practiced.includes(command) ? progress : { ...progress, practiced: [...progress.practiced, command] };
}

/** Records a panel or screen opened from a lesson, or a lesson read (readMark); returns the new progress (unchanged object if already known). */
export function recordOpened(progress: LearningProgress, target: ShowTarget | string): LearningProgress {
  const opened = progress.opened ?? [];
  return opened.includes(target) ? progress : { ...progress, opened: [...opened, target] };
}
