// The checked-in TypeScript bindings (codex-rs/app-server-protocol/schema/typescript) are the
// *stable* export. The TUI runs with `experimentalApi: true` and uses a few experimental
// fields; their building blocks (CollaborationMode, CollaborationModeMask) are stable types,
// so only the containers are declared here. Source of truth:
//   `suffice app-server generate-ts --experimental --out <dir>`
//   v2/TurnStartParams.ts          -> collaborationMode?: CollaborationMode | null
//   v2/CollaborationModeListParams -> Record<string, never>
//   v2/CollaborationModeListResponse -> { data: Array<CollaborationModeMask> }
import type { CollaborationMode } from "@protocol/CollaborationMode";
import type { CollaborationModeMask } from "@protocol/v2/CollaborationModeMask";
import type { TurnStartParams } from "@protocol/v2/TurnStartParams";

export type TurnStartParamsWithMode = TurnStartParams & {
  /** For `settings.developer_instructions`, null means the mode's built-in instructions. */
  collaborationMode?: CollaborationMode | null;
};

export type CollaborationModeListResponse = { data: Array<CollaborationModeMask> };
