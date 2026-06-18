# DWG-import & kozijn-detectie — integratie-onderzoek (extern project `dxf-to-ifc-kozijnen`)

Onderzoek (18 jun 2026): wat uit het aparte project
`…/Maryam/AI Dev/Projects OWN/dxf-to-ifc-kozijnen` is herbruikbaar voor Open Frame Studio,
en hoe het zou inpluggen. Dit doc is een **analyse + gefaseerd plan**; er is nog geen code
in OFS gewijzigd.

## 1. Wat het externe project is
Een **DXF/DWG → IFC kozijnen-converter** (los van OFS), met een Streamlit-UI. Pijplijn:

```
DWG/DXF ──► dwg-reader.exe (Rust + acadrust) ──► JSON {dimensions, inserts, blocks, modelspace_polys}
                                                      │
DXF ──────► ezdxf (Python) ───────────────────────────┤
                                                      ▼
                              app.py: DwgDoc → auto_detect_single_file → params → build_ifc (ifcopenshell) ──► .ifc
```

Naast de app een **AutoLISP-route** (`dxf_to_ifc.lsp`/`.dcl`) die binnen AutoCAD een selectie naar
JSON schrijft en `generate_ifc.py --from-json` aanroept.

| Component | Wat | Waarde voor OFS |
|---|---|---|
| **`dwg-reader/` (Rust, `acadrust`)** | `main.rs` (~150 r): leest DWG/DXF, extraheert per block + modelspace de **LwPolylines** (shoelace-area ≥ 10), plus **Inserts** (block-refs met positie/rotatie/schaal) en **Dimensions** (waarde + 2 punten) → JSON. | **Hoog** — OFS kan nu géén binair DWG. `acadrust` is pure Rust → past in onze stack; vervangt de geplande externe ODA-converter. |
| **`auto_detect_single_file` (app.py 1510–1800)** | Detecteert uit één tekening: W×H (uit dimensions), kozijntype (transom / schuin / grid), buiten/binnen-profiel, tussendorpel-z, stijl/dorpel-indelingen. Helpers: `_detect_axis_divisions`, `_detect_transom_from_segments`, `_detect_slant_drop`, `classify_fw`, `_section_polys`, … | **Hoog, maar groot** — een **nieuwe** capaciteit (heel-kozijn-tekening → parametrisch kozijn). OFS importeert nu enkel profiel-*doorsnedes*. |
| `build_ifc` (ifcopenshell), `parse_ifc*.py`, LISP | IFC-generatie/parsing in Python; in-CAD conversie. | **Laag** — OFS heeft al native Rust IFC-export/-import. Parallel spoor, niet overnemen. |

## 2. `acadrust` — de DWG-lezer
- **crates.io / github.com/hakanaktt/acadrust**, v0.4.0, **pure Rust** (geen C-bindings).
- Leest **DWG R13–R2018** (binair) én DXF (ASCII/binair); 41 entity-types incl. LINE/ARC/CIRCLE/
  ELLIPSE/LWPOLYLINE/DIMENSION/INSERT; optioneel **serde**; "failsafe mode" voor fout-tolerant parsen.
- **Licentie: MPL-2.0** (weak-copyleft op bestandsniveau). Bruikbaar als *dependency* zonder dat
  OFS-code MPL wordt; wel een bewuste licentiekeuze voor de OpenAEC-Foundation-context → **open vraag
  voor projecteigenaar.**
- Deps zijn alle pure Rust (`nom`, `byteorder`, `flate2`, `nalgebra`, …).

API-patroon (uit `dwg-reader/main.rs`, direct herbruikbaar):
```rust
let doc = DwgReader::from_file(path)?.read();        // of DxfReader voor .dxf
for entity in doc.entities() { match entity {
    EntityType::LwPolyline(p) => /* p.vertices[i].location.{x,y} */,
    EntityType::Insert(i)     => /* i.block_name, i.insert_point, i.rotation, i.x_scale() */,
    EntityType::Dimension(d)  => /* d.base.actual_measurement, first/second point */,
    _ => {}
}}
// blocks: doc.block_records → br.entity_handles → doc.get_entity(handle)
```

## 3. Twee integratie-niveaus voor OFS

### Niveau 1 — DWG **profiel-doorsnede**-import (klein, hoge ROI)
Voer DWG-profieltekeningen in dezelfde pijplijn als onze DXF-profielimport. Onze contour-logica
(best-match chaining + planaire-graaf face-trace, [profiel-dxf-dwg-import-onderzoek.md](profiel-dxf-dwg-import-onderzoek.md))
werkt op een lijst **`Poly { pts, closed, layer }`**. We hoeven dus alleen acadrust-entities naar
`Poly` te mappen, niet de contour-extractie te herschrijven.

**Refactor-stap:** splits in [`dxf_profile.rs`](../ofs-core/src/import/dxf_profile.rs:64) de tweede helft
(`polys → ImportedProfile`: closed/`chain`/`outer_contour` → bbox → `detect_sponning` → schatten) af
naar een herbruikbare `fn profile_from_polys(polys: &[Poly]) -> Result<ImportedProfile, String>`.
- DXF-pad: `parse_dxf_profile` (tekst → `extract_polylines` → polys) → `profile_from_polys`.
- DWG-pad (nieuw `import/dwg.rs`): `acadrust read` → map LINE/ARC/CIRCLE/ELLIPSE/LWPOLYLINE naar
  `Poly` (hergebruik `arc_pts`/bulge-tessellatie) → `profile_from_polys`.

