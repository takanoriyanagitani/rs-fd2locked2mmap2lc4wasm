# Simple Benhmark

## macOS

- CPU: M3 Max 48 GB
- df: Avail 127 Gi

| mode           | user | sys | real | cpu% | rate      | ratio  |
|:--------------:|:----:|:---:|:----:|:----:|:---------:|:------:|
| native(simple) |  3.0 | 5.7 | 29.4 | 29%  | 557 MiB/s | (100%) |
| wasm(nosimd)   | 10.9 | 5.6 | 36.2 | 45%  | 453 MiB/s |   81%  |
| wasm(simd)     |  2.1 | 5.8 | 28.7 | 27%  | 570 MiB/s |  102%  |
| wasm(unroll8)  |  1.9 | 4.2 | 18.1 | 33%  | 905 MiB/s |  162%  |

## linux

- CPU: 13th gen Core i7 64GB
- df: Avail 194 Gi

| mode           | user | sys | real | cpu% | rate        | ratio  |
|:--------------:|:----:|:---:|:----:|:----:|:-----------:|:------:|
| native(simple) |  4.4 | 0.4 |  4.8 | 99%  | 3,413 MiB/s | (100%) |
| wasm(unroll8)  |  4.4 | 0.4 |  4.8 | 99%  | 3,413 MiB/s |  100%  |
