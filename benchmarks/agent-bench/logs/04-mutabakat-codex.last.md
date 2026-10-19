Implemented the full specification in [solution.py](/Users/muzafferberkesavas/agent-bench/solutions/04-mutabakat-codex/solution.py).

Validation passed with the exact requested command:

`14 passed, 1 warning in 0.05s`

The implementation covers Turkish amount/date parsing, resilient invoice loading with line-numbered errors, sequential payment IDs, TRY conversion with the seven-day previous-rate fallback, the three prioritized matching rules, one-payment-once behavior, per-match `fark_try`, and the JSON report CLI.