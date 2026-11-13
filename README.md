# Open Frame Studio v2026.46.2

**Open-source kozijnontwerp voor de Nederlandse bouwketen.** De desktopapp combineert tekenen, projectgegevens en een beperkte IFC4-uitwisseling. Deze bètaversie is een ontwikkelstand, geen gevalideerd pakket voor producentencalculatie, normverklaringen of machineaansturing.

Built by the [OpenAEC Foundation](https://github.com/OpenAEC-Foundation) — accessible, professional-grade tools for Architecture, Engineering & Construction.

![Platform: Windows](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-blue)
![Built with Tauri 2](https://img.shields.io/badge/Built%20with-Tauri%202-FFC131)

> **Release-status:** `v2026.46.2` is de laatste release (alfa) voor ontwerp- en BIM-uitwisselproeven. De installers zijn geen productie- of normvrijgave; zie de [releasecontroles](docs/release-readiness.md) en [GitHub Releases](https://github.com/OpenAEC-Foundation/open-frame-studio/releases).

> **Updaten:** de desktopapp heeft nog geen automatische updater. Een nieuwe GitHub-release verschijnt daarom niet vanzelf in een al geïnstalleerde app. Download de passende installer rechtstreeks van de [laatste release (`latest`)](https://github.com/OpenAEC-Foundation/open-frame-studio/releases/latest) en voer die uit, of werk bij via de OpenAEC Installer. Die leest de laatste release uit een feed die periodiek wordt vernieuwd, dus een update kan er een paar uur later staan. Een release moet daarvoor als stabiele `latest` zijn gemarkeerd; een prerelease ziet de installer niet.

> **Versie bumpen:** draai `node scripts/bump-version.mjs --next` voor het volgende `JJJJ.W.N`-nummer
> volgens de ISO-week in Nederland, of geef expliciet een nummer op (bijvoorbeeld `2026.46.0`).
> Het script werkt Cargo, Tauri, UI, beide lockfiles, losse webpagina's en de README-titel bij.
> Controleer daarna met `node scripts/check-version.mjs vJJJJ.W.N` en voeg releasenotities toe.

---

## What is Open Frame Studio?

Open Frame Studio is een desktopapp voor het tekenen, documenteren en uitwisselen van kozijnen. De Rust-kern wordt gedeeld met een beperkte browservariant. De cloud-API en losse werkvloerpagina's zijn nog niet aangesloten op dezelfde projectworkflow.

### Key Features

- **2D Kozijn Editor** — Interactive SVG editor with grid subdivision, drag handles, dimension lines, and cell-type assignment
- **Interactive Profile Editor** — Visual point-by-point cross-section editing with drag handles, edge splitting, mirroring, snap-to-grid, and undo/redo
- **AI Configurator Assistant** — Natural language kozijn design via OpenAI-compatible API (works with OpenAI, Ollama, or any compatible endpoint). Type "Maak een draaikiep 900x1400" and it creates the kozijn.
- **Template Library** — Pre-built templates: draaikiepraam, dubbel draaikiepraam, schuifpui, voordeur, melkmeisje, melkmeisje met bovenlicht
- **Profielbibliotheek** — Voorbeeldprofielen voor hout, aluminium, PVC en hout-aluminium; systeem- en leveranciersdetails vragen controle door een producent
- **3D Viewer** — Three.js-based 3D preview with frame extrusion and transparent glass panels
- **IFC4-uitwisseling (bèta)** — Los kozijn, project, beperkte typebibliotheek, afzonderlijke onderdelen en een indicatief IFC/JSON/CSV-overdrachtspakket. De gedetailleerde export accepteert momenteel rechthoekige kozijnen met vaste beglazing; de `OFS_`-eigenschappen zijn niet als NBVT ILS v2.0 gevalideerd.
- **DXF Export** — Tekeningen met maatlijnen; controleer maatvoering en bewerkingen vóór productie
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

## Changelog v2026.46.2 (installer-prerelease, 2026-11-13)

- **IFC-export blijft werken na een maatwijziging.** Vakmaten werden bij maat-, profiel- en vakmaatwijzigingen herschaald zonder de tussenstijlen mee te rekenen, waardoor de maatketen niet meer sloot en elke IFC-export en producentenoverdracht werd geweigerd (1200 → 1210 mm was al genoeg). De keten sluit nu altijd; een getypte vakmaat blijft exact staan en de overige vakken vangen het verschil op. De browserversie herschaalt nu ook kozijnen met meerdere vakken.
- **IFC-weigeringen worden vooraf uitgelegd.** Vrije indeling, aanbouw en niet-rechthoekige kozijnen worden gemeld vóór het opslaan-dialoog, met de naam van het kozijn. LOD 300 met draaiende delen of deuren valt automatisch terug op LOD 200, met melding. Nieuwe knop **Terug naar raster** in de eigenschappen als herstelpad voor de vrije indeling.
- **Glasmaat in de sponning.** Kozijnen zonder gekozen profiel gaven dagmaat − 8 mm (een ruit kleiner dan de opening). Glas en glaslatten rekenen nu met de sponning uit het profiel of de materiaalnorm (hout 17, PVC 20, alu 25 mm): hout geeft dagmaat + 24 mm, gelijk aan een expliciet gekozen profiel. Het producentenpakket krijgt zo een bestelmaat die niet kleiner is dan de zichtmaat.

## Changelog v2026.46.1 (installer-prerelease, 2026-11-09)

- SuperCloud Route A in ontwikkeling: een browserbundel met relatieve WASM-bestanden, een Docs-adapter voor `.ofs` openen/opslaan met versiecontrole en alleen-lezen, plus drie alleen-lezen `frame_*`-assistenttools. Echte werkplekintegratie en desktop-MCP-pariteit vragen nog acceptatie.
- Docker-imageworkflow uit de Blueprint-template toegevoegd. Deze bouwt de grafische Linux-app uit de `.deb` van dezelfde release, test stream/gezondheid en publiceert na slagen een ondertekende GHCR-image. De GitHub-containerproef moet nog slagen voordat de image als beschikbaar geldt.
- De releaseworkflow herstelt de GitHub-rechten voor conceptrelease-assets en koppelt conceptreleases expliciet aan de juiste tag.

## Changelog v2026.46.0 (installer-prerelease, 2026-11-09)

- IFC4-export voor één kozijn, projectkozijnen, een beperkte `IfcWindowType`-bibliotheek en een onderdelenproject met stabiele `GlobalId`'s. Ongeschikte vormen en indelingen worden vóór bestandsschrijven afgewezen; LOD400 is uitgeschakeld totdat profiel- en verbindingsdetails zijn uitgewerkt.
- Indicatieve producentenoverdracht met onderdelen-IFC, `overdracht.json` en `onderdelen.csv`. De kruisverwijzing koppelt IFC-onderdelen aan zaag- en glaslijstregels; zij is geen importadapter of CNC-vrijgave.
- Normcontroles zijn een indicatieve voorcontrole. De CNC-weergave toont zaagsneden als preview; generieke G-code-export is geblokkeerd totdat producentprofielen en machinebewerkingen zijn gevalideerd.
- IFC-inspectie en wijzigingsvergelijking op identiteit; een eigen onderdelen-IFC kan beperkt worden hersteld als bewerkbaar basisproject. Ontbrekende productiegegevens worden daarbij niet ingevuld.
- Een visuele linkimport van het onderdelenmodel is in Revit 2026.5 uitgevoerd. Tekla, SketchUp, hsbDesign, native bewerkbaarheid en producentencalculatie vragen nog acceptatieproeven. Zie het [acceptatieprotocol](docs/ifc-acceptatie-revit-tekla-sketchup-hsbcad.md).
- De desktop- en webfunctionaliteit lopen nog uiteen. Zie [release-readiness](docs/release-readiness.md) voor de vereiste bouw-, test- en uitwisselcontroles.

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
| Styling | OpenAEC Design System (CSS custom properties) |

---

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Tauri CLI 2](https://v2.tauri.app/start/prerequisites/) voor `cargo tauri` en de systeembibliotheken die Tauri op het gekozen platform vereist

### Build from Source

```bash
# Clone the repository
git clone https://github.com/OpenAEC-Foundation/Open-Frame-Studio.git
cd Open-Frame-Studio

# Install frontend dependencies
cd ui && npm ci && cd ..

# Run in development mode
cargo tauri dev

# Build for production
cargo tauri build
```

### Browser Preview (no Tauri required)

```bash
cd ui && npm run dev
```

`npm run dev` laat de interface zien zonder desktopbackend en gebruikt alleen een WASM-bundel als diens versie overeenkomt met de UI; anders toont de app een herkenbare demo-preview. Een actuele browserbuild met `ofs-wasm` wordt gemaakt met `npm run build:web`; daarvoor zijn `wasm-pack` en de Rust-target `wasm32-unknown-unknown` nodig.

**Webbeperking:** de browser heeft geen volledige desktoppariteit. Bestandsimport/-export en meerdere project-, vliesgevel-, calculatie- en productiecommando's hebben daar nog geen echte implementatie. Gebruik de browserweergave niet als bewijs van een werkende producentenworkflow.

### Docker voor OpenAEC SuperCloud

De [Docker-image-workflow](.github/workflows/docker-image.yml) bouwt na een geslaagde installerrelease een container **uit de `.deb` en containertemplate van diezelfde releasecommit**. De gewone Linux-desktopapp draait daarin als niet-rootgebruiker op een virtueel scherm en is via één HTTP-poort te bekijken. De workflow controleert het app-proces, `GET /gezond`, een screenshot van de stream en het geheugengebruik voordat hij de image naar GHCR pusht. De [v2026.46.1-containerproef](https://github.com/OpenAEC-Foundation/open-frame-studio/actions/runs/42686749360) is geslaagd; de [schermafdruk](docs/release-evidence/v2026.46.1-docker-stream.png) toont de gestreamde app. Een release met deze containertemplate kan ook handmatig via de workflow en zijn `tag`-invoer worden verwerkt.

De GHCR-image is gepubliceerd, maar een anonieme pull van `2026.46.1` kreeg op 9 november 2026 een `403`. Voor de opdracht hieronder is voorlopig toegang tot de package en een `docker login ghcr.io` nodig; [issue #8](https://github.com/OpenAEC-Foundation/open-frame-studio/issues/8) houdt publieke bereikbaarheid open.

Na publicatie van de image:

```bash
docker run -p 127.0.0.1:3000:3000 -v "$PWD":/openaec -v ofs-config:/config ghcr.io/openaec-foundation/open-frame-studio:2026.46.1
```

Open dan `http://localhost:3000`. `/openaec` bevat gebruikersbestanden; `/config` bewaart instellingen. `OPENAEC_SCREEN` stelt de virtuele schermmaat in (standaard `1920x1080`) en `OPENAEC_RESTART=0` schakelt automatisch herstarten van de app uit. De stream heeft zelf geen aanmelding: plaats hem bij servergebruik achter de authenticatieproxy van de SuperCloud. De huidige app leest nog geen startbestand uit een CLI-argument en levert geen MCP-server; `OPENAEC_OPEN` en `OPENAEC_MCP` zijn voor Open Frame Studio dus nog niet beschikbaar. De image-tag `latest` wordt alleen bij een volledige release bijgewerkt.

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
├── ofs-wasm/               # Browserbinding van ofs-core; eigen build nodig
├── ofs-cloud/              # Losse API-scaffold, nog geen gedeelde projectdienst
├── ofs-web/                # Losse prototypepagina's voor configuratie/werkvloer
├── ui/                     # Svelte frontend
│   └── src/
│       ├── App.svelte      # Main app with workspace tabs
│       ├── components/     # Shell, Editor, Panels, Profile Editor, Viewer3D
│       ├── stores/         # Svelte stores (project, kozijn, profiles, AI assistant, history)
│       ├── styles/         # OpenAEC design tokens & themes
│       └── lib/            # Tauri invoke wrapper, AI tools en beperkte browseradapters
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

- **IFC4** — toegepast voor de ondersteunde kozijn- en onderdelenexport; conformiteit en importgedrag worden per doelpakket getoetst
- **NBVT ILS Houten Kozijnen v2.0** — referentie voor een toekomstige veld-voor-veld toets; de huidige `OFS_`-eigenschappen claimen geen ILS-conformiteit
- **Bbl/KVT/CE** — bestaande controles zijn indicatief; zij vervangen geen projectspecifieke normbeoordeling, gevalideerde DoP of producentverklaring

---

## Contributing

Contributions are welcome! Please open an issue or pull request.

---

## License

De repository bevat momenteel tegenstrijdige licentievermeldingen: [LICENSE](LICENSE) en eerdere README-versies noemen CC BY-SA 4.0, terwijl Cargo-metadata en schermen AGPL-3.0 noemen. De projecteigenaar moet de bedoelde licentie vastleggen voordat een nieuwe binaire release wordt gepubliceerd.

© 2026 OpenAEC Foundation

---

## Related Projects

- [Open Field Studio](https://github.com/OpenAEC-Foundation/Open-Field-Studio) — Construction inspection & QA tool
- [Open PDF Studio](https://github.com/OpenAEC-Foundation/open-pdf-studio) — PDF editor & annotator
- [Open Heatloss Studio](https://github.com/OpenAEC-Foundation/open-heatloss-studio) — Heat loss calculations (ISSO 51:2023)
- [building.py](https://github.com/OpenAEC-Foundation/building.py) — Python library for BIM modeling
