# Open Frame Studio vs. "Alles-in-één kozijnentool" — gap-analyse & MoSCoW

Bron: `Alles-in-een-kozijnentool.pdf` (marktverkenning + functioneel ontwerp, 17 juni 2026).
Vergelijking opgesteld 2026-06-17 tegen de OFS-codebase. Technische herbeoordeling: 2026-11-09.

## Kernconclusie
OFS bevat bouwstenen voor hout, kunststof, aluminium, hout-aluminium, vliesgevels en IFC. De aanwezigheid van modules bewijst nog geen uitwisselbaarheid of productiegeschiktheid. De IFC-export schrijft nu één kozijn, meerdere projectkozijnen, een bibliotheek met herbruikbare `IfcWindowType`-objecten of een projectbestand met afzonderlijke onderdelen. LOD300 beschrijft rechthoekige vaste beglazing met echte vakken, stijlen en glasdiktes; overige vormen, vrij ingedeelde vakken en beweegbare delen worden voor deze gedetailleerde export afgewezen. De project-, bibliotheek- en onderdelenexport weigeren vliesgevels en kozijncombinaties zolang die anders zouden ontbreken. De import leest maat- en onderdeelmetadata, vergelijkt vaste IDs en kan een eigen onderdelen-IFC herstellen als bewerkbaar **basisproject**. Hij bouwt nog geen bewerkbaar kozijn uit een willekeurig IFC-bestand op. LOD400, typen voor beweegbare delen/deuren, gevalideerde ILS-eigenschappen en uitwisseling met producentensoftware ontbreken nog. Daarom is marktvervanging nog geen aangetoonde eigenschap van het pakket.

## De 14 modules vs OFS
| # | Module | OFS | Toelichting |
|---|---|---|---|
| 1 | CRM & verkoop | 🟡 | `quotation`+PDF-offerte; geen CRM/handtekening/iDEAL |
| 2 | Configurator & calculatie | 🟡 | editor, `calculation`, `pricing`, multi-materiaal en catalogusimport; producentprijzen en bestelregels niet gevalideerd |
| 3 | CAD & 3D-tekenen | 🟡 | 2D (vrij/getoogd/rond/polygon) en 3D; niet alle vormen hebben een getrouwe IFC- of productie-uitvoer |
| 4 | Plausibiliteit & normcontrole | 🟡 | `validation` (+glaslat/vluchtraam), `ids`; botsing/statisch beperkt |
| 5 | BIM & IFC | 🟡 | IFC-export per kozijn, project, beperkte `IfcWindowType`-bibliotheek en afzonderlijke onderdelen. Metadata-import, basisreconstructie van eigen onderdelenexport en wijzigingsvergelijking; geen bewezen bewerkbare roundtrip met externe BIM- en producentensoftware of productiegeschikte profiel- en verbindingsdetails. |
| 6 | Compliance-engine | 🟡 | indicatieve `certification`, `performance_class`, `energy`/`thermal`; concept-DoP met invulvelden, geen gevalideerde FPC/ITT of volledige Bbl/KVT/CE-toets |
| 7 | Werkvoorbereiding, CNC & MES | 🟡 | zaag-/glaslijsten, optimalisatie en indicatieve zaagpreview; generieke G-code is geblokkeerd, geen gevalideerde machinepostprocessor of verbonden MES |
| 8 | Inkoop & bestellen | 🟡 | `procurement`; geen 1-klik online bestellen |
| 9 | PM & planning | 🟡 | `planning`+productieplan; transport/capaciteit beperkt |
| 10 | Montage-/opleveringsapp | ❌ | ofs-web werkvloer; geen mobiele inmeet-/oplevering-app |
| 11 | Nazorg/garantie | ❌ | alleen `bcf`; geen service-/garantiedossier |
| 12 | Duurzaamheid/circulariteit | 🟡 | indicatieve massa- en circulariteitsberekening; geen gevalideerde EPD/NMD/MPG/Madaster-koppeling |
| 13 | ERP/financieel/API | ❌🟡 | `ofs-cloud` = losgekoppelde scaffolding; geen facturatie/voorraad |
| 14 | Rapportage/analytics | ❌ | geen dashboards |

