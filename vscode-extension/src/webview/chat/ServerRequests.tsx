// The server's requests shown inline, like the TUI's approval overlay
// (bottom_pane/approval_overlay.rs) and request_user_input view: command and file-change
// approvals, and the questions plan mode asks. Answers go back through the host by request id.
import type { CommandExecutionRequestApprovalParams } from "@protocol/v2/CommandExecutionRequestApprovalParams";
import type { FileChangeRequestApprovalParams } from "@protocol/v2/FileChangeRequestApprovalParams";
import type { ToolRequestUserInputParams } from "@protocol/v2/ToolRequestUserInputParams";
import { useState } from "react";
import { useApp } from "../app/context";
import type { PendingServerRequest } from "../app/state";
import { Icon } from "../components/icons";
import { Button } from "../components/ui";

export function ServerRequests() {
  const { state } = useApp();
  if (state.serverRequests.length === 0) return null;
  return (
    <div className="sf-requests">
      {state.serverRequests.map((r) => (
        <RequestCard key={r.requestId} request={r} />
      ))}
    </div>
  );
}

function RequestCard({ request }: { request: PendingServerRequest }) {
  switch (request.method) {
    case "item/commandExecution/requestApproval":
    case "item/fileChange/requestApproval":
      return <ApprovalCard request={request} />;
    case "item/tool/requestUserInput":
      return <QuestionsCard request={request} params={request.params as ToolRequestUserInputParams} />;
    default:
      return <UnsupportedCard request={request} />;
  }
}

function ApprovalCard({ request }: { request: PendingServerRequest }) {
  const { ctl, t } = useApp();
  const isCommand = request.method === "item/commandExecution/requestApproval";
  const p = request.params as CommandExecutionRequestApprovalParams & FileChangeRequestApprovalParams;
  const answer = (decision: "accept" | "acceptForSession" | "decline" | "cancel") => ctl.answerServerRequest(request.requestId, { decision });
  return (
    <div className="sf-request" role="alertdialog" aria-label={isCommand ? t("approval.commandTitle") : t("approval.fileTitle")}>
      <div className="sf-request-title">
        <Icon name={isCommand ? "terminal" : "file"} /> {isCommand ? t("approval.commandTitle") : t("approval.fileTitle")}
      </div>
      {isCommand && p.command && <pre className="sf-request-cmd">{p.command}</pre>}
      {p.reason && <p className="sf-muted">{t("approval.reason", { reason: p.reason })}</p>}
      <div className="sf-row sf-wrap">
        <Button variant="primary" onClick={() => answer("accept")}>
          {t("approval.accept")}
        </Button>
        <Button onClick={() => answer("acceptForSession")}>{t("approval.acceptSession")}</Button>
        <Button variant="ghost" onClick={() => answer("decline")}>
          {t("approval.decline")}
        </Button>
        <Button variant="danger" onClick={() => answer("cancel")}>
          {t("approval.cancel")}
        </Button>
      </div>
    </div>
  );
}

function QuestionsCard({ request, params }: { request: PendingServerRequest; params: ToolRequestUserInputParams }) {
  const { ctl, t } = useApp();
  const [answers, setAnswers] = useState<Record<string, string>>({});
  return (
    <form
      className="sf-request"
      onSubmit={(e) => {
        e.preventDefault();
        const result = Object.fromEntries(params.questions.map((q) => [q.id, { answers: answers[q.id] ? [answers[q.id]!] : [] }]));
        ctl.answerServerRequest(request.requestId, { answers: result });
      }}
    >
      <div className="sf-request-title">
        <Icon name="info" /> {t("approval.questionsTitle")}
      </div>
      {params.questions.map((q) => (
        <fieldset key={q.id} className="sf-question">
          <legend>{q.header || q.question}</legend>
          {q.header && <p>{q.question}</p>}
          {q.options?.map((o) => (
            <label key={o.label} className="sf-radio">
              <input type="radio" name={q.id} checked={answers[q.id] === o.label} onChange={() => setAnswers({ ...answers, [q.id]: o.label })} />
              <span className="sf-radio-dot" aria-hidden="true" />
              <span>
                <span className="sf-radio-label">{o.label}</span>
                {o.description && <span className="sf-radio-desc">{o.description}</span>}
              </span>
            </label>
          ))}
          {(q.isOther || !q.options?.length) && (
            <input
              className="sf-input"
              type={q.isSecret ? "password" : "text"}
              placeholder={t("approval.other")}
              aria-label={q.question}
              onChange={(e) => setAnswers({ ...answers, [q.id]: e.target.value })}
            />
          )}
        </fieldset>
      ))}
      <Button variant="primary" type="submit">
        {t("approval.submitAnswers")}
      </Button>
    </form>
  );
}

function UnsupportedCard({ request }: { request: PendingServerRequest }) {
  const { ctl, t } = useApp();
  return (
    <div className="sf-request">
      <div className="sf-request-title">
        <Icon name="warning" /> {request.method}
      </div>
      <Button onClick={() => ctl.answerServerRequest(request.requestId, {})}>{t("approval.decline")}</Button>
    </div>
  );
}
