# SuperCloud Route A — eerste Docs-koppeling

Broncontract: [OpenAEC Blueprint, SuperCloud-koppeling](https://github.com/OpenAEC-Foundation/OpenAEC-Blueprint/blob/main/docs/supercloud-koppeling.md). Dit document beschrijft de **eerste implementatiesnede**, geen acceptatie van GitHub-issue #9.

## Wat nu werkt in de browserbron

- `cd ui && npm run build:platform` bouwt de actuele WASM-kern en een Vite-bundel met relatieve `./`-basis. Script, CSS, lettertypen en WASM worden uit dezelfde bundel geladen, zodat plaatsing onder `/werkplek/app/open-frame-studio/` mogelijk is.
- Als `window.openaecHost` aanwezig is, opent de app `initialFileId` na het starten of na `openaec-host-ready`. De `bytes` van een `.ofs`-bestand worden als UTF-8 gedecodeerd en met de bestaande WASM-projectparser geladen.
- Openen vanuit het menu vraagt `host.list()` op en toont de `.ofs`-bestanden in die map. Opslaan stuurt `Uint8Array` naar `host.save(bytes, {id, expectedVersion})`; Opslaan als gebruikt `{name}`. De teruggegeven ID, naam en versie worden de actieve bestandsreferentie, ook wanneer de host bij een conflict een kopie heeft gemaakt.
- Een `readOnly`-bestand krijgt een zichtbare melding. Muterende webcommando's worden geweigerd zolang die referentie actief is. Opslaan als kan een nieuwe bewerkbare kopie maken. Hostfouten worden als melding getoond zonder het huidige project als opgeslagen te markeren.
- De platformbuild weigert modelcommando's als de WASM-kern ontbreekt. Een vertraagde Docs-open wordt genegeerd als de projectgeneratie, de bewerkingsversie of de actuele WASM-snapshot intussen veranderde. Bij een vertraagde save blijven tussentijdse edits als niet opgeslagen gemarkeerd en wordt een inmiddels geopend ander project niet overschreven.
- In de platformbuild zijn de externe AI-tab en links die een nieuw venster openen verborgen.
- De platformbuild registreert vóór de UI-start `window.__OPDS_APP__` met asynchrone `tools()`, `definitions()` en `run(name, args?)`. De MCP-laag wordt pas bij de eerste aanroep als aparte chunk geladen; `run` wacht op de WASM-start en haalt per aanroep een actuele kernsnapshot op. Dezelfde JSON-bron voor de definities wordt als `dist/mcp-tools.json` uitgegeven.

## Eerste alleen-lezen assistenttools

| Tool | Invoer | Uitvoer |
| --- | --- | --- |
| `frame_stand` | Leeg object of geen argumenten | Project-ID, naam, nummer en formaatversie; Docs-ID, bestandsnaam, versie en alleen-lezenstatus; niet-opgeslagenstatus, actieve selectie en aantallen kozijnen/vliesgevels. |
| `frame_kozijnen` | `query?: string` (merk/naam), `offset?: integer >= 0`, `limit?: integer 1..50` (standaard 20) | `total`, `offset`, `limit` en compacte kozijnrijen met ID, merk, naam, materiaal, maten, vorm, aantal vakken en IFC-herkomst. |
| `frame_kozijn` | `id?: string`; zonder ID geldt het geselecteerde kozijn | Compacte kozijnspecificatie met profiel, maten, vorm, plaatsing, raster en aantallen vullingen. Hoogstens de eerste 20 vakken worden individueel beschreven; `omitted` telt de overige. |

De tools geven een object terug en veranderen niets aan het model. Namen en tekstvelden worden begrensd; antwoorden boven 30.000 tekens worden geweigerd. Ongeldige argumenten, een onbekende tool, geen project, geen selectie en een ontbrekend kozijn leveren een afgewezen Promise met respectievelijk `ongeldige_argumenten`, `onbekende_tool`, `geen_project`, `geen_kozijn_geselecteerd` en `kozijn_niet_gevonden`. Zonder werkende WASM-kern geeft `run` `niet_in_browser: OFS-rekenkern ontbreekt`. Docs Openen/Opslaan zijn opzettelijk geen assistenttools: het hostcontract laat die acties aan de gebruiker.

## Reproduceerbare broncontrole

```text
node --test ui/src/lib/platformHost.test.js
node --test ui/src/lib/frameTools.test.js
cd ui && npm run build:platform
```

De adaptertests controleren UTF-8/ArrayBuffer, alleen `.ofs`, versiegestuurd opslaan, een door de host gemaakte conflictkopie, alleen-lezen en Opslaan als. De tooltests controleren definities, selectie, filter/paginering, begrensde uitvoer en foutgedrag. De bundelbouw controleert de relatieve assetpaden, een aparte toolchunk en `dist/mcp-tools.json`. Deze tests zijn nog geen proef in een echte SuperCloud-werkplek.

## Nog open voor acceptatie van issue #9

1. Test de bundel in een iframe met `sandbox="allow-scripts allow-modals"` zonder `allow-same-origin`, met een nagebootste host en daarna in de echte werkplek. Controleer initial open, menu Openen, Opslaan, Opslaan als, conflict en alleen-lezen, plus console/CSP en nul ongeoorloofde netwerkverzoeken.
2. Vervang de eenvoudige genummerde Docs-keuze door een toegankelijke bestandskiezer. Neem taal/thema en blijvende instellingen over via `host.settings`; browsertabopslag is in de werkplek slechts tijdelijk.
3. Bied de drie tools via een desktop-MCP-server aan en gebruik daarbij de gedeelde, transportneutrale dispatcher. OFS heeft momenteel geen desktop-MCP-server, dus de Blueprint-eis voor dezelfde toolcode op browser en desktop is nog niet aantoonbaar gehaald. Verifieer `window.__OPDS_APP__` en `mcp-tools.json` ook in de echte SuperCloud-assistent.
4. De browser ondersteunt niet alle desktopfuncties. Verboden native acties moeten een expliciete `niet_in_browser`-fout geven en mogen geen stille schijnresultaten opleveren. Evalueer de overige webshims en profiel-/sjabloonopslag per werkstroom.

Zonder die proeven blijft deze route een ontwikkelstand. De bestaande Route B-afhankelijkheden uit issue #9 (desktop-MCP, beheerde modus en Linux arm64) zijn hiermee niet ingevuld.
