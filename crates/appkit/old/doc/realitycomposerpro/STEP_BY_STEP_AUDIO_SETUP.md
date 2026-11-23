# Step-by-Step: Audio Setup

> Guide to audio playback and spatial audio in rust-kit-pro

---

## 🗺 Documentation Map

- [← Back to Reality Composer Pro Guide](./JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [🔊 Audio Engine API](../../../../../phi/crates/rust-kit-pro/src/audio_engine.rs)
- [🏠 RealityKit Home](../realitykit/-realitykit.md)

---

## 📊 Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| **Audio Engine** | ✅ Working | cpal backend initialized |
| **Playback** | ✅ Working | WAV, MP3, OGG support |
| **Spatial Audio** | ✅ Working | 3D positioned sounds |
| **Volume Control** | ✅ Working | Master + per-source |
| **Looping** | ✅ Working | Seamless loops |
| **Panning** | ✅ Working | Stereo positioning |
| **AudioResource** | ✅ Working | ECS component |
| **Streaming** | ⚠️ Partial | Basic streaming only |
| **Effects** | ❌ Not Started | Reverb, EQ planned |

**Last Updated:** 2024-01-20  
**App Status:** ✅ Running & Ready  
**Crate:** `phi/crates/rust-kit-pro/src/audio_engine.rs`

---

## Overview

rust-kit-pro includes a full audio engine with spatial 3D audio support, allowing you to place sound sources in your scene and hear them from the correct directions.

---

## Features

| Feature | Description |
|---------|-------------|
| **Playback** | WAV, MP3, OGG support |
| **Spatial Audio** | 3D positioned sounds |
| **Volume Control** | Per-source and master volume |
| **Looping** | Seamless loop support |
| **Panning** | Stereo left/right control |

---

## Step 1: Initialize Audio Engine

```rust
use rust_kit_pro::audio_engine::AudioEngine;

fn init_audio() -> Result<(), String> {
    let mut engine = AudioEngine::new();
    engine.init()?;
    
    println!("Audio initialized at {} Hz", engine.sample_rate);
    Ok(())
}
```

---

## Step 2: Load Audio File

```rust
fn load_sound(engine: &mut AudioEngine, path: &str) -> Result<AudioPlaybackController, String> {
    let controller = engine.load_audio(path)?;
    println!("Loaded: {}", path);
    Ok(controller)
}
```

**Supported Formats:**
- `.wav` - Uncompressed audio
- `.mp3` - Compressed audio
- `.ogg` - Vorbis audio

---

## Step 3: Play Audio

```rust
fn play_sound(engine: &mut AudioEngine, controller: &AudioPlaybackController) {
    engine.play(controller);
}

fn pause_sound(engine: &mut AudioEngine, controller: &AudioPlaybackController) {
    engine.pause(controller);
}

fn stop_sound(engine: &mut AudioEngine, controller: &AudioPlaybackController) {
    engine.stop(controller);
}
```

---

## Step 4: Spatial Audio (3D Positioning)

```rust
fn setup_spatial_audio(
    engine: &mut AudioEngine,
    controller: &AudioPlaybackController,
    position: [f32; 3]
) {
    // Set 3D position
    engine.set_position(controller, position);
    
    // Pan left/right based on X position (-1.0 to 1.0)
    engine.set_pan(controller, 0.0); // Center
}
```

**Position Coordinates:**
- `[x, y, z]` where x is left/right
- Sounds auto-pan based on position
- Distance attenuation applied

---

## Step 5: Volume Control

```rust
fn set_volumes(engine: &mut AudioEngine, controller: &AudioPlaybackController) {
    // Master volume (all sounds)
    engine.set_master_volume(0.8); // 80%
    
    // Per-source volume
    engine.set_volume(controller, 0.5); // 50%
}
```

---

## Step 6: Looping

```rust
fn loop_sound(engine: &mut AudioEngine, controller: &AudioPlaybackController) {
    engine.set_looping(controller, true);
    engine.play(controller); // Will loop forever
}
```

---

## Full Example

```rust
use rust_kit_pro::audio_engine::AudioEngine;

fn setup_scene_audio() -> Result<(), String> {
    let mut engine = AudioEngine::new();
    engine.init()?;
    
    // Load sounds
    let bgm = engine.load_audio("assets/audio/background.mp3")?;
    let sfx = engine.load_audio("assets/audio/click.wav")?;
    
    // Setup BGM
    engine.set_looping(&bgm, true);
    engine.set_volume(&bgm, 0.3);
    engine.play(&bgm);
    
    // Setup spatial SFX
    engine.set_position(&sfx, [5.0, 0.0, 2.0]); // Right side
    engine.set_volume(&sfx, 0.8);
    engine.play(&sfx);
    
    Ok(())
}
```

---

## Shutdown

```rust
fn cleanup(engine: &mut AudioEngine) {
    engine.shutdown();
}
```

---

## Next Steps

- [Animation Editor](STEP_BY_STEP_ANIMATION_EDITOR.md)
- [Physics Simulation](STEP_BY_STEP_PHYSICS_SIMULATION.md)

---

*Generated for rust-kit-pro v0.1.0*
