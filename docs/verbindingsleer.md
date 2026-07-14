# Verbindingsleer kozijnen — beslistabel en tekenregels

Bronnen-gedreven naslagwerk (onderzoek 2026-07-14) voor hoe Open Frame Studio
verbindingen bepaalt, tekent (2D) en bouwt (3D/productie). Zelfde opzet als
`profielmaten-onderzoek.md`: de domein-expert corrigeert, zijn correcties gaan
boven elke bron.

Zekerheid: ●● = meerdere onafhankelijke bronnen, ● = één goede bron,
○ = afgeleid/aanname (expliciet checken).

## 1. Beslistabel: materiaal × ontmoeting → verbinding

| Materiaal | Ontmoeting | Verbinding | Wie loopt door | 2D-aanzicht | 3D |
|---|---|---|---|---|---|
| Hout kozijn | stijl × boven-/onderdorpel | dubbele pen-en-gat (of 2× deuvel) ●● | **dorpel** ●● | haakse naad; dorpelrandlijn loopt door | dorpel volle breedte, stijl gekort (onderdorpel: stijlkop contraprofiel over afwaterend vlak) |
| Hout kozijn | tussenstijl × dorpel / kalf × stijl | pen-en-gat + contraprofiel ●● | dorpel resp. stijl | doorlopend deel ononderbroken, T-deel stuit haaks | T-deel gekort, kop met contourfrees |
| Hout vleugel (raam/deur) | hoek | slis (4/5/6-vlaks) of deuvel Ø14 ●● | **stijl** ●● | verticale lijnen doorlopend | vleugelstijl volle hoogte |
| Hout | — | **zuiver verstek is VERBODEN** (alleen kops-op-kops mag niet, KVT 15) ●● | — | nooit een 45°-diagonaal tekenen bij hout | — |
| PVC | hoek (kader én vleugel) | 45° verstek, gelast ●● | geen | diagonale lasnaad binnenhoek→buitenhoek | beide leden 45° afgekort |
| PVC | T (tussenstijl/-dorpel) | gecontramald + geschroefd/aangelast, nooit verstek ●● | kader | rechte stuiknaad | T-lid recht gekort, kop volgt kadercontour |
| Alu | hoek | 45° verstek + hoekspieën geperst/gelijmd ●● | geen | diagonale hoeklijn | beide leden 45°, spie onzichtbaar |
| Alu | T | recht geraveld + T-verbinders, dichting gevulkaniseerd ● (Reynaers) | kader | rechte stuiknaad | T-lid recht gekort met contourkop |

KVT-maten (katern 15): pen ≥25 lang, ≥25 breed, 12–20 dik; gat max +0,1 mm;
houtbreedte <114 → enkele pen-gat of dubbele deuvel, ≥114 → dubbel;
deuvels kozijn ≥2× Ø12 (raam Ø14); kruising tussenstijl×tussendorpel: pen ≥12,5
met 1–2 mm vrije ruimte. ●●

**LET OP (wijziging t.o.v. v0.5.2):** v0.5.2 tekende hout stijl-doorlopend.
Volgens KVT/vakonderwijs (industrieelproducerenmethout.nl, klusidee) is dat bij
KOZIJNEN omgekeerd: dorpels lopen door, stijlen staan ertussen (op de
onderdorpel met contraprofiel). Stijl-doorlopend geldt wél voor stelkozijnen
(KVT 27.3.5) en voor vleugels. In deze release is hout-kozijn teruggezet naar
**dorpel-doorlopend** — expert: graag bevestigen. ○ (bronnen spreken elkaar
deels tegen; vakonderwijs-bron is het duidelijkst)

## 2. Tekenconventies 2D (NL)

- Hoofdaanzicht = **buitenaanzicht** (VKG 1.8); horizontale doorsnede: buiten
  onder; verticale doorsnede: buiten links. ●●
- Draaisymbool: brede zijde driehoek = scharnierzijde, punt = kruk-/slotzijde. ●●
- Doorgetrokken lijn = naar buiten draaiend; streep-/stippellijn = naar binnen
  draaiend; pijl = schuivend deel. ●● (VKG 1.8)
- Verstek → diagonale naadlijn binnenhoek→buitenhoek; stuik/pen-gat → randlijn
  doorlopend deel ononderbroken, stuitend deel eindigt met haakse naadlijn. ●
- Duitsland/Polen tekenen van binnenuit (DIN links/rechts) — aanzichtzijde moet
  op termijn een documentinstelling worden. ●●

## 3. Interactiepatronen uit commerciële software (referentie voor de editor)

- **WH Okna**: tool kiezen → klik in vak plaatst stijl/dorpel op klikpositie;
  klik op maatgetal → typ waarde; contextmenu "uitlijnen op". (handleiding p. 44-61)
- **LogiKal**: raster → maten → vulling per vak (auto-doorschakelen, Ctrl+klik
  voor meerdere vakken); verbindingen per categorie (hoeken kader/deur/raam,
  T-verbindingen) in positie-eigenschappen, niet per stuk; 3D is een view op
  hetzelfde model (View → 3D). (help.orgadata.com)
- **Klaes 3D**: elke wijziging herrekent zaaglengtes/hoeken direct; doorsnedes
  worden uit het model gegenereerd, nooit apart getekend.
- **MatrixKozijn**: merkenlijst → kozijnvorm → sjabloon → knooppunten (per stuk
  wijzigbaar: blauw hoekje = hoekverbinding, groen streepje = tussenverbinding)
  → vakken (vak-voorkeuzes bibliotheek) → randen.
- **Toelevering Online** (minimale alfa-lat): (1) materiaal+buitenmaat,
  (2) stijlen/dorpels toevoegen/verplaatsen/verwijderen, (3) vulling per vak met
  draairichting, (4) eigenschappen — live prijs.

## 4. Consequenties voor Open Frame Studio

1. Eén model (layout-boom) → 2D en 3D zijn beide pure projecties; nooit een
   tweede geometrie bijhouden. Elke mutatie ververst beide (fix: setKozijnLayout
   ververst nu de geometrie-payload).
2. Verbindingen per ontmoetingstype automatisch toewijzen (beslistabel §1) met
   defaults per materiaal en override per knooppunt (Matrix-model, R-VERB-1);
   validatie tegen KVT-maten in de plausibiliteitsmodule (R-VERB-2).
3. Zaaglijst: verstek = meten over de lange punt (netmaat korte zijde apart
   vermelden); tussenstijl/tussendorpel nooit beide doorlopend op een kruising.
4. Vleugel = geneste knoop in het vak met eigen (afgeleide) maatketen — WH Okna's
   lagenmodel; vast glas in kozijn ≠ vaste vleugel.
5. Draaisymbolen conform §2 in élk pad (matrix, layout, proto) — zelfde
   tekenfunctie, geen drie implementaties.

## 5. Bronnen

- KVT katern 15 (verbindingen), 27.3.5 (stelkozijnen), 61.1 (houtafmetingen) — kvt-online.nl
- VKG Keurmerk kwaliteitseisen 1.8 (aanduidingen op tekeningen) — vkgkeurmerk.nl
- industrieelproducerenmethout.nl — kozijnverbindingen, contraprofiel/contramallen
- kunststofkozijn.nl kennisbank — hoekverbindingen PVC (verstek/HVL)
- Reynaers techblog — verstek + ravelen T-stijlen alu
- VEKA verwerkingsdocumentatie — schuine/haakse lassen
- help.orgadata.com (LogiKal), klaes.de, whokna.com.pl handleiding 2023,
  knowledge-base.matrix-software.com, toeleveringonline.nl
