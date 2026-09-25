// Goal item 15: the row under the chat input and the panels it expands, laid out like the
// Claude Code extension (chips under the box, panels opening above it). Every control maps to an
// existing protocol field: turn/start.invisible, turn/start.collaborationMode (from
// collaborationMode/list), thread/goal/*, review/start, the TUI's permission presets
// (approvalPolicy + sandboxPolicy), UserInput skill items, picked file/folder paths written into
// the message text like the TUI's @ picker, turn/start.model/effort.
import type { ReasoningEffort } from "@protocol/ReasoningEffort";
import { useEffect, useState } from "react";
import type { MessageKey } from "../../shared/i18n";
import { useApp } from "../app/context";
import type { PermissionPreset } from "../app/state";
import { Icon } from "../components/icons";
import { Button, Chip, Empty, Notice } from "../components/ui";

export function Toolbar() {
  const { state, ctl, t } = useApp();
  const c = state.composer;
  const attachedSkills = state.init?.attachedSkills.length ?? 0;
  const modeLabel = state.chat.goal ? t("panel.modeGoal") : c.mode === "plan" ? t("panel.modePlan") : t("panel.modeDefault");
  const modelEntry = state.models.find((m) => m.id === (c.model ?? state.threadModel ?? state.config?.model));
  const modelLabel = modelEntry?.displayName ?? c.model ?? state.threadModel ?? state.config?.model ?? t("panel.modelDefault");
  const effort = c.effort ?? modelEntry?.defaultReasoningEffort ?? null;
  return (
    <div className="sf-toolbar" role="toolbar">
      <Chip icon={c.invisible ? "eyeOff" : "eye"} active={c.invisible} onClick={() => ctl.toggleInvisible()} title={t("panel.invisibleTitle")}>
        {t("panel.invisible")}
      </Chip>
      <Chip icon="layers" active={c.panel === "mode" || c.mode === "plan" || !!state.chat.goal} onClick={() => ctl.togglePanel("mode")} title={t("panel.mode")}>
        {modeLabel}
      </Chip>
      <Chip icon="sparkles" active={c.panel === "skills"} onClick={() => ctl.togglePanel("skills")} title={t("panel.skillsTitle")} badge={attachedSkills}>
        {t("panel.skills")}
      </Chip>
      <Chip icon="folder" active={c.panel === "files" || c.files.length > 0} onClick={() => ctl.togglePanel("files")} title={t("panel.filesTitle")} badge={c.files.length}>
        {c.files.length > 0 ? t(c.fileMode === "select" ? "panel.filesChipSelected" : "panel.filesChipBlocked") : t("panel.files")}
      </Chip>
      <span className="sf-toolbar-spacer" />
      <Chip icon="chip" active={c.panel === "model"} onClick={() => ctl.togglePanel("model")} title={`${t("panel.model")} · ${t("panel.effort")}`}>
        {modelLabel}
        {effort ? ` · ${t(`effort.${effort}` as MessageKey)}` : ""}
      </Chip>
    </div>
  );
}

export function Panels() {
  const { state } = useApp();
  switch (state.composer.panel) {
    case "mode":
      return <ModePanel />;
    case "skills":
      return <SkillsPanel />;
    case "files":
      return <FilesPanel />;
    case "model":
      return <ModelPanel />;
    default:
      return null;
  }
}

function PanelFrame({ title, children }: { title: string; children: React.ReactNode }) {
  const { ctl, t } = useApp();
  return (
    <div className="sf-panel" role="dialog" aria-label={title}>
      <header className="sf-panel-head">
        <span>{title}</span>
        <button type="button" className="sf-icon-btn" onClick={() => ctl.togglePanel(null)} aria-label={t("common.close")}>
          <Icon name="x" />
        </button>
      </header>
      <div className="sf-panel-body">{children}</div>
    </div>
  );
}

