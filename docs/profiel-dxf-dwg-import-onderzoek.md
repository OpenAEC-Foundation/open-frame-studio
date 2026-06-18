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

## Status
Onderzoek vastgelegd; geen code gewijzigd. Keuze A/B/C ligt bij gebruiker (domein-expert).
Backend-werk (nieuwe parser) is niet lokaal te compileren → via review + CI (#4481904).
