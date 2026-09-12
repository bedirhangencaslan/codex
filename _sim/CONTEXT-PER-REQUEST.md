# Everything the model sees, on every request, in both agents

`CONTEXT-KARSILASTIRMA.md` compared the two agents' **first** requests. The reading window is not
chosen there. It is chosen on the second request and every one after, where the conversation has
grown — and what those requests are made of had never been written down for either agent.

Both halves below are captured request bodies, not reconstructions: `relay_all.py` dumps every body
and its headers, and `ctxmap.py` labels each message into one block. Same task, same corpus, same
relay, same day (2026-09-12), runs taken within the hour.

Blocks: **prompt** = the agent's own system prompt · **skills** = a skills/permissions block sent as
its own system message · **agents_md** = AGENTS.md wherever it is carried · **task** = the user turn ·
**tools** = serialized tool specs · **assistant** = the model's own prior turns, including reasoning
and tool-call arguments · **tool_out** = what the tools answered.

## OpenCode — `wire-stock-rep11`, 24 loop requests, $0.0265

| req | msgs | prompt | skills | agents_md | task | tools | assistant | tool_out | **total** |
|---|---|---|---|---|---|---|---|---|---|
| 002 | 2 | 8,888 | 701 | 22,894 | 519 | 21,803 | 0 | 0 | 54,805 |
| 003 | 4 | 8,888 | 701 | 22,894 | 519 | 21,803 | 190 | 3,916 | 58,911 |
| 004 | 8 | 8,888 | 701 | 22,894 | 519 | 21,803 | 1,015 | 27,105 | 82,925 |
| 006 | 15 | 8,888 | 701 | 22,894 | 519 | 21,803 | 2,145 | 85,752 | 142,702 |
| 010 | 27 | 8,888 | 701 | 22,894 | 519 | 21,803 | 4,018 | 145,810 | 204,633 |
| 015 | 43 | 8,888 | 701 | 22,894 | 519 | 21,803 | 6,784 | 181,870 | 243,459 |
| 020 | 57 | 8,888 | 701 | 22,894 | 519 | 21,803 | 9,305 | 218,646 | 282,756 |
| 025 | 72 | 8,888 | 701 | 22,894 | 519 | 21,803 | 25,757 | 249,696 | **330,258** |

Five columns never move. `task` is 519 characters on the first request and 519 on the last.

## Suffice — `wire-sufficefork-rep22`, 19 requests, $0.0299

| req | msgs | prompt | skills | agents_md | task | tools | assistant | tool_out | **total** |
|---|---|---|---|---|---|---|---|---|---|
| 001 | 4 | 16,521 | 5,592 | 23,958 | 519 | 14,683 | 0 | 0 | 61,273 |
| 003 | 9 | 16,521 | 5,592 | 23,958 | 519 | 14,683 | 1,075 | 6,205 | 68,553 |
| 005 | 38 | 16,521 | 5,592 | 23,958 | 519 | 14,683 | 7,931 | 220,826 | 290,030 |
| 006 | 41 | 16,521 | 5,592 | 23,958 | **1,872** | 14,683 | 9,111 | 252,396 | 324,133 |
| **007** | **7** | 16,521 | 5,592 | 23,958 | **7,882** | 14,683 | 194 | **0** | 68,830 |
| 010 | 20 | 16,521 | 5,592 | 23,958 | 7,882 | 14,683 | 5,202 | 214,576 | 288,414 |
| 011 | 25 | 16,521 | 5,592 | 23,958 | **9,235** | 14,683 | 6,255 | 248,327 | 324,571 |
| **012** | **9** | 16,521 | 5,592 | 23,958 | **8,397** | 14,683 | 438 | **0** | 69,589 |
| 019 | 40 | 16,521 | 5,592 | 23,958 | 8,397 | 14,683 | 18,564 | 178,096 | 265,811 |

## The two things this shows

### 1. It compacts. Twice. OpenCode never does.

At requests 7 and 12 `tool_out` falls to zero, the message count collapses from 41 to 7 and from 25
to 9, and `task` — the user block — grows from 519 characters to 7,882 and then 8,397. That is the
compaction summary being carried forward in place of the conversation it replaced.

On the wire:

```
req  6   prompt 81,483   out 4,154     <- writing the summary
req  7   prompt 15,794                 <- restart, cache gone
req 11   prompt 80,319   out 5,788     <- writing the summary
req 12   prompt 16,013                 <- restart, cache gone
```

Each one costs three ways: the summary itself (4,154 and 5,788 output tokens, the two most expensive
generations in the run), the cached prefix discarded, and whatever work has to be redone because its
detail is gone. OpenCode ran the same task to 330,258 characters — larger than anything the fork
reached — and never compacted.

### 2. The fixed prefix is bigger, and differently shaped

| | OpenCode | Suffice |
|---|---|---|
| prompt | 8,888 | **16,521** |
| skills | 701 | **5,592** |
| agents_md | 22,894 | 23,958 |
| task | 519 | 519 |
| tools | **21,803** | 14,683 |
| **fixed total** | **54,805** | **61,273** |

The fork spends 1.9× on its prompt and 8× on its skills block, and saves it back on tool specs. Net
+6,468 characters on every single request.

## Why this is the right basis for the next experiment

The window is chosen at request 003 in both agents. At that moment the model is looking at:

| | OpenCode req 003 | Suffice req 003 |
|---|---|---|
| fixed prefix | 54,805 | 61,273 |
| its own prior turn | 190 | 1,075 |
| tool output so far | 3,916 | 6,205 |
| **total** | **58,911** | **68,553** |

Two agents, the same decision, 9,642 characters apart — and now every one of those characters is
attributable to a named block. A rig that mutates *one block at a time at that exact request*, with
20 sends and a known correct answer, costs half a cent a variant. That is the design; this document
is the input it needs.

## Caveat

One run each. The compaction count is a property of this run, not a measured rate — though the fork
compacted in `rep4` too and OpenCode has not compacted in any of the eight runs recorded.
