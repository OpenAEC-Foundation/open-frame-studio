# Releasegereedheid — v0.7.0-beta.1

Status op 9 november 2026: **bronkandidaat voor GitHub `main`; geen publieke installer of productievrijgave**. De laatst gepubliceerde versie is v0.6.0. Een geslaagde bronbuild zegt niets over de werking van een gebundelde desktopapp of de import in een producentenpakket.

## Afgebakende bètascope

- Lokale desktopapp met `.ofs`-projecten, 2D/3D-ontwerp, calculatie- en productiemodules.
- IFC4 voor rechthoekige kozijnen met vaste beglazing: los kozijn, project, typebibliotheek en project met losse onderdelen. LOD400, bijzondere vormen, vrije vakindeling, beweegbare delen, vliesgevels en combinaties vallen buiten de gedetailleerde IFC-uitvoer.
- IFC-inspectie en vergelijking op vaste IDs. Alleen een **eigen** onderdelen-IFC kan als bewerkbaar basisproject worden hersteld; ontbrekende systeem- en productiegegevens blijven onbekend.
- Indicatief producentenpakket met IFC, JSON en CSV. Het is een neutraal uitwisselcontract, geen gevalideerde producentimport, bestelling of machineprogramma.
- Browserweergave is beperkt; cloud-API en losse werkvloerpagina's zijn niet aangesloten op de desktopprojectworkflow.

## Reproduceerbare broncontrole

Voer deze controles uit op **dezelfde commit** als de kandidaatbuild. CI moet ze automatisch herhalen; bewaar commit-ID, platform, versies en uitvoer bij het testrapport.

```text
node scripts/check-version.mjs
cd ui && npm ci && npm run build && cd ..
cargo test -p ofs-core --lib --locked
cargo check -p ofs-core -p open-frame-studio --locked
```

Voor de echte browserbuild zijn `wasm-pack` en de target `wasm32-unknown-unknown` nodig. Voer `cd ui && npm run build:web` uit en controleer dat `ui/dist/wasm` de pas gebouwde WASM en zijn versie bevat. Een gewone Vite-build mag geen bewijs van een actuele browserkern heten.

Maak de vaste IFC-proefset met:

```text
cargo run -p ofs-core --example ifc_acceptance_fixture -- <uitvoermap>
```

De unit- en parsercontroles toetsen structuur, maat en ID-stabiliteit. De [applicatieproef](ifc-acceptatie-revit-tekla-sketchup-hsbcad.md) toetst afzonderlijk of elk doelpakket het bestand op de juiste plek, schaal en objecthiërarchie ontvangt.

## Gates vóór een publieke release of installer

| Gate | Slaagvoorwaarde | Huidige stand |
|---|---|---|
| Versie en bron | UI, Cargo, lockfile, Tauri, README en losse pagina's hebben één versie; distributies komen uit dezelfde commit | Lokale versiecontrole en bronbuild geslaagd; GitHub-CI na push nog afwachten |
| Desktopinstallatie | Een gebundelde installer start, opent/bewaart een bestaand `.ofs`-project en exporteert de proefset op een schone machine | Nog niet uitgevoerd voor v0.7.0-beta.1 |
| Browser | Nieuwe WASM wordt gebouwd en geladen; niet-ondersteunde acties geven geen schijnresultaat | Lokale `build:web` en browserproef met maken/dupliceren geslaagd; gedeployde site nog controleren |
| BIM-referentie | Revit, Tekla, SketchUp en hsbDesign: import zonder fouten; maat, positie, eigenschappen, objecttelling en revisie aantoonbaar | Alleen visuele Revit-linkproef voor onderdelenmodel uitgevoerd |
| Producent | Eén afgesproken systeemkozijn met leveranciersprofielen, verbindingen, glas, toleranties en prijs-/zaag-/machine-uitvoer vergeleken met een gecontroleerde referentie | Nog geen producentadapter of acceptatie |
| Norm-/machinevrijgave | Onbekende invoer geeft onbekend of onvoldoende; CNC en controles worden pas als vrijgegeven aangeboden na systeem- en machinevalidatie | Fail-closed voor ontbrekende verklaringen en generieke G-code; producent- en normvalidatie blijft open |
| Licentie | `LICENSE`, Cargo-metadata, UI en README noemen dezelfde door projecteigenaar bevestigde licentie | Open: CC BY-SA 4.0 en AGPL-3.0 staan nu door elkaar |

Een bètacodepush naar GitHub is geen release: maak pas een versie-tag en publiceer pas binaries nadat de relevante gates geslaagd zijn. Zet acceptatiebevindingen en de exacte ondersteunde producttypen in de releasenotities. Bestaande exportbestanden met oude ILS-achtige propertysetnamen kunnen andere namen hebben dan nieuwe `OFS_`-eigenschappen; controleer migratie bij afnemers.

### Lokale integratiecontrole op 9 november 2026

`cargo test -p ofs-core -p ofs-wasm -p open-frame-studio --locked` slaagde met 138 kern-tests, één WASM-regressietest en één test voor veilig projectopslaan. `cargo build -p open-frame-studio --locked`, `npm run build:web`, de versiecontrole en de vijf IFC-proefbestanden met producentenoverdracht slaagden eveneens. Dit is een controle van de bron- en lokale browserbuild, geen installatietest of acceptatie door een fabrikant.
