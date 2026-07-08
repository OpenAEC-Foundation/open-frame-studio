# Profielmaten-onderzoek — correctielijst en geometrie-regels

**Status:** onderzoeksronde juli 2026, ter review
**Betreft:** Open Frame Studio v0.5.0 — profieldata (`profiles/`), maatketens (`ofs-core`), tekenlaag (`KozijnCanvas`)

## Aan de reviewer

Dit document is het naslagwerk bij de datacorrectie van juli 2026. Na v0.5.0 bleek dat profielmaten en sponning-/glaslatdieptes niet met de werkelijkheid klopten. Deze ronde is eerst onderzocht (KVT-online, NPR 3577-uitleg, fabrikantendocumentatie VEKA/Gealan/Kömmerling/Reynaers/Schüco, DTS/TO-fabrieksdocumentatie, VMRG, EN 12020-2) en pas daarna gecorrigeerd.

**Jij bent de beste bron.** Elke maat hieronder staat er mét herkomst en een eerlijke zekerheidsindicatie. Waar bronnen ontbraken is een default gekozen met motivatie — niets is stilzwijgend verzonnen. Loop de tabellen na en corrigeer wat niet strookt met jouw fabriekspraktijk: jouw correcties gaan boven elke publieke bron en worden direct in de data en in dit document verwerkt. Let vooral op de regels gemarkeerd **afgeleid** of **unverified** — daar is de bewijslast het dunst.

Zekerheidslegenda:

| Markering | Betekenis |
|---|---|
| ●● | bevestigd door meerdere onafhankelijke bronnen |
| ● | één bron |
| ○ | afgeleid (consistentiecheck of naamconventie, geen directe bron) |
| ⚠ | unverified — geen bron gevonden; waarde staat er met vlag, niet als feit |

---

## 1. Diagnose: vier systemische fouten

1. **Sponninghoogte schaalde mee met de profieldiepte** (17 → 38 mm). Oorzaak: de import-heuristiek `glazing_rebate = width × 0,36` (`ofs-core/src/import/dxf_profile.rs:145` en `catalog.rs:324`) — 67 mm breed → 24, 90 mm → 32. Die reeks is letterlijk in alle hout-JSONs terechtgekomen. Per norm is de sponninghoogte echter **vast**: hout 17 mm (KVT 12.2), PVC 18–20 mm en alu 13,5–27 mm per systeem. ●●
2. **Glasmaat werd berekend als dagmaat − 2×4 mm** (`production.rs:11,540-548`) in plaats van sponningmaat − 2×omtrekspeling. Bij 17 mm sponning en 5 mm speling hoort glasmaat = dagmaat + 24 mm; de uitvoer bestelde dus glas **32 mm te klein per richting**. ●●
3. **Reynaers: bouwdiepte en aanzicht verwisseld** — CS 77 stond met depth 68; actuele hoofdsystemen (MasterLine 8, SlimLine 38) ontbraken. ●
4. **Niet-bestaande producten/maten in de bibliotheek**: Gealan "S 9000 Plus (92 mm)", glaslat 56×60, raamhout 54×67, dorpel 67×150, tussenstijl 54 mm breed, PVC-wanddikte "Klasse A (3 mm)", Deceuninck 70 mm glas in 76 mm bouwdiepte. ●●/○ per geval, zie tabellen.

---

## 2. Hout (KVT / fabriekspraktijk)

### 2.1 Sponning en glasinval

| Grootheid | Was (v0.5.0) | Wordt | Bron | Zekerheid |
|---|---|---|---|---|
| Sponninghoogte (glasinval, vlakrichting) | 17–38, schaalde met diepte | **17 mm vast**, alle houtprofielen | KVT 12.2; Uitleg NPR 3577 (Kenniscentrum Glas 2018) | ●● |
| — alternatief | — | 20 mm (KVT-alternatief); 18–20 als triple-advies | KVT 12.2 | ●● |
| Sponningdiepte vast glas (bouwdiepterichting) | zelfde foute reeks 17–38 | **51 mm** (fabriek 57; DTS 51/53/57 × 17) | KVT-tekening 14.01; TO-binnendetaillering 2024; DTS-folder | ●● |
| Aanslagsponning draaiende delen | idem | **21–30 mm** diep, fabrieksdefault **29** met 6 mm lucht | KVT 14.01; TO 2024 | ●● |
| Achterhout | ontbrak | ≥13 mm (bij sponning 17) / ≥15 mm (bij 20) | KVT 12.2 | ●● |
| Sponninghoogte bovenkant onder-/tussendorpel | uniform | mag **14** (binnensponning) / **16** (buitensponning) | KVT 14.2 + 13.2/tekening 13.01 | ●● |

