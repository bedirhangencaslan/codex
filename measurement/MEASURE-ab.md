
## A/B by task (newest runs per side)

| task | side | n | mean $ | min $ | max $ | req | hit | $ / tool call |
|---|---|---|---|---|---|---|---|---|
| docs | base | 3 | 0.0073 | 0.0024 | 0.0143 | 6.0 | 63.3% | 0.0009 |
| docs | fork | 2 | 0.0104 | 0.0049 | 0.0159 | 10.5 | 90.4% | 0.0009 |
| git | base | 3 | 0.0120 | 0.0102 | 0.0153 | 26.3 | 93.4% | 0.0005 |
| git | fork | 3 | 0.0094 | 0.0071 | 0.0133 | 24.3 | 93.2% | 0.0004 |
| rs | base | 1 | 0.0108 | 0.0108 | 0.0108 | 13.0 | 85.1% | 0.0008 |
| rs | fork | 1 | 0.0340 | 0.0340 | 0.0340 | 29.0 | 89.2% | 0.0006 |
| sepet | base | 3 | 0.0214 | 0.0145 | 0.0267 | 15.0 | 77.3% | 0.0015 |
| sepet | fork | 3 | 0.0377 | 0.0289 | 0.0524 | 25.3 | 86.2% | 0.0013 |

Run-to-run spread usually exceeds the side difference; read `min`/`max` before `mean`, and `$ / tool call` when the two sides did different amounts of work.