function Radio({ checked, onChange, label, description }: { checked: boolean; onChange: () => void; label: string; description?: string }) {
  return (
    <label className={`sf-radio ${checked ? "is-checked" : ""}`}>
      <input type="radio" checked={checked} onChange={onChange} />
      <span className="sf-radio-dot" aria-hidden="true" />
      <span>
        <span className="sf-radio-label">{label}</span>
        {description && <span className="sf-radio-desc">{description}</span>}
      </span>
    </label>
  );
}

const PERMISSIONS: Array<{ id: PermissionPreset; label: MessageKey; desc: MessageKey }> = [
  { id: "readOnly", label: "panel.permReadOnly", desc: "panel.permReadOnlyDesc" },
  { id: "auto", label: "panel.permAuto", desc: "panel.permAutoDesc" },
  { id: "full", label: "panel.permFull", desc: "panel.permFullDesc" },
];

function ModePanel() {
  const { state, ctl, t } = useApp();
  const [objective, setObjective] = useState("");
  const goal = state.chat.goal;
  // Modes come from collaborationMode/list, so any mode the server adds is listed too.
  const modes = state.modes.length ? state.modes : [{ name: "Default", mode: "default" as const, model: null, reasoning_effort: null }];
  const describe = (mode: string | null) =>
    mode === "plan" ? t("panel.modePlanDesc") : mode === "default" ? t("panel.modeDefaultDesc") : undefined;
  return (
    <PanelFrame title={t("panel.mode")}>
      <div className="sf-panel-group">
        {modes.map((m) => (
          <Radio
            key={m.name}
            checked={(state.composer.mode ?? "default") === (m.mode ?? "default")}
            onChange={() => ctl.setMode(m.mode ?? "default")}
            label={m.mode === "plan" ? t("panel.modePlan") : m.mode === "default" ? t("panel.modeDefault") : m.name}
            description={describe(m.mode)}
          />
        ))}
      </div>

      <div className="sf-panel-group">
        <div className="sf-panel-subtitle">
          <Icon name="target" size={13} /> {t("panel.modeGoal")}
        </div>
        <p className="sf-muted">{t("panel.modeGoalDesc")}</p>
        {goal ? (
          <>
            <p className="sf-goal-objective">{goal.objective}</p>
            <p className="sf-muted">{t("panel.goalStatus", { status: t(`goal.${goal.status}` as MessageKey), tokens: goal.tokensUsed })}</p>
            <div className="sf-row">
              {goal.status === "paused" ? (
                <Button onClick={() => void ctl.setGoalStatus("active")}>{t("panel.goalResume")}</Button>
              ) : (
                <Button onClick={() => void ctl.setGoalStatus("paused")}>{t("panel.goalPause")}</Button>
              )}
              <Button variant="ghost" onClick={() => void ctl.clearGoal()}>
                {t("panel.goalClear")}
              </Button>
            </div>
          </>
        ) : (
          <form
            className="sf-row"
            onSubmit={(e) => {
              e.preventDefault();
              if (objective.trim()) void ctl.setGoal(objective.trim()).then(() => setObjective(""));
            }}
          >
            <input className="sf-input" value={objective} onChange={(e) => setObjective(e.target.value)} placeholder={t("panel.goalPlaceholder")} aria-label={t("panel.goalPlaceholder")} />
            <Button variant="primary" type="submit" disabled={!objective.trim()}>
              {t("panel.goalSet")}
            </Button>
          </form>
        )}
      </div>

      <div className="sf-panel-group">
        <div className="sf-panel-subtitle">
          <Icon name="search" size={13} /> {t("panel.modeReview")}
        </div>
        <p className="sf-muted">{t("panel.modeReviewDesc")}</p>
        <Button onClick={() => void ctl.startReview()} disabled={state.chat.activeTurnId !== null}>
          {t("panel.reviewStart")}
        </Button>
      </div>

      <div className="sf-panel-group">
        <div className="sf-panel-subtitle">
          <Icon name="key" size={13} /> {t("panel.permissions")}
        </div>
        <Radio checked={state.composer.permission === null} onChange={() => ctl.setPermission(null)} label={t("panel.permDefault")} description={t("panel.permDefaultDesc")} />
        {PERMISSIONS.map((p) => (
          <Radio key={p.id} checked={state.composer.permission === p.id} onChange={() => ctl.setPermission(p.id)} label={t(p.label)} description={t(p.desc)} />
        ))}
      </div>
    </PanelFrame>
  );
}

