import type { tr } from "./tr";

/** English. Typed against the Turkish reference dictionary: a missing key is
 * a compile error, an extra key is too. */
export const en: Record<keyof typeof tr, string> = {
  "locale.name": "English",

  "nav.sessions": "Sessions",
  "nav.learn": "Learn",
  "nav.skills": "Skills",
  "nav.commands": "Commands",
  "nav.styles": "Styles",
  "nav.aria": "Main navigation",
  "nav.language": "Interface language",

  "sessions.search": "Search threads…",
  "sessions.new": "New session",
  "sessions.untitled": "(untitled session)",
  "sessions.week": "this week",
  "sessions.avgCached": "avg cached",
  "sessions.fresh": "fresh tokens",
  "sessions.count": "sessions",
  "sessions.cacheNow": "cache · 420 s TTL",
  "sessions.requests": "req",
  "sessions.compactions": "compactions",

  "warmth.instant": "INSTANT",
  "warmth.fast": "FAST",
  "warmth.cold": "COLD",

  "rail.selected": "Selected Session",
  "rail.cachedInput": "CACHED INPUT",
  "rail.timeline": "Request Timeline",
  "rail.summary": "Summary",
  "rail.fresh": "fresh",
  "rail.cached": "cached",
  "rail.output": "output",
  "rail.keepAlive": "keep-alive",
  "gauge.cachedInput": "Cached input",

  "skills.title": "Skills",
  "skills.subtitle": "every block that enters the prompt is measured",
  "skills.promptLoad": "Prompt load",
  "skills.lighter": "lighter",
  "skills.tokens": "tokens",
  "skills.on": "on",
  "skills.off": "off",

  "commands.title": "Commands",
  "commands.subtitle": "safe tries run in an ephemeral thread, never saved to history",
  "commands.try": "Try",
  "commands.trialIdle": "Trial area",
  "commands.trialHint": "Pick a command on the left; it runs in an ephemeral thread, never saved to history.",

  "styles.title": "Styles",
  "styles.subtitle": "picking a style only changes the look; session data is untouched",
  "styles.apply": "Apply",
  "styles.active": "Active",
  "styles.thermal.name": "Thermal Instrument Pro",
  "styles.thermal.tagline": "cache warmth as the interface's physics",
  "styles.blueprint.name": "Blueprint Notebook",
  "styles.blueprint.tagline": "a measurement log on millimeter paper",
  "styles.abyss.name": "Abyssal Terminal",
  "styles.abyss.tagline": "dual-phosphor OLED terminal",

  "common.loading": "Loading…",
  "common.empty": "Nothing here yet",
  "common.mockBadge": "sample data",
  "common.liveBadge": "live: app-server",
  "common.trialFailed": "The trial could not finish",

  "learn.title": "Learn",
  "learn.subtitle": "short lessons; every try runs in an ephemeral thread, never saved to history",
  "learn.progress": "progress",
  "learn.lesson": "Lesson",
  "learn.prev": "Previous",
  "learn.next": "Next",
  "learn.markDone": "Mark as done",
  "learn.done": "Done",
  "learn.reset": "Reset progress",
  "learn.tryIt": "Try it now",
  "learn.tip": "Tip",
  "learn.backToModules": "Back to modules",

  "learn.slash.title": "/ Commands",
  "learn.slash.desc": "Learn the command language that runs the session, in six short lessons — from status to context management.",

  "learn.slash.intro.title": "What is a / command?",
  "learn.slash.intro.body":
    "Typing / in the composer opens the command list. These commands are not messages sent to the model; they operate the session itself: show status, summarize history, fork the conversation. Running a / command adds no tokens to the prompt — a distinction that matters for cost.",
  "learn.slash.intro.tip": "Don't remember the exact name? Type / and a few letters; the list filters as you type.",

  "learn.slash.status.title": "Read the state: /status",
  "learn.slash.status.body":
    "/status gives the session's identity at a glance: which model, which provider, cache warmth and token counters. The fresh/cached split is the bill itself: cached tokens are billed at a fifth of the fresh price. When a session feels slow or expensive, this is the first place to look.",
  "learn.slash.status.tip": "The speed indicator (⚡ Instant / ● Fast / ◌ Warming up) summarizes the same data live.",

  "learn.slash.context.title": "Manage context: /compact and /recap",
  "learn.slash.context.body":
    "As the conversation grows, the context window fills. /compact reduces history to a dense summary in a single request — but it is not free: in measurements, one compaction cost ~73k tokens of cold input. So trust the automatic compaction at the 80k threshold; run /compact by hand only when the topic deliberately changes. /recap writes a summary without deleting anything.",
  "learn.slash.context.tip": "Compaction summarizes tool output too; the model may re-read a file afterwards. That is normal.",

  "learn.slash.sessions.title": "Session lifecycle: /new, /resume, /fork",
  "learn.slash.sessions.body":
    "/new opens a clean slate, /resume continues a saved session where it left off. /fork is the least known but most powerful: it opens a new session with a copy of the history — when you want to attempt something risky, your main session stays intact.",
  "learn.slash.sessions.tip": "Changing direction deep into a long session? /fork + /compact is the cheapest route.",

  "learn.slash.skills.title": "Invoke skills: /skills and $name",
  "learn.slash.skills.body":
    "/skills lists the skills enabled for this project. You invoke one inside a message with $skill-name. Careful: every enabled skill adds its instruction block to every request's prompt — a measured cost. The Skills screen shows how many tokens each one adds; turn off the ones you don't use.",
  "learn.slash.skills.tip": "This project's own finding: turning off the irrelevant skills block was the one variable that survived measurement in both directions.",

  "learn.slash.safety.title": "Safe practice: ephemeral threads",
  "learn.slash.safety.body":
    "Every \"Try it now\" button on this screen runs the command in an ephemeral thread: never written to history, never shown in your session list, gone when it closes. There is nothing you can break here — try commands freely and get to know the shape of their output.",
  "learn.slash.safety.tip": "The same guarantee holds for the trial pane on the Commands screen.",
};
