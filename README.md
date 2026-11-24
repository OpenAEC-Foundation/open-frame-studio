# Open Frame Studio v2026.48.4

**Open-source kozijnontwerp voor de Nederlandse bouwketen.** De desktopapp combineert tekenen, projectgegevens en een beperkte IFC4-uitwisseling. Deze bètaversie is een ontwikkelstand, geen gevalideerd pakket voor producentencalculatie, normverklaringen of machineaansturing.

Built by the [OpenAEC Foundation](https://github.com/OpenAEC-Foundation) — accessible, professional-grade tools for Architecture, Engineering & Construction.

![Platform: Windows](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-blue)
![Built with Tauri 2](https://img.shields.io/badge/Built%20with-Tauri%202-FFC131)

> **Releasekanaal** — Open Frame Studio publiceert de actuele installers via GitHub Latest. De software is bedoeld voor ontwerp- en BIM-uitwisselproeven. [Releasecontroles](docs/release-readiness.md) · [GitHub Releases](https://github.com/OpenAEC-Foundation/open-frame-studio/releases)

> **Installeren en bijwerken** — Download de installer voor je besturingssysteem via de [laatste GitHub-release](https://github.com/OpenAEC-Foundation/open-frame-studio/releases/latest) en voer die uit. De OpenAEC Installer heeft Open Frame Studio op 19 november 2026 uit zijn catalogus verwijderd. [Wijziging in de Installer-catalogus](https://github.com/OpenAEC-Foundation/openaec-installer/commit/b6da4a008b1df85488a06b87daf039c7bbef864f)

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
- **OpenAEC Blueprint layout** — Frame Studio’s ribbon, workspace tabs, project explorer and properties panel follow the [Frame shell profile](https://github.com/OpenAEC-Foundation/OpenAEC-Blueprint/blob/2eb8a958bbfe864dea81237e8890cf2405620b7d/design/frame-layout-review.html). New installations start in light mode. Existing theme and panel preferences are retained.

### Materialen

| Categorie | Typen |
|---|---|
| **Hout** | KVT standaard profielen (54/67/78/90mm series), meranti, accoya, vuren, eiken, hardhout FSC, naaldhout |
| **Aluminium** | Standaard aluminium profielen |
| **Kunststof (PVC)** | Standaard PVC profielen |
| **Hout-Aluminium** | Combinatieprofielen |

---

## Changelog v2026.48.4 (2026-11-24)

- In het vooraanzicht volgt de gestippelde buitenlijn van de spouwlat de dikte aan elke ingeschakelde kozijnzijde. Bij een profiel van 121 × 33 mm is de uitsteek dus 33 mm. Aangrenzende latten sluiten aan op de buitenhoek.
- De A3-werkplaatstekening toont de vrije vakmaten en de hartpositie van tussenstijlen vanaf links en tussenregels vanaf onder. Bij ronde en getoogde kozijnen volgen de vakmaten de dagcontour.
- De release bevat de bijgewerkte 4K-demofilm *Open Frame Studio* met Nederlandse vrouwenstem. [Download de demofilm](https://github.com/OpenAEC-Foundation/open-frame-studio/releases/download/v2026.48.4/Open.Frame.Studio_Demofilm_4K_NL.mp4)

## Changelog v2026.48.3 (2026-11-24)

- De liggende A3-werkplaatstekening neemt tussenstijlen en tussendorpels uit het 2D-aanzicht over. Bij rechthoekige kozijnen staan vaknummer en vulling op de overeenkomstige plaats, ook bij een plaatselijke vakverdeling.

## Changelog v2026.48.2 (2026-11-24)

- Een spouwlat is per zijde in te stellen en te verwijderen. De 2D-tekening toont bij rechthoekige kozijnen de spouwlat aan de gekozen zijde. Na opslaan en heropenen blijven de overige zijden leeg als daar geen spouwlat is gekozen.

## Changelog v2026.48.1 (2026-11-24)

- De twintig werkruimten gebruiken de Frame-specifieke OpenAEC Blueprint. Het lint groepeert opdrachten bij een smal venster en onthoudt welke groepen en tabbladen de gebruiker zichtbaar houdt.
- De 2D-editor en de 3D-viewer delen de projectverkenner en het eigenschappenpaneel. Bij wisselen van thema volgen de achtergrond en het raster van de 3D-viewer de nieuwe kleuren. De camerastand blijft behouden.
- Nieuwe installaties starten in het lichte thema. Een bestaande themakeuze en opgeslagen paneelbreedtes blijven beschikbaar. De zelfstandige configurator, werkvloer en AR-preview gebruiken dezelfde vormgeving.

## Changelog v2026.47.2 (2026-11-20)

- De onderdelen-IFC schrijft de zichtbreedte en zichthoogte van ruiten in de juiste standaardvelden. Beglazing is selecteerbaar als `IfcWindow` met `CPset_31.Onderdeel = Ruit`. De noordrichting heeft twee componenten.
- Na een materiaalwissel gebruikt de IFC-export de tussenstijlbreedte van het gekozen profiel. De zaagoptimalisatie verdeelt verschillende profielartikelen over aparte handelsstaven.
- Afgekeurde beglazing is herkenbaar in de glaslijst. Inkoop sluit die regels uit en de producentenoverdracht meldt welke ruit eerst gecorrigeerd moet worden.

## Changelog v2026.47.1 (2026-11-19)

- Kunststof en aluminium kozijnen gebruiken de maten en dichtingsniveaus van het gekozen profielsysteem. Een los aanslagprofiel tegen de dagkant krijgt eigen stukken, booglengtes en zaaghoeken in de kortlijst.
- Het beslag volgt de draaiwijze, de scharnierzijde en de beschikbare montageruimte van elke vleugel. Bij het spiegelen blijven handmatige krukhoogte en aantallen behouden.
- De 3D-doorsnede toont de aansluiting van vleugel, aanslag en rubbers. De beslag- en rubberlijsten en de PDF-, Excel- en CSV-uitvoer vermelden de bijbehorende stukreferenties.

## Changelog v2026.47.0 (2026-11-19)

- Bij ronde en getoogde kozijnen volgen de glassponningen, glaslatten en draaiende vleugels de boog. De glaslijst vermeldt de bijbehorende glasmaat.
- De houtkortlijst geeft stralen, booglengtes, kopse rondingen en zaaghoeken voor kozijnhout, raamhout, tussenstijlen en tussendorpels.
- Pen-en-gatverbindingen worden per houtdeel berekend. Bij de boogvoeten en de twee naden van een rond kozijn kun je per verbinding de penzijde en penlengte kiezen. De houtkortlijst en de PDF-, Excel- en CSV-uitvoer koppelen de penlengte, gatdiepte en zaaghoek aan het juiste uiteinde.
- Met **Pen-en-gat bekijken** in de 3D-viewer schuif je de houtdelen uit elkaar om de schouders, pennen en blinde gaten te bekijken.

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

### Een Linux-download verifiëren

Voor releases met Sigstore-bijlagen download je het Linux-pakket en het gelijknamige bestand met `.sigstore.json` erachter. De releaseworkflow ondertekent de DEB, AppImage en RPM afzonderlijk. De GitHub-build vermeldt bij **Verify downloaded Linux packages** de workflow-identiteit en de broncommit. Vul de releasetag en bestandsnaam in en controleer met Cosign 3 en GitHub CLI.

```bash
read -rp 'Releasetag: ' tag
read -rp 'Naam van het gedownloade pakket: ' package
repo=OpenAEC-Foundation/open-frame-studio
ref=refs/heads/main  # Gebruik refs/tags/$tag als de build door een tagpush is gestart.
commit=$(gh api "repos/$repo/commits/$tag" --jq .sha)
cosign verify-blob --bundle "$package.sigstore.json" \
  --certificate-identity "https://github.com/$repo/.github/workflows/release.yml@$ref" \
  --certificate-github-workflow-sha "$commit" \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com "$package"
```

De bundel bevat het certificaat en het transparantielogbewijs. [Sigstore-documentatie voor bestandsverificatie](https://docs.sigstore.dev/cosign/verifying/verify/)

### Docker voor OpenAEC SuperCloud

De [Docker-image-workflow](.github/workflows/docker-image.yml) bouwt na een geslaagde installerrelease een container **uit de `.deb` en containertemplate van diezelfde releasecommit**. De gewone Linux-desktopapp draait daarin als niet-rootgebruiker op een virtueel scherm en is via één HTTP-poort te bekijken. De workflow controleert het app-proces, `GET /gezond`, een screenshot van de stream en het geheugengebruik voordat hij de image naar GHCR pusht. De [v2026.46.1-containerproef](https://github.com/OpenAEC-Foundation/open-frame-studio/actions/runs/42686749360) is geslaagd; de [schermafdruk](docs/release-evidence/v2026.46.1-docker-stream.png) toont de gestreamde app. Een release met deze containertemplate kan ook handmatig via de workflow en zijn `tag`-invoer worden verwerkt.

De [containerrun voor v2026.48.3](https://github.com/OpenAEC-Foundation/open-frame-studio/actions/runs/45372052645) publiceerde de linux/amd64-image met Cosign-handtekening, bouwattest en SPDX-SBOM. Het gemeten rustgeheugen was 379,6 MiB. Op 24 november 2026 vereiste GHCR aanmelding voor een pull-token. Gebruik een account met leestoegang en `docker login ghcr.io`. [Issue #8](https://github.com/OpenAEC-Foundation/open-frame-studio/issues/8)

```bash
docker run --rm -p 127.0.0.1:3000:3000 -v "$PWD":/openaec -v ofs-config:/config ghcr.io/openaec-foundation/open-frame-studio:latest
```

Open `http://localhost:3000`. De tag `latest` volgt de laatste volledige containerrelease. Voor reproduceerbare installaties kun je de versie of digest uit de releasenotities gebruiken. De desktop draait als UID 1000. Geef deze gebruiker schrijftoegang tot de gemounte werkmap. Plaats de stream bij servergebruik achter de authenticatieproxy van de SuperCloud.

| Instelling in deze bronversie | Gebruik |
| --- | --- |
| `/openaec`, `/config` | Openen, opslaan en exporteren beginnen in de werkmap. Het configuratievolume bewaart de voorkeuren. |
| `OPENAEC_OPEN` | Opent een leesbaar `.ofs`-bestand onder `/openaec`, bijvoorbeeld `-e OPENAEC_OPEN='Projecten/Voordeur.ofs'`. Opslaan werkt hetzelfde bestand bij. |
| `OPENAEC_LANG`, `OPENAEC_THEME` | Starttaal `nl`, `en`, `de` of `tr` (ook regiocodes zoals `en-GB`). Kies `default`, `light`, `dark`, `blue`, `amber-navy`, `warm-ember` of `highContrast` als thema. Een ingestelde waarde geldt bij iedere start. Bij weglaten gelden de opgeslagen voorkeuren. |
| `OPENAEC_SCREEN`, `OPENAEC_RESTART` | Schermmaat (standaard `1920x1080`) en automatisch herstarten van de app (`1` aan, `0` uit). |
| `OPENAEC_MCP_TOKEN_FILE` | Activeert de alleen-lezen MCP-service. Mount een alleen-lezen secret, bijvoorbeeld in `/run/secrets/ofs-mcp`, en geef dit pad door. Een token van `openssl rand -hex 32` is geschikt. |

Verbind de SuperCloud-assistent binnen het afgeschermde containernetwerk met `http://<containernaam>:3001/mcp` en de header `Authorization: Bearer <token>`. Publiceer alleen de streampoort. De MCP-service leest per aanvraag de laatst opgeslagen versie van het startbestand en gebruikt dezelfde `frame_*`-tools als de desktopadapter. Voor een andere projectkoppeling start je een container met de gewenste `OPENAEC_OPEN`. De HTTP-route accepteert serververkeer zonder browser-Origin.

Starten, modelleren, opslaan en de MCP-berekeningen werken zonder uitgaand internetverkeer. Updates komen via de containerimage. De optionele assistent gebruikt op aanvraag het ingestelde modelendpoint (standaard `localhost:11434`), de Blender-koppeling gebruikt `127.0.0.1:9876` en de help- en feedbackknoppen openen GitHub. De [containertest voor pull requests](.github/workflows/container-smoke.yml) controleert de grafische sessie, het opslaan van het startproject en een MCP-aanroep in een container met het externe netwerk uitgeschakeld.

---

### SDK, reken-API en SuperCloud

Voor een indicatieve calculatie van een vastbeglaasd houten kozijn zijn een native SDK voor Windows en Linux en een stateloze HTTP-service beschikbaar in de bron. Dezelfde rekenkern wordt gebruikt door de desktopapp. [SDK en extern voorbeeld](ofs-sdk/README.md) · [HTTP-contract en AWS-configuratie](packaging/aws/README.md)

Een SuperCloud-werkplek kan projecten via Docs openen en opslaan. De assistent kan de kozijnen uitlezen en hun oppervlakte berekenen. Dezelfde vier tools zijn beschikbaar in Claude Code. [SuperCloud-installatie en test](docs/supercloud-route-a.md) · [Claude Code instellen](packaging/mcp/README.md) · [Platform- en releasecontroles](docs/ecosystem-capability-matrix-2026.48.4.md)

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
├── ofs-cloud/              # Headless calculatie-API en lokaal projectprototype
├── ofs-sdk/                # Windows-DLL en Linux-library met C-ABI
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