function SkillsPanel() {
  const { state, ctl, t } = useApp();
  const attached = new Set(state.init?.attachedSkills.map((s) => s.path));
  const skills = state.skills.filter((s) => s.enabled);
  return (
    <PanelFrame title={t("panel.skillsTitle")}>
      {skills.length === 0 ? (
        <Empty>{t("panel.skillsEmpty")}</Empty>
      ) : (
        <ul className="sf-check-list">
          {skills.map((s) => (
            <li key={s.path}>
              <label className="sf-check">
                <input type="checkbox" checked={attached.has(s.path)} onChange={() => ctl.toggleAttachedSkill(s)} />
                <span>
                  <span className="sf-check-label">{s.interface?.displayName ?? s.name}</span>
                  <span className="sf-check-desc">{s.shortDescription ?? s.description}</span>
                </span>
              </label>
            </li>
          ))}
        </ul>
      )}
      <Button variant="ghost" icon="gear" onClick={() => ctl.go("skills")}>
        {t("nav.skills")}
      </Button>
    </PanelFrame>
  );
}

interface TreeNode {
  name: string;
  path: string;
  dir: boolean;
}

function joinPath(parent: string, name: string): string {
  const sep = parent.includes("\\") ? "\\" : "/";
  return `${parent.replace(/[\\/]$/, "")}${sep}${name}`;
}

