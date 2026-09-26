Implemented the full specification in [solution.py](/Users/muzafferberkesavas/agent-bench/solutions/01-kanbagi-codex/solution.py), including compatibility, stock expiry, donor eligibility, request matching, and the JSON report CLI.

Verification with the required command:

```
BENCH_SOLUTION=$PWD ../../.venv/bin/python -m pytest ../../projects/01-kanbagi/tests -q
```

Result: **15 passed**.