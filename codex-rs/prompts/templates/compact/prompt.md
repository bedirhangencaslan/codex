You are performing a CONTEXT CHECKPOINT COMPACTION. Everything except the user's own messages is about to be deleted. Write the handoff another LLM will use to resume the task.

Number the user's requests in order starting at 1, matching the order of the user messages in this conversation. Emit one line per request you are asked to summarize:

`N. <request in a few words> - <outcome>`

The outcome must say what actually happened. For example: completed, and how; partially done, and what is left; noted for later, not started; the user was asked something and answered; the user was given information (restate the answer if it is still needed); the user rejected the proposal, and why; abandoned or superseded by a later request.

After the list, add only what the next LLM cannot recover from the user messages alone:
- Decisions, constraints, and preferences that still bind
- Files, symbols, commands, and data it would otherwise have to rediscover
- The concrete next step

Keep each entry to one or two lines. Do not restate the user's requests verbatim; they are preserved. Do not narrate tool calls, only what they established.

Stay under 3000 tokens. Anything past that is cut.