NB: één veld (`glazingRebate`) kan vast en draaiend niet allebei uitdrukken; het datamodel krijgt gescheiden waarden per toepassing.

### 2.2 Kozijnhout-secties (eerste getal = aanzicht, tweede = diepte)

| Onderdeel | Was | Wordt | Bron | Zekerheid |
|---|---|---|---|---|
| Stijl / boven-/onderdorpel standaard | 67×114 ✓ | 67×114 (blijft) | KVT 61.1; DTS | ●● |
| Onderdorpel "67×150" | 8 profielen | **67×114** standaard; zwaar **67×139**; hefschuif 67×139 (of 67×122 bij 56 mm deur). 67×150 is geen KVT-reeks- of handelsmaat | KVT 61.1; DTS-folder p.8/23-25; TO 2024 | ●● |
| Tussenstijl / tussendorpel "54×114" | 4 JSONs | **90×114** (KVT 30.5.3 staat 67 of 90 toe; typen C/D onmogelijk in 67; verschil met stijl ≤25 mm). 54 mm bestaat niet als kozijn-tussenstijl | KVT 30.5.3; TO 2024 p.11-12 | ●● |
| Raamhout "54×67" | aanwezig | bestaat niet; kleinste KVT-maat is **54×78** (reeks 54×78/90/102, 66×78…139, 78×78/90/102) | KVT 61.1; TO 2024 p.3 | ● |
| Raamhout "84×90" | aanwezig | **78×90** (KVT-reeks) | KVT 61.1 | ● |
| Raamhout 69×90 (draaikiep) | aanwezig ✓ | blijft — fabrieksstandaard draaikiep, zichtdeel 55 mm | TO 2024 p.3 | ●● |
| Meranti "54×114 (roede)" | aanwezig | herzien: echte glasroede is veel lichter | TO 2024 | ● |

### 2.3 Glaslatten

| Grootheid | Was | Wordt | Bron | Zekerheid |
|---|---|---|---|---|
| Assortiment | o.a. "Glaslat 56×60" (bestaat niet; dat is raamhoutformaat) | norm-min **15×17**; fabriek **17×17** (dubbel glas, verdekt genageld); handel **15×28 / 17×28 / 20×34,5** (isolatie/triple) | KVT 12.3.2; NPR 3577-uitleg; TO/DTS; bakkerdehouthandel.nl | ●● |
| Glaslathoogte-eis | geen validatie | ≥17 **én** ≥ sponninghoogte | KVT 12.3.2 | ●● |
| Glaslatbreedte (voet/oplegbreedte) | veld leeg in alle JSONs | ≥15 buiten / ≥13 binnen | KVT 12.3.2 | ●● |
| Default in code (`glaslat.rs:47-48`) | 15×17 ✓ | blijft (norm-minimum), evt. 17×17 als fabrieksdefault | KVT 12.3.2 | ●● |
| Binnen-/buitenbeglazing | — | binnenbeglazing default bij draaiende delen; onderaan neuslat 5 mm vrij | NPR 3577 | ●● |

Door de gebruiker eerder genoemde 16×28 en 22×28 zijn in geen enkele bron aangetroffen — 15×28/17×28 gebruikt. **Graag bevestigen of jullie werkelijk 16×28/22×28 voeren.**

### 2.4 Onderdorpel-geometrie (ontbrak volledig in het model)

