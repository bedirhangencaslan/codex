import type { MessageKey } from "./i18n";

/** A lesson holds no prose — only dictionary keys, so every locale carries
 * the full curriculum and a missing translation is a compile error. */
export interface LearnLesson {
  id: string;
  titleKey: MessageKey;
  bodyKey: MessageKey;
  tipKey?: MessageKey;
  /** Runs in an ephemeral thread via DataProvider.runCommandTrial. */
  tryCommand?: string;
}

export interface LearnModule {
  id: string;
  titleKey: MessageKey;
  descKey: MessageKey;
  lessons: LearnLesson[];
}

/** The curriculum. Adding a module = one entry here + its dictionary keys;
 * the Learn screen renders whatever this list contains. */
export const LEARN_MODULES: LearnModule[] = [
  {
    id: "slash",
    titleKey: "learn.slash.title",
    descKey: "learn.slash.desc",
    lessons: [
      {
        id: "intro",
        titleKey: "learn.slash.intro.title",
        bodyKey: "learn.slash.intro.body",
        tipKey: "learn.slash.intro.tip",
      },
      {
        id: "status",
        titleKey: "learn.slash.status.title",
        bodyKey: "learn.slash.status.body",
        tipKey: "learn.slash.status.tip",
        tryCommand: "/status",
      },
      {
        id: "context",
        titleKey: "learn.slash.context.title",
        bodyKey: "learn.slash.context.body",
        tipKey: "learn.slash.context.tip",
        tryCommand: "/recap",
      },
      {
        id: "sessions",
        titleKey: "learn.slash.sessions.title",
        bodyKey: "learn.slash.sessions.body",
        tipKey: "learn.slash.sessions.tip",
        tryCommand: "/fork",
      },
      {
        id: "skills",
        titleKey: "learn.slash.skills.title",
        bodyKey: "learn.slash.skills.body",
        tipKey: "learn.slash.skills.tip",
        tryCommand: "/skills",
      },
      {
        id: "safety",
        titleKey: "learn.slash.safety.title",
        bodyKey: "learn.slash.safety.body",
        tipKey: "learn.slash.safety.tip",
        tryCommand: "/diff",
      },
    ],
  },
];

const PROGRESS_PREFIX = "suffice.learn.";

export function loadProgress(moduleId: string): Set<string> {
  try {
    const raw = globalThis.localStorage?.getItem(PROGRESS_PREFIX + moduleId);
    if (raw) return new Set(JSON.parse(raw) as string[]);
  } catch {
    // unreadable progress is just empty progress
  }
  return new Set();
}

export function saveProgress(moduleId: string, done: Set<string>): void {
  try {
    globalThis.localStorage?.setItem(PROGRESS_PREFIX + moduleId, JSON.stringify([...done]));
  } catch {
    // best-effort only
  }
}