Materiaalmodel: hout, pvc, aluminium en hout-aluminium; staal ontbreekt. Vliesgevel heeft een eigen model maar wordt in de huidige IFC-projectuitvoer geweigerd. Doelgroepen zijn architect, tekenaar en producent; hun volledige uitwisselworkflow is nog niet gevalideerd.
Normen: de bestaande EN 14351-1-, KOMO/BRL-, RC/NEN 5096-, Uw- en BENG-controles zijn deels vereenvoudigd of indicatief. Een geslaagde modulecheck is geen normverklaring. Circulariteit wordt indicatief berekend zonder volledige marktdata.

## MoSCoW-roadmap voor OFS → alles-in-één
**Must** — het verbindende fundament + bestaande sterktes verdiepen
- `ofs-cloud` verbinden tot het ene datamodel ("één keer invoeren, overal gebruiken").
- IFC als uitwisselcontract uitwerken en extern valideren: kozijncombinatie met afzonderlijke onderdelen, uitbreiding van typebibliotheek naar beweegbare delen/deuren en bijzondere vormen, herimport met behoud van eigenschappen en proefuitwisseling met ontwerp- en producentensoftware.
- Compliance-diepte: concept-DoP tot onderbouwde verklaring uitwerken, Uw conform de toepasselijke berekeningsmethode en toetsgegevens aantonen.
- Productie-outputs compleet maken (zaaglijst incl. glaslat/vakvulling — deels gedaan deze sessie).

**Should**
- Circulariteit/materiaalpaspoort met gevalideerde EPD/NMD/MPG-bronnen en eventuele Madaster-koppeling.
- Inkoop: online bestellen per leverancier.
- Plausibiliteit: botsing- en statische voorcontrole.

**Could**
- CRM + digitale handtekening/betaling, montage-app, nazorg/garantie-dossier, analytics-dashboards.

**Won't (nu)** — koppelen i.p.v. bouwen
- Volledige ERP/boekhouding: via open API koppelen aan bestaand ERP.

## Belangrijke nuance
Veel modules bestaan als code maar zijn deels scaffolding; *breedte* is groot, *diepte/rijpheid* varieert. `ofs-cloud` is disconnected. De markeringen duiden technische dekking aan en zijn geen onafhankelijke toets op normconformiteit of geschiktheid voor calculatie en productie. Sessiewerk 2026-06-17 (vakvullingen, glaslatten, ramen, vluchtraam) versterkte modules 3/4/6/7.

## IFC-implementatiemijlpaal 2026-11-09
- LOD300 gebruikt de werkelijke rechthoekige vakindeling: vier kozijndelen, tussenstijlen/-regels en één glasvolume per vast beglaasd vak met de ingestelde glasdikte. Bronplaatsing X/Y/Z is in millimeters en de rotatie in graden om de verticale as; de IFC-geometrie en -plaatsing worden in meters geschreven.
- De export weigert ongeldige maten, bijzondere vormen, vrije vakindeling, uitbreidingen en niet-vast beglaasde vakken op LOD300. LOD400 blijft uitgeschakeld tot systeemprofielen en verbindingen echt zijn uitgewerkt. LOD200 blijft een vereenvoudigde doos voor rechthoekige kozijnen.
- Veronderstelde Uw, Uf, Psi, dagmaten, afwerking, buitentoepassing en niet gekozen beslag worden niet langer als vaststaand gegeven in de IFC gezet. Eigen eigenschappen dragen `OFS_`-namen in plaats van een onbewezen ILS-conformiteitsclaim; dit verandert de propertysetnamen voor afnemers van oude exports.
- Controle: 109 `ofs-core`-tests met standaardfuncties geslaagd; Svelte-build geslaagd. Een tweevaks voorbeeldbestand is door de onafhankelijke parser `web-ifc` geopend als één `IfcWindow` met zeven geometrie-instanties. Dit bewijst nog geen correcte import in elke BIM- of producententoepassing.

