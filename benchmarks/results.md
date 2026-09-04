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
| Tether (32KB Profile) | 14.96x | 249.6 MB/s | 309.8 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 12.36x | 222.9 MB/s | 302.8 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 5.88x | 267.0 MB/s | 621.7 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.02x | 536.3 MB/s | 909.7 MB/s | 1170.8 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 0.97x | 7.4 MB/s | 8.8 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 13.95x | 4.5 MB/s | 2.1 MB/s | 4076.6 KB / 7182.3 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Linear Trend

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 31.83x | 327.7 MB/s | 479.5 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 21.78x | 321.9 MB/s | 327.5 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 7.53x | 266.6 MB/s | 283.8 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.00x | 618.9 MB/s | 584.1 MB/s | 1175.6 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 1.83x | 14.5 MB/s | 17.5 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 2446.48x | 6.0 MB/s | 2.4 MB/s | 2351.4 KB / 5464.0 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Periodic Square

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 402.21x | 460.7 MB/s | 941.3 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 201.97x | 309.7 MB/s | 973.0 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 391.77x | 2735.5 MB/s | 3029.9 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 79.87x | 6663.2 MB/s | 1431.4 MB/s | 794.1 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 39.75x | 13.6 MB/s | 16.3 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 2424.24x | 7.4 MB/s | 2.4 MB/s | 2381.1 KB / 5696.3 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Gaussian Noise

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 4.60x | 210.4 MB/s | 266.2 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 4.39x | 199.8 MB/s | 250.3 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 3.18x | 198.0 MB/s | 481.6 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.23x | 501.8 MB/s | 1196.2 MB/s | 1135.4 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 0.97x | 4.9 MB/s | 5.9 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 3.14x | 4.2 MB/s | 1.9 MB/s | 5437.9 KB / 8238.4 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Adversarial Random

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 1.00x | 183.1 MB/s | 1072.8 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 1.00x | 176.4 MB/s | 857.4 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 1.00x | 818.0 MB/s | 2084.5 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 1.00x | 1014.1 MB/s | 1404.5 MB/s | 1568.7 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 0.97x | 3.7 MB/s | 4.5 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 1.00x | 3.5 MB/s | 1.6 MB/s | 5839.4 KB / 9304.0 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Float Temperature

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 5.95x | 493.7 MB/s | 480.8 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 5.59x | 434.0 MB/s | 380.5 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 6.51x | 600.2 MB/s | 1156.7 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 4.00x | 1329.2 MB/s | 1240.1 MB/s | 979.7 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 5.52x | 7.1 MB/s | 8.5 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 7.48x | 7.8 MB/s | 2.1 MB/s | 3011.7 KB / 6485.4 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Real-World: Environmental IoT

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 35.57x | 436.2 MB/s | 466.6 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 25.73x | 384.3 MB/s | 429.3 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 15.10x | 602.5 MB/s | 937.3 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 8.04x | 1680.5 MB/s | 936.4 MB/s | 881.5 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 7.87x | 7.6 MB/s | 9.1 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 31.90x | 8.6 MB/s | 2.2 MB/s | 2794.2 KB / 6281.5 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Real-World: Wearable IMU

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 8.76x | 261.8 MB/s | 298.2 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 7.81x | 241.5 MB/s | 264.9 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 6.58x | 279.7 MB/s | 594.1 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.78x | 714.0 MB/s | 1311.3 MB/s | 1065.7 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 1.84x | 13.0 MB/s | 15.6 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 7.30x | 4.6 MB/s | 2.1 MB/s | 4009.4 KB / 6940.5 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Real-World: Industrial Power

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 11.75x | 234.3 MB/s | 304.2 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 10.10x | 217.2 MB/s | 268.7 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 5.60x | 262.5 MB/s | 588.2 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.80x | 670.8 MB/s | 1278.2 MB/s | 1063.0 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 1.85x | 13.3 MB/s | 16.0 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 10.55x | 4.9 MB/s | 2.2 MB/s | 3588.7 KB / 6566.5 KB | zstd level 1 (windowLog=15, 32KB window) |


## Lossless Image Extension Benchmark (§9)

Evaluated row-by-row streaming causal spatial prediction vs. PNG and Lossless WebP:

### Category: Flat UI Screenshot

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether-Image (Streaming 2D) | 96.80x | 160.0 MB/s | 685.3 MB/s | 3.5 KB / 3.0 KB | Row-by-row streaming, O(width) RAM |
| PNG (libpng / zlib) | 115.79x | 2.0 MB/s | 5.0 MB/s | 1302.4 KB / 64.0 KB | Full image buffer + Deflate |
| Lossless WebP | 564.97x | 0.8 MB/s | 1.6 MB/s | 2035.7 KB / 128.0 KB | Spatial predictor + VP8L entropy |


### Category: Synthetic 2D Gradient

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether-Image (Streaming 2D) | 14.46x | 57.5 MB/s | 370.0 MB/s | 3.5 KB / 3.0 KB | Row-by-row streaming, O(width) RAM |
| PNG (libpng / zlib) | 141.24x | 47.5 MB/s | 118.9 MB/s | 194.4 KB / 64.0 KB | Full image buffer + Deflate |
| Lossless WebP | 1024.00x | 7.3 MB/s | 14.6 MB/s | 129.1 KB / 128.0 KB | Spatial predictor + VP8L entropy |


### Category: Photographic Texture

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether-Image (Streaming 2D) | 3.04x | 34.2 MB/s | 70.0 MB/s | 3.5 KB / 3.0 KB | Row-by-row streaming, O(width) RAM |
| PNG (libpng / zlib) | 4.44x | 19.5 MB/s | 48.7 MB/s | 194.2 KB / 64.0 KB | Full image buffer + Deflate |
| Lossless WebP | 4.82x | 2.5 MB/s | 5.0 MB/s | 142.3 KB / 128.0 KB | Spatial predictor + VP8L entropy |

