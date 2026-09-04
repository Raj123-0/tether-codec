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
| Tether (32KB Profile) | 14.96x | 260.3 MB/s | 316.4 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 12.36x | 251.3 MB/s | 300.8 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 5.88x | 276.4 MB/s | 570.4 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.02x | 541.4 MB/s | 966.1 MB/s | 1170.8 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 1.12x | 12.4 MB/s | 14.9 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 13.95x | 5.1 MB/s | 2.0 MB/s | 4076.6 KB / 7182.3 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Linear Trend

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 31.83x | 363.9 MB/s | 431.0 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 21.78x | 291.1 MB/s | 412.4 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 7.53x | 283.5 MB/s | 506.8 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.00x | 569.3 MB/s | 620.8 MB/s | 1175.6 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 1.03x | 25.9 MB/s | 31.1 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 2446.48x | 6.0 MB/s | 2.5 MB/s | 2351.4 KB / 5464.0 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Periodic Square

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 64.21x | 492.4 MB/s | 625.9 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 38.50x | 409.7 MB/s | 622.7 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 391.77x | 2191.7 MB/s | 1987.3 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 79.87x | 7012.3 MB/s | 1101.9 MB/s | 794.1 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 63.33x | 14.6 MB/s | 17.5 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 2424.24x | 7.4 MB/s | 2.3 MB/s | 2381.1 KB / 5696.3 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Gaussian Noise

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 4.60x | 182.7 MB/s | 206.1 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 4.39x | 164.5 MB/s | 189.1 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 3.18x | 123.7 MB/s | 350.1 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.23x | 362.0 MB/s | 650.1 MB/s | 1135.4 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 2.21x | 8.9 MB/s | 10.6 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 3.14x | 4.2 MB/s | 1.9 MB/s | 5437.9 KB / 8238.4 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Adversarial Random

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 1.00x | 190.6 MB/s | 804.1 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 1.00x | 171.4 MB/s | 1057.4 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 1.00x | 774.0 MB/s | 2025.9 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 1.00x | 1055.8 MB/s | 1312.0 MB/s | 1568.7 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 21.37x | 11.8 MB/s | 14.2 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 1.00x | 3.5 MB/s | 1.6 MB/s | 5839.4 KB / 9304.0 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Float Temperature

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 5.95x | 493.8 MB/s | 418.1 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 5.59x | 370.5 MB/s | 383.9 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 6.51x | 630.4 MB/s | 1333.8 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 4.00x | 1338.7 MB/s | 1143.2 MB/s | 979.7 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 12.60x | 9.0 MB/s | 10.8 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 7.48x | 7.7 MB/s | 2.1 MB/s | 3011.7 KB / 6485.4 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Real-World: Environmental IoT

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 8.36x | 448.2 MB/s | 481.0 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 7.68x | 449.2 MB/s | 471.5 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 15.10x | 492.0 MB/s | 876.5 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 8.04x | 1549.4 MB/s | 986.3 MB/s | 881.5 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 16.30x | 9.2 MB/s | 11.1 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 31.90x | 8.3 MB/s | 2.1 MB/s | 2794.2 KB / 6281.5 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Real-World: Wearable IMU

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 8.76x | 282.8 MB/s | 285.7 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 7.81x | 230.0 MB/s | 270.6 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 6.58x | 197.8 MB/s | 586.9 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.78x | 627.7 MB/s | 1312.0 MB/s | 1065.7 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 1.08x | 24.1 MB/s | 28.9 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 7.30x | 4.9 MB/s | 2.1 MB/s | 4009.4 KB / 6940.5 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Real-World: Industrial Power

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 11.75x | 258.0 MB/s | 283.5 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 10.10x | 232.7 MB/s | 256.4 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 5.60x | 261.7 MB/s | 588.9 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.80x | 493.6 MB/s | 939.2 MB/s | 1063.0 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 1.08x | 22.3 MB/s | 26.8 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 10.55x | 4.9 MB/s | 2.1 MB/s | 3588.7 KB / 6566.5 KB | zstd level 1 (windowLog=15, 32KB window) |


## Lossless Image Extension Benchmark (§9)

Evaluated row-by-row streaming causal spatial prediction vs. PNG and Lossless WebP:

### Category: Flat UI Screenshot

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether-Image (Streaming 2D) | 37.60x | 361.1 MB/s | 850.3 MB/s | 3.5 KB / 3.0 KB | Row-by-row streaming, O(width) RAM |
| PNG (libpng / zlib) | 115.79x | 2.0 MB/s | 5.0 MB/s | 1302.5 KB / 64.0 KB | Full image buffer + Deflate |
| Lossless WebP | 564.97x | 0.8 MB/s | 1.6 MB/s | 2035.6 KB / 128.0 KB | Spatial predictor + VP8L entropy |


### Category: Synthetic 2D Gradient

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether-Image (Streaming 2D) | 14.46x | 56.6 MB/s | 314.4 MB/s | 3.5 KB / 3.0 KB | Row-by-row streaming, O(width) RAM |
| PNG (libpng / zlib) | 141.24x | 45.3 MB/s | 113.3 MB/s | 194.4 KB / 64.0 KB | Full image buffer + Deflate |
| Lossless WebP | 1024.00x | 7.0 MB/s | 14.1 MB/s | 129.1 KB / 128.0 KB | Spatial predictor + VP8L entropy |


### Category: Photographic Texture

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether-Image (Streaming 2D) | 3.04x | 34.1 MB/s | 69.4 MB/s | 3.5 KB / 3.0 KB | Row-by-row streaming, O(width) RAM |
| PNG (libpng / zlib) | 4.44x | 16.0 MB/s | 40.0 MB/s | 194.2 KB / 64.0 KB | Full image buffer + Deflate |
| Lossless WebP | 4.82x | 2.4 MB/s | 4.8 MB/s | 142.3 KB / 128.0 KB | Spatial predictor + VP8L entropy |