## IFC-projectexport 2026-11-09
- Een gedeelde exportkern maakt losse kozijnen en meerdere kozijnen uit het project. Projectnaam en -nummer staan in `IfcProject`; elk kozijn houdt zijn eigen GlobalId uit de kozijn-UUID, merk, serie en projectplaatsing. Eén ruimtelijke hiërarchie en één containmentrelatie verbinden de elementen. De volgorde van kozijnen in het project verandert de IFC-uitvoer niet.
- Voor de ruimtelijke hiërarchie zijn de locatie en het gebouw nog neutrale placeholders. Een nieuwe project-UUID blijft bij hernoemen behouden; oudere `.ofs`-projecten krijgen die UUID bij het openen en leggen hem bij de eerstvolgende opslag vast. Echte gebouw-/verdiepingsplaatsing is een vervolgstap.
- De STEP-tekenreeksen worden volgens ISO 10303-21:2002 gecodeerd, zodat ook Nederlandse namen met accenten en aanvullende Unicode-tekens worden uitgewisseld. `web-ifc` las de proefnamen `René 😀` en `K-É` correct terug.
- Controle: 113 `ofs-core`-tests geslaagd. Een voorbeeld met twee kozijnen opent in `web-ifc` als één project, één verdieping, twee ramen en twaalf geometrie-instanties. Dit is een parsercontrole, geen producentenacceptatie.

