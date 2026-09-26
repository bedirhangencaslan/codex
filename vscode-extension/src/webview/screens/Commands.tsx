// Goal item 3: the Suffice course. Two tabs:
//   Lessons   - everything the extension offers, in three levels (shared/lessons.ts): read, run the
//               real command in the chat box, open the panel or screen being taught. A "try" step
//               completes only when the controller actually runs the command and a "show" step when
//               the lesson opened its target, so progress means the user did it.
//   Reference - every command of every registered set, grouped by category.
// Both render whatever sets are registered; new lessons, levels or command sets need no change here.
// Progress lives in the host's globalState (`learning`); the open tab, level and lesson in the view state.
import { useEffect, useState } from "react";
import type { MessageKey } from "../../shared/i18n";
import {
  EMPTY_PROGRESS,
  lessonComplete,
  lessonProgress,
  lessonSets,
  nextLesson,
  setOf,
  stepDone,
  type LearningProgress,
  type Lesson,
  type LessonSet,
  type LessonStep,
  type ShowTarget,
} from "../../shared/lessons";
import { CATEGORY_ORDER, commandSets, type SlashCommandEntry } from "../../shared/slashCommands";
import { useApp } from "../app/context";
import type { AppState } from "../app/state";
import { Icon } from "../components/icons";
import { Badge, Button, Notice, SearchBox } from "../components/ui";

type Tab = "lessons" | "reference";

const learning = (state: AppState): LearningProgress => state.init?.learning ?? EMPTY_PROGRESS;

interface ViewState {
  commandsTab?: Tab;
  openLesson?: string | null;
  level?: string;
}

export function CommandsScreen() {
  const { t, ctl } = useApp();
  const view = ctl.bridge.viewState<ViewState>() ?? {};
  const [tab, setTabState] = useState<Tab>(view.commandsTab ?? "lessons");
  const [openLesson, setOpenLessonState] = useState<string | null>(view.openLesson ?? null);
  const [filter, setFilter] = useState("");

  const remember = (patch: ViewState) => ctl.bridge.setViewState<ViewState>({ ...(ctl.bridge.viewState<ViewState>() ?? {}), ...patch });
  const setTab = (next: Tab) => {
    setTabState(next);
    remember({ commandsTab: next });
  };
  const setOpenLesson = (id: string | null) => {
    setOpenLessonState(id);
    remember({ openLesson: id });
  };
  const lookUp = (command: string) => {
    setFilter(command);
    setTab("reference");
  };

  return (
    <div className="sf-screen">
      <header className="sf-screen-head">
        <h1>{t("commands.title")}</h1>
        <p className="sf-muted">{t("commands.intro")}</p>
      </header>
      <div className="sf-segmented sf-tabs" role="tablist">
        {(["lessons", "reference"] as const).map((id) => (
          <button key={id} type="button" role="tab" aria-selected={tab === id} className={tab === id ? "is-active" : ""} onClick={() => setTab(id)}>
            {t(id === "lessons" ? "commands.tabLessons" : "commands.tabReference")}
          </button>
        ))}
      </div>
      {tab === "lessons" ? (
        <Lessons openLesson={openLesson} setOpenLesson={setOpenLesson} lookUp={lookUp} remember={remember} initialLevel={view.level} />
      ) : (
        <Reference filter={filter} setFilter={setFilter} />
      )}
    </div>
  );
}

// --- lessons ---------------------------------------------------------------------------------

function setProgress(set: LessonSet, progress: LearningProgress): { done: number; total: number } {
  return { done: set.lessons.filter((l) => lessonComplete(l, progress)).length, total: set.lessons.length };
}

