# Open Frame Studio v0.6.0

**Free, open-source kozijn (window frame) design software for the Dutch construction industry.**

Built by the [OpenAEC Foundation](https://github.com/OpenAEC-Foundation) — accessible, professional-grade tools for Architecture, Engineering & Construction.

![License: CC BY-SA 4.0](https://img.shields.io/badge/License-CC%20BY--SA%204.0-lightgrey.svg)
![Platform: Windows](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-blue)
![Built with Tauri 2](https://img.shields.io/badge/Built%20with-Tauri%202-FFC131)

> **Versie bumpen:** draai `node scripts/bump-version.mjs <nieuwe-versie>` (bv. `0.5.2`).
> Dat werkt de versie in één keer overal bij: `Cargo.toml` (alle Rust-crates + `ofs-cloud /health`),
> `ui/package.json` (waaruit de UI-titelbalk/menu/instellingen leest via `ui/src/lib/version.js`),
> `src-tauri/tauri.conf.json`, de `ofs-web`-pagina's en de README-titel. Voeg daarna een
> changelog-regel toe en draai `npm install` in `ui/` voor de lockfile.

---

## What is Open Frame Studio?

Open Frame Studio is a desktop application for designing, documenting, and exporting kozijnen (window and door frames). A free, professional-grade alternative to commercial kozijnsoftware.

### Key Features

- **2D Kozijn Editor** — Interactive SVG editor with grid subdivision, drag handles, dimension lines, and cell-type assignment
- **Interactive Profile Editor** — Visual point-by-point cross-section editing with drag handles, edge splitting, mirroring, snap-to-grid, and undo/redo
- **AI Configurator Assistant** — Natural language kozijn design via OpenAI-compatible API (works with OpenAI, Ollama, or any compatible endpoint). Type "Maak een draaikiep 900x1400" and it creates the kozijn.
- **Template Library** — Pre-built templates: draaikiepraam, dubbel draaikiepraam, schuifpui, voordeur, melkmeisje, melkmeisje met bovenlicht
- **80+ Profielen** — Uitgebreide profielbibliotheek voor hout, aluminium, PVC en hout-aluminium met realistische sponning-doorsnedes
- **3D Viewer** — Three.js-based 3D preview with frame extrusion and transparent glass panels
- **IFC Export** — Generate IFC files with ILS Houten Kozijnen v2.0 property sets for BIM workflows
- **DXF Export** — Workshop-ready DXF drawings with dimension lines
- **PDF Kozijnstaat** — Professional kozijnstaat reports (PDF + Excel)
- **Werkplaatstekening** — A3 workshop drawings with title block and OpenAEC branding
- **Blender/Bonsai Integration** — Send kozijnen directly to Blender via TCP socket for IFC modeling
- **Undo/Redo** — Full snapshot-based undo/redo with Ctrl+Z / Ctrl+Y
- **Dark & Light Theme** — OpenAEC Design System with Amber accent (#D97706)

### Materialen

| Categorie | Typen |
|---|---|
| **Hout** | KVT standaard profielen (54/67/78/90mm series), meranti, accoya, vuren, eiken, hardhout FSC, naaldhout |
| **Aluminium** | Standaard aluminium profielen |
| **Kunststof (PVC)** | Standaard PVC profielen |
| **Hout-Aluminium** | Combinatieprofielen |

---

## Changelog v0.6.0 (2026-07-14)

### Vrije 2D-editor = bron van waarheid, 3D en zaaglijst volgen 1-op-1
- **Hoofd-editor is nu de vrije tekeneditor** — elk kozijn met één knop om te zetten ("Vrije indeling activeren"); vak klikken → vullingpaneel (glas/raam/deur/paneel/borstwering, scharnierzijde, draairichting, glaslat, beslag); Ctrl+klik plaatst een tussenstijl op de klikpositie (Ctrl+Alt = tussendorpel); deellijnen slepen; undo/redo over de hele indeling
- **Proto → toepassen is 1-op-1** — de proto-tab tekent op de echte kozijnmaten en profielbreedte; toepassen ververst direct geometrie, maatvoering en 3D
- **3D volgt het model exact** — getrapte omtrek (melkmeisje) met dorpel onder het zijlicht en metselwerk i.p.v. een gat; verstek-hoeken (alu/PVC) echt op 45° afgekort; glas ín de sponning; vleugels als echte raamhout-/deurhout-extrusies; stolp = twee vleugels + stolpnaald
- **Zaaglijst = tekening** — layout-kozijnen krijgen een eigen ledenresolutie: doorlopende tussenstijlen, borstweringdorpel, per vak glas/vleugel/paneel/glaslatten; verstek besteld over de lange punt (ook bij default-verbindingen)
- **Verbindingen volgens verbindingsleer** — hout: dorpels doorlopend, stijlen ertussen gepend (KVT katern 15), haakse naden; PVC/alu: 45°-lasnaad, T-stijlen recht gecontramald; beslistabel + bronnen in `docs/verbindingsleer.md`; bewuste keuze in het verbindingenpaneel wint nu altijd van het materiaal-default
- **NL-tekenconventies (VKG 1.8)** — buitenaanzicht met draaisymbolen: brede zijde = scharnierzijde, doorgetrokken = naar buiten, streeplijn = naar binnen, pijl = schuivend
- **Vulling-instellingen blijven bewaard** — glaslat/draairichting/beslag overleven nu opslaan, kozijnwissel en undo (backend-model uitgebreid)
- **Maatketting sluitend** — kolom/rij toevoegen reserveert de dividerbreedte; browser kan nu ook dorpel- en lidprofielen instellen
- **CI draait `cargo test`** — 104 tests, waaronder regressietests op al het bovenstaande

---

## Changelog v0.5.2 (2026-07-09)

### Veldtest — 11 punten
- **Dubbelklik op vakvulling** — voegt geen ongewilde deellijn meer toe; bewerkingen resetten de zoom/pan niet langer (auto-fit alleen bij een nieuw kozijn)
- **Kleuren mee naar selectie** — gekozen RAL-kleur wordt nu optimistisch opgeslagen én in 2D en 3D getekend (backend gaf het kozijn ongewijzigd terug)
- **Vakken uitwisselen** — nieuwe ⇄-knop wisselt de inhoud van twee vakken in de vrije indeling
- **Vakvulling-diepte** — nieuw inzet/diepte-veld (`setbackMm`) per paneel, verwerkt in de 3D-plaatsing
- **Sjablonen kunststof + aluminium** — VEKA, Kömmerling, Reynaers CS 77, Schüco AWS 75.SI+ (verstek-hoeken)
- **Houtprofielen opgeschoond** — merk-clones verwijderd (goemaat/weekamp/webo/hebo/raamwerk), alle hout generiek
- **Hout aangevuld** — bovendorpel (waterhol), tussendorpel/kalf, glasroede 32×12 (gebronde KVT-maten)
- **Raamhout gecorrigeerd** — niet-bestaande `54×67` overal vervangen door KVT-`69×90`, sash-default 69mm, opdek-variant
- **Hoekverbindingen** — één gedeelde hoek-helper voedt zaaglijst én tekening; verstek-polygonen (alu/PVC), stijl-doorloop (hout)
- **Melkmeisje** — correct als deur + zijlicht met borstwering (metselwerk) eronder, getrapte omtrek; niet langer een raam met een leeg gat

### Versiebeheer
- **Eén bron van waarheid** — UI leest de versie uit `package.json`, `ofs-cloud` uit `CARGO_PKG_VERSION`
- **`scripts/bump-version.mjs`** — bumpt de versie in één commando overal (Cargo/package.json/tauri/ofs-web/README)

---

## Changelog v0.4.0 (2026-04-08)

### Rendering Overhaul
- **Arc/boog rendering herschreven** — correcte wiskundige berekening (center_y, acos hoeken, SVG sweep-flag), boog gaat nu naar boven
- **Rond kozijn** — 360° gesplitst in twee semicirkels, gevulde donut-rendering met outer + inner cirkel
- **Stijlen clippen** — bij getoogde kozijnen starten stijlen op de boogaanzet (arch_height), niet meer op y=0
- **Sponninglijnen** op alle frame members — 17mm offset lijn toont de sponning/glasgroef
- **Raamhout realistisch** — 3-laags rendering: buitenkant, sponninglijn, glaslatlijn (stippel) per EN standaard
- **Deurhout realistisch** — dikke onderdorpel (150mm), drempellijn, zelfde 3-laags detail
- **EN 12519 opening symbolen** — draai (driehoek basis=scharnier, punt=kruk), kiep, draai-kiep, schuif
- **Scharnier- en kruk-indicatoren** — kleine cirkels aan scharnierkant, T-lijn aan krukkant

### Maatvoering (NEN 3576)
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
| Export / Import | Native Rust in `ofs-core` (IFC, DXF, PDF, glTF, XLSX, CSV) |
| Styling | OpenAEC Design System (CSS custom properties + Tailwind) |

---

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/OpenAEC-Foundation/Open-Frame-Studio.git
cd Open-Frame-Studio

# Install frontend dependencies
cd ui && npm install && cd ..

# Run in development mode
cd ui && npm run tauri dev

# Build for production
cd ui && npm run tauri build
```

### Browser Preview (no Tauri required)

```bash
cd ui && npm run dev
```

The app includes a full browser mock layer, so you can develop and preview the UI without the Tauri runtime. There is also a real web build (`npm run build:web`, backed by the `ofs-wasm` crate).

**Web mode limitations:** the browser version does not yet match the desktop app. Vliesgevel (curtain wall) editing and file export/import (IFC, DXF, PDF, XLSX, CNC) are desktop-only until the wasm module is rebuilt against the current `ofs-core`. Custom templates do work in web mode — they are persisted in `localStorage`.

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
│       ├── validation.rs   # Business rules
│       ├── export/         # Native export — IFC, DXF, PDF, glTF, XLSX, CSV
│       └── import/         # Native import — IFC, DXF profiles, supplier catalogs
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
└── profiles/               # Profile library (JSON)
    ├── wood/               # Houten profielen
    ├── aluminum/           # Aluminium profielen
    ├── pvc/                # PVC profielen
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
