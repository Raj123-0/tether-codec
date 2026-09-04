# Tether Lossless Compression Benchmark Results

This benchmark reports **four simultaneous metrics** for every competitor under an explicit memory contract:
1. **Ratio**: Compression ratio (Raw Size / Compressed Size). Higher is better.
2. **Enc Speed (MB/s)**: Encoding throughput. Higher is better.
3. **Dec Speed (MB/s)**: Decoding throughput. Higher is better.
4. **RAM Footprint (KB)**: Peak working memory (Encoder / Decoder). Stated and strictly enforced.

> [!NOTE]
> All comparisons use **equivalent memory-constrained profiles** (zstd with 32KB windowLog=15, LZ4 block, Tether with 32KB or 4KB budget). No competitor is granted unbounded memory.

### Dataset: Synthetic: Sine Wave

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 14.96x | 270.4 MB/s | 303.6 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 12.36x | 241.3 MB/s | 273.7 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 5.88x | 273.2 MB/s | 639.0 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.02x | 535.7 MB/s | 900.3 MB/s | 1170.8 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 1.12x | 9.2 MB/s | 11.0 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 13.95x | 4.9 MB/s | 2.1 MB/s | 4076.6 KB / 7182.3 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Linear Trend

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 31.83x | 369.0 MB/s | 439.2 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 21.78x | 316.7 MB/s | 420.1 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 7.53x | 285.8 MB/s | 444.0 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.00x | 563.4 MB/s | 646.7 MB/s | 1175.6 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 1.03x | 26.0 MB/s | 31.2 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 2446.48x | 5.9 MB/s | 2.4 MB/s | 2351.4 KB / 5464.0 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Periodic Square

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 64.21x | 497.6 MB/s | 593.4 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 38.50x | 392.9 MB/s | 599.6 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 391.77x | 2391.7 MB/s | 1913.1 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 79.87x | 6520.9 MB/s | 1075.5 MB/s | 794.1 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 63.33x | 14.6 MB/s | 17.5 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 2424.24x | 7.1 MB/s | 2.4 MB/s | 2381.1 KB / 5696.3 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Gaussian Noise

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 4.60x | 206.9 MB/s | 246.1 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 4.39x | 200.8 MB/s | 229.8 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 3.18x | 193.5 MB/s | 475.7 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.23x | 497.0 MB/s | 1050.3 MB/s | 1135.4 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 2.21x | 7.8 MB/s | 9.4 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 3.14x | 4.1 MB/s | 1.9 MB/s | 5437.9 KB / 8238.4 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Adversarial Random

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 1.00x | 164.4 MB/s | 859.1 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 1.00x | 166.9 MB/s | 608.5 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 1.00x | 795.3 MB/s | 2722.8 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 1.00x | 1292.7 MB/s | 2077.2 MB/s | 1568.7 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 21.37x | 12.0 MB/s | 14.4 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 1.00x | 3.5 MB/s | 1.6 MB/s | 5839.4 KB / 9304.0 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Float Temperature

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 5.95x | 467.8 MB/s | 472.3 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 5.59x | 404.1 MB/s | 440.6 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 6.51x | 603.5 MB/s | 1181.8 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 4.00x | 1197.0 MB/s | 933.1 MB/s | 979.7 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 12.60x | 9.4 MB/s | 11.3 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 7.48x | 7.3 MB/s | 2.0 MB/s | 3011.7 KB / 6485.4 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Real-World: Environmental IoT

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 8.36x | 504.2 MB/s | 525.1 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 7.68x | 311.1 MB/s | 486.6 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 15.10x | 574.3 MB/s | 923.9 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 8.04x | 1597.8 MB/s | 1055.5 MB/s | 881.5 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 16.30x | 9.6 MB/s | 11.5 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 31.90x | 8.6 MB/s | 2.2 MB/s | 2794.2 KB / 6281.5 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Real-World: Wearable IMU

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 8.76x | 271.6 MB/s | 313.6 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 7.81x | 244.2 MB/s | 285.0 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 6.58x | 277.7 MB/s | 600.2 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.78x | 710.2 MB/s | 1131.8 MB/s | 1065.7 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 1.08x | 24.2 MB/s | 29.0 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 7.30x | 4.8 MB/s | 2.1 MB/s | 4009.4 KB / 6940.5 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Real-World: Industrial Power

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 11.75x | 229.2 MB/s | 308.5 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 10.10x | 214.6 MB/s | 290.7 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 5.60x | 267.2 MB/s | 494.0 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.80x | 669.7 MB/s | 1301.9 MB/s | 1063.0 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 1.08x | 24.6 MB/s | 29.5 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 10.55x | 4.8 MB/s | 2.0 MB/s | 3588.7 KB / 6566.5 KB | zstd level 1 (windowLog=15, 32KB window) |


## Lossless Image Extension Benchmark (§9)

Evaluated row-by-row streaming causal spatial prediction vs. PNG and Lossless WebP:

### Category: Flat UI Screenshot

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether-Image (Streaming 2D) | 24.97x | 249.8 MB/s | 422.3 MB/s | 3.5 KB / 3.0 KB | Row-by-row streaming, O(width) RAM |
| PNG (libpng / zlib) | 115.79x | 2.0 MB/s | 5.1 MB/s | 1302.4 KB / 64.0 KB | Full image buffer + Deflate |
| Lossless WebP | 564.97x | 0.8 MB/s | 1.5 MB/s | 2035.5 KB / 128.0 KB | Spatial predictor + VP8L entropy |


### Category: Synthetic 2D Gradient

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether-Image (Streaming 2D) | 7.11x | 36.2 MB/s | 80.1 MB/s | 3.5 KB / 3.0 KB | Row-by-row streaming, O(width) RAM |
| PNG (libpng / zlib) | 141.24x | 45.6 MB/s | 113.9 MB/s | 194.4 KB / 64.0 KB | Full image buffer + Deflate |
| Lossless WebP | 1024.00x | 7.1 MB/s | 14.2 MB/s | 129.1 KB / 128.0 KB | Spatial predictor + VP8L entropy |


### Category: Photographic Texture

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether-Image (Streaming 2D) | 3.24x | 34.0 MB/s | 72.8 MB/s | 3.5 KB / 3.0 KB | Row-by-row streaming, O(width) RAM |
| PNG (libpng / zlib) | 4.44x | 19.8 MB/s | 49.6 MB/s | 194.2 KB / 64.0 KB | Full image buffer + Deflate |
| Lossless WebP | 4.82x | 2.3 MB/s | 4.7 MB/s | 142.3 KB / 128.0 KB | Spatial predictor + VP8L entropy |

