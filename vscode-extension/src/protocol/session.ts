// Typed app-server calls, one method per protocol request. This is the webview-side
// counterpart of the TUI's request layer (codex-rs/tui/src/app_server_session.rs): the same
// methods, the same parameters, so the extension drives Suffice exactly the way the terminal
// UI does and every cost mechanism stays on the server side where it lives.
import type { ClientInfo } from "@protocol/ClientInfo";
import type { FuzzyFileSearchParams } from "@protocol/FuzzyFileSearchParams";
import type { FuzzyFileSearchResponse } from "@protocol/FuzzyFileSearchResponse";
import type { InitializeResponse } from "@protocol/InitializeResponse";
import type { ServerNotification } from "@protocol/ServerNotification";
import type { ServerRequest } from "@protocol/ServerRequest";
import type { ConfigBatchWriteParams } from "@protocol/v2/ConfigBatchWriteParams";
import type { ConfigReadParams } from "@protocol/v2/ConfigReadParams";
import type { ConfigReadResponse } from "@protocol/v2/ConfigReadResponse";
import type { ConfigValueWriteParams } from "@protocol/v2/ConfigValueWriteParams";
import type { ConfigWriteResponse } from "@protocol/v2/ConfigWriteResponse";
import type { FsReadDirectoryParams } from "@protocol/v2/FsReadDirectoryParams";
import type { FsReadDirectoryResponse } from "@protocol/v2/FsReadDirectoryResponse";
import type { GetAccountParams } from "@protocol/v2/GetAccountParams";
import type { GetAccountResponse } from "@protocol/v2/GetAccountResponse";
import type { LoginAccountParams } from "@protocol/v2/LoginAccountParams";
import type { LoginAccountResponse } from "@protocol/v2/LoginAccountResponse";
import type { ModelListParams } from "@protocol/v2/ModelListParams";
import type { ModelListResponse } from "@protocol/v2/ModelListResponse";
import type { ReviewStartParams } from "@protocol/v2/ReviewStartParams";
import type { ReviewStartResponse } from "@protocol/v2/ReviewStartResponse";
import type { SkillsConfigWriteParams } from "@protocol/v2/SkillsConfigWriteParams";
import type { SkillsConfigWriteResponse } from "@protocol/v2/SkillsConfigWriteResponse";
import type { SkillsListParams } from "@protocol/v2/SkillsListParams";
import type { SkillsListResponse } from "@protocol/v2/SkillsListResponse";
import type { ThreadCompactStartParams } from "@protocol/v2/ThreadCompactStartParams";
import type { ThreadCompactStartResponse } from "@protocol/v2/ThreadCompactStartResponse";
import type { ThreadGoalClearParams } from "@protocol/v2/ThreadGoalClearParams";
import type { ThreadGoalClearResponse } from "@protocol/v2/ThreadGoalClearResponse";
import type { ThreadGoalGetParams } from "@protocol/v2/ThreadGoalGetParams";
import type { ThreadGoalGetResponse } from "@protocol/v2/ThreadGoalGetResponse";
import type { ThreadGoalSetParams } from "@protocol/v2/ThreadGoalSetParams";
import type { ThreadGoalSetResponse } from "@protocol/v2/ThreadGoalSetResponse";
import type { ThreadListParams } from "@protocol/v2/ThreadListParams";
import type { ThreadListResponse } from "@protocol/v2/ThreadListResponse";
import type { ThreadReadParams } from "@protocol/v2/ThreadReadParams";
import type { ThreadReadResponse } from "@protocol/v2/ThreadReadResponse";
import type { ThreadResumeParams } from "@protocol/v2/ThreadResumeParams";
import type { ThreadResumeResponse } from "@protocol/v2/ThreadResumeResponse";
import type { ThreadSetNameParams } from "@protocol/v2/ThreadSetNameParams";
import type { ThreadSetNameResponse } from "@protocol/v2/ThreadSetNameResponse";
import type { ThreadStartParams } from "@protocol/v2/ThreadStartParams";
import type { ThreadStartResponse } from "@protocol/v2/ThreadStartResponse";
import type { ThreadTurnsListParams } from "@protocol/v2/ThreadTurnsListParams";
import type { ThreadTurnsListResponse } from "@protocol/v2/ThreadTurnsListResponse";
import type { TurnInterruptParams } from "@protocol/v2/TurnInterruptParams";
import type { TurnInterruptResponse } from "@protocol/v2/TurnInterruptResponse";
import type { TurnStartResponse } from "@protocol/v2/TurnStartResponse";
import type { CollaborationModeListResponse, TurnStartParamsWithMode } from "./experimental";
import type { JsonRpcConnection } from "./jsonrpc";