function FilesPanel() {
  const { state, ctl, t } = useApp();
  const root = ctl.cwd;
  const [children, setChildren] = useState<Record<string, TreeNode[]>>({});
  const [open, setOpen] = useState<Set<string>>(new Set());
  const [filter, setFilter] = useState("");
  const [hits, setHits] = useState<TreeNode[] | null>(null);
  const selected = new Set(state.composer.files.map((f) => f.path));
  // A running chat keeps the list it started with; an edit applies to the next chat.
  const mode = state.composer.fileMode;
  const chatAccess = ctl.chatBlocks;
  const changedForChat =
    chatAccess !== null &&
    (chatAccess.picks.length !== selected.size ||
      chatAccess.picks.some((path) => !selected.has(path)) ||
      (selected.size > 0 && chatAccess.mode !== mode));
  // Upstream Codex: the unelevated Windows sandbox refuses any read restriction, so blocking needs
  // the elevated one (windows-sandbox-rs/src/lib.rs).
  const needsElevated = selected.size > 0 && /^[A-Za-z]:[\\/]/.test(root ?? "") && state.config?.windowsSandbox !== "elevated";

  const load = async (dir: string) => {
    if (children[dir]) return;
    const r = await ctl.session.fsReadDirectory({ path: dir }).catch(() => ({ entries: [] }));
    const nodes = r.entries
      .filter((e) => !e.fileName.startsWith(".git"))
      .map((e) => ({ name: e.fileName, path: joinPath(dir, e.fileName), dir: e.isDirectory }))
      .sort((a, b) => Number(b.dir) - Number(a.dir) || a.name.localeCompare(b.name));
    setChildren((c) => ({ ...c, [dir]: nodes }));
  };

  useEffect(() => {
    if (root) void load(root);
  }, [root]);

  useEffect(() => {
    if (!filter.trim() || !root) {
      setHits(null);
      return;
    }
    const handle = setTimeout(() => {
      void ctl.session
        .fuzzyFileSearch({ query: filter, roots: [root], cancellationToken: null })
        .then((r) => setHits(r.files.slice(0, 30).map((f) => ({ name: f.file_name, path: joinPath(f.root, f.path), dir: f.match_type === "directory" }))))
        .catch(() => setHits([]));
    }, 150);
    return () => clearTimeout(handle);
  }, [filter, root]);

  const toggleFile = (node: TreeNode) => (selected.has(node.path) ? ctl.detachFile(node.path) : ctl.attachFile({ name: node.name, path: node.path }));

  const renderNodes = (dir: string, depth: number): React.ReactNode =>
    (children[dir] ?? []).map((node) => (
      <li key={node.path}>
        {node.dir ? (
          <div className="sf-tree-row" style={{ paddingLeft: 8 + depth * 14 }}>
            <input type="checkbox" checked={selected.has(node.path)} onChange={() => toggleFile(node)} aria-label={node.name} />
            <button
            type="button"
            className="sf-tree-toggle"
            onClick={() => {
              const next = new Set(open);
              if (next.has(node.path)) next.delete(node.path);
              else {
                next.add(node.path);
                void load(node.path);
              }
              setOpen(next);
            }}
            aria-expanded={open.has(node.path)}
          >
            <Icon name={open.has(node.path) ? "chevronDown" : "chevronRight"} size={12} />
            <Icon name="folder" size={13} /> {node.name}
            </button>
          </div>
        ) : (
          <label className="sf-tree-row" style={{ paddingLeft: 8 + depth * 14 }}>
            <input type="checkbox" checked={selected.has(node.path)} onChange={() => toggleFile(node)} />
            <span className="sf-tree-spacer" aria-hidden="true" />
            <Icon name="file" size={13} /> {node.name}
          </label>
        )}
        {node.dir && open.has(node.path) && <ul className="sf-tree">{renderNodes(node.path, depth + 1)}</ul>}
      </li>
    ));

  return (
    <PanelFrame title={t("panel.filesTitle")}>
      {changedForChat && (
        <Notice tone="info">
          <div className="sf-stack-tight">
            <span>{t("panel.filesLocked")}</span>
            <div>
              <Button variant="secondary" icon="plus" onClick={() => ctl.newThread()}>
                {t("panel.filesApplyNew")}
              </Button>
            </div>
          </div>
        </Notice>
      )}
      {needsElevated && <Notice tone="warning">{t("panel.filesElevated")}</Notice>}
      <div className="sf-mode-switch">
        <button type="button" className={`sf-mode-switch-label ${mode === "block" ? "is-active" : ""}`} onClick={() => ctl.setFileMode("block")}>
          {t("panel.filesModeBlock")}
        </button>
        <button
          type="button"
          role="switch"
          aria-checked={mode === "select"}
          aria-label={`${t("panel.filesModeBlock")} / ${t("panel.filesModeSelect")}`}
          className={`sf-switch ${mode === "select" ? "is-on" : ""}`}
          onClick={() => ctl.setFileMode(mode === "select" ? "block" : "select")}
        >
          <span className="sf-switch-thumb" aria-hidden="true" />
        </button>
        <button type="button" className={`sf-mode-switch-label ${mode === "select" ? "is-active" : ""}`} onClick={() => ctl.setFileMode("select")}>
          {t("panel.filesModeSelect")}
        </button>
      </div>
      <p className="sf-muted sf-small">{t(mode === "block" ? "panel.filesModeBlockDesc" : "panel.filesModeSelectDesc")}</p>
      <input className="sf-input" value={filter} onChange={(e) => setFilter(e.target.value)} placeholder={t("panel.filesFilter")} aria-label={t("panel.filesFilter")} />
      {state.composer.files.length > 0 && (
        <div className="sf-row sf-wrap">
          <span className="sf-muted">{t("panel.filesSelected", { count: state.composer.files.length })}</span>
          <Button variant="ghost" icon="x" onClick={() => ctl.clearFiles()}>
            {t("panel.filesClear")}
          </Button>
        </div>
      )}
      <ul className="sf-tree sf-tree-root">
        {hits
          ? hits.map((node) => (
              <li key={node.path}>
                <label className="sf-tree-row">
                  <input type="checkbox" checked={selected.has(node.path)} onChange={() => toggleFile(node)} />
                  <Icon name={node.dir ? "folder" : "file"} size={13} /> {node.path.replace(`${root}`, "").replace(/^[\\/]/, "")}
                </label>
              </li>
            ))
          : root && renderNodes(root, 0)}
      </ul>
    </PanelFrame>
  );
}

