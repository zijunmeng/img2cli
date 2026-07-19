# Roadmap - img2cli

This file tracks future architecture improvements, user experience features, and optimization milestones for the `img2cli` desktop application and daemon. Completed milestones are tracked in release notes and git history.

---

## 1. Platform Direction & Build Strategy
`img2cli` aims to remain a highly performant, single-binary background daemon with a lightweight, lazy-loaded configuration interface.
* **Primary Target**: Windows (`x86_64-pc-windows-gnu` / setup installer and portable zip).
* **Secondary Targets**: macOS (Apple Silicon & Intel DMG) and Linux (Portable AppImage).
* **Guiding Principle**: Zero-dependency background operation with low memory overhead (<25MB idle).

---

## 2. Milestone 1: Performance & Latency Optimization (Short-Term)

### A. Pre-emptive Screen Freezing (Zero UI Flicker)
* **Goal**: Drop region capture startup lag from `100ms - 200ms` down to near `0ms`.
* **Action**:
  1. Capture display pixels directly inside the Rust backend handler (`rdev` callback) using low-level OS drawing/capture APIs (Windows GDI, macOS Quartz/CGDisplay).
  2. Cache the captured image buffer instantly in memory.
  3. Wake and launch the fullscreen Tauri Webview window asynchronously to render the crop handlers over the cached memory image.

### B. SSH Keep-Alive Connection Pooling
* **Goal**: Reduce SFTP upload latency from `1.5s` down to under `200ms`.
* **Action**:
  1. Maintain a single multiplexed connection channel to the active remote host.
  2. Implement a background thread heartbeat ping loop to keep the SSH channel warm.
  3. Send captured screenshot byte streams directly through the pre-existing warm channel, avoiding TCP/SSH handshakes at trigger time.

---

## 3. Milestone 2: AI-First Pipeline Enhancements (Mid-Term)

### A. Local OCR & Code Block Extraction
* **Goal**: Automatically convert screenshot images into copyable Markdown text code blocks to save up to 90% of model API tokens.
* **Action**:
  1. Link Rust bindings to native platform OCR engines (Windows OCR Runtime, macOS Vision API).
  2. Scan cropped region pixels for code structures and terminal outputs.
  3. If text/code is recognized, automatically format it as a markdown code block (`````language ... `````) and paste it directly to the active cursor instead of uploading a graphical image.

### B. Zero-Trust API Key & Password Masking
* **Goal**: Prevent accidental leaks of credentials (e.g. OpenAI `sk-...`, passwords, private IPs) to public AI models.
* **Action**:
  1. Perform real-time OCR checks on selection crop.
  2. Run regex matching against standard private key formats, internal server IPs, and database strings.
  3. Alert the user with a single-click "Mask/Blur" option directly on the crop UI.

---

## 4. Milestone 3: Editor Annotation Overlays (Long-Term)

### A. Inline Crop Annotations
* **Goal**: Add vector-based drawing directly on the screenshot selection area.
* **Action**:
  1. Draw lightweight vector elements (Arrow, Highlight Rectangle, Mosaic/Blur brush) on a transparent HTML5 `<canvas>` inside the Vue crop overlay.
  2. Render annotations locally to the cropped image buffer before initiating compression or upload.

---

## 5. Milestone 4: Memory-Aware Screen Pinning (Future)

### A. Static Window Pinning (贴图)
* **Goal**: Pin screenshot captures on top of other applications without incurring Tauri Webview multi-process memory bloat.
* **Action**:
  1. Avoid full-blown Webview window instances for pinned frames.
  2. Explore creating lightweight, raw OS-native windows (via Rust `tao` or simple custom Win32/Cocoa window bindings) that render static image frames using CPU/GPU directly, keeping memory footprints under 30MB.
