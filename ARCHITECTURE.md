# Tether Architecture & Algorithmic Design

Tether is a lossless compression codec engineered specifically for **streaming numeric and sensor telemetry data under strict, stated hardware memory budgets**.

---

## 1. System Overview

```
Numeric Stream (i64, f64, i32, f32) ──► Fixed Block Framing (e.g. 256 samples)
                                              │
                    ┌─────────────────────────┴─────────────────────────┐
                    ▼                                                   ▼
            Integer Streams                                     Float Streams
                    │                                                   │
     Predictor Tournament (selector.rs)                     Gorilla XOR Predictor
    ┌───────────────┼───────────────┬───────────────┐                   │
    ▼               ▼               ▼               ▼                   │
Constant Mode  Delta-of-Delta     Delta        Adaptive FIR             │
(11B payload)  (2nd Diff)      (Wrapping)     (Order-3 LMS)             │
    │               │               │               │                   │
    └───────────────┼───────────────┴───────────────┘                   │
                    ▼                                                   │
         Zero-Run Residual Packing (delta_xor.rs) ◄─────────────────────┘
                    │
            ┌───────┴───────────────────────────┐
            ▼                                   ▼
    Symbols (0..127, 128..192, 193..255)   Raw Extra Bits Stream
            │                                   │
    Adaptive rANS Table (1024 sum)              │
    (rescaling <= 2048 count)                   │
            │                                   │
    Backward rANS Encoder (32-bit state)        │
            │                                   │
            └───────┬───────────────────────────┘
                    ▼
            Framed Block Output:
            [Header: 11-25 B] [Extra Bits] [rANS Payload]
```

---

## 2. Predictive Coding Subsystem

### 2.1 Constant Block Mode (`PredictorMode::Constant = 4`)
When all samples in a block share the identical value, Tether emits an 11-byte block:
`[mode: 1B] [count: 2B] [value: 8B]`.
This delivers $>180\times$ compression on constant periods and $>1.5$ GB/s throughput with zero entropy coder overhead.

### 2.2 Baseline Integer Delta (`PredictorMode::Delta = 1`)
For integer streams ($x_n$), the baseline computes wrapping differences:
$$d_n = x_n - x_{n-1} \pmod{2^{64}}$$
Signed differences are mapped to non-negative integers via zigzag encoding:
$$z(d) = (d \ll 1) \oplus (d \gg 63)$$

### 2.3 Delta-of-Delta (`PredictorMode::DeltaOfDelta = 5`)
Models linear kinematic velocity using second-order difference extrapolation:
$$\hat{x}_n = 2 x_{n-1} - x_{n-2} = x_{n-1} + (x_{n-1} - x_{n-2})$$
$$e_n = x_n - \hat{x}_n = (x_n - x_{n-1}) - (x_{n-1} - x_{n-2})$$
Linear trends and constant clock increments produce exact zero residuals ($e_n = 0$) instantly with zero convergence delay.

### 2.4 Gorilla Float XOR (`PredictorMode::Xor = 2`)
For IEEE 754 floating-point streams, consecutive samples often share sign, exponent, and high mantissa bits. Tether computes bitwise differences against the preceding word:
$$xor_n = \text{bits}(x_n) \oplus \text{bits}(x_{n-1})$$

### 2.5 Adaptive Linear Predictor (`PredictorMode::AdaptiveLinear = 3`)
For complex physical trajectories and resonant oscillations, an Order-3 Finite Impulse Response (FIR) filter predicts:
$$\hat{x}_n = \left(\sum_{i=1}^3 c_i \cdot x_{n-i} + 2^{\text{SHIFT}-1}\right) \gg \text{SHIFT}$$
where $\text{SHIFT} = 8$ (fixed-point scale 256).
Coefficients adapt online using sign-sign Least Mean Squares (zero division, zero floating point math):
$$c_i \leftarrow \text{clamp}\left(c_i + \mu \cdot \text{sgn}(e_n) \cdot \text{sgn}(x_{n-i}), -2048, 2048\right)$$