### Niveau 2 — DWG/DXF **heel-kozijn**-tekening → parametrisch Kozijn (groot, nieuw)
Port van `auto_detect_single_file` + helpers naar Rust, gekoppeld aan OFS's `Kozijn`-model:
W×H uit dimensions, type (transom/schuin/grid), profiel-blokken, tussendorpel, stijl/dorpel-indelingen.
~600 regels Python met veel ruimtelijke heuristiek; **een eigen project, geen losse stap.** Wel de
logische opvolger zodra Niveau 1 staat (het hergebruikt dezelfde acadrust-reader + `profile_from_polys`).

## 4. Integratiepunt & feature-gating in `ofs-core`
- Nieuwe module `ofs-core/src/import/dwg.rs` (in [`import/mod.rs`](../ofs-core/src/import/mod.rs)),
  achter een **`dwg`-feature** met `acadrust` als optionele dep.
- **Cargo** ([`ofs-core/Cargo.toml`](../ofs-core/Cargo.toml)): `acadrust = { version = "0.4", optional = true }`;
  `features.dwg = ["dep:acadrust"]`; toevoegen aan `default`.
- **Wasm valt er automatisch buiten:** `ofs-wasm` bouwt `ofs-core` met `default-features = false`
  ([`ofs-wasm/Cargo.toml`](../ofs-wasm/Cargo.toml:10)), dus `acadrust` komt **nooit** in de
  browser-bundle (DWG-lezen hoort ook niet in de browser; web-users uploaden DXF). DWG is dus een
  **desktop/Tauri-only** capaciteit — schoon.
- **Command/UI:** nieuw `src-tauri/src/commands/import_dwg.rs` (naast `import_profile.rs`), ribbon-knop
  "Profiel DWG". Web-build toont de knop niet of geeft een nette "alleen desktop"-melding.

## 5. Risico's & open vragen
1. **Licentie MPL-2.0** — akkoord voor OFS/OpenAEC? (Dependency-gebruik is doorgaans prima; bevestiging nodig.)
2. **Geen lokale Rust-toolchain** — acadrust-integratie compileert pas op CI (zelfde gate als de
   DXF-herbouw, #4481904). Blind toevoegen van een dependency is hoger risico dan onze eigen edits.
3. **acadrust-robuustheid op echte data — nog ONBEWEZEN.** Empirische check (18 jun): de prebuilt
   `dwg-reader.exe` op onze 627 profiel-DXF's geeft **leeg** (`{dimensions:[],inserts:[],blocks:{},
   modelspace_polys:[]}`) — ook op bestanden mét LWPOLYLINE. Oorzaak vrijwel zeker de **kozijn-
   specifieke filters** in `main.rs` (alleen modelspace-handle-matches · alleen LwPolyline · shoelace
   ≥ 10), afgestemd op kozijn-*aanzichten* met dimensions/inserts/blocks — niet op profiel-doorsnedes
   (die zijn LINE+ARC). Niet uit te sluiten is dat acadrust's DXF-handle-model afwijkt. **We hebben
   geen leveranciers-DWG-samples** om acadrust zelf te bewijzen → Fase 0 is nu **inconclusief**.
   `acadrust` blijft veelbelovend op papier (pure Rust, MPL, R13–2018, 208/208-claim) maar **onbewezen
   op onze bestanden**. → Eerst echte DWG's verzamelen.
4. **acadrust ↔ onze entity-dekking** — de OFS-mapping mag de dwg-reader-filters NIET overnemen: itereer
   álle entities en neem **LINE/ARC/CIRCLE/ELLIPSE/LWPOLYLINE** mee (acadrust levert die), niet enkel
   LwPolyline. De prebuilt exe is dus geen drop-in; we schrijven onze eigen acadrust-mapping.

## 6. Aanbeveling (gefaseerd)
- **Fase 0 (nu, geen OFS-code):** **(a)** verzamel echte leveranciers-**DWG**-bestanden en draai er de
  prebuilt `dwg-reader.exe` op → bewijst dat acadrust ze leest vóór we een dependency toevoegen.
  (Prebuilt exe op onze profiel-DXF's gaf leeg, maar dat is zijn kozijn-filter, geen acadrust-test —
  zie §5.3.) **(b)** licentie-akkoord MPL-2.0 ophalen.
- **Fase 1 (klein):** `profile_from_polys`-refactor + `import/dwg.rs` (Niveau 1) achter `dwg`-feature;
  ribbon "Profiel DWG". Hergebruikt de volledige contour-pijplijn. CI-gated.
- **Fase 2 (groot, apart):** port `auto_detect_single_file` → Rust, gekoppeld aan `Kozijn` (Niveau 2).

**Kernpunt:** de hoogste ROI met de laagste verstoring is **Niveau 1** — native DWG-profielimport,
die volledig leunt op onze net-verbeterde contour-extractie en buiten de wasm-build blijft.