/**
 * The reasoning levels the model reports (model/list `supportedReasoningEfforts`, in its order), as
 * a slider with one stop per level and the level names above their stops. Only levels the model
 * lists are offered: the wire clamps anything else (GLM-5.3 accepts only low, high and max).
 */
function EffortSlider({
  levels,
  value,
  label,
  name,
  onChange,
}: {
  levels: ReasoningEffort[];
  value: ReasoningEffort | null;
  label: string;
  name: (level: ReasoningEffort) => string;
  onChange: (level: ReasoningEffort) => void;
}) {
  const index = Math.max(0, value ? levels.indexOf(value) : 0);
  const last = Math.max(1, levels.length - 1);
  // A stop sits at thumb/2 + (width - thumb) * i / last; the labels use the same formula.
  const at = (i: number) => `calc(var(--sf-thumb) / 2 + (100% - var(--sf-thumb)) * ${i / last})`;
  return (
    <div className="sf-effort">
      <div className="sf-effort-labels" aria-hidden="true">
        {levels.map((level, i) => (
          <button
            key={level}
            type="button"
            tabIndex={-1}
            className={`sf-effort-label ${i === index ? "is-active" : ""}`}
            style={{ left: at(i) }}
            onClick={() => onChange(level)}
          >
            {name(level)}
          </button>
        ))}
      </div>
      <input
        type="range"
        className="sf-effort-range"
        min={0}
        max={levels.length - 1}
        step={1}
        value={index}
        aria-label={label}
        aria-valuetext={name(levels[index]!)}
        onChange={(e) => onChange(levels[Number(e.target.value)]!)}
      />
      <div className="sf-effort-stops" aria-hidden="true">
        {levels.map((level, i) => (
          <span key={level} className={`sf-effort-stop ${i <= index ? "is-filled" : ""}`} style={{ left: at(i) }} />
        ))}
      </div>
    </div>
  );
}

function ModelPanel() {
  const { state, ctl, t } = useApp();
  const current = state.composer.model ?? state.threadModel ?? state.config?.model ?? state.models.find((m) => m.isDefault)?.id ?? null;
  const entry = state.models.find((m) => m.id === current);
  const efforts = entry?.supportedReasoningEfforts ?? [];
  const effort = state.composer.effort ?? entry?.defaultReasoningEffort ?? null;
  return (
    <PanelFrame title={`${t("panel.model")} · ${t("panel.effort")}`}>
      <div className="sf-panel-group">
        {state.models.map((m) => (
          <Radio key={m.id} checked={current === m.id} onChange={() => ctl.setModel(m.id)} label={m.displayName} description={m.description} />
        ))}
      </div>
      {efforts.length > 0 && (
        <div className="sf-panel-group">
          <div className="sf-panel-subtitle">{t("panel.effort")}</div>
          <EffortSlider
            levels={efforts.map((e) => e.reasoningEffort as ReasoningEffort)}
            value={effort as ReasoningEffort | null}
            label={t("panel.effort")}
            name={(level) => t(`effort.${level}` as MessageKey)}
            onChange={(level) => ctl.setEffort(level)}
          />
        </div>
      )}
    </PanelFrame>
  );
}
