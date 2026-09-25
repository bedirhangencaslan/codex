// Goal item 3: the slash command course. Two tabs:
//   Lessons   - short lessons (shared/lessons.ts): read, run the real command in the chat box, then
//               a quick check. A "try" step completes only when the controller actually runs the
//               command, so progress means the user has done it, not just clicked.
//   Reference - every command of every registered set, grouped by category.
// Both render whatever sets are registered; new lessons or command sets need no change here.
// Progress lives in the host's globalState (`learning`), the open tab/lesson in the view state.
import { useState } from "react";
import type { MessageKey } from "../../shared/i18n";
import { EMPTY_PROGRESS, lessonComplete, lessonProgress, lessonSets, nextLesson, stepDone, type LearningProgress, type Lesson, type LessonStep } from "../../shared/lessons";
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
        <Lessons openLesson={openLesson} setOpenLesson={setOpenLesson} lookUp={lookUp} />
      ) : (
        <Reference filter={filter} setFilter={setFilter} />
      )}
    </div>
  );
}

// --- lessons ---------------------------------------------------------------------------------

function Lessons({ openLesson, setOpenLesson, lookUp }: { openLesson: string | null; setOpenLesson: (id: string | null) => void; lookUp: (c: string) => void }) {
  const { t, ctl, state } = useApp();
  const progress = learning(state);
  const all = lessonSets().flatMap((s) => s.lessons);
  const current = all.find((l) => l.id === openLesson);
  if (current) return <LessonView lesson={current} onBack={() => setOpenLesson(null)} onOpen={setOpenLesson} lookUp={lookUp} />;

  const next = nextLesson(progress);
  return (
    <div className="sf-stack">
      {lessonSets().map((set) => {
        const done = set.lessons.filter((l) => lessonComplete(l, progress)).length;
        return (
          <section key={set.id} className="sf-command-set">
            <header className="sf-command-set-head">
              <h2>{t(set.titleKey)}</h2>
              <ProgressBar done={done} total={set.lessons.length} label={t("learn.lessonsDone", { done, total: set.lessons.length })} />
            </header>
            <ol className="sf-lesson-list">
              {set.lessons.map((lesson, i) => {
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
                        <span className="sf-lesson-cmds">
                          {lesson.commands.map((c) => (
                            <code key={c}>/{c}</code>
                          ))}
                        </span>
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
        );
      })}
      {!next && <Notice tone="success">{t("learn.allDone")}</Notice>}
      {(progress.practiced.length > 0 || Object.keys(progress.quizzes).length > 0) && (
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
  const progress = learning(state);
  const p = lessonProgress(lesson, progress);
  const complete = p.done === p.total;
  const all = lessonSets().flatMap((s) => s.lessons);
  const after = all.slice(all.indexOf(lesson) + 1).find((l) => !lessonComplete(l, progress)) ?? all[all.indexOf(lesson) + 1];

  return (
    <article className="sf-stack">
      <div>
        <Button variant="ghost" icon="back" onClick={onBack}>
          {t("learn.back")}
        </Button>
      </div>
      <header className="sf-lesson-head">
        <h2>{t(lesson.titleKey)}</h2>
        <ProgressBar done={p.done} total={p.total} label={t("learn.stepsDone", p)} />
      </header>
      <p className="sf-muted">{t(lesson.introKey)}</p>
      <div className="sf-row sf-wrap">
        <span className="sf-muted">{t("learn.covers")}:</span>
        {lesson.commands.map((c) => (
          <button key={c} type="button" className="sf-cmd-link" onClick={() => lookUp(c)}>
            /{c}
          </button>
        ))}
      </div>
      <ol className="sf-steps">
        {lesson.steps.map((step, i) => (
          <li key={i} className={`sf-step sf-step-${step.kind} ${step.kind !== "read" && stepDone(step, progress) ? "is-done" : ""}`}>
            <Step step={step} />
          </li>
        ))}
      </ol>
      {complete && (
        <Notice tone="success">
          <div className="sf-stack-tight">
            <span>{t("learn.lessonComplete")}</span>
            {after && (
              <div>
                <Button variant="primary" icon="chevronRight" onClick={() => onOpen(after.id)}>
                  {t("learn.nextLesson", { title: t(after.titleKey) })}
                </Button>
              </div>
            )}
          </div>
        </Notice>
      )}
    </article>
  );
}

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

  const answered = progress.quizzes[step.id];
  const correct = answered === step.answer;
  return (
    <div className="sf-step-box">
      <div className="sf-step-title">
        <Icon name="target" size={14} />
        <span>
          <strong>{t("learn.quizTitle")}: </strong>
          {t(step.questionKey)}
        </span>
      </div>
      <div className="sf-quiz-options" role="radiogroup" aria-label={t(step.questionKey)}>
        {step.options.map((option) => {
          const picked = answered === option;
          const tone = picked ? (option === step.answer ? "is-right" : "is-wrong") : "";
          return (
            <button key={option} type="button" role="radio" aria-checked={picked} className={`sf-quiz-option ${tone}`} disabled={correct} onClick={() => ctl.answerQuiz(step.id, option)}>
              /{option}
            </button>
          );
        })}
      </div>
      {answered !== undefined && (correct ? <Notice tone="success">{t("learn.correct")} {t(step.explainKey)}</Notice> : <Notice tone="warning">{t("learn.wrong")}</Notice>)}
    </div>
  );
}

function ProgressBar({ done, total, label }: { done: number; total: number; label: string }) {
  return (
    <span className="sf-progress">
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