| Regel | Waarde | Bron | Zekerheid |
|---|---|---|---|
| Afschot sponning onder-/tussendorpel | ≥9° (praktijk 10°) | KVT 14.2; TO 2024 | ●● |
| Sponningbodem | 45 mm afwaterend | KVT-tekening 14.01 | ●● |
| Opstand/lijmlat | ≥8 breed × ≥14 hoog (vast) / ≥15 (draaiend) | KVT-tekening 14.01 (één tekening) | ● |
| Buitenopstand | 12–17 mm | KVT 14.01 | ● |
| Waterhol óf alu lekdorpel | **verplicht** in boven- en tussendorpels | KVT 13.5/14.5 | ●● |
| Vrije ruimte onder de neus | 15 mm | KVT 11 | ●● |
| Dorpelhoogte woningtoegang | ≤20 mm | KVT 11 | ●● |
| Neuten (kunststof) | 55 mm hoog standaard (leverbaar tot 150), breedtes 67/90, contraprofiel op dorpelsponning | DTS-folder (KOMO 20676) | ●● |
| Hout boven maaiveld | pas ≥50 mm; bergingsdorpel-verbinding ≥100 mm | KVT 11.3 | ●● |
| Voorsponning binnendraaiende deur | 6×9 mm; een "12 mm dievenklauw-sponning" bestaat niet (dievenklauw = beslag, pin ø9–11) | TO 2024 p.12; dekozijnenman.frl | ●● |

### 2.5 Niet aangepast (geen bron — vlag "unverified")

Hout-Uf-waarden (1,1–2,0) en maxGlassWeight (40–140 kg), Weekamp 56×67 stomp, HEBO-"kunststof"-entries in `profiles/wood/hebo.json` (staan in de houtcategorie én zijn als HEBO-product niet geverifieerd — verplaatsen/vlaggen). **Correcties uit de praktijk zeer welkom.**

---

## 3. PVC (systeemhuizen)

Kernles: bij PVC zijn **Falzhöhe** (totale sponninghoogte incl. lucht) en **Glaseinstand** (gedekte glasrand = getekende glasinval) twee verschillende grootheden — het datamodel krijgt er twee velden voor.

### 3.1 VEKA Softline 82

| Grootheid | Was | Wordt | Bron | Zekerheid |
|---|---|---|---|---|
| Wanddikte | "Klasse A (3 mm)" | Klasse A (DIN EN 12608) = **≥2,8 zichtvlakken / ≥2,5 overig**; "3 mm" bestaat niet als klassewaarde | tmp-fenster.de; classicfenster.de | ●● |
| Glaseinstand | 28 (was Falzhöhe als glasinval getekend) | **20 mm**; Falzhöhe 28 blijft als apart veld | VEKA TPI Softline 82 MD (07/2020) p.1: letterlijk "Glasfalzhöhe 28 / Glaseinstand 20" | ●● |
| Glasinval vaste beglazing (AD) | — | 25 mm | Baltic Technikheft 82MD p.10-11 (maattekeningen) | ○ |
| AD/MD-variant | één entry: sealingLevels 2 + Uf 1,0 (combinatie bestaat niet) | **AD: 2 niveaus / Uf 1,2. MD: 3 niveaus / Uf 1,1** — variantveld toegevoegd | VEKA TPI p.1+3; fensterblick.de | ●● |
| Aanzichten | sightline 61 (onbebrond) | kozijn 73 (breed 83/106); vleugel 84 (zichtdeel 51); combinatie 124; T-stijl 94/124; stulp 74, naad 8 | VEKA TPI p.1-2; Baltic p.4-9 | ●● |
| Falzluft kozijn–vleugel | — | 12 mm | Baltic-sneden | ○ |
| Glaslat-parametriek | — | cliphoogte **25 constant**; latbreedte = **59,5 − glasdikte** (24→35,5 … 52→7,5; 54 via 15°-lat art. 107.278). Glasdikte bepaalt de lat, niet andersom | Baltic Technikheft p.35 | ● |
| Glasbereik | 24–52 ✓ | blijft | VEKA TPI | ●● |

### 3.2 Gealan S 9000