## IFC-typebibliotheek 2026-11-09
- De projectexport kan ook een bibliotheekbestand schrijven met een [IfcProjectLibrary](https://standards.buildingsmart.org/IFC/RELEASE/IFC4/FINAL/HTML/schema/ifckernel/lexical/ifcprojectlibrary.htm), één [IfcWindowType](https://standards.buildingsmart.org/IFC/RELEASE/IFC4/FINAL/HTML/schema/ifcsharedbldgelements/lexical/ifcwindowtype.htm) per kozijn en een [IfcRepresentationMap](https://standards.buildingsmart.org/IFC/RELEASE/IFC4/ADD1/HTML/schema/ifcgeometryresource/lexical/ifcrepresentationmap.htm) per type. Dezelfde LOD300-geometrie wordt gebruikt, maar zonder projectplaatsing of raaminstanties. De type-ID verschilt bewust van de ID van een geplaatste instantie.
- Het type bevat eigen maat-, materiaal- en glasgegevens. De bibliotheek is voorlopig beperkt tot rechthoekige vaste beglazing; een ongeschikt type laat de export zonder bestand stoppen.
- Controle: 115 `ofs-core`-tests geslaagd, desktopapp en UI bouwen. `web-ifc` leest twee raamtypen, twee geometriekaarten en twee declaraties; de geometriekaarten bevatten vijf en zeven onderdelen. Importeren en parametrisch aanpassen in ontwerp- of producentensoftware moet nog apart worden getoetst.

## IFC-onderdelenexport 2026-11-09
- Een aparte knop **Onderdelen IFC** schrijft een projectbestand met één [`IfcElementAssembly`](https://standards.buildingsmart.org/IFC/RELEASE/IFC4/FINAL/HTML/schema/ifcproductextension/lexical/ifcelementassembly.htm) per kozijn, afzonderlijke [`IfcMember`](https://standards.buildingsmart.org/IFC/RELEASE/IFC4/FINAL/HTML/schema/ifcsharedbldgelements/lexical/ifcmember.htm)-profielen en [`IfcPlate`](https://standards.buildingsmart.org/IFC/RELEASE/IFC4/FINAL/HTML/schema/ifcsharedbldgelements/lexical/ifcplate.htm)-ruiten. Onderdelen hebben eigen vaste `GlobalId`, merkteken/onderdeelrol, geometrie en materiaalassociatie; glasdelen dragen zichtmaat en dikte in `OFS_Onderdeel`. De samenstelling is ruimtelijk geplaatst, de onderdelen zijn eraan geaggregeerd. De exportvolgorde van kozijnen verandert de output niet.
- De [NBVT ILS Houten Kozijnen v2.0](https://nbvt.nl/wp-content/uploads/2023/07/KOZIJNEN_ILS-Houten-Kozijnen_versie-2.0_2023.pdf) vraagt bij voorkeur een kozijnsamenstelling met losse onderdelen. De huidige export volgt dat principe, maar gebruikt `IfcPlate` voor glas en eigen `OFS_`-eigenschappen. ILS-conformiteit is daarom niet geclaimd.
- De geometrie heeft vooralsnog rechthoekige profieldoorsneden en mist systeemspecifieke sponningen, glaslatten, verbindingen, beslag, bewerkingen en CNC-gegevens. Dit is een overdrachtsmodel om onderdelen te herkennen en vergelijken; het is geen LOD400 of bewezen import voor calculatie/productie. Alleen rechthoekige kozijnen met vaste beglazing worden toegestaan; een ongeschikt project laat de export zonder bestand stoppen.
- Controle: 117 `ofs-core`-tests geslaagd, desktopapp en UI bouwen. `web-ifc` las een proefproject met 2 samenstellingen, 9 profielen, 3 glasplaten, 4 materiaalrelaties en 12 geometrie-instanties. Import in echte ontwerp- en producentensoftware blijft een acceptatieproef.

## IFC-import, roundtrip en assen 2026-11-09
- De metadata-import leest IFC4 STEP-entiteiten, ge-escapete STEP-teksten, meter-/millimetereenheden, raam- en deurafmetingen, eigenschappen, `IfcElementAssembly` en bijbehorende `IfcMember`/`IfcPlate`-onderdelen. Ontbrekende maten blijven onbekend. Niet-ondersteunde eenheden en dubbele STEP-/GlobalIds geven een fout. **IFC Inspectie** wijzigt het project niet; het later toegevoegde **IFC Basis herstellen** is een aparte, beperkte herstelactie.
- IFC Roundtrip vergelijkt de stabiele GlobalId van hetzelfde kozijn en zijn onderdelen, inclusief naam, maat en eigenschappen. Een wijziging wordt niet meer gemaskeerd door vergelijken op naam of een verzonnen standaardmaat. Een externe IFC met andere IDs verschijnt als verwijderd/toegevoegd totdat een expliciete koppelregel is vastgesteld.
- De exportassen volgen nu de [`IfcWindow`-definitie](https://standards.buildingsmart.org/IFC/RELEASE/IFC4/FINAL/HTML/schema/ifcsharedbldgelements/lexical/ifcwindow.htm): lokaal X = breedte, Z = hoogte, Y = diepte. De onafhankelijke `web-ifc`-geometriecontrole meet in een proefproject 1,20 × 1,50 × 0,114 m en 0,90 × 1,20 × 0,114 m op de juiste projectposities. Ook de 12 losse onderdelen blijven zichtbaar.
- Controle op dit tussenpunt: 122 `ofs-core`-tests geslaagd; desktop- en UI-build slagen. Producentacceptatie en willekeurige IFC-geometrie vallen buiten deze metadata-importer; zie het latere basisherstel hieronder.

## Koppeling IFC-onderdelen aan calculatielijsten 2026-11-09
- Elk afzonderlijk profiel en glasdeel in de onderdelenexport krijgt nu `OFS_CalculatieIndicatief`. Het profiel verwijst naar zijn bestaande zaaglijst-ID met netto-/brutolengte, hoeken, omschrijving en aantal. Glas verwijst naar het glaslijst-ID met opgegeven type, indicatieve bestelmaat, dikte en aantal. Zo kan een afnemer de vaste IFC-onderdeel-ID koppelen aan een calculatieregel.
- Deze waarden zijn expliciet **indicatief**. De productiecode gebruikt bij ontbrekende profielsnapshots nog historische maatregels en kent geen gevalideerde systeemverbindingen of machine-instructies. Als een getekend onderdeel geen corresponderende of geldige calculatieregel heeft, stopt de onderdelenexport vóór het schrijven van een bestand.
- De metadata-import leest deze gekoppelde eigenschappen terug; een regressietest toetst de profiel- en glaslijst-ID's. 122 `ofs-core`-tests slagen en `web-ifc` blijft het proefbestand met 12 afzonderlijke geometrieën openen.

## Eerste pakketacceptatie 2026-11-09
- De opdrachtgever heeft **Revit, Tekla, SketchUp en hsbcad** als eerste doelpakketten aangewezen. Het concrete [acceptatieprotocol](ifc-acceptatie-revit-tekla-sketchup-hsbcad.md) onderscheidt zichtbare BIM-referentie, bewerkbare objecten en productiegeschikte data.
- In **Revit 2026.5** is het onderdelen-IFC als geladen link in *Manage Links* bevestigd. In de 3D-weergave waren beide kozijnen en hun onderdelen zichtbaar. Selectie toonde één gekoppeld model; dit is dus alleen een geslaagde visuele referentie-import. Maatvoering binnen Revit, eigenschappen, revisiegedrag, native bewerkbaarheid en productie zijn niet geaccepteerd.
- Tekla, SketchUp en hsbDesign zijn nog niet met echte applicaties beproefd. OFS levert op dit moment uitsluitend IFC4; voor sommige conversieroutes in Tekla is IFC2x3 een vereiste of voorkeur. Een geschikte producentenadapter en een veld-voor-veld vergelijking met NBVT ILS blijven open.

## Indicatieve producentenoverdracht 2026-11-09
- Een nieuw exportpakket schrijft het onderdelen-IFC samen met `overdracht.json` en `onderdelen.csv`. Elk profiel en elke ruit krijgt een expliciete kruisverwijzing tussen de daadwerkelijk geschreven IFC-GlobalId en de bestaande zaag- of glaslijst-ID. De exporter leest het IFC na schrijven terug, controleert de koppelingen en publiceert pas daarna een nieuwe map.
- Het overdrachtscontract heeft versie 1 en vermeldt nadrukkelijk dat maten en hoeveelheden indicatief zijn. Het is een neutrale mappingbasis, geen import in een fabrikantpakket en geen CNC-vrijgave. De herhaalbare IFC-acceptatie-fixture maakt ook dit pakket aan.
- Het bronmodel bewaart nu onderdeel-ID's voor rechthoekige gridvakken en -verdelingen. Bij `add_column` en `add_row` houden bestaande onderdelen hun IFC-GlobalId; nieuwe onderdelen krijgen een nieuw ID. Oude projecten zonder opgeslagen onderdeel-ID's behouden hun eerdere volgnummer-ID's, die bij de eerste structurele bewerking worden vastgelegd. Een duplicaat-ID wordt vóór export geweigerd. Losse merk- en zaaglijstnummers kunnen bij een splitsing nog wijzigen; de GlobalId is de primaire revisiesleutel.
- Controle na deze uitbreiding: 131 `ofs-core`-tests, desktopcontrole en UI-build geslaagd. Een voorbeeld met twee kozijnen levert 12 unieke gekoppelde IFC-onderdeel-ID's op (9 profielen en 3 ruiten); `web-ifc` leest het meegeleverde onderdelenmodel met dezelfde telling. De acceptatie-fixture bevat nu ook een IFC-revisie met een nieuw vak: 12 bestaande GlobalIds blijven, 2 nieuwe komen erbij. `web-ifc` leest die revisie als 2 samenstellingen, 10 profielen en 4 ruiten.

## IFC-basisherstel 2026-11-09
- De IFC-ribbon bevat **IFC Basis herstellen**. Alleen een eigen rechthoekig onderdelen-IFC met vaste beglazing kan vanuit een leeg project worden hersteld. Maten, grid, materiaal, kleuren, glaswaarden, projectplaatsing en onderdeel-GlobalIds komen terug; project-UUID is in nieuwe exports als `OFS_Project.ProjectUUID` opgenomen.
- Niet in het IFC aanwezige leverancierprofielen, glaslaagopbouw, verbindingen en machinebewerkingen blijven onbekend. Herstelde kozijnen dragen een vlag `recoveredFromIfc`; de desktop-CNC-commando's weigeren dan G-code en CNC-onderdelen. De basisreconstructie is niet gelijk aan een productiegeschikte roundtrip.
- Controle: 133 `ofs-core`-tests, desktopcontrole en UI-build geslaagd. De herstelde proef met twee kozijnen kan opnieuw als IFC worden geëxporteerd met dezelfde kozijn- en onderdeel-ID's; onvolledige OFS-eigenschappen en een gewoon raam-IFC worden voor bewerkbaar herstel geweigerd.

## Norm- en CNC-voorcontrole 2026-11-09

- De CE-, Bbl- en SKH/KOMO-voorcontroles geven geen automatisch geslaagde verklaring of normtoets meer voor ontbrekende producent- en projectgegevens. De oude vaste grens `Uw <= 1,65` wordt niet meer als algemene verbouweis gepresenteerd; de toepasselijke eis moet per bouwwerk worden bepaald.
- De CNC-weergave bevat alleen indicatieve zaagsneden. Vaste, niet-gevalideerde pen- en groefbewerkingen zijn verwijderd. De generieke G-code-postprocessor en het desktop-exportcommando weigeren machinebestanden totdat een producent- en machineconfiguratie is gevalideerd.
- Controle na deze wijzigingen: 138 `ofs-core`-tests geslaagd en desktopcode gecontroleerd. Dit is geen productievrijgave.
