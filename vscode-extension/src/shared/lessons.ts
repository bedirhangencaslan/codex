// Slash command lessons (goal item 3): the teaching half of the command guide; the reference
// list is the other half.
//
// A lesson is a short sequence of steps:
//   read  - one explanation paragraph
//   try   - run a real command in the chat composer; the step completes when the controller
//           actually runs that command (LearningProgress.practiced), not when a button is clicked
//   quiz  - pick the command for a situation; exactly one option is right
// Lessons are grouped in sets. The built-in set teaches the Suffice commands the extension runs
// itself; registerLessonSet() adds more (a project's own commands, a later course) without
// changing the screen. Texts: `learn.<lesson>.*` in learn.en.ts / learn.tr.ts.
import type { MessageKey } from "./i18n";

export type LessonStep =
  | { kind: "read"; textKey: MessageKey }
  | { kind: "try"; command: string; textKey: MessageKey; draftKey: MessageKey }
  | { kind: "quiz"; id: string; questionKey: MessageKey; options: string[]; answer: string; explainKey: MessageKey };

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
  lessons: Lesson[];
}

/** What the user has done, persisted in globalState (`learning`). */
export interface LearningProgress {
  /** Command names the user has run from the extension's composer. */
  practiced: string[];
  /** quiz id -> the option the user picked last. */
  quizzes: Record<string, string>;
  /** Reference entries the user has opened (`<set>:<command>`). */
  explored: string[];
}

export const EMPTY_PROGRESS: LearningProgress = { practiced: [], quizzes: {}, explored: [] };

const k = (lesson: string, part: string) => `learn.${lesson}.${part}` as MessageKey;
const read = (lesson: string, part: string): LessonStep => ({ kind: "read", textKey: k(lesson, part) });
const tryIt = (lesson: string, command: string): LessonStep => ({
  kind: "try",
  command,
  textKey: k(lesson, `try.${command}`),
  draftKey: k(lesson, `try.${command}.draft`),
});
const quiz = (lesson: string, id: string, options: string[], answer: string): LessonStep => ({
  kind: "quiz",
  id: `${lesson}.${id}`,
  questionKey: k(lesson, `quiz.${id}`),
  options,
  answer,
  explainKey: k(lesson, `quiz.${id}.explain`),
});

const lesson = (id: string, commands: string[], steps: LessonStep[]): Lesson => ({
  id,
  titleKey: k(id, "title"),
  introKey: k(id, "intro"),
  commands,
  steps,
});

const builtIn: LessonSet = {
  id: "suffice-basics",
  titleKey: "learn.setBasics",
  lessons: [
    lesson("basics", ["status", "pwd"], [
      read("basics", "what"),
      read("basics", "popup"),
      tryIt("basics", "status"),
      tryIt("basics", "pwd"),
      quiz("basics", "folder", ["status", "pwd", "new"], "pwd"),
    ]),
    lesson("sessions", ["new", "resume", "rename", "compact", "copy"], [
      read("sessions", "fresh"),
      tryIt("sessions", "rename"),
      read("sessions", "compact"),
      tryIt("sessions", "resume"),
      quiz("sessions", "unrelated", ["compact", "new", "resume"], "new"),
      quiz("sessions", "long", ["compact", "new", "copy"], "compact"),
    ]),
    lesson("modes", ["plan", "goal", "review"], [
      read("modes", "plan"),
      tryIt("modes", "plan"),
      read("modes", "goal"),
      tryIt("modes", "goal"),
      read("modes", "review"),
      quiz("modes", "approach", ["goal", "plan", "review"], "plan"),
      quiz("modes", "longTask", ["goal", "plan", "review"], "goal"),
    ]),
    lesson("cost", ["invisible", "mention", "skills", "status"], [
      read("cost", "context"),
      read("cost", "invisible"),
      tryIt("cost", "invisible"),
      read("cost", "mention"),
      tryIt("cost", "mention"),
      tryIt("cost", "skills"),
      quiz("cost", "side", ["invisible", "new", "compact"], "invisible"),
    ]),
    lesson("control", ["model", "permissions"], [
      read("control", "model"),
      tryIt("control", "model"),
      read("control", "permissions"),
      tryIt("control", "permissions"),
      quiz("control", "readOnly", ["model", "permissions", "status"], "permissions"),
    ]),
    lesson("terminal", ["vim", "theme", "mcp", "diff"], [
      read("terminal", "why"),
      read("terminal", "where"),
      quiz("terminal", "which", ["plan", "vim", "invisible"], "vim"),
    ]),
  ],
};

const sets: LessonSet[] = [builtIn];

export function lessonSets(): readonly LessonSet[] {
  return sets;
}

export function registerLessonSet(set: LessonSet): void {
  const existing = sets.findIndex((s) => s.id === set.id);
  if (existing >= 0) sets[existing] = set;
  else sets.push(set);
}

/** A step is done when its command was run, or its quiz was answered correctly. Reading is done by being shown. */
export function stepDone(step: LessonStep, progress: LearningProgress): boolean {
  if (step.kind === "try") return progress.practiced.includes(step.command);
  if (step.kind === "quiz") return progress.quizzes[step.id] === step.answer;
  return true;
}

export function lessonProgress(lesson: Lesson, progress: LearningProgress): { done: number; total: number } {
  const graded = lesson.steps.filter((s) => s.kind !== "read");
  return { done: graded.filter((s) => stepDone(s, progress)).length, total: graded.length };
}

export function lessonComplete(lesson: Lesson, progress: LearningProgress): boolean {
  const { done, total } = lessonProgress(lesson, progress);
  return done === total;
}

/** The first lesson not yet complete, across all sets; undefined when everything is done. */
export function nextLesson(progress: LearningProgress): Lesson | undefined {
  return sets.flatMap((s) => s.lessons).find((l) => !lessonComplete(l, progress));
}

/** Records a command run from the composer; returns the new progress (unchanged object if already known). */
export function recordPracticed(progress: LearningProgress, command: string): LearningProgress {
  return progress.practiced.includes(command) ? progress : { ...progress, practiced: [...progress.practiced, command] };
}