| Grootheid | Was | Wordt | Bron | Zekerheid |
|---|---|---|---|---|
| "S 9000 Plus (92 mm)" | volledig profiel (Uf 0,82, glas 60) | **verwijderd** — geen bewijs dat het bestaat; S 9000-platform is uitsluitend 82,5 mm | gealan.de/de/systeme/s-9000 | ○ |
| Glasaanslaghoogte | 28 | **18 mm** | GEALAN Zubehör-/Zusatzprofile 03-2020 | ●● |
| Overslaghoogte kozijn–vleugel | — | 26 mm; karakteristieke 15°-schuinte | idem | ●● |
| Falzluft / stulpnaad | — | 12 / 6 mm | STOLMA-systeemsneden p.85 | ● |
| Aanzichten (STOLMA-uitvoering) | 62 (onbebrond) | kozijn 70; DK-combinatie 110; T-stijl 92 (combi 172); stulp 74 (combi 154) — van tekeningen afgelezen | STOLMA-sneden | ○ |
| maxGlass | 52/54 | **54 (STV)** aangehouden; gealan.de noemt inmiddels 56/58 — als noot in de data, niet overgenomen | gealan.de vs STV-opgave | ● |

### 3.3 Kömmerling

| Systeem | Was | Wordt | Bron | Zekerheid |
|---|---|---|---|---|
| 88 MD: Uf | 1,2 | **0,95** (officieel standaard); glas tot **58**; kozijnaanzicht 74; centrische staalversterking = systeemkenmerk | koemmerling.com; bew24-fenster.de | ●● |
| 76 MD: maxGlass | 50 | **56** ("bis 56 mm" officieel; 50 = oudere dealeropgave — noot); Uf 1,0 klopt | koemmerling.com | ●● |
| 76 AD (ontbrak) | — | toegevoegd: 5 kamers / 2 niveaus / glas ≤50 / Uf 1,1 / kozijnaanzicht 67 | koemmerling.com; aanzicht dealeropgave | ● |

### 3.4 Overig PVC

| Item | Besluit | Bron | Zekerheid |
|---|---|---|---|
| Deceuninck Elegant 76: maxGlassThickness 70 | fysiek onmogelijk (70 mm vulling in 76 mm bouwdiepte laat 6 mm voor beide schalen) → **unverified-vlag**, corrigeren met fabrieksdata, niet gokken | eigen consistentiecheck | ○ |
| Schüco LivIng 82 (kandidaat-toevoeging) | 82 mm / 7 kamers / lasbare EPDM; AS 2 niveaus Uf 1,0, MD 3 niveaus Uf 0,96; glas 16–54 (officieel; dealers 24–52); kozijnaanzicht 70, min. combinatie 120 | Schüco-brochure P4151/07.20 p.8-9; widuro.de; bew24-fenster.de | ●● |
| Gealan S 8000, VEKA Softline 70, Gealan-KUBUS, Aluplast IDEAL, overig Deceuninck | niet onderzocht deze ronde → **unverified-vlag**, niet aangepast | — | ⚠ |

---

## 4. Aluminium (Reynaers / Schüco)

### 4.1 Reynaers