function Lessons({
  openLesson,
  setOpenLesson,
  lookUp,
  remember,
  initialLevel,
}: {
  openLesson: string | null;
  setOpenLesson: (id: string | null) => void;
  lookUp: (c: string) => void;
  remember: (patch: ViewState) => void;
  initialLevel: string | undefined;
}) {
  const { t, ctl, state } = useApp();
  const progress = learning(state);
  const sets = lessonSets();
  const all = sets.flatMap((s) => s.lessons);
  const next = nextLesson(progress);
  const [level, setLevelState] = useState<string>(initialLevel ?? (next ? setOf(next.id)?.id : undefined) ?? sets[0]!.id);
  const setLevel = (id: string) => {
    setLevelState(id);
    remember({ level: id });
  };
  const current = all.find((l) => l.id === openLesson);
  if (current) return <LessonView lesson={current} onBack={() => setOpenLesson(null)} onOpen={setOpenLesson} lookUp={lookUp} />;

  const doneAll = all.filter((l) => lessonComplete(l, progress)).length;
  const active = sets.find((s) => s.id === level) ?? sets[0]!;
  const nextSet = next ? setOf(next.id) : undefined;
  return (
    <div className="sf-stack">
      <section className="sf-course-hero">
        <div className="sf-course-hero-top">
          <span className="sf-course-hero-label">{t("learn.overall")}</span>
          <ProgressBar done={doneAll} total={all.length} label={t("learn.lessonsDone", { done: doneAll, total: all.length })} wide />
        </div>
        {next ? (
          <button type="button" className="sf-course-next" onClick={() => setOpenLesson(next.id)}>
            <span className="sf-course-next-main">
              {nextSet && <span className="sf-course-level-tag">{t(nextSet.titleKey)}</span>}
              <span className="sf-lesson-title">{t(next.titleKey)}</span>
              <span className="sf-lesson-intro">{t(next.introKey)}</span>
            </span>
            <span className="sf-course-next-go">
              {t(lessonProgress(next, progress).done > 0 ? "learn.continue" : "learn.start")}
              <Icon name="chevronRight" size={13} />
            </span>
          </button>
        ) : (
          <Notice tone="success">{t("learn.allDone")}</Notice>
        )}
      </section>

      <div className="sf-level-tabs" role="tablist" aria-label={t("learn.levels")}>
        {sets.map((set, i) => {
          const p = setProgress(set, progress);
          return (
            <button key={set.id} type="button" role="tab" aria-selected={set.id === active.id} className={`sf-level-tab ${set.id === active.id ? "is-active" : ""} ${p.done === p.total ? "is-complete" : ""}`} onClick={() => setLevel(set.id)}>
              <span className="sf-level-dots" aria-hidden="true">
                {Array.from({ length: Math.min(3, i + 1) }, (_, d) => (
                  <i key={d} />
                ))}
              </span>
              <span className="sf-level-name">{t(set.titleKey)}</span>
              <span className="sf-level-count">
                {p.done}/{p.total}
              </span>
            </button>
          );
        })}
      </div>

      <section className="sf-command-set">
        {active.descKey && <p className="sf-muted sf-level-desc">{t(active.descKey)}</p>}
        <ol className="sf-lesson-list">
          {active.lessons.map((lesson, i) => {
            const p = lessonProgress(lesson, progress);
            const complete = p.done === p.total;
            const isNext = next?.id === lesson.id;
            return (
              <li key={lesson.id}>
                <button type="button" className={`sf-lesson-card ${isNext ? "is-next" : ""} ${complete ? "is-complete" : ""}`} onClick={() => setOpenLesson(lesson.id)}>
                  <span className="sf-lesson-num" aria-hidden="true">
                    {complete ? <Icon name="check" size={13} /> : i + 1}
                  </span>
                  <span className="sf-lesson-main">
                    <span className="sf-lesson-title">{t(lesson.titleKey)}</span>
                    <span className="sf-lesson-intro">{t(lesson.introKey)}</span>
                    {lesson.commands.length > 0 && (
                      <span className="sf-lesson-cmds">
                        {lesson.commands.map((c) => (
                          <code key={c}>/{c}</code>
                        ))}
                      </span>
                    )}
                  </span>
                  <span className="sf-lesson-side">
                    {complete ? <Badge tone="success">{t("learn.completeBadge")}</Badge> : <span className="sf-muted">{t("learn.stepsDone", p)}</span>}
                    <span className="sf-lesson-go">{t(complete ? "learn.again" : p.done > 0 ? "learn.continue" : "learn.start")}</span>
                  </span>
                </button>
              </li>
            );
          })}
        </ol>
      </section>

      {(progress.practiced.length > 0 || Object.keys(progress.quizzes).length > 0 || (progress.opened ?? []).length > 0) && (
        <div>
          <Button variant="ghost" icon="refresh" onClick={() => ctl.resetLearning()}>
            {t("learn.reset")}
          </Button>
        </div>
      )}
    </div>
  );
}

