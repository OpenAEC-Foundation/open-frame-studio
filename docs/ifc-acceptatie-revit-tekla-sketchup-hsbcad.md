# IFC-acceptatie: Revit, Tekla, SketchUp en hsbcad

Status: 9 november 2026, bronkandidaat v2026.46.0 voor een installer-prerelease. Deze vier pakketten zijn door de opdrachtgever als eerste doel genoemd. De hier genoemde proeven betreffen de huidige, beperkte IFC4-export van Open Frame Studio (OFS), niet een claim dat ieder kozijn of iedere productieketen al uitwisselbaar is.

## Uitwisselcontract en toepassingsgrens

| OFS-uitvoer | Bedoeld gebruik | Huidige inhoud | Nog niet aangetoond |
|---|---|---|---|
| Los kozijn / project, LOD300 | BIM-referentie en hoeveelheden | `IfcWindow` met buitenmaat, vaste vakken, stijlen, regels, glas en projectplaatsing | native parametrisch bewerken, vrije vormen, beweegbare delen, deuren |
| Onderdelenproject | Elementen herkennen voor werkvoorbereiding | `IfcElementAssembly` met afzonderlijke `IfcMember`-profielen en `IfcPlate`-ruiten, materiaal, vaste GlobalIds, indicatieve zaag-/glaslijstwaarden | systeemsponningen, beslag, verbindingen, geverifieerde zaagwaarden, CNC |
| Typebibliotheek | Hergebruik van typen | `IfcProjectLibrary`, `IfcWindowType` en representatiekaarten | directe opname als native familie/component in ieder doelpakket |

