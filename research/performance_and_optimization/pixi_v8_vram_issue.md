There is a significant degradation in VRAM management in v8 when creating Sprites from textures.

This leads to excessive GPU memory consumption, which can cause browser crashes, particularly in Safari on iOS where available VRAM is less in comparison with other platforms.

### Memory Consumption Breakdown

| Pixi Version | Method Used | Playground Tab - Memory Footprint | Image Cache | GPU Memory | JS Memory | GPU Process - Memory Footprint | GPU Process - GPU Memory |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **v7** | `Texture.from` | 254MB | 732MB | 26.6MB | 69,232K | 1.1GB | 26.6MB |
| **v8** | `Texture.from` | 377MB | 9.4KB | **747MB** | 126,000K | **2.2GB** | 761MB |
| **v8** | `Assets.load` | **1.0GB** | 9.4KB | 26.0MB | 174,320K | **1.2GB** | 42.5MB |

*   **Pixi v8 using `Texture.from` results in a massive increase in GPU memory consumption** compared to v7.
*   **Pixi v8 using `Assets.load` results in significantly higher overall memory usage**, especially on the main process.
*   **The issue is particularly problematic in Safari (iOS 17.5.1), where the tab crashes immediately.**

