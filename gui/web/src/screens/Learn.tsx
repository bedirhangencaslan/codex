import { useEffect, useMemo, useRef, useState } from "react";
import {
  LEARN_MODULES,
  fmtPercent,
  loadProgress,
  makeT,
  saveProgress,
  type DataProvider,
  type LearnModule,
  type Locale,
} from "@suffice/gui-core";

type TrialState = { output: string; running: boolean; error: string | null };

/** Guided curriculum: modules → lessons, all prose from the locale
 * dictionary. Tries run in an ephemeral thread; progress is local. */
export function Learn({ provider, locale }: { provider: DataProvider; locale: Locale }) {
  const t = useMemo(() => makeT(locale), [locale]);
  const [module, setModule] = useState<LearnModule | null>(null);
  const [lessonIx, setLessonIx] = useState(0);
  const [done, setDone] = useState<Set<string>>(new Set());
  const [trial, setTrial] = useState<TrialState | null>(null);
  const outRef = useRef<HTMLPreElement>(null);

  useEffect(() => {
    if (module) setDone(loadProgress(module.id));
    setLessonIx(0);
    setTrial(null);
  }, [module]);

  useEffect(() => {
    setTrial(null);
  }, [lessonIx]);

  useEffect(() => {
    outRef.current?.scrollTo({ top: outRef.current.scrollHeight });
  }, [trial?.output]);

  const markDone = (lessonId: string) => {
    if (!module) return;
    const next = new Set(done);
    next.add(lessonId);
    setDone(next);
    saveProgress(module.id, next);
  };

  const runTry = (command: string, lessonId: string) => {
    setTrial({ output: "", running: true, error: null });
    provider
      .runCommandTrial(command, (chunk) => setTrial((cur) => (cur ? { ...cur, output: cur.output + chunk } : cur)))
      .then(() => {
        setTrial((cur) => (cur ? { ...cur, running: false } : cur));
        markDone(lessonId);
      })
      .catch((e: unknown) =>
        setTrial((cur) => (cur ? { ...cur, running: false, error: e instanceof Error ? e.message : String(e) } : cur)),
      );
  };

  if (!module) {
    return (
      <>
        <div className="pagehead">
          <h1>{t("learn.title")}</h1>
        </div>
        <p style={{ color: "var(--sf-muted)", marginTop: 0, maxWidth: "68ch" }}>{t("learn.subtitle")}</p>
        <div className="modulegrid">
          {LEARN_MODULES.map((m) => {
            const progress = loadProgress(m.id);
            const pct = Math.round((progress.size / m.lessons.length) * 100);
            return (
              <button key={m.id} className="modulecard" onClick={() => setModule(m)}>
                <b>{t(m.titleKey)}</b>
                <span>{t(m.descKey)}</span>
                <span className="moduleprog">
                  <span className="bar" aria-hidden="true">
                    <i style={{ width: `${Math.max(2, pct)}%` }} />
                  </span>
                  <span className="num">
                    {progress.size}/{m.lessons.length} · {fmtPercent(locale, pct)} {t("learn.progress")}
                  </span>
                </span>
              </button>
            );
          })}
        </div>
      </>
    );
  }

  const lesson = module.lessons[lessonIx];
  const pct = Math.round((done.size / module.lessons.length) * 100);

  return (
    <>
      <div className="pagehead">
        <h1>
          {t(module.titleKey)}{" "}
          <small>
            · {t("learn.lesson")} {lessonIx + 1}/{module.lessons.length}
          </small>
        </h1>
        <button className="ghost" onClick={() => setModule(null)}>
          ← {t("learn.backToModules")}
        </button>
      </div>

      <div className="lessonwrap">
        <aside className="lessonlist" aria-label={t("learn.title")}>
          {module.lessons.map((l, i) => (
            <button
              key={l.id}
              className={`lessonitem${i === lessonIx ? " on" : ""}`}
              aria-current={i === lessonIx ? "step" : undefined}
              onClick={() => setLessonIx(i)}
            >
              <span className={`tick${done.has(l.id) ? " done" : ""}`} aria-hidden="true">
                {done.has(l.id) ? "✓" : i + 1}
              </span>
              {t(l.titleKey)}
            </button>
          ))}
          <div className="moduleprog" style={{ padding: "12px 14px" }}>
            <span className="bar" aria-hidden="true">
              <i style={{ width: `${Math.max(2, pct)}%` }} />
            </span>
            <span className="num">{fmtPercent(locale, pct)} {t("learn.progress")}</span>
          </div>
        </aside>

        <article className="lessonbody panel">
          <header>
            <h2>{t(lesson.titleKey)}</h2>
          </header>
          <div className="lessonprose">
            <p>{t(lesson.bodyKey)}</p>
            {lesson.tipKey && (
              <p className="lessontip">
                <b>{t("learn.tip")}:</b> {t(lesson.tipKey)}
              </p>
            )}
            {lesson.tryCommand && (
              <div className="lessontry">
                <button className="btn-primary" disabled={trial?.running} onClick={() => runTry(lesson.tryCommand!, lesson.id)}>
                  {t("learn.tryIt")}: <code>{lesson.tryCommand}</code>
                </button>
                {trial && (
                  <pre ref={outRef} className="trialout num" aria-live="polite">
                    {trial.output || (trial.running ? "…" : "")}
                    {trial.error && `\n⚠ ${t("common.trialFailed")}: ${trial.error}`}
                  </pre>
                )}
              </div>
            )}
          </div>
          <footer className="lessonnav">
            <button className="ghost" disabled={lessonIx === 0} onClick={() => setLessonIx(lessonIx - 1)}>
              ← {t("learn.prev")}
            </button>
            {done.has(lesson.id) ? (
              <span className="pill ok">
                <i />
                {t("learn.done")}
              </span>
            ) : (
              <button className="ghost" onClick={() => markDone(lesson.id)}>
                {t("learn.markDone")}
              </button>
            )}
            <button
              className="ghost"
              disabled={lessonIx === module.lessons.length - 1}
              onClick={() => setLessonIx(lessonIx + 1)}
            >
              {t("learn.next")} →
            </button>
          </footer>
        </article>
      </div>
    </>
  );
}
