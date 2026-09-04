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
    ┌───────────────┴───────────────┐                                   │
    ▼                               ▼                                   │
Delta Predictor         Adaptive Linear FIR                             │
(Wrapping Diff)         (Order-3, Sign LMS)                             │
    │                               │                                   │
    └───────────────┬───────────────┘                                   │
                    ▼                                                   │
              Residual Class & Bit Split (delta_xor.rs) ◄───────────────┘
                    │
            ┌───────┴───────────────────────────┐
            ▼                                   ▼
    Magnitude Symbols (0..192)          Raw Extra Bits Stream
            │                                   │
    Adaptive rANS Table (1024 sum)              │
    (rescaling <= 2048 count)                   │
            │                                   │
    Backward rANS Encoder (32-bit state)        │
            │                                   │
            └───────┬───────────────────────────┘
                    ▼
            Framed Block Output:
            [Header: 15-31 B] [Extra Bits] [rANS Payload]
```

---

## 2. Predictive Coding Subsystem

### 2.1 Baseline Integer Delta
For integer streams ($x_n$), the baseline computes wrapping differences:
$$d_n = x_n - x_{n-1} \pmod{2^{64}}$$
Signed differences are mapped to non-negative integers via zigzag encoding:
$$z(d) = (d \ll 1) \oplus (d \gg 63)$$

### 2.2 Gorilla Float XOR
For IEEE 754 floating-point streams, consecutive samples often share sign, exponent, and high mantissa bits. Tether computes bitwise differences against the preceding word:
$$xor_n = \text{bits}(x_n) \oplus \text{bits}(x_{n-1})$$

### 2.3 Adaptive Linear Predictor (Order-3 FIR with Sign-Sign LMS)
For smooth physical trajectories, accelerations, and sensor drift, an Order-3 Finite Impulse Response (FIR) filter predicts:
$$\hat{x}_n = \left(\sum_{i=1}^3 c_i \cdot x_{n-i} + 2^{\text{SHIFT}-1}\right) \gg \text{SHIFT}$$
where $\text{SHIFT} = 8$ (fixed-point scale 256).
The prediction residual is:
$$e_n = x_n - \hat{x}_n$$
Coefficients adapt online using sign-sign Least Mean Squares (zero division, zero floating point math):
$$c_i \leftarrow \text{clamp}\left(c_i + \mu \cdot \text{sgn}(e_n) \cdot \text{sgn}(x_{n-i}), -2048, 2048\right)$$
State size is strictly constant: 3 coefficients (`[i32; 3]`) + 3 history words (`[i64; 3]`) = 36 bytes of RAM.

### 2.4 Per-Block Tournament Selection (`selector.rs`)
For each block of 256 samples, Tether evaluates the candidate predictors on data already buffered:
$$\text{Score}_{\text{Delta}} = \sum |x_n - x_{n-1}|$$
$$\text{Score}_{\text{FIR}} = \sum |x_n - \hat{x}_n|$$
**Selection rule**: Adaptive FIR is selected only if $\text{Score}_{\text{FIR}} \times 100 < \text{Score}_{\text{Delta}} \times 95$ (at least 5% superior). Otherwise, the simpler Delta is chosen. The choice is signaled via 1 byte in the block header.

---

## 3. Entropy Coding Subsystem (rANS & Adaptive Tables)

### 3.1 32-bit Streaming rANS
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

### 3.2 Magnitude Classification & Escape Handling
Residuals $R \ge 0$ are classified into single symbols:
- $0 \le R \le 127$: Symbol $R$, 0 extra bits.
- $R \ge 128$: Symbol $128 + w$ where $w = 64 - \text{clz}(R) \in [8, 64]$. The lower $w - 1$ bits are emitted to the bitstream.
- Alphabet is strictly bounded to 256 symbols. Every sample produces exactly 1 rANS symbol.

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
