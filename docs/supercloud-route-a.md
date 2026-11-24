# Open Frame Studio in de SuperCloud

De browserbundel gebruikt Route A. De SuperCloud verzorgt toegang en versiebeheer voor projecten in Docs. [OpenAEC Blueprint, SuperCloud-koppeling](https://github.com/OpenAEC-Foundation/OpenAEC-Blueprint/blob/main/docs/supercloud-koppeling.md)

## Wat nu werkt in de browserbron

- `cd ui && npm run build:platform` bouwt de actuele WASM-kern en een Vite-bundel met relatieve `./`-basis. Script, CSS, lettertypen en WASM worden uit dezelfde bundel geladen, zodat plaatsing onder `/werkplek/app/open-frame-studio/` mogelijk is.
- Als `window.openaecHost` aanwezig is, opent de app `initialFileId` na het starten of na `openaec-host-ready`. De `bytes` van een `.ofs`-bestand worden als UTF-8 gedecodeerd en met de bestaande WASM-projectparser geladen.
- Bestand → Openen toont de `.ofs`-bestanden van `host.list()` in een toetsenbordbedienbare dialoog. Enter bevestigt de keuze en Escape sluit de dialoog. Opslaan gebruikt `host.save(bytes, {id, expectedVersion})`. Opslaan als gebruikt `{name}`. De app volgt de naam, ID en versie die de host teruggeeft, ook bij een conflictkopie. Bij een download buiten een Docs-map geeft de host een lege ID terug en blijft het document zonder Docs-koppeling.
- De app neemt ondersteunde taal- en themakeuzes van de host over, ook na het event `openaec-host-preferences`. Paneelbreedtes, opengeklapte panelen, lintindeling, taal en thema worden onder `frame.*` via `host.settings` bewaard.
- Een `readOnly`-bestand krijgt een zichtbare melding. Muterende webcommando's worden geweigerd zolang die referentie actief is. Opslaan als kan een nieuwe bewerkbare kopie maken. Hostfouten worden als melding getoond zonder het huidige project als opgeslagen te markeren.
- De platformbuild weigert modelcommando's als de WASM-kern ontbreekt. Een vertraagde Docs-open wordt genegeerd als de projectgeneratie, de bewerkingsversie of de actuele WASM-snapshot intussen veranderde. Bij een vertraagde save blijven tussentijdse edits als niet opgeslagen gemarkeerd en wordt een inmiddels geopend ander project niet overschreven.
- In de platformbuild zijn de externe AI-tab en links die een nieuw venster openen verborgen.
- De platformbuild registreert vóór de UI-start `window.__OPDS_APP__` met asynchrone `tools()`, `definitions()` en `run(name, args?)`. De MCP-laag wordt pas bij de eerste aanroep als aparte chunk geladen; `run` wacht op de WASM-start en haalt per aanroep een actuele kernsnapshot op. Dezelfde JSON-bron voor de definities wordt als `dist/mcp-tools.json` uitgegeven.

## Alleen-lezen assistenttools

| Tool | Invoer | Uitvoer |
| --- | --- | --- |
| `frame_stand` | Leeg object of geen argumenten | Project-ID, naam, nummer en formaatversie; Docs-ID, bestandsnaam, versie en alleen-lezenstatus; niet-opgeslagenstatus, actieve selectie en aantallen kozijnen/vliesgevels. |
| `frame_kozijnen` | `query?: string` (merk/naam), `offset?: integer >= 0`, `limit?: integer 1..50` (standaard 20) | `total`, `offset`, `limit` en compacte kozijnrijen met ID, merk, naam, materiaal, maten, vorm, aantal vakken en IFC-herkomst. |
| `frame_kozijn` | `id?: string`; zonder ID geldt het geselecteerde kozijn | Compacte kozijnspecificatie met profiel, maten, vorm, plaatsing, raster en aantallen vullingen. Hoogstens de eerste 20 vakken worden individueel beschreven; `omitted` telt de overige. |
| `frame_oppervlakte` | `id?: string`, standaard alle kozijnen | Berekent het bruto oppervlak op buitenmaten in m² met dezelfde functie als het dashboard. Bij gebogen vormen telt de omsluitende rechthoek. `missingDimensions` telt kozijnen die bij gebrek aan geldige maten buiten de som vallen. |

De tools geven een object terug en veranderen niets aan het model. Namen en tekstvelden worden begrensd; antwoorden boven 30.000 tekens worden geweigerd. Ongeldige argumenten, een onbekende tool, geen project, geen selectie en een ontbrekend kozijn leveren een afgewezen Promise met respectievelijk `ongeldige_argumenten`, `onbekende_tool`, `geen_project`, `geen_kozijn_geselecteerd` en `kozijn_niet_gevonden`. Zonder werkende WASM-kern geeft `run` `niet_in_browser: OFS-rekenkern ontbreekt`. Docs Openen/Opslaan zijn opzettelijk geen assistenttools: het hostcontract laat die acties aan de gebruiker.

## Lokale MCP-verbinding met een opgeslagen project

Een desktopclient kan de stdio-server uit de broncheckout starten met Node 22.12 of hoger. Geef als argument het absolute pad naar het ene project dat de assistent mag lezen.

```json
{
  "mcpServers": {
    "open-frame-studio": {
      "command": "node",
      "args": ["C:/OpenFrameStudio/packaging/mcp/stdio.mjs", "C:/Projecten/voorbeeld.ofs"]
    }
  }
}
```

Vervang beide voorbeeldpaden door de eigen paden. De stdio-server luistert op de invoer en uitvoer van het proces en heeft geen npm-installatie nodig. Elke gereedschapsaanroep leest de actuele opgeslagen versie van dit ene bestand, tot 32 MiB. Geef voor `frame_kozijn` een ID uit `frame_kozijnen` op. De browser rapporteert daarnaast de actuele selectie en onopgeslagen bewerkingen in de werkplek. Beide transporten gebruiken `ui/src/lib/frameTools.js` voor berekeningen en resultaten.

## Reproduceerbare controles

```text
node --test ui/src/lib/platformHost.test.js ui/src/lib/frameTools.test.js ui/src/lib/hostWorkspaceSettings.test.js packaging/mcp/stdio.test.mjs
cd ui && npm run build:platform
node scripts/smoke-platform.mjs
```

Voor de browserproef zijn Playwright en Chromium nodig. Gebruik een lokaal geïnstalleerde `playwright` of zet `OFS_PLAYWRIGHT` op het pad naar het pakket en eventueel `OFS_BROWSER` op het uitvoerbare browserbestand. De proef maakt zelf een tijdelijk project in het browsergeheugen en gebruikt een lokale testhost. Ze controleert openen, alleen-lezen, Opslaan als, versieconflict, sneltoetsen, paneelinstellingen, taal, thema en de vier tooldefinities in een iframe met `sandbox="allow-scripts allow-modals"`. Ze controleert ook de browserconsole en externe verzoeken. De platformbundel neemt de SVG-pictogrammen op in het document, zodat ze binnen deze sandbox zichtbaar zijn.

## Acceptatie in de werkplek

De echte eindproef voor issue #9 vereist de platformbundel in een aangemelde SuperCloud-werkplek met een eigen Docs-testmap. Open een `.ofs` vanuit Docs, maak via Opslaan als een kopie, bewaar een nieuwe versie en vraag de assistent naar `frame_stand` en `frame_oppervlakte`. Vergelijk bestandsversie en resultaat met Docs en het dashboard. Dezelfde route kan met een oudere alleen-lezenversie worden herhaald. Lokale sjabloonopslag, Blender en uitvoer die een native bestandspad nodig heeft, horen bij de desktopversie.