export type Notification = ServerNotification;
export type NotificationOf<M extends Notification["method"]> = Extract<Notification, { method: M }>;
export type ServerRequestOf<M extends ServerRequest["method"]> = Extract<ServerRequest, { method: M }>;

export class AppServerSession {
  constructor(private readonly rpc: JsonRpcConnection) {}

  /** `initialize` then `initialized`. experimentalApi matches the TUI (app_server_connection.rs). */
  async initialize(clientInfo: ClientInfo): Promise<InitializeResponse> {
    const response = await this.rpc.request<InitializeResponse>("initialize", {
      clientInfo,
      capabilities: { experimentalApi: true, requestAttestation: false },
    });
    this.rpc.notify("initialized");
    return response;
  }

  onNotification(listener: (notification: Notification) => void): () => void {
    return this.rpc.onNotification((n) => listener(n as unknown as Notification));
  }

  // --- threads (local session history lives behind these) --------------------------------
  threadStart(params: ThreadStartParams) {
    return this.rpc.request<ThreadStartResponse>("thread/start", params);
  }
  threadResume(params: ThreadResumeParams) {
    return this.rpc.request<ThreadResumeResponse>("thread/resume", params);
  }
  threadList(params: ThreadListParams) {
    return this.rpc.request<ThreadListResponse>("thread/list", params);
  }
  threadRead(params: ThreadReadParams) {
    return this.rpc.request<ThreadReadResponse>("thread/read", params);
  }
  threadTurnsList(params: ThreadTurnsListParams) {
    return this.rpc.request<ThreadTurnsListResponse>("thread/turns/list", params);
  }
  threadSetName(params: ThreadSetNameParams) {
    return this.rpc.request<ThreadSetNameResponse>("thread/name/set", params);
  }
  threadCompactStart(params: ThreadCompactStartParams) {
    return this.rpc.request<ThreadCompactStartResponse>("thread/compact/start", params);
  }

  // --- turns -------------------------------------------------------------------------------
  turnStart(params: TurnStartParamsWithMode) {
    return this.rpc.request<TurnStartResponse>("turn/start", params);
  }
  turnInterrupt(params: TurnInterruptParams) {
    return this.rpc.request<TurnInterruptResponse>("turn/interrupt", params);
  }
  reviewStart(params: ReviewStartParams) {
    return this.rpc.request<ReviewStartResponse>("review/start", params);
  }

  // --- goals and modes ---------------------------------------------------------------------
  threadGoalGet(params: ThreadGoalGetParams) {
    return this.rpc.request<ThreadGoalGetResponse>("thread/goal/get", params);
  }
  threadGoalSet(params: ThreadGoalSetParams) {
    return this.rpc.request<ThreadGoalSetResponse>("thread/goal/set", params);
  }
  threadGoalClear(params: ThreadGoalClearParams) {
    return this.rpc.request<ThreadGoalClearResponse>("thread/goal/clear", params);
  }
  collaborationModeList() {
    return this.rpc.request<CollaborationModeListResponse>("collaborationMode/list", {});
  }

  // --- catalog -----------------------------------------------------------------------------
  modelList(params: ModelListParams) {
    return this.rpc.request<ModelListResponse>("model/list", params);
  }
  skillsList(params: SkillsListParams) {
    return this.rpc.request<SkillsListResponse>("skills/list", params);
  }
  skillsConfigWrite(params: SkillsConfigWriteParams) {
    return this.rpc.request<SkillsConfigWriteResponse>("skills/config/write", params);
  }

  // --- config ------------------------------------------------------------------------------
  configRead(params: ConfigReadParams) {
    return this.rpc.request<ConfigReadResponse>("config/read", params);
  }
  configValueWrite(params: ConfigValueWriteParams) {
    return this.rpc.request<ConfigWriteResponse>("config/value/write", params);
  }
  configBatchWrite(params: ConfigBatchWriteParams) {
    return this.rpc.request<ConfigWriteResponse>("config/batchWrite", params);
  }

  // --- files -------------------------------------------------------------------------------
  fuzzyFileSearch(params: FuzzyFileSearchParams) {
    return this.rpc.request<FuzzyFileSearchResponse>("fuzzyFileSearch", params);
  }
  fsReadDirectory(params: FsReadDirectoryParams) {
    return this.rpc.request<FsReadDirectoryResponse>("fs/readDirectory", params);
  }

  // --- account -----------------------------------------------------------------------------
  accountRead(params: GetAccountParams) {
    return this.rpc.request<GetAccountResponse>("account/read", params);
  }
  accountLoginStart(params: LoginAccountParams) {
    return this.rpc.request<LoginAccountResponse>("account/login/start", params);
  }
}
