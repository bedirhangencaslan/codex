Done — [solution.py](solution.py) passes the full acceptance suite: 12/12 tests in 0.15s, with comfortable margins on every threshold.

What's in it (numpy only, no ML libraries):

- **relu / softmax / cross_entropy** — softmax is row-wise with per-row max subtraction for stability; cross-entropy uses natural log with the sample mean.
- **MLP** — init follows the spec's exact order and distribution: `default_rng(seed)`, then `W1 ~ N(0, √(2/n_in))`, then `W2 ~ N(0, √(2/n_hidden))`, zero biases, so same seed reproduces bit-identical weights.
- **Backward pass, derived by hand**: for softmax followed by mean cross-entropy, the output delta collapses to `(P − Y)/N` with Y one-hot. From there: `dW2 = HᵀdZ2`, `db2 = rowsum(dZ2)`, then through the ReLU gate `dZ1 = (dZ2·W2ᵀ) ⊙ 1[Z1>0]`, giving `dW1 = XᵀdZ1`, `db1 = rowsum(dZ1)`. No fixture data was involved — this is the general analytic gradient.
- **fit / step / predict / save / load_mlp** — full-batch GD with one grads+step pair per epoch, in-place `p -= lr·g` updates, argmax predictions, and JSON round-trip.

Measured results: train accuracy 0.996 (≥ 0.97), test accuracy 0.983 (≥ 0.90), training 0.04s (≤ 60s), 84 parameters (< 2000). I also ran a gradient check on a fresh random configuration (4 inputs, 7 hidden, 3 outputs, different seeds than the tests use) — worst relative error vs. central finite differences was 2.2e-11, far under the 1e-4 threshold, confirming the derivation holds generally rather than on the fixture's specific values.