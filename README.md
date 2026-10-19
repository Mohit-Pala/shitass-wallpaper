# shitass-wallpaper

Run a WGSL shader as the Windows desktop wallpaper, using `wgpu` (DX12/Vulkan) and the
Progman/WorkerW trick to draw behind the desktop icons.

```
cargo run --release                      # built-in shader
cargo run --release -- path\to\my.wgsl   # your own shader
```

A shader needs `vs_main` and `fs_main` entry points and this uniform at `@group(0) @binding(0)`:

```wgsl
struct Uniforms {
    resolution: vec2<f32>,
    time: f32,
    _pad: f32,
    mouse: vec2<f32>,
    _pad2: vec2<f32>,
};
```

See `shaders/default.wgsl`. Close the wallpaper by ending the process; Explorer repaints the
original wallpaper.

## Status

Scaffold only. Single window over the whole virtual desktop, 60 fps cap, cursor polled globally.
Not done yet: per-monitor windows, pause on fullscreen apps, shader hot-reload, tray icon.