| Grootheid | Was | Wordt | Bron | Zekerheid |
|---|---|---|---|---|
| Bouwdieptes | CS 77 depth 68 (2×), CS 86-HI depth 77, CP 155 depth 68 | systeemnaam = bouwdiepte: **CS 77 → 77, CS 86 → 86, CP 155 → 155** (CP 155 afgeleid uit naamconventie) | reynaers.com; naamconventie | ● / ○ (CP155) |
| Aanzichtbreedtes (width-veld bevatte de dieptes) | 77/86/155 | opnieuw te bepalen; CS 77-aanzichten niet onderzocht → **unverified-vlag** | — | ⚠ |
| MasterLine 8 (ontbrak) | — | kozijn 77 / vleugel 87; aanzicht binnendraaiend 53+37=97 (buitendraaiend 20/118/138); **sponninghoogte 27**; glaslat 25 hoog, klik; Ω-steg PA6.6 GF25 **40 mm** (vlakke deuren 32); wanddikte 1,6–2,5; Uf 1,9/1,5/1,2 (std/HI/HI+ bij 119 mm sectie); glas ≤62 vast / ≤72 draaiend; vleugel-overlap 6, schaduwvoeg 7, vleugel 10 mm dieper; EPDM-middendichting (overlap aanslaglip 4,5) | reynaers.com ML8 technical info; ML8-bestektekst; ATG 3067 | ●● |
| ML8 glas-minimum | — | **24** (ATG) aangehouden voor validatie; reynaers.com noemt 13 — conservatieve keuze, ATG is het keuringsdocument | ATG 3067 vs reynaers.com | conflict, keuze gedocumenteerd |
| SlimLine 38 (ontbrak, steellook) | — | Classic kozijn 99 / vleugel 86 (Cubic 76/72, Ferro 76/86); combinatie-aanzicht ±67; **sponninghoogte 13,5**; glas 16–55; Uf 1,9 | reynaers.com SL38-pagina's; reynaersrus.com | ●● |
| CS 77-detailwaarden (Uf 2,2/1,2/1,0, glas 63/72) | aanwezig | niet geverifieerd deze ronde → vlag | — | ⚠ |

SL38 (13,5) naast ML8 (27) bewijst: **sponninghoogte is systeem-specifiek, niet materiaal-generiek** — de uniforme 25 in beide alu-JSONs kon niet blijven.

### 4.2 Schüco AWS

| Grootheid | Was | Wordt | Bron | Zekerheid |
|---|---|---|---|---|
| Sightline 70.HI | 79 (+Bottom 104 / Mullion 94 / Vent 41 — alle onbebrond) | velden gesplitst in kozijn-/vleugel-/combinatie-aanzicht; officieel: kozijnprofielen vanaf 51; pfosten 76–250; 75.SI+ maatketen 41+7+59=107 (min. combi 91) | Schüco Systemeigenschaften 75.SI/70.HI (lbprofile.de p.67,71,84-89) | ● |
| Glasdiktes 70.HI | max 50 | vast 11–52 / vleugel 21–62 | idem p.71 | ● |
| Glasdiktes 75.SI+ | max 50 | 19–55 / 19–65 | idem | ● |
| Glasdiktes 90.SI+ | max 62 | 28–63 / 28–68 | idem | ● |
| Uf 90.SI+ | 0,8 | "tot **0,71**" | brochure P3933 | ● |
| Uf 75.SI+ | 1,3 | **1,2** bij 117 mm declaratie-aanzicht met PA-stegen (tot 0,92 met PT) | idem | ● |
| Uf 70.HI | 1,5 ✓ | blijft | idem | ● |
| Sealing levels 70.HI | 2 | **3** (fabrikantsopgave, MD) — **bronconflict**: fensternorm.com zegt 2, schueco.com zegt 3; vermoedelijk telverschil (glasdichtingen wel/niet meegeteld). Conflictnotitie in de data | schueco.com vs fensternorm.com | conflict, keuze gedocumenteerd |
| AWS-sponninghoogte | 25 generiek | **niet publiek gevonden** → 25 laten staan met unverified-vlag | — | ⚠ |
| Stegbreedte Schüco | — | niet publiek → niet verzonnen | — | ⚠ |

### 4.3 Alu-doorsnedemodel en vleugeldiepte

| Regel | Waarde | Bron | Zekerheid |
|---|---|---|---|
| Doorsnede-opbouw | buitenschaal + isolatiesteg + binnenschaal (i.p.v. PVC-achtige kamers); wanddikte nominaal 1,6–2,5 | ML8-bestek §1/§3 | ●● |
| Steg-referentiereeks | insulbar 10–54 mm met Uf-trap 24→2,6 / 34→1,9 / 42→1,5 W/m²K | insulbar standard program | ● |
| Toleranties | EN 12020-2 (cat. A ±0,15–0,25; cat. B om holten ±0,20–0,40 bij t≤3); VMRG eist EN 12020-2 | DIN EN 12020-2:2001 tabel 2; VMRG 2015 | ●● |
| Vleugeldiepte (`ventDepth`, nieuw veld) | patroon: vleugel = kozijn + 10 (Schüco 70/80, 75/85, 90/100; ML8 77/87; Aliplast 75/84); **uitzondering SL38 Classic 99/86** (vleugel ondieper) | Schüco-tabel p.71; reynaers.com; profilnet.eu | ●● |

