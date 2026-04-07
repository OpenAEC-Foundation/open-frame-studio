# Open Frame Studio v0.1.3

**Free, open-source kozijn (window frame) design software for the Dutch construction industry.**

Built by the [OpenAEC Foundation](https://github.com/OpenAEC-Foundation) — accessible, professional-grade tools for Architecture, Engineering & Construction.

![License: CC BY-SA 4.0](https://img.shields.io/badge/License-CC%20BY--SA%204.0-lightgrey.svg)
![Platform: Windows](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-blue)
![Built with Tauri 2](https://img.shields.io/badge/Built%20with-Tauri%202-FFC131)

---

## What is Open Frame Studio?

Open Frame Studio is a desktop application for designing, documenting, and exporting kozijnen (window and door frames). It fills the gap between expensive commercial tools like DeltaPi and MatrixKozijn and manual CAD workflows.

### Key Features

- **2D Kozijn Editor** — Interactive SVG editor with grid subdivision, drag handles, dimension lines, and cell-type assignment
- **Interactive Profile Editor** — Visual point-by-point cross-section editing with drag handles, edge splitting, mirroring, snap-to-grid, and undo/redo (DeltaPi-level functionality)
- **AI Configurator Assistant** — Natural language kozijn design via OpenAI-compatible API (works with OpenAI, Ollama, or any compatible endpoint). Type "Maak een draaikiep 900x1400" and it creates the kozijn.
- **Template Library** — Pre-built templates: draaikiepraam, dubbel draaikiepraam, schuifpui, voordeur, melkmeisje, melkmeisje met bovenlicht
- **80+ Profielen** — Verified profile library from 17+ Dutch and European manufacturers (HEBO, Goemaat, WEBO, Weekamp, KVT, Reynaers, Schüco, Gealan, VEKA, Kömmerling, Aluplast, Deceuninck, and more)
- **3D Viewer** — Three.js-based 3D preview with frame extrusion and transparent glass panels
- **IFC Export** — Generate IFC files with ILS Houten Kozijnen v2.0 property sets for BIM workflows
- **DXF Export** — Workshop-ready DXF drawings with dimension lines
- **PDF Kozijnstaat** — Professional kozijnstaat reports (PDF + Excel)
- **Werkplaatstekening** — A3 workshop drawings with title block and OpenAEC branding
- **Blender/Bonsai Integration** — Send kozijnen directly to Blender via TCP socket for IFC modeling
- **Undo/Redo** — Full snapshot-based undo/redo with Ctrl+Z / Ctrl+Y
- **Dark & Light Theme** — OpenAEC Design System with Amber accent (#D97706)

### Materials Supported

| Category | Manufacturers |
|---|---|
| **Hout** | KVT Standaard, ABC Profiel, Raamhout NL, WEBO, HEBO, Goemaat, Weekamp, Raamwerk, Meranti, Accoya, Hardhout FSC, Naaldhout |
| **Aluminium** | Reynaers, Schüco, Generic |
| **Kunststof (PVC)** | Gealan, VEKA, Kömmerling, Aluplast, Deceuninck, Generic |
| **Hout-Aluminium** | Hout-Aluminium combinaties |

---

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop framework | [Tauri 2.0](https://v2.tauri.app/) |
| Frontend | [Svelte 5](https://svelte.dev/) + [Vite 6](https://vitejs.dev/) |
| Core logic | Rust (`ofs-core` crate) |
| 2D Editor | SVG (reactive Svelte bindings) |
| 3D Viewer | [Three.js](https://threejs.org/) |
| IFC Export | Python sidecar ([IfcOpenShell](https://ifcopenshell.org/)) |
| DXF Export | Python sidecar ([ezdxf](https://ezdxf.readthedocs.io/)) |
| PDF Export | Python sidecar ([ReportLab](https://www.reportlab.com/)) |
| Styling | OpenAEC Design System (CSS custom properties + Tailwind) |

---

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Python](https://www.python.org/) 3.10+ (for export features)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/OpenAEC-Foundation/Open-Frame-Studio.git
cd Open-Frame-Studio

# Install frontend dependencies
cd ui && npm install && cd ..

# Install Python dependencies (optional, for export)
cd python && pip install -r requirements.txt && cd ..

# Run in development mode
cd ui && npm run tauri dev

# Build for production
cd ui && npm run tauri build
```

### Browser Preview (no Tauri required)

```bash
cd ui && npm run dev
```

The app includes a full browser mock layer, so you can develop and preview the UI without the Tauri runtime.

---

## Project Structure

```
Open-Frame-Studio/
├── Cargo.toml              # Rust workspace root
├── ofs-core/               # Pure Rust library (no Tauri dependency)
│   └── src/
│       ├── kozijn.rs       # Core data model
│       ├── geometry.rs     # 2D/3D geometry computation
│       ├── grid.rs         # Grid subdivision + templates
│       ├── profile.rs      # Profile definitions
│       └── validation.rs   # Business rules
├── src-tauri/              # Tauri application
│   └── src/
│       ├── main.rs         # Entry point + command registration
│       ├── state.rs        # Application state
│       └── commands/       # Tauri IPC commands
├── ui/                     # Svelte frontend
│   └── src/
│       ├── App.svelte      # Main app with workspace tabs
│       ├── components/     # Shell, Editor, Panels, Profile Editor, Viewer3D
│       ├── stores/         # Svelte stores (project, kozijn, profiles, AI assistant, history)
│       ├── styles/         # OpenAEC design tokens & themes
│       └── lib/            # Tauri invoke wrapper, AI tools & system prompt, browser mocks
├── python/                 # Python sidecar for exports
│   ├── ofs_ifc/            # IFC generation (IfcOpenShell)
│   ├── ofs_dxf/            # DXF generation (ezdxf)
│   └── ofs_pdf/            # PDF kozijnstaat + werkplaatstekening
└── profiles/               # Profile library (JSON)
    ├── wood/               # 12 manufacturers
    ├── aluminum/           # 3 manufacturers
    ├── pvc/                # 6 manufacturers
    └── wood-aluminum/      # Combinatieprofielen
```

---

## Internal Units

All dimensions are stored in **millimeters** (kozijnindustrie standaard). The UI can display in mm or meters.

---

## Standards

- **ILS Houten Kozijnen v2.0** — Dutch BIM standard property sets for wooden window frames
- **IFC 4** — Industry Foundation Classes for BIM interoperability

---

## Contributing

Contributions are welcome! Please open an issue or pull request.

---

## License

This work is licensed under the [Creative Commons Attribution-ShareAlike 4.0 International License](https://creativecommons.org/licenses/by-sa/4.0/).

© 2026 OpenAEC Foundation

---

## Related Projects

- [Open Field Studio](https://github.com/OpenAEC-Foundation/Open-Field-Studio) — Construction inspection & QA tool
- [Open PDF Studio](https://github.com/OpenAEC-Foundation/open-pdf-studio) — PDF editor & annotator
- [Open Heatloss Studio](https://github.com/OpenAEC-Foundation/open-heatloss-studio) — Heat loss calculations (ISSO 51:2023)
- [building.py](https://github.com/OpenAEC-Foundation/building.py) — Python library for BIM modeling
