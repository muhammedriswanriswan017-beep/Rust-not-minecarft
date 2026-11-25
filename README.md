# HPC Craft — Rust Edition (v0.1)

An independent, single-player voxel sandbox written in Rust with Bevy 0.15.3. No Minecraft source, textures, account, or launcher is required. This is a source-code prototype, not a compiled release or a drop-in replacement for Minecraft Java Edition.

## Windows quick start

1. Extract the ZIP to a normal folder.
2. Install Rust using the official installer at https://rustup.rs/ (stable MSVC toolchain). If prompted, install Visual Studio C++ Build Tools with the Desktop development with C++ workload and Windows SDK. Restart your terminal afterward.
3. Double-click `Play.bat` or open a terminal in the extracted HPC-Craft directory and run `cargo run --release`.
4. The first build downloads and compiles Bevy and its dependencies; internet access and free disk space are needed. Subsequent builds reuse the local build cache.
5. Click inside the game to capture the mouse. Press Esc to release it; close the window to quit.

To build without launching: `cargo build --release`. The executable is `target/release/hpc-craft.exe`. To run the included tests: `cargo test`.

## Controls

| Key | Action |
| --- | --- |
| W A S D | Move |
| Mouse | Look around |
| Space | Jump / fly up |
| Left Ctrl | Sprint |
| F | Toggle creative flight |
| Left Shift | Fly down |
| Left mouse | Break targeted block (except bottom layer) |
| Right mouse | Place selected block next to targeted block |
| 1–7 / mouse wheel | Select block type |
| R | Return to spawn |
| F5 | Save block changes |
| F9 | Reload saved world |
| Esc | Release mouse / pause movement |

## Implementation

- Fixed 128 × 48 × 128 voxel world, divided into 64 horizontal chunks (16 × 16 columns each).
- Deterministic terrain, hills, sandy patches, trees and underground cave pockets.
- Seven unlimited building materials; original flat vertex colours; no external assets.
- Face-culling chunk meshes: only exposed faces are emitted. Edits rebuild the affected chunk and adjacent chunks at boundaries.
- First-person movement, gravity, jumping, collision detection, grid ray traversal and creative flight.
- Local world saving (`hpc-world.bin` in the working directory). Save manually with F5. The next launch automatically loads a valid save. Player position is reset on load. To reset the world, close the game and rename the save file.
- Included logic tests for deterministic terrain, chunk invalidation, ray targeting and face winding.

## Limitations

No infinite streaming, mobs, crafting, health, multiplayer, Minecraft world import, pixel textures or day/night cycle yet. Lighting is simple and shadows are disabled. Bevy is pinned to 0.15.3 for API consistency. Rust/Cargo and a GPU window are not available in the creation environment, so compilation, unit tests and gameplay have not been verified. Performance has not been benchmarked.

## Linux

On a supported Linux desktop, install stable Rust, a C/C++ toolchain, pkg-config and the X11/Vulkan graphics development/runtime packages required by Bevy for your distro. Then run `cargo run --release`.
