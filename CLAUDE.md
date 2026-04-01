# Open Frame Studio

Parametrische kozijntekensoftware onder de OpenAEC Foundation.

## Tech Stack

- **Frontend**: Svelte 4 + Vite 5 (in `ui/`)
- **Backend**: Tauri 2.0 + Rust (in `src-tauri/`)
- **Core library**: Rust crate `ofs-core` (geometry, grid, kozijn, profile, validation)
- **Export sidecar**: Python (in `python/`) — IFC, DXF, PDF generatie
- **Blender integration**: TCP socket naar Bonsai addon op port 9876

## Project Structure

```
ofs-core/          Rust library — kozijn model, geometry, profiles, validation
src-tauri/         Tauri app — commands, blender bridge, state management
  src/commands/    Tauri commands (kozijn, project, export_ifc, export_dxf, export_pdf, blender)
  src/blender/     Blender TCP socket client
  src/models/      Shared data models
ui/                Svelte frontend
  src/components/  UI components (shell, editor, panels, viewer3d, project)
  src/stores/      Svelte stores (kozijn, project, profiles, history, ui)
  src/lib/tauri.js Browser mock layer for dev without Tauri
  src/styles/      CSS tokens + app styles (OpenAEC Design System)
python/            Python sidecar
  ofs_ifc/         IFC generator + ILS properties
  ofs_dxf/         DXF generator
  ofs_pdf/         PDF kozijnstaat + werkplaatstekening
profiles/          Kozijnprofiel JSON data (wood, pvc, aluminum, wood-aluminum)
```

## Conventions

- Internal unit: **millimeters**
- Project file format: `.ofs` (JSON)
- UI: Ribbon toolbar, Backstage file menu, dark/light themes
- Design tokens: Amber #D97706, Space Grotesk (headings), Inter (body)
- Language in code: English. UI labels and domain terms: Dutch where appropriate (kozijn, stijl, dorpel, etc.)

## Development

```bash
# Frontend
cd ui && npm install && npm run dev

# Tauri (requires Rust toolchain)
cargo tauri dev

# Python sidecar
cd python && pip install -r requirements.txt
```

## Build

```bash
cargo tauri build
```

## Workspace

Rust workspace with two members: `src-tauri` and `ofs-core`. Shared dependencies in root `Cargo.toml`.