### 2.6 Per-Block Tournament Selection (`selector.rs`)
For each block of 256 samples, Tether evaluates the candidate predictors:
1. If all samples are equal $\implies$ select `Constant`.
2. Evaluate $\text{Score}_{\text{Delta}}$, $\text{Score}_{\text{DeltaOfDelta}}$, and $\text{Score}_{\text{FIR}}$.
3. If Delta-of-Delta beats Delta by $\ge 5\%$ and is $\le$ FIR $\implies$ select `DeltaOfDelta`.
4. If FIR beats Delta by $\ge 5\%$ and is $<$ Delta-of-Delta $\implies$ select `AdaptiveLinear`.
5. Otherwise, select `Delta`.

---

## 3. Entropy Coding Subsystem (rANS & Zero-Run Packing)

### 3.1 Zero-Run Residual Packing (`delta_xor.rs`)
In streaming telemetry and filtered images, runs of exact zero residuals are extremely frequent. Tether uses an alphabet mapping that eliminates redundant rANS operations:
- `0`: Single zero residual.
- `1..127`: Small literal residuals $1..127$.
- `128..192`: Prefix for larger residuals with bit width $w = \text{symbol} - 128 \in [8, 64]$.
- `193..255`: **Zero-run symbols** representing runs of zeros of length $2..64$ (`193` $\implies 2$ zeros, `255` $\implies 64$ zeros).

This reduces rANS symbol counts by up to **$33\times$** on telemetry streams.

### 3.2 32-bit Streaming rANS
Tether uses 32-bit range Asymmetric Numeral Systems (rANS) with state $x \in [L, b \cdot L - 1]$, where:
- $L = 2^{23} = 8,388,608$
- Emission base $b = 256$ (1 byte at a time)
- Normalized frequency sum $M = 1024$ ($2^{10}$)

Encoding symbol $s$ with frequency $f$ and cumulative start $C(s)$:
$$\text{Renormalize: while } x \ge (2^{21} \cdot f) \implies \text{emit}(x \ \& \ 0xFF), \ x \leftarrow x \gg 8$$
$$\text{Encode: } x \leftarrow \left(\lfloor x / f \rfloor \ll 10\right) + C(s) + (x \pmod f)$$

Decoding slot $m = x \ \& \ 1023$, symbol $s = \text{LUT}[m]$:
$$x \leftarrow f(s) \cdot (x \gg 10) + (m - C(s))$$
$$\text{Renormalize: while } x < L \implies x \leftarrow (x \ll 8) \ | \ \text{read\_byte}()$$

### 3.3 Bounded Adaptive Table & Deterministic Rescaling
- Total frequency count is normalized to $M = 1024$.
- When cumulative observations reach 2048: counts are halved ($c_s \leftarrow \max(1, c_s \gg 1)$) and renormalized using deterministic integer largest-remainder allocation.
- Both encoder and decoder execute identical integer math. Zero floating point drift.

---

## 4. Strict Memory Contract

| Profile | Target Hardware | Max Working RAM | Block Samples | Peak Encoder RAM | Peak Decoder RAM |
|:---|:---|:---:|:---:|:---:|:---:|
| **Micro4KB** | Cortex-M0/M3 (<= 8KB SRAM) | 4,096 B | 128 | 3.2 KB | 2.2 KB |
| **Embedded16KB** | Cortex-M4/Wearables (<= 32KB SRAM) | 16,384 B | 256 | 5.7 KB | 2.4 KB |
| **Embedded32KB** (Default) | Cortex-M7/IoT Gateways | 32,768 B | 256 | 5.7 KB | 2.4 KB |
| **Desktop64KB** | Edge Gateways / Linux SBC | 65,536 B | 512 | 10.5 KB | 3.6 KB |

Every structure has statically knowable size. Memory consumption never grows with stream length. Verified by `tests/memory_ceiling.rs` and `mem_profile/ci_check.py`.

---

## 5. Streaming 2D Image Extension

For lossless images, Tether swaps the 1D temporal predictor for a 2D spatial predictor (None, Sub, Up, Average, Paeth, Plane, MED) evaluated row-by-row.
- Working buffer: exactly 2 rows ($O(\text{width})$ RAM).
- A 1920-wide image uses $\approx 3.8$ KB of working memory.
- Residuals feed directly into the existing adaptive rANS module.
