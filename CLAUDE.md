# Open Frame Studio

Parametrische kozijntekensoftware onder de OpenAEC Foundation.

## Tech Stack

- **Frontend**: Svelte 5 + Vite 6 (in `ui/`)
- **Backend**: Tauri 2.0 + Rust (in `src-tauri/`)
- **Core library**: Rust crate `ofs-core` — kozijn/vliesgevel model, geometry, profiles, validation, calculation, pricing, production, CNC
- **Export/Import**: native Rust in `ofs-core` (`src/export/`, `src/import/`) — IFC, DXF, PDF, glTF, XLSX, CSV
- **Web build**: `ofs-wasm` (wasm-bindgen wrapper over `ofs-core`) vervangt de Tauri backend in de browser
- **Cloud API**: `ofs-cloud` — axum REST API op port 3456
- **Blender integration**: TCP socket naar Bonsai addon op port 9876

## Project Structure

```
ofs-core/          Rust library — kozijn model, geometry, grid, profiles, validation,
                   calculation, pricing, quotation, thermal/energy, production, planning
  src/export/      Native export — IFC, DXF, PDF (kozijnstaat, labels, offerte), glTF, XLSX, CSV
  src/import/      Native import — IFC, DXF profiles, supplier catalogs (XLSX)
  src/cnc/         G-code generation + machine postprocessors
  src/vliesgevel/  Vliesgevel model — grid, geometry, production, validation
src-tauri/         Tauri app — commands, blender bridge, state management
  src/commands/    Tauri commands (kozijn, project, profiles, export_*, import_*, pricing,
                   production, cnc, vliesgevel, blender, ...)
  src/blender/     Blender TCP socket client
  src/models/      Shared data models
ofs-wasm/          wasm-bindgen wrapper over ofs-core for the web build
ofs-cloud/         Cloud API (axum) — project CRUD, in-memory storage; export/quotation
                   endpoints are stubs
ofs-web/           Standalone HTML pages — kozijn configurator, werkvloer (workshop) app,
                   AR preview (model-viewer)
ui/                Svelte frontend (desktop + web)
  src/components/  UI components (shell, editor, panels, profile-editor, viewer3d, project)
  src/stores/      Svelte stores (kozijn, project, profiles, vliesgevel, production, history, ui, ...)
  src/lib/tauri.js Backend abstraction — Tauri IPC in the app, ofs-wasm in the browser
  src/locales/     svelte-i18n translations (nl, en, de)
  src/styles/      CSS tokens + app styles (OpenAEC Design System)
profiles/          Kozijnprofiel JSON data (wood, pvc, aluminum, wood-aluminum)
```

## Conventions

- Internal unit: **millimeters**
- Project file format: `.ofs` (JSON)
- UI: Ribbon toolbar, Backstage file menu, dark/light themes
- Design tokens: Amber #D97706, Space Grotesk (headings), Inter (body)
- Language in code: English. UI strings via svelte-i18n (nl, en, de). Domain terms: Dutch where appropriate (kozijn, stijl, dorpel, etc.)

## Development

```bash
# Frontend (browser mock without Tauri)
cd ui && npm install && npm run dev

# Frontend in web mode (builds ofs-wasm first; requires wasm-pack)
cd ui && npm run dev:web

# Tauri (requires Rust toolchain)
cargo tauri dev

# Cloud API (http://localhost:3456)
cargo run -p ofs-cloud
```

## Build

```bash
# Desktop app
cargo tauri build

# Web build (ofs-wasm + Vite)
cd ui && npm run build:web
```

## Workspace

Rust workspace with four members: `src-tauri`, `ofs-core`, `ofs-wasm`, `ofs-cloud`. Shared dependencies in root `Cargo.toml`. `ofs-core` has `export` and `import` features (default on); `ofs-wasm` builds it with `default-features = false`. `ofs-web` is plain HTML, not a crate.