De export is IFC4 STEP met **meter als IFC-lengte-eenheid**. OFS rekent intern in millimeters en geeft maatvelden met suffix `_mm` ook in millimeters door. Een kozijn is lokaal X = breedte, Y = diepte en Z = hoogte, conform de [buildingSMART-definitie van IfcWindow](https://standards.buildingsmart.org/IFC/RELEASE/IFC4/FINAL/HTML/schema/ifcsharedbldgelements/lexical/ifcwindow.htm). Alle OFS-eigenschappen hebben eigen namen. De export claimt geen buildingSMART-certificering, NBVT-ILS-conformiteit of gevalideerde productiedata.

## Doelpakketten

| Pakket | Gedocumenteerde route | OFS-acceptatie die nu nodig is | Stand |
|---|---|---|---|
| **Revit 2026** | IFC4 als gekoppeld model; [Autodesk: IFC-bestanden koppelen](https://help.autodesk.com/cloudhelp/2026/ENU/Revit-DocumentPresent/files/GUID-6708CFD6-0AD7-461F-ADE8-6527423EC895.htm) | Project- en onderdelenmodel zichtbaar op juiste plek; merkteken, maat en eigenschappenset beschikbaar; link kan na revisie worden bijgewerkt. Native Revit-familie apart toetsen. | **Deelproef geslaagd:** `parts.ifc` staat met geladen status in *Manage Links*. Twee afzonderlijke kozijnen en hun vak-/profiellijnen waren zichtbaar in de 3D-weergave. Geselecteerd object was één RVT-link. Parameteroverdracht, revisie en native bewerkbaarheid zijn niet getoetst. |
| **Tekla Structures 2026** | IFC-referentiemodel; [Trimble: IFC import](https://support.tekla.com/doc/tekla-structures/2026/int_ifc_import). Omzetting van geschikte lineaire objecten naar native onderdelen is mogelijk, maar hangt af van profiel en eenheid; [Trimble: conversiebeheer](https://support.tekla.com/doc/tekla-structures/2026/int_conversion_management). | Referentiepositie en maten; `IfcMember`/`IfcPlate` als afzonderlijke herkenbare onderdelen; selectieve conversie met behoud van ID/merk; profieldoorsnede en materiaaltoewijzing; terugexport vergelijken. | **Nog niet getest** in Tekla. Rechthoekige OFS-profielen zijn geen leveranciersprofielen. Tekla adviseert in zijn documentatie IFC2x3 voor bepaalde conversieroutes; OFS levert nu alleen IFC4. |
| **SketchUp** | [SketchUp IFC-import](https://help.sketchup.com/en/importing-and-exporting-ifc-files) ondersteunt IFC2x3 en IFC4 in de gedocumenteerde actuele versies. | Elk kozijn op projectpositie, juiste maat/oriëntatie, groepshiërarchie en objectklassen; onderdelen en attribuutnamen terugvinden; opnieuw exporteren en vergelijken. | **Nog niet getest** in SketchUp. Import als zichtbare SketchUp-geometrie zegt op zichzelf niets over maakbaarheid. |
| **hsbcad / hsbDesign** | [hsbcad IFC Import Manager](https://www.hsbcad.academy/toolsets/ifc-import-export) voor hsbDesign binnen Revit/AutoCAD. | Vaststellen welke IFC-klassen, parameters, profielen en materiaalkoppelingen de gekozen hsbDesign-versie kan omzetten; proefberekening en werkvoorbereiding op één echt systeemkozijn; afwijkingen terugmelden. | **Nog niet getest** met hsbDesign. hsbcad richt zich mede op prefab houtbouw; een werkende kozijncalculatie- en CNC-import is hiermee niet bewezen. |

### Uitgevoerde Revit-proef

- Omgeving: Autodesk Revit 2026.5, nieuw tijdelijk metrisch multidisciplinair project, `parts.ifc` uit het OFS-proefproject met een kozijn van 1200 × 1500 mm en een van 900 × 1200 mm. De IFC werd met **Link IFC**, *Auto - IFC Origin to Internal Origin* gekoppeld.
- Waarneming: *Manage Links* toonde `parts.ifc` onder IFC met geladen vinkje. In de standaard 3D-weergave waren twee gescheiden kozijnen met stijlen en glaspanelen zichtbaar; selecteren gaf **RVT Links: Linked Revit Model: parts.ifc: 1**. Het tijdelijke Revit-project is zonder opslaan gesloten.
- Reikwijdte: dit is een geslaagde **visuele referentie-import** voor de onderdelenvariant. Schaal is in deze proef niet met Revit-maatvoering nagemeten; psets, vaste GlobalIds binnen Revit, typebibliotheek, links updaten en parametrische reconstructie zijn nog open.
- De onafhankelijke `web-ifc`-controle van dezelfde exportvorm las eerder twee samenstellingen, negen profielen, drie glasplaten en twaalf geometrie-instanties. Dat is een parsercontrole, geen acceptatie door een producent.

## Acceptatieprotocol voor de vier pakketten

Maak de vier bronbestanden met de herhaalbare fixture in [`ofs-core/examples/ifc_acceptance_fixture.rs`](../ofs-core/examples/ifc_acceptance_fixture.rs):

```powershell
cargo run -p ofs-core --example ifc_acceptance_fixture -- "C:\pad\naar\acceptatiebestanden"
```

De uitvoer heet `01-los-kozijn.ifc`, `02-project.ifc`, `03-onderdelen.ifc` en `04-typebibliotheek.ifc`. `05-onderdelen-revisie.ifc` bevat hetzelfde project met een extra kolom in het eerste kozijn. De generator maakt tevens een aparte `OFS-overdracht-…`-map met het onderdelen-IFC en de hieronder beschreven producentenoverdracht. Gebruik per pakket dezelfde gegenereerde set en leg de pakketversie, importinstellingen, screenshots, objecttelling, gemeten maten, attribuutmapping en afwijkingen vast. Bewaar de set bij het testrapport: een nieuwe fixture-uitvoering maakt nieuwe project-/kozijn-UUID's. Binnen één uitvoering zijn de kozijn- en onderdeel-GlobalIds in de verschillende bestanden stabiel. De opgeslagen `partIds` houden de identiteit van bestaande grid-onderdelen vast bij een splitsing via `add_column` of `add_row`; nieuwe onderdelen krijgen nieuwe IDs. Bestaande `.ofs`-bestanden zonder `partIds` behouden hun eerdere IFC-ID's en leggen deze vast zodra de grid wordt bewerkt. Structurele bewerkingen buiten deze modeloperaties vragen een aparte identiteitstoets.

## Producentenoverdracht (indicatief)

De knop **Producentenoverdracht** maakt in een gekozen map een nieuwe submap met `onderdelen.ifc`, `overdracht.json` en `onderdelen.csv`. De JSON heeft `schemaVersion: 1`, project-ID/-nummer, status en één record per IFC-profiel of ruit. Elk record bevat de werkelijke `partGlobalId` uit het zojuist geschreven IFC, de `assemblyGlobalId`, het kozijn-ID, onderdeelmerk, rol, materiaal, lijst-ID, aantallen en de beschikbare indicatieve zaag- of glasmaten. De CSV bevat dezelfde kernvelden als puntkomma-gescheiden UTF-8 voor een eerste mappingproef; tekstvelden zijn gequote en mogelijke spreadsheetformules worden in de CSV als tekst gemarkeerd. De JSON bewaart de oorspronkelijke waarde.

De export leest zijn eigen IFC terug en weigert een onvolledige koppeling tussen IFC-onderdeel en zaag-/glaslijst. Hij schrijft eerst naar een tijdelijke submap en maakt de definitieve map pas na een geslaagde controle zichtbaar. Dit is **geen** importadapter voor een specifiek producentenpakket en **geen** vrijgave voor machines of bestellingen. Een producent moet zijn profiel-, verbinding-, glas- en bewerkingsregels nog op deze velden mappen en de uitkomst vergelijken met een gecontroleerde referentie.

Bij de fixture-revisie bevat het oorspronkelijke onderdelen-IFC 12 onderdelen en de revisie 14. Alle 12 bestaande GlobalIds komen terug; twee nieuwe onderdelen krijgen een nieuwe ID. Deze controle betreft de modelbewerking `add_column` en zegt nog niets over objectmatching na bewerking door een extern pakket.

## Bewerkbaar basisproject uit eigen IFC

**IFC Basis herstellen** leest uitsluitend een door OFS geschreven onderdelen-IFC met rechthoekige vakken en vaste beglazing. De actie werkt vanuit een nieuw leeg project. De kern reconstrueert projectnaam/-nummer en, bij nieuwe exports, project-UUID; per kozijn de UUID, naam, merk, buitenmaten, kozijnbreedte/-diepte, materiaal, kleuren, kolom- en rijmaten, glastype/-dikte/Ug, projectpositie/rotatie en de bestaande IFC-onderdeel-ID's. Een test exporteert twee kozijnen, herstelt ze als bewerkbaar `.ofs`-model en exporteert het herstelde model opnieuw met dezelfde onderdeel-ID's.

Dit is een **basisreconstructie**. Het IFC bevat geen betrouwbare leverancier-ID voor de dorpel en tussenprofielen, geen profielsnapshot, glaslaagopbouw, verbindingen, beslag of machinebewerkingen. De herstelroute verzint die gegevens niet: onbekende tussenprofielen krijgen een herkenbare controletekst en de glaslaagopbouw blijft onbekend. CNC-export vanuit een hersteld basisproject is geblokkeerd. Willekeurige IFC-bestanden, deuren, beweegbare delen, vliesgevels en afwijkende vakvormen worden niet als bewerkbaar OFS-project geaccepteerd. **IFC Inspectie** blijft beschikbaar om zulke bestanden alleen te lezen.

| Proef | Concrete slaagvoorwaarde |
|---|---|
| A. Los kozijn LOD300 | IFC opent zonder fout; één raam op juiste locatie; gemeten buitenmaat en diepte binnen 1 mm van bron; vakindeling zichtbaar. |
| B. Project LOD300 | Twee ramen blijven afzonderlijk selecteerbaar; merktekens en plaatsingen kloppen; geen stille weglatingen. |
| C. Onderdelen | Twee samenstellingen, negen profielobjecten en drie glasobjecten; materiaal, rol, merk, GlobalId en indicatieve calculatiereferentie per onderdeel terugvindbaar. |
| D. Typebibliotheek | Twee verschillende typen worden geïmporteerd of als bibliotheek herkend; als het pakket dit niet ondersteunt, expliciet vastleggen en via geplaatste IFC-instanties testen. |
| E. Revisie | `03-onderdelen.ifc` en `05-onderdelen-revisie.ifc` vergelijken: bestaand kozijn, bestaande stijlen en ruiten behouden hun GlobalId; nieuwe stijl en ruit krijgen nieuwe IDs. Merk- en lijstnummers kunnen opschuiven en zijn geen objectidentiteit. |
| F. Producentketen | Leveranciersprofiel, verbinding, glasopbouw en bewerking via een afgesproken mapping omzetten; calculatie-/zaag-/CNC-uitvoer vergelijken met handmatig gecontroleerde referentie. Geen automatische productie zonder die proef. |

Een **geslaagde BIM-referentie-import** is een andere acceptatiestatus dan **native bewerkbaar** of **productiegereed**. Per doelpakket moet de status afzonderlijk worden vastgesteld. De huidige Revit-proef bereikt uitsluitend de eerste status voor de onderdelenvariant.

## Gegevens die een producent moet aanleveren

Voor een werkende calculatie-/productiebrug zijn minimaal nodig: één representatief kozijnproject inclusief bron-IFC en verwachte output; profielcatalogus met doorsneden en systeemcodes; verbindingen, sponningen, glaslatten en toleranties; zaag- en bewerkingsregels; glasbestelregels; prijs-/artikelcodes; machine- of software-importformaat en versie; testresultaat van de beoogde producent. Zonder deze gegevens kan OFS wel onderdelen en indicatieve lijsten uitwisselen, maar geen betrouwbare machine-instructie of prijs claimen.

De [NBVT ILS Houten Kozijnen v2.0](https://nbvt.nl/wp-content/uploads/2023/07/KOZIJNEN_ILS-Houten-Kozijnen_versie-2.0_2023.pdf) is een belangrijke Nederlandse toets voor het houten deel van dit contract. Een aparte veld-voor-veld toets is nodig voordat een ILS-conforme export kan worden aangeboden.