---

## 5. Geometrie-regels tekenlaag en productie (R1–R12)

Deze regels hanteert de software na de correctie. Elk met bron; de nummers worden in code-commentaar aangehaald.

| # | Regel | Bron |
|---|---|---|
| **R1** | Sponninghoogte komt uit systeemdata en schaalt **nooit** mee met de profieldiepte. Hout 17 (KVT-alt 20; triple 18–20); VEKA Glaseinstand 20 (vast AD 25); Gealan 18; ML8 27; SL38 13,5; CS77/Schüco AWS onbekend → 25 met unverified-vlag. Canvas-fallback 17 blijft. | KVT 12.2; fabrikantendocs §3-4 |
| **R2** | Maatketen glas: sponningmaat = dagmaat + 2×sponninghoogte; omtrekspeling = max(5; ⅓×sponninghoogte); **glasmaat = sponningmaat − 2×omtrekspeling** (= dagmaat + 24 bij 17/5). Achterhout ≥13 (17-sponning) / ≥15 (20-sponning). Praktijkcheck: glasmaat = sponningmaat − 6 à 10 mm. | NPR 3577-uitleg; KVT 12.2; glasdiscount-inmeetregel |
| **R3** | Glaslat: hoogte ≥ max(17; sponninghoogte). Tekenoffset glaslatlijn t.o.v. sponninglijn = **glaslathoogte − sponninghoogte** (0 bij 17×17-lat op 17-sponning; 11 bij 28-lat) — géén vaste 5/6 mm en niet de voetbreedte. Lengtes: horizontaal sponningmaat − 1; verticaal (binnen) sponningmaat − 2×latdikte − 1. Binnenbeglazing default bij draaiende delen; neuslat onderaan 5 mm vrij. | KVT 12.3.2; NPR 3577 |
| **R4** | Vleugel-overdek vooraanzicht: binnendraaiend valt de vleugel (sponningdiepte − lucht) **over** de daglijn heen — sponningdiepte 21–30 (default 29), lucht 6, kaderdichtingsgroef 8×9, zichtdeel raamhout 55; buitendraaiend overdek 17. Vleugel dus buiten het dagrect tekenen, niet erbinnen (was: gap +2 erbinnen). | KVT 14.01; TO 2024 p.3/11-12 |
| **R5** | Glaslijn in de vleugel: inset = sashWidth − sponninghoogte vanaf de buitenrand (= 17 vanaf de dagrand), zelfde conventie als de kozijnleden. | KVT 12.01-terminologie (h vanaf dagkant) |
| **R6** | Onderdorpel: afschot ≥9° (praktijk 10°), sponningbodem 45 afwaterend, opstand ≥8×≥14/15, buitenopstand 12–17, waterhol óf alu lekdorpel verplicht in boven-/tussendorpels, 15 mm vrij onder de neus, toegangsdorpel ≤20. Bovensponning dorpels mag 14 (binnen) / 16 (buiten). | KVT 14.2, 13.5/14.5, 11, tek. 14.01/13.01 |
| **R7** | Kozijnhout-secties: eerste getal = aanzicht, tweede = diepte; stijl/dorpel 67×114 standaard, tussenstijl/-dorpel 90×114, draaikiep-raamhout 69×90, verschil stijl–dorpel ≤25 mm. | KVT 61.1, 30.5.3; TO 2024 |
| **R8** | PVC: Falzhöhe ≠ Glaseinstand — twee velden. VEKA-glaslat: cliphoogte 25 constant, breedte = 59,5 − glasdikte. Falzluft kozijn–vleugel 12. Overslaghoogte Gealan 26. Stulpnaad VEKA 8 / Gealan 6. Wanddikte Klasse A 2,8/2,5. | VEKA TPI; Baltic; GEALAN/STOLMA; EN 12608 |
| **R9** | Alu: ventDepth = frameDepth + 10 (uitzondering SL38); doorsnede = buitenschaal + steg (ML8 40, deur 32) + binnenschaal, wanddikte 1,6–2,5; glaslat 25 klik; overlap 6 + schaduwvoeg 7 (ML8). | ML8-bestek; Schüco-tabel; insulbar |
| **R10** | Beglazingsblokjes: steunblok 50/75/100 lang × (glasdikte + 2) breed × omtrekspeling dik; hart ≤ ¼ sponninglengte uit de hoek; stelblokjes pas >1 m²; wig 6° op afwaterende dorpel. | NPR 3577-uitleg; DTS p.7 |
| **R11** | Neut: kunststof 55 hoog standaard (tot 150), breedtes 67/90; hout pas ≥50 mm boven maaiveld. Voorsponning binnendraaiende deur 6×9 mm. | DTS (KOMO 20676); KVT 11.3; TO 2024 p.12 |
| **R12** | Import-defaults: de heuristiek `width × 0,36` is vervangen door de R1-waarden per materiaal — anders herintroduceert elke import de schaalfout. | eigen constatering (dxf_profile.rs:145; catalog.rs:324) |