function LessonView({ lesson, onBack, onOpen, lookUp }: { lesson: Lesson; onBack: () => void; onOpen: (id: string) => void; lookUp: (c: string) => void }) {
  const { t, ctl, state } = useApp();
  useEffect(() => ctl.markLessonRead(lesson.id), [lesson.id]);
  const progress = learning(state);
  const p = lessonProgress(lesson, progress);
  const complete = p.done === p.total;
  const all = lessonSets().flatMap((s) => s.lessons);
  const index = all.indexOf(lesson);
  const previous = all[index - 1];
  const following = all[index + 1];
  const set = setOf(lesson.id);

  return (
    <article className="sf-stack">
      <div className="sf-row sf-wrap sf-lesson-crumbs">
        <Button variant="ghost" icon="back" onClick={onBack}>
          {t("learn.back")}
        </Button>
        {set && <span className="sf-course-level-tag">{t(set.titleKey)}</span>}
      </div>
      <header className="sf-lesson-head">
        <h2>{t(lesson.titleKey)}</h2>
        <ProgressBar done={p.done} total={p.total} label={t("learn.stepsDone", p)} />
      </header>
      <p className="sf-muted">{t(lesson.introKey)}</p>
      {lesson.commands.length > 0 && (
        <div className="sf-row sf-wrap">
          <span className="sf-muted">{t("learn.covers")}:</span>
          {lesson.commands.map((c) => (
            <button key={c} type="button" className="sf-cmd-link" onClick={() => lookUp(c)}>
              /{c}
            </button>
          ))}
        </div>
      )}
      <ol className="sf-steps">
        {lesson.steps.map((step, i) => (
          <li key={i} className={`sf-step sf-step-${step.kind} ${step.kind !== "read" && stepDone(step, progress) ? "is-done" : ""}`}>
            <Step step={step} />
          </li>
        ))}
      </ol>
      {complete && <Notice tone="success">{t("learn.lessonComplete")}</Notice>}
      <nav className="sf-lesson-nav">
        {previous ? (
          <Button variant="ghost" icon="back" onClick={() => onOpen(previous.id)}>
            {t(previous.titleKey)}
          </Button>
        ) : (
          <span />
        )}
        {following && (
          <Button variant={complete ? "primary" : "secondary"} onClick={() => onOpen(following.id)}>
            {t("learn.nextLesson", { title: t(following.titleKey) })}
            <Icon name="chevronRight" size={13} />
          </Button>
        )}
      </nav>
    </article>
  );
}

const SHOW_ICON: Record<ShowTarget, "folder" | "sparkles" | "pencil" | "chip" | "layers" | "history" | "key" | "gear"> = {
  "panel:mode": "layers",
  "panel:skills": "sparkles",
  "panel:files": "folder",
  "panel:prefs": "pencil",
  "panel:model": "chip",
  "screen:history": "history",
  "screen:skills": "sparkles",
  "screen:api": "key",
  "screen:settings": "gear",
};

function Step({ step }: { step: LessonStep }) {
  const { t, ctl, state } = useApp();
  const progress = learning(state);
  if (step.kind === "read") return <p className="sf-step-text">{t(step.textKey)}</p>;

  if (step.kind === "try") {
    const done = stepDone(step, progress);
    return (
      <div className="sf-step-box">
        <div className="sf-step-title">
          <Icon name="terminal" size={14} />
          <span>{t(step.textKey)}</span>
        </div>
        <div className="sf-command-example">
          <code>{t(step.draftKey)}</code>
          {done ? (
            <Badge tone="success">
              <Icon name="check" size={11} /> {t("learn.practiced")}
            </Badge>
          ) : (
            <Button variant="primary" icon="send" onClick={() => ctl.setDraft(t(step.draftKey))}>
              {t("learn.tryButton")}
            </Button>
          )}
        </div>
        {!done && <p className="sf-muted sf-small">{t("learn.tryHint")}</p>}
      </div>
    );
  }

  if (step.kind === "show") {
    const done = stepDone(step, progress);
    return (
      <div className="sf-step-box">
        <div className="sf-step-title">
          <Icon name={SHOW_ICON[step.target]} size={14} />
          <span>{t(step.textKey)}</span>
        </div>
        <div className="sf-row sf-wrap">
          <Button variant={done ? "secondary" : "primary"} icon="chevronRight" onClick={() => ctl.showLessonTarget(step.target)}>
            {t("learn.showButton")}
          </Button>
          {done && (
            <Badge tone="success">
              <Icon name="check" size={11} /> {t("learn.seen")}
            </Badge>
          )}
        </div>
      </div>
    );
  }

  const answered = progress.quizzes[step.id];
  const options = step.kind === "quiz" ? step.options.map((o) => ({ value: o, label: `/${o}` })) : step.optionKeys.map((key, i) => ({ value: String(i), label: t(key) }));
  const answer = step.kind === "quiz" ? step.answer : String(step.answer);
  const correct = answered === answer;
  return (
    <div className="sf-step-box">
      <div className="sf-step-title">
        <Icon name="target" size={14} />
        <span>
          <strong>{t("learn.quizTitle")}: </strong>
          {t(step.questionKey)}
        </span>
      </div>
      <div className={`sf-quiz-options ${step.kind === "choice" ? "is-statements" : ""}`} role="radiogroup" aria-label={t(step.questionKey)}>
        {options.map((option) => {
          const picked = answered === option.value;
          const tone = picked ? (option.value === answer ? "is-right" : "is-wrong") : "";
          return (
            <button key={option.value} type="button" role="radio" aria-checked={picked} className={`sf-quiz-option ${tone}`} disabled={correct} onClick={() => ctl.answerQuiz(step.id, option.value)}>
              {option.label}
            </button>
          );
        })}
      </div>
      {answered !== undefined && (correct ? <Notice tone="success">{t("learn.correct")} {t(step.explainKey)}</Notice> : <Notice tone="warning">{t("learn.wrong")}</Notice>)}
    </div>
  );
}

