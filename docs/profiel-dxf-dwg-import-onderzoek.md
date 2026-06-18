# Profiel-DXF/DWG-import — onderzoek leveranciersdoorsnedes

Onderzoek (18 jun 2026): kunnen we echte profiel­doorsnedes (Profilschnitte) van
kunststof- en aluminium-kozijn­leveranciers via de bestaande import inlezen, zodat
de `crossSection` in OFS klopt i.p.v. de huidige nep-L-vorm?

Aanleiding: de spec-*waarden* (bouwdiepte/Uf/aanzichtbreedte) in `profiles/` kloppen
goed (web-geverifieerd tegen Gealan S 9000 en Schüco AWS 70.HI), maar de
`crossSection`-polygon is voor élk profiel dezelfde placeholder
`[[0,0],[15,0],[15,rebate],[depth,rebate],[depth,depth],[0,depth]]` → de profiel-editor
(`ProfileCanvas.svelte`) tekent daardoor een blok-L i.p.v. de echte meerkamer-doorsnede.

## 1. Waar leveranciers CAD publiceren

| Leverancier | Bron | Formaat | Toegang |
|---|---|---|---|
| **Reynaers** (alu) | [reynaers.com/en/architects/download-cad-bim](https://www.reynaers.com/en/architects/download-cad-bim) — MasterLine 8, SlimLine 38, CS 77, CS 59 | **DWG** + sectietekeningen, BIM (Revit) | registratie |
| **Schüco** (alu) | architectenportaal "Schüco CAD" + [archiproducts](https://www.archiproducts.com/en/products/schuco/aluminium-thermal-break-window-schuco-aws-70-hi_34019), [bimobject](https://www.bimobject.com/en/schueco/product/schueco_aws_70_hi_easy_access) | DWG, BIM | registratie |
| **Gealan / Veka / Kömmerling** (pvc) | [3Dfindit (CADENAS)](https://www.3dfindit.com/en/cad-bim-library/manufacturer/gealan?path=gealan) — Gealan, [Kömmerling](https://www.3dfindit.com/en/cad-bim-library/manufacturer/kommerling?path=kommerling) | DWG/**DXF**/STEP (exportkeuze) | gratis account |
| **Veka** (pvc) | [Bibliocad](https://www.bibliocad.com/en/library/veka-windows-details_1561/), [CadBull](https://cadbull.com/detail/46824/Veka-system-section-detail-dwg-file) | DWG-sectiedetails | gratis/account |
| Generiek | 3Dfindit, Bibliocad, CADforum, BIMobject, Polantis | DWG/DXF/BIM | wisselend |

**Belangrijk:** de meeste leveranciers leveren primair **DWG** (binair AutoCAD) en BIM
(Revit). **DXF** (ASCII) is vooral te krijgen via portals als 3Dfindit (exportformaat
kiezen) of door zelf DWG→DXF te converteren (bv. ODA File Converter, gratis).

## 2. Formaat: DWG vs DXF
- **DWG = binair** → de huidige import (`parse_dxf_profile`) leest tekst; DWG kan **niet**
  rechtstreeks. Vereist conversie naar DXF (ODA File Converter / export uit CAD / 3Dfindit).
- **DXF = ASCII** (group-code/value-paren) → in principe leesbaar, maar zie §4.

## 3. Huidige import (`ofs-core/src/import/dxf_profile.rs`)
- Leest DXF-tekst, herkent entities: **LINE** (10/20/11/21), **LWPOLYLINE** (10/20),
  **POLYLINE+VERTEX** (10/20).
- Verzamelt alle punten → **`convex_hull(points)`** → bounding-box → `width`/`depth`;
  `sightline = width*0.8`, `glazingRebate = width*0.36` (geschat, niet uit geometrie).
- UI-pad: ribbon **"Profiel DXF"** → command `import_profile` → `parse_dxf_profile`.

## 4. Verdict — kan de huidige import echte leveranciers-DXF aan? **Nee.**
Vier blokkades:

1. **Convex hull vernietigt het profiel.** Een profieldoorsnede is **concaaf**
   (meerkamer + glassponning-inkeping). `convex_hull()` maakt er de buitenste omhullende
   van → alle kamers én de sponning-inkeping verdwijnen; je houdt een blok over.
   Dit alleen al maakt de import ongeschikt, óók bij een perfecte DXF.
2. **Ontbrekende entities.** Geen **SPLINE** (ronde kamerwanden/afrondingen),
   **ARC/CIRCLE** (hoekafrondingen, rubbergroeven), **ELLIPSE**, **HATCH**, en geen
   **INSERT/BLOCK**-resolutie. Leveranciers-DXF zit vol met deze → geometrie gaat verloren
   of staat op de verkeerde plek (block-offsets worden genegeerd).
3. **Geen layer-filtering.** Alle punten worden op één hoop gegooid; maatlijnen, glas,
   staalversterking, rubber en arcering vervuilen de puntenwolk (en dus de hull).
4. **Geen units / oriëntatie.** `$INSUNITS` wordt niet gelezen; X=breedte/Y=diepte wordt
   aangenomen; aanzicht en sponning worden geschat, niet uit de tekening gehaald.

Plus: **DWG wordt helemaal niet ondersteund** (binair).

## 5. Aanbevolen route
**A. Importpijplijn geschikt maken (voorkeur voor accurate doorsnedes):**
1. DWG→DXF buiten de tool (ODA File Converter) of vraag leveranciers om DXF/exporteer via 3Dfindit.
2. Nieuwe DXF-parser: entities **LWPOLYLINE (incl. bulge→arc), ARC, CIRCLE, SPLINE, ELLIPSE**,
   plus **INSERT/BLOCK**-resolutie en **layer-filtering** (alleen contour-layer).
3. **Convex hull schrappen** → de werkelijke gesloten polyline behouden; de **buitencontour**
   isoleren (grootste gesloten lus) en interne kamers optioneel apart bewaren.
4. Units uit `$INSUNITS`; aanzicht/bouwdiepte/sponning **uit de geometrie** afleiden
   (sponning-inkeping detecteren op de concave contour, niet op de hull).
5. Overweeg de Rust-crate `dxf` (kabeljau/dxf-rs) i.p.v. handmatig parsen.

**B. Alternatief zonder CAD-import:** doorsnedes **handmatig overtrekken** in de bestaande
profiel-editor (`ProfileCanvas` — vertex-editor met `crossSection`), op basis van de
leverancier-sectietekening/PDF. Arbeidsintensiever maar volledig in eigen beheer en exact te
controleren; geen DWG/DXF-afhankelijkheid.

**C. Pragmatische tussenstap:** per familie een **realistische parametrische** doorsnede
genereren (correcte aanzicht×bouwdiepte, glassponning-stap, frame↔raam-overdek, N kamers,
Gealan-15°-schuinte) — benadering, maar veel beter dan de placeholder en consistent.

## 6. Bevindingen op echte sample-DXF's + prototype-validatie
Map `samples/` bevat **502 DXF-bestanden** (een "ps…"-systeem, ASCII DXF, AC1027/AutoCAD 2013).
Geanalyseerd + getest met een Node-prototype (`temp/dxf-proto.mjs`).

**Structuur (essentieel):**
- **Layer-gebaseerd.** In veel bestanden staan LINE/ARC op layer **`DIMENSIONS`** = maatlijnen,
  niet de contour. De doorsnede staat op layer **`0`** (of een contour-layer). → **layer-filtering verplicht.**
- **Contour = geketende LINE+ARC** (bv. `ps1023_Default`: 24 LINE + 16 ARC op layer 0), of een
  gesloten **LWPOLYLINE/POLYLINE** (met bulge-arcs), of **ELLIPSE**.
- Sommige bestanden (bv. `ps1013`) zijn **alleen maatlijn-/detailtekeningen** → geen contour.
- **HATCH** = vulling (te negeren of als boundary-bron).
- Entity-woordenschat over 502: LINE 462 · ARC 367 · LWPOLYLINE 207 · HATCH 187 · ELLIPSE 92 ·
  POLYLINE 45 · SPLINE 33 · INSERT 7 · SOLID/CIRCLE 4.

**Prototype-resultaat (aanpak gevalideerd):** layer-filter + arc/bulge/ellipse-tessellatie +
**segment-chaining → grootste gesloten lus** levert echte concave contouren:
`ps1023` 89 pt (gesloten, 230×35), `ps100` 317 pt (119×20), `ps1036` 225 pt (196×17,5).
Dimensie-only bestanden worden correct als "geen contour" gemarkeerd. **De convex-hull vervalt.**
Restpunten: een enkel bestand sluit nog niet (gap-bridging/tolerantie nodig); parser moet tot de
`ENTITIES`-sectie scopen (HEADER/TABLES/BLOCKS overslaan); SPLINE/INSERT nog toevoegen.

→ **Verdict bijgesteld: JA, de import kán dit aan na herbouw** — de aanpak is op de echte
bestanden bewezen. Route A is haalbaar.

## Status
Route **A — import herbouwd** ✅. `ofs-core/src/import/dxf_profile.rs` is herschreven naar het
gevalideerde algoritme: DXF-tokenizer, scope op de `ENTITIES`-sectie, **layer-filtering**
(annotaties/maatlijnen eruit), tessellatie van **LINE / ARC / CIRCLE / ELLIPSE /
LWPOLYLINE(bulge)**, **segment-chaining → grootste gesloten lus** (convex hull verwijderd), en
sponning/aanzicht uit de echte contour. Publieke API ongewijzigd (`parse_dxf_profile`,
`ImportedProfile`, `Sponning`); callers (`commands/import_profile.rs`, `catalog.rs`) ongemoeid.
Review-agent: **compiles clean** (geen lokale cargo; echte gate = CI #4481904).

**Dekkingsmeting (Node-prototype `temp/dxf-proto.mjs --all`, 627 DXF's):**
**257 gesloten contour (41%) · 275 open (44%) · 95 geen contour (15%) · 0 errors.**
De open-bestanden bevatten vooral LINE+ARC+ELLIPSE → ze sluiten niet door **chaining-gaps**,
niet door missende SPLINE (slechts 14 open-bestanden hebben SPLINE, 6 INSERT). De 95 zonder
contour zijn vrijwel zeker maatlijn-/annotatiebestanden.

## 7. Gap-bridging-onderzoek (18 jun 2026) — diagnose + besluit
De "open"-bestanden zijn gemeten (`temp/dxf-diag.mjs`, `temp/dxf-bridge.mjs`, `temp/dxf-face.mjs`)
om de juiste fix te kiezen i.p.v. een tolerantie te gokken. Drie bevindingen:

**a) Ruimere chaining-tolerantie helpt NIET — dood spoor.** Sweep van de join-tolerantie
(0.05 → 0.2 → 0.5 → 1.0 → 2.0 mm) levert vrijwel vlakke dekking (267→265→259→259→270) en
op de middenwaarden zelfs *minder*: grotere tolerantie maakt nieuwe fóute joins die eerder-correcte
sluitingen breken. De gemeten head-tail-gap van de grootste open lus zit bij **214 van de 275**
bestanden **>5 mm** uit elkaar — het is dus géén bijna-sluitende lus maar een **gefragmenteerde
keten**. De fragmenten zíjn er wel (nearest-bridge-gap vooral 0.1–0.5 mm), maar het probleem is
**topologisch, niet gap-grootte**: de greedy keten slaat bij een junction (waar een kamerwand de
buitencontour raakt) de verkeerde tak in.

**b) Best-match chaining — veilige winst, geïmplementeerd.** I.p.v. *first-match* (eerste endpoint
binnen tolerantie) kiest de chainer nu de *dichtstbijzijnde* endpoint over alle 4 oriëntaties.
Dat houdt de wandeling bij junctions op de echte buitencontour. Apples-to-apples op identieke set
(525 bestanden, cap op 7 hele-raam-tekeningen van 39k–76k segments): **253 → 267 gesloten (+14,
48,2 % → 50,9 %)**, zonder tolerantie te wijzigen. ✅ Doorgevoerd in `dxf_profile.rs::chain`
(en gespiegeld in `temp/dxf-proto.mjs`).

**c) Planaire-graaf buitencontour-trace — geïmplementeerd als hybride fallback.** ✅
Algoritme (`temp/dxf-face.mjs`, gespiegeld in `dxf_profile.rs::outer_contour`): knopen snappen
(bridget 0.1–0.5 mm gaps op graaf-niveau, snapTol 0,5 mm), dangling spurs snoeien, grootste
samenhangende component isoleren, alle faces tracen via de hoek-next-edge-regel, buitenste face =
max |area|. Reproduceert de bekende-goede contouren exact (ps1023 230×35, ps100 119×20, ps1036
196×17,5). **Cruciaal: het draait als *fallback*, alléén wanneer best-match-chaining de lus niet
sluit** — zo ziet het nooit de kleine schone profielen die het anders over-collapste, en die blijven
via chaining correct. **Degenerate-rejection** (fill = area/bbox < 0,12 → falen i.p.v. garbage) vangt
de teruggevouwen/collineaire traces af.

**Hybride-meting (`temp/dxf-hybrid.mjs`, 525 meetbare bestanden, snapTol 0,5):**
chaining sluit 267, face-trace herstelt nog **152** → **419 gesloten (79,8 %)** · 106 open · 0 errors.
(snapTol 0,3 → 404.) De ~152 herstelde zijn bij inspectie overwegend echt (incl. kleine accessoires
als rubbers/glaslatten en lange stijlen die buiten een naïeve plausibiliteitsband vallen). Dit
**verdubbelt** de dekking t.o.v. chaining-alleen (267 → 419). Unit-tests dekken spur-pruning +
gap-bridging.

**Nog te doen — bijgestelde prioriteit:**
1. ~~Gap-bridging via ruimere tolerantie~~ — **dood spoor (gemeten).** Best-match-chaining ✅ gedaan.
2. ~~Hybride face-trace-fallback~~ — ✅ geïmplementeerd (267 → 419, 79,8 %). Mogelijke verfijning:
   adaptieve snapTol per profielgrootte voor de laatste ~106 open bestanden.
3. **SPLINE** (14 open) + **INSERT/BLOCK** (6) — kleine winst.
4. `$INSUNITS`/units.
5. **DWG→DXF-conversie** (ODA File Converter) voor binaire bronnen.

Validatie op de samples via de `temp/dxf-*.mjs`-prototypes (Node, lokaal); de echte (Rust) run pas
na wasm/CI-herbouw (#4481904).