Aanvullende productie-/validatieregels (uitbreiding, alle ●● tenzij vermeld):

- **Glaslatbevestiging** (KVT 12.5/12.5.1): schroef ≥3,5 dik, hechtlengte ≥15 in kozijnhout (17-lat → schroef ≥32), eindafstand ≤50, h.o.h. ≤200; nagel ≥1,8×38 h.o.h. ≤150; buitenbeglazing RVS verplicht. Neuslat: bolkopschroef ≥4,0 / hechtlengte ≥20 / h.o.h. ≤300; nagel 1,8×43 ≤150; neus ±5 mm buiten de dorpel.
- **Afdichting** (NPR 3577; ML8-bestek §6/§9; VMRG): rugvulling ≥4; kitvoeg ≥4×6 afwaterend; hielafdichting onder + 200 mm op; alu drainage vleugel 15×5 h.o.h. ≤500, drukegalisatie ø5 op 250 uit hoeken; beglazingsdruk 500–1500 N/m.
- **Validatie toe te voegen**: glaslathoogte ≥ sponninghoogte; achterhout ≥13/15; afstand tussen aanslagen ≥15; aanslagvlak <7 mm na afschaven = maatregel vereist (KVT 12.2/12.3.2).
- **Sash-default**: één default 69 mm (was inconsistent 67 op production.rs:611 vs 54 op :636; KVT-minimum 66×90, fabriek 69×90).
- **Bonus certificering** (●): standaard KVT-kozijn met 17 mm sponningen voldoet aan 30 minuten brandwerendheid (bouwwereld.nl) — bruikbaar als plausibiliteitsregel.

---

## 6. Conflicten en gaten — gekozen defaults met motivatie

| # | Kwestie | Besluit + motivatie |
|---|---|---|
| C1 | Aanname vooraf "glasinval hout ≥18" | Bronnen zeggen **17** (norm-minimum, meerdere bronnen) → 17 gekozen. |
| C2 | Aanname "glasinval PVC ≥25" | Geen universele 25: VEKA 20 (draaiend/MD), 25 alleen vaste beglazing AD (afgeleid), Gealan 18 → **per systeem opslaan**. |
| C3 | Hypothese "glaslathoogte = sponninghoogte − steldikte" | **Verworpen** — strijdig met KVT 12.3.2 (hoogte ≥ sponninghoogte); steldikte (≥5) zit in de omtrekspeling. |
| C4 | Schüco 70.HI dichtingsniveaus 2 vs 3 | **3** (fabrikant schueco.com, MD) met conflictnotitie; fensternorm telt glasdichtingen vermoedelijk niet mee. |
| C5 | Gealan maxGlass 52/54 vs site-opgave 56/58 | **54 (STV)** aangehouden; nieuwe site-opgave als noot in de data. |
| C6 | Niet publiek gevonden: Schüco-stegbreedte, AWS-sponninghoogte, CS77-sponning/aanzichten, VEKA sightline-bron | **Unverified-vlag** in de data; geen waarden verzonnen. |
| C7 | ML8 glas-minimum 13 (site) vs 24 (ATG) | **24** voor validatie — conservatief; ATG is het keuringsdocument. |
| C8 | Niet onderzocht deze ronde | Hout-Uf/glasgewichten, Gealan S 8000, VEKA Softline 70, KUBUS, Aluplast, Deceuninck (behalve de bewezen-onmogelijke 70 mm), Weekamp 56×67, HEBO-kunststof, Schüco ASS 77 PD, CP155-aanzichten → vlag "unverified", niet aangepast. |