function ProgressBar({ done, total, label, wide = false }: { done: number; total: number; label: string; wide?: boolean }) {
  return (
    <span className={`sf-progress ${wide ? "is-wide" : ""}`}>
      <span className="sf-progress-bar" aria-hidden="true">
        <span style={{ width: `${(done / Math.max(1, total)) * 100}%` }} />
      </span>
      {label}
    </span>
  );
}

// --- reference -------------------------------------------------------------------------------

function Reference({ filter, setFilter }: { filter: string; setFilter: (v: string) => void }) {
  const { t, ctl, state } = useApp();
  const explored = new Set(learning(state).explored);
  const q = filter.toLowerCase().replace(/^\//, "");
  // A lookup from a lesson ("/plan" chip) opens that command directly.
  const exact = commandSets().flatMap((s) => s.commands.map((c) => ({ set: s.id, c }))).find((x) => x.c.name === q);
  const [open, setOpen] = useState<string | null>(exact ? `${exact.set}:${exact.c.name}` : null);

  const expand = (key: string) => {
    setOpen(open === key ? null : key);
    ctl.markExplored(key);
  };

  const matches = (c: SlashCommandEntry) =>
    !q || c.name.includes(q) || c.aliases.some((a) => a.includes(q)) || t(c.descKey).toLowerCase().includes(q);

  return (
    <>
      <SearchBox value={filter} onChange={setFilter} placeholder={t("commands.search")} />
      {commandSets().map((set) => {
        const total = set.commands.length;
        const done = set.commands.filter((c) => explored.has(`${set.id}:${c.name}`)).length;
        return (
          <section key={set.id} className="sf-command-set">
            <header className="sf-command-set-head">
              <h2>{t(set.titleKey)}</h2>
              <span title={set.source}>
                <ProgressBar done={done} total={total} label={t("commands.lessonProgress", { done, total })} />
              </span>
            </header>
            {CATEGORY_ORDER.map((category) => {
              const commands = set.commands.filter((c) => c.category === category && matches(c));
              if (commands.length === 0) return null;
              return (
                <div key={category} className="sf-command-group">
                  <h3>{t(`commands.category.${category}` as MessageKey)}</h3>
                  <ul className="sf-command-list">
                    {commands.map((c) => {
                      const key = `${set.id}:${c.name}`;
                      const isOpen = open === key;
                      return (
                        <li key={c.name} className={`sf-command ${isOpen ? "is-open" : ""}`}>
                          <button type="button" className="sf-command-head" onClick={() => expand(key)} aria-expanded={isOpen}>
                            <code className="sf-command-name">/{c.name}</code>
                            {explored.has(key) && <Icon name="check" size={12} className="sf-command-seen" />}
                            <span className="sf-command-desc">{t(c.descKey)}</span>
                            <Icon name={isOpen ? "chevronDown" : "chevronRight"} size={12} />
                          </button>
                          {isOpen && (
                            <div className="sf-command-body">
                              <div className="sf-row sf-wrap">
                                <Badge tone={c.guiAction ? "success" : "neutral"}>{c.guiAction ? t("commands.inExtension") : t("commands.terminalOnly")}</Badge>
                                {c.availableDuringTask && <Badge>{t("commands.flagDuringTask")}</Badge>}
                                {c.inlineArgs && <Badge>{t("commands.flagArgs")}</Badge>}
                                {c.availableInSideConversation && <Badge>{t("commands.flagSide")}</Badge>}
                                {c.availableWhenThreadUnavailable && <Badge>{t("commands.flagNoThread")}</Badge>}
                                {c.aliases.map((a) => (
                                  <Badge key={a}>/{a}</Badge>
                                ))}
                              </div>
                              {c.whenKey && (
                                <p>
                                  <strong>{t("commands.whenToUse")}: </strong>
                                  {t(c.whenKey)}
                                </p>
                              )}
                              {c.exampleKey && (
                                <div className="sf-command-example">
                                  <span className="sf-muted">{t("commands.example")}</span>
                                  <code>{t(c.exampleKey)}</code>
                                  {c.guiAction && (
                                    <Button variant="primary" icon="send" onClick={() => ctl.setDraft(t(c.exampleKey!))}>
                                      {t("commands.tryIt")}
                                    </Button>
                                  )}
                                </div>
                              )}
                            </div>
                          )}
                        </li>
                      );
                    })}
                  </ul>
                </div>
              );
            })}
          </section>
        );
      })}
    </>
  );
}
