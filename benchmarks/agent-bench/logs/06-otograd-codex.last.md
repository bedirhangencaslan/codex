Implemented the full reverse-mode autograd engine in [solution.py](/Users/muzafferberkesavas/agent-bench/solutions/06-otograd-codex/solution.py).

It includes graph construction and topological backward propagation, broadcasting-aware gradients, reused-node accumulation, matmul/activation/reduction operations, stable Tensor-based cross-entropy, deterministic Net initialization, momentum SGD, training, prediction, and JSON save/load.

Acceptance run:

```bash
17 passed, 1 warning in 1.03s
```