---

## 7. Terminologie en veld-semantiek (datamodel-hygiëne)

- KVT 12.01-richtingen: **h = sponninghoogte** (vlakrichting), **m = sponningbreedte** (bouwdiepterichting). PVC: Falzhöhe ≠ Glaseinstand (zie R8).
- De snapshot-keten klopt technisch (`sponning_diepte` ← `glazingRebate`, `sponning_hoogte` ← `sponning.depth`, via `ProfileSelector.svelte:25-28`) — na datacorrectie werkt de hele keten zonder codewijziging in dat pad.
- `KvtProfileType` A/A1/B/B1/C/D (`ofs-core/src/profile.rs:136-153`) is **handelsjargon** (66×110-profielhandel), geen KVT-normterm — KVT gebruikt A–H voor overspanningstabellen (katern 30). Gedocumenteerd, niet als norm gepresenteerd.

---

## 8. Bronnenlijst

**Normen/richtlijnen (online):** KVT-online (katernen 11, 12, 13, 14, 30, 61 + tekeningen 12.01/13.01/14.01), Uitleg NPR 3577 (Kenniscentrum Glas 2018), DIN EN 12608 (wanddikteklassen), DIN EN 12020-2:2001 (alu-toleranties), VMRG Kwaliteitseisen 2015.

**Fabrieks-/leveranciersdocumentatie:** TO-binnendetaillering profielcombinaties 2024; DTS-folder (KOMO 20676, raadsma.nl); VEKA Technische Produktinformationen Softline 82 MD (07/2020); Baltic Technikheft VEKA 82MD (10/2021); GEALAN Zubehör-/Zusatzprofile 03-2020 + gealan.de; STOLMA Gealan-systeemsneden; koemmerling.com; Schüco Systemeigenschaften AWS 75.SI/70.HI + brochure P3933 + LivIng-brochure P4151/07.20; reynaers.com (ML8/SL38) + ML8-bestektekst + ATG 3067; insulbar standard program; bakkerdehouthandel.nl, dekkerhout.nl, bew24-fenster.de, fensterblick.de, tmp-fenster.de, classicfenster.de, bouwwereld.nl.

**Lokaal archief:** `C:\Users\martijn\.pdf-toolkit-files\` (kvt-12.01/13.01/14.01.pdf, npr3577-uitleg.pdf, to-binnendetaillering-2024.pdf, dts-folder.pdf) en `...\ofs-research\` (schueco-75si-systemeigenschaften.pdf, masterline8-bestektekst.pdf, atg3067-masterline8.pdf, insulbar-standard-program-us.pdf, din-en-12020-2-tolerances.pdf, vmrg-kwaliteitseisen-2015.pdf).

**Codeverwijzingen (huidige waarden geverifieerd op regel):** `profiles/wood/kvt-standaard.json:12-53` (glazingRebate/sponning-reeks 17→24), `ofs-core/src/import/dxf_profile.rs:145` en `catalog.rs:324` (×0,36-heuristiek), `ofs-core/src/production.rs:11,540-548,608-642` (glasmaat/glaslat/sash-defaults), `ofs-core/src/profile.rs:12-30,43-61,100-108,136-186`, `ui/src/components/editor/KozijnCanvas.svelte:641,811,954-975` (glaslatlijn, sponninglijn, vleugelplaatsing), `ui/src/components/panels/ProfileSelector.svelte:25-28`.
