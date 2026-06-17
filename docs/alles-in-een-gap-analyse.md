# Open Frame Studio vs. "Alles-in-één kozijnentool" — gap-analyse & MoSCoW

Bron: `Alles-in-een-kozijnentool.pdf` (marktverkenning + functioneel ontwerp, 17 juni 2026).
Vergelijking opgesteld 2026-06-17 tegen de OFS-codebase.

## Kernconclusie
De PDF stelt dat geen enkel marktpakket multi-materiaal (hout+pvc+alu) **én** aluminium-gevelbouw **én** een echte BIM-roundtrip combineert. **OFS dekt die drie hiaten architectonisch al**: profielen hout/pvc/alu/hout-alu, een `vliesgevel`-module (curtain wall) en IFC in+uit met `ifc_roundtrip`. De gaten zitten aan de **bedrijfs-/levenscyclus-randen** en bij **circulariteit**, niet in de kerntekentool.

## De 14 modules vs OFS
| # | Module | OFS | Toelichting |
|---|---|---|---|
| 1 | CRM & verkoop | 🟡 | `quotation`+PDF-offerte; geen CRM/handtekening/iDEAL |
| 2 | Configurator & calculatie | ✅ | editor, `calculation`, `pricing`, multi-materiaal, `catalog`-import |
| 3 | CAD & 3D-tekenen | ✅ | 2D (vrij/getoogd/rond/polygon), 3D (glTF), `edge`, Blender/Bonsai |
| 4 | Plausibiliteit & normcontrole | 🟡 | `validation` (+glaslat/vluchtraam), `ids`; botsing/statisch beperkt |
| 5 | BIM & IFC | ✅ | IFC in+uit, `ifc_roundtrip`, `bcf`, `ids` — dekt markthiaat |
| 6 | Compliance-engine | 🟡 | `certification` (EN 14351-1-checks), `performance_class` (Rw indicatief), `energy`/`thermal` (Uw/BENG/g), RC1–6; geen DoP-generatie/FPC/ITT |
| 7 | Werkvoorbereiding, CNC & MES | ✅ | `production`, `optimization`, `cnc`, labels, ofs-web werkvloer; MES beperkt |
| 8 | Inkoop & bestellen | 🟡 | `procurement`; geen 1-klik online bestellen |
| 9 | PM & planning | 🟡 | `planning`+productieplan; transport/capaciteit beperkt |
| 10 | Montage-/opleveringsapp | ❌ | ofs-web werkvloer; geen mobiele inmeet-/oplevering-app |
| 11 | Nazorg/garantie | ❌ | alleen `bcf`; geen service-/garantiedossier |
| 12 | Duurzaamheid/circulariteit | ❌ | `energy`/BENG deels; geen EPD/NMD/MPG/Madaster/materiaalpaspoort |
| 13 | ERP/financieel/API | ❌🟡 | `ofs-cloud` = losgekoppelde scaffolding; geen facturatie/voorraad |
| 14 | Rapportage/analytics | ❌ | geen dashboards |

Materiaal: hout/pvc/alu/hout-alu ✓ + `vliesgevel` ✓. Gebruikers: productie/tekenaars/architecten ✓.
Normen: EN 14351-1 ✓(checks), EN ISO 10077-2 🟡(vereenvoudigd), KOMO/BRL ✓(checks), RC/NEN 5096 ✓, Rw 🟡(indicatief), BENG ✓. Circulariteit ❌.

## MoSCoW-roadmap voor OFS → alles-in-één
**Must** — het verbindende fundament + bestaande sterktes verdiepen
- `ofs-cloud` verbinden tot het ene datamodel ("één keer invoeren, overal gebruiken").
- Compliance-diepte: DoP-generatie + Uw conform EN ISO 10077-2 (nu vereenvoudigd).
- Productie-outputs compleet maken (zaaglijst incl. glaslat/vakvulling — deels gedaan deze sessie).

**Should**
- Circulariteit/materiaalpaspoort (EPD/NMD/MPG, Madaster) — de marktbrede differentiator.
- Inkoop: online bestellen per leverancier.
- Plausibiliteit: botsing- en statische voorcontrole.

**Could**
- CRM + digitale handtekening/betaling, montage-app, nazorg/garantie-dossier, analytics-dashboards.

**Won't (nu)** — koppelen i.p.v. bouwen
- Volledige ERP/boekhouding: via open API koppelen aan bestaand ERP.

## Belangrijke nuance
Veel modules bestaan als code maar zijn deels scaffolding; *breedte* is groot, *diepte/rijpheid* varieert. `ofs-cloud` is disconnected. Sessiewerk 2026-06-17 (vakvullingen, glaslatten, ramen, vluchtraam) versterkt modules 3/4/6/7.
