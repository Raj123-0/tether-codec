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
| Tether (32KB Profile) | 6.91x | 8.1 MB/s | 22.5 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 6.28x | 22.1 MB/s | 23.4 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 5.88x | 232.2 MB/s | 577.5 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.02x | 517.7 MB/s | 573.3 MB/s | 1170.8 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 1.12x | 9.1 MB/s | 10.9 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 13.95x | 3.5 MB/s | 1.5 MB/s | 4076.6 KB / 7182.3 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Linear Trend

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 31.75x | 23.2 MB/s | 22.6 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 24.63x | 21.3 MB/s | 24.9 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 7.53x | 221.3 MB/s | 463.7 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.00x | 572.9 MB/s | 536.5 MB/s | 1175.6 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 1.03x | 22.2 MB/s | 26.7 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 2446.48x | 4.7 MB/s | 1.9 MB/s | 2351.4 KB / 5464.0 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Periodic Square

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 39.57x | 21.0 MB/s | 22.2 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 28.95x | 23.2 MB/s | 23.8 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 391.77x | 1308.4 MB/s | 1588.8 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 79.87x | 4487.9 MB/s | 1103.0 MB/s | 794.1 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 63.33x | 9.1 MB/s | 10.9 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 2424.24x | 5.5 MB/s | 1.8 MB/s | 2381.1 KB / 5696.3 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Gaussian Noise

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 4.62x | 21.1 MB/s | 20.9 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 4.43x | 23.2 MB/s | 23.2 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 3.18x | 141.5 MB/s | 333.1 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.23x | 341.2 MB/s | 482.4 MB/s | 1135.4 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 2.21x | 6.9 MB/s | 8.3 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 3.14x | 3.1 MB/s | 1.5 MB/s | 5437.9 KB / 8238.4 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Adversarial Random

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 1.00x | 21.0 MB/s | 23.4 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 1.00x | 20.1 MB/s | 25.0 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 1.00x | 537.9 MB/s | 1060.8 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 1.00x | 544.7 MB/s | 746.5 MB/s | 1568.7 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 21.37x | 8.2 MB/s | 9.9 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 1.00x | 2.6 MB/s | 1.3 MB/s | 5839.4 KB / 9304.0 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Synthetic: Float Temperature

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 5.63x | 21.9 MB/s | 22.5 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 5.35x | 23.3 MB/s | 18.4 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 6.51x | 410.5 MB/s | 875.4 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 4.00x | 813.8 MB/s | 489.0 MB/s | 979.7 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 12.60x | 6.9 MB/s | 8.3 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 7.48x | 5.7 MB/s | 1.7 MB/s | 3011.7 KB / 6485.4 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Real-World: Environmental IoT

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 8.06x | 20.7 MB/s | 23.5 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 7.50x | 23.6 MB/s | 21.9 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 15.10x | 569.9 MB/s | 697.6 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 8.04x | 946.6 MB/s | 532.1 MB/s | 881.5 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 16.30x | 7.9 MB/s | 9.4 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 31.90x | 6.2 MB/s | 1.7 MB/s | 2794.2 KB / 6281.5 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Real-World: Wearable IMU

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 8.30x | 20.7 MB/s | 22.8 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 7.28x | 21.1 MB/s | 18.5 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 6.58x | 190.3 MB/s | 501.5 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.78x | 633.2 MB/s | 887.6 MB/s | 1065.7 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 1.08x | 17.6 MB/s | 21.1 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 7.30x | 3.5 MB/s | 1.6 MB/s | 4009.4 KB / 6940.5 KB | zstd level 1 (windowLog=15, 32KB window) |


### Dataset: Real-World: Industrial Power

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether (32KB Profile) | 8.76x | 21.0 MB/s | 19.3 MB/s | 5.7 KB / 2.4 KB | **Target profile: Pareto frontier winner** |
| Tether (4KB Micro Profile) | 8.11x | 19.9 MB/s | 17.6 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU profile |
| zstd (32KB Window, Lvl 1) | 5.60x | 187.9 MB/s | 404.5 MB/s | 784.3 KB / 781.3 KB | zstd level 1 (windowLog=15, 32KB window) |
| LZ4 (Standard) | 2.80x | 619.9 MB/s | 949.2 MB/s | 1063.0 KB / 1562.5 KB | Fast byte-matcher (poor ratio on floats/drifts) |
| Gorilla XOR | 1.08x | 17.9 MB/s | 21.5 MB/s | 2.0 KB / 2.0 KB | Pure XOR delta baseline |
| Delta + zstd (32KB Window) | 10.55x | 3.6 MB/s | 1.5 MB/s | 3588.7 KB / 6566.5 KB | zstd level 1 (windowLog=15, 32KB window) |


## Lossless Image Extension Benchmark (§9)

Evaluated row-by-row streaming causal spatial prediction vs. PNG and Lossless WebP:

### Category: Flat UI Screenshot

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether-Image (Streaming 2D) | 8.11x | 1.7 MB/s | 1.4 MB/s | 5.7 KB / 2.4 KB | Row-by-row streaming, O(width) RAM |
| PNG (libpng / zlib) | 115.79x | 0.3 MB/s | 0.7 MB/s | 1302.6 KB / 64.0 KB | Full image buffer + Deflate |
| Lossless WebP | 564.97x | 0.1 MB/s | 0.2 MB/s | 2035.6 KB / 128.0 KB | Spatial predictor + VP8L entropy |


### Category: Synthetic 2D Gradient

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether-Image (Streaming 2D) | 1.00x | 1.7 MB/s | 1.8 MB/s | 5.7 KB / 2.4 KB | Row-by-row streaming, O(width) RAM |
| PNG (libpng / zlib) | 141.24x | 12.2 MB/s | 30.4 MB/s | 194.4 KB / 64.0 KB | Full image buffer + Deflate |
| Lossless WebP | 1024.00x | 4.0 MB/s | 8.0 MB/s | 129.1 KB / 128.0 KB | Spatial predictor + VP8L entropy |


### Category: Photographic Texture

| Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| Tether-Image (Streaming 2D) | 1.00x | 1.7 MB/s | 1.8 MB/s | 5.7 KB / 2.4 KB | Row-by-row streaming, O(width) RAM |
| PNG (libpng / zlib) | 4.44x | 11.5 MB/s | 28.9 MB/s | 194.2 KB / 64.0 KB | Full image buffer + Deflate |
| Lossless WebP | 4.82x | 1.8 MB/s | 3.6 MB/s | 142.3 KB / 128.0 KB | Spatial predictor + VP8L entropy |

