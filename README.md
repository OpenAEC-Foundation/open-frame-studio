# Open Frame Studio v0.2.0

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

## Changelog v0.2.0 (2026-04-08)

### Rendering Overhaul
- **Arc/boog rendering herschreven** — correcte wiskundige berekening (center_y, acos hoeken, SVG sweep-flag), boog gaat nu naar boven
- **Rond kozijn** — 360° gesplitst in twee semicirkels, gevulde donut-rendering met outer + inner cirkel
- **Stijlen clippen** — bij getoogde kozijnen starten stijlen op de boogaanzet (arch_height), niet meer op y=0
- **Sponninglijnen** op alle frame members — 17mm offset lijn toont de sponning/glasgroef
- **Raamhout realistisch** — 3-laags rendering: buitenkant, sponninglijn, glaslatlijn (stippel) per EN standaard
- **Deurhout realistisch** — dikke onderdorpel (150mm), drempellijn, zelfde 3-laags detail
- **EN 12519 opening symbolen** — draai (driehoek basis=scharnier, punt=kruk), kiep, draai-kiep, schuif
- **Scharnier- en kruk-indicatoren** — kleine cirkels aan scharnierkant, T-lijn aan krukkant

### Maatvoering (NEN 3576 / GA Kozijn stijl)
- **3-niveau systeem** — Niveau 1: houtdiktes + vakmaten (complete maatketen), Niveau 2: dagmaat, Niveau 3: buitenwerkse maat
- **Rechts-labels 90° geroteerd** — geen overlap meer bij verticale dimensies
- **Invoerveld vergroot** — 100/zoom breed, font schaalt mee met zoom

### Nieuwe Features
- **Hoekoplossingen UI** — JointPanel met 4 hoeken (pen/slis, verstek, contramal, stomp), quick-apply knoppen, kleurgecodeerde canvas-indicatoren
- **Eigen sjablonen** — "Opslaan als sjabloon" knop in ribbon, dynamische lijst met ingebouwd + custom
- **Profielbibliotheek browser** — "Laden uit bibliotheek" in ShapeManager, laadt profiel parameters
- **Vrij tekenen profielen** — freeform drawing mode: sleep punten, dubbelklik = punt toevoegen, verwijder punten, undo, 5mm snap-grid
- **Houtsoort per onderdeel** — per-member materiaal velden (top/bottom/left/right_material) in datamodel
- **Sash profiel selector gewired** — raam/deur profielen nu wijzigbaar via dropdown (was TODO)

### Data Verbeteringen
- **21 profiel JSONs verrijkt** — alle ~65 profielen (hout, aluminium, PVC, hout-alu) hebben nu L-vormige sponning cross-sections
- **Profiel doorsnede** — hatch pattern en sponning annotatie in ProfileCrossSection component
- **Edge/spouwlat indicatoren** — paarse stippellijnen op canvas per frame-zijde
- **Default 4 hoekverbindingen** — elk nieuw kozijn start met 4 pen/slis joints

### Bugfixes
- **Calculatie tabel** — 5 verkeerde i18n keys gecorrigeerd, colgroup herbalanceerd naar 100%
- **3D Viewer** — `$state()` reactivity fix, WebGL detectie, deep change tracking via JSON serialisatie
- **AI Assistent** — default Ollama endpoint, gedetailleerde foutmelding met oplossingsrichtingen
- **ArchedTrapezoid** — toegevoegd aan is_arched check (geen dubbel bovendorpel meer)

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
