Done — all 14 acceptance tests pass on the first run.

[solution.py](/Users/muzafferberkesavas/agent-bench/solutions/04-mutabakat-suffice/solution.py) implements the full spec with standard library only:

- **Loading**: TR amount parsing (`1.234,56`), both date formats normalized to ISO, malformed invoice rows collected as `{line, reason}` without crashing, payments assigned `p1, p2, …` in file order.
- **Currency**: `to_try` uses the exact-day rate, falls back to the nearest prior rate within 7 days (per-currency aware), raises `ValueError` beyond that, and rounds to 2 decimals; TRY passes through.
- **Matching**: invoices processed in file order, each payment consumed at most once. Rule priority is exact (fatura_no in description + tolerance), then amount+date(≤3 days)+first customer word, then the exactly-two split payments summing within tolerance. Amount comparisons use original currency when both sides match, otherwise TRY equivalents.
- **fark_try**: same-currency differences computed in that currency first, then converted using the *invoice* date to avoid fake FX noise; mixed-currency differences come from TRY equivalents.
- **CLI**: `python3 solution.py report --fixtures <DIR>` prints the exact summary JSON with `toplam_fark_try` and `hatali_satirlar`.

No fixture values are hardcoded — the rules are generic and the fixture file is only read at runtime via `--fixtures`.