# Open Frame Studio · distributie en integratie

De distributiecontrole hoort bij [v2026.48.4](https://github.com/OpenAEC-Foundation/open-frame-studio/releases/tag/v2026.48.4), gebouwd uit [0b2b34e](https://github.com/OpenAEC-Foundation/open-frame-studio/commit/0b2b34e3149fa298eb03f8232b4c26e9f85fc8ee). De SDK, nieuwe API en integratietests in deze branch zijn vervolgwerk voor [issue #6](https://github.com/OpenAEC-Foundation/open-frame-studio/issues/6). De tag bevat de eerdere releasecode.

## Desktop en browser

| Platform | Bewijs voor v2026.48.4 | Vervolgcontrole |
| --- | --- | --- |
| Windows x64 | De gedownloade [systeeminstaller](https://github.com/OpenAEC-Foundation/open-frame-studio/releases/download/v2026.48.4/Open.Frame.Studio_2026.48.4_x64-setup.exe) en [gebruikersinstaller](https://github.com/OpenAEC-Foundation/open-frame-studio/releases/download/v2026.48.4/Open.Frame.Studio_2026.48.4_x64_user-setup.exe) geven `Valid` met Impertio Studio B.V. en een Microsoft RSA-tijdstempel. | Uit de gebruikersinstaller uitgepakt geeft `open-frame-studio.exe` de status `NotSigned`. De aangepaste workflow tekent de EXE vóór het verpakken en controleert de geïnstalleerde kopie. Het bewijs van die wijziging hoort bij de eerstvolgende release uit deze bron. |
| macOS Intel en Apple Silicon | Het platformdoel van de [installerworkflow](https://github.com/OpenAEC-Foundation/open-frame-studio/actions/runs/45397718969) is een universele DMG. | Een praktijkproef op beide architecturen en een Developer ID-/notarisatiecontrole blijven open. |
| Linux x86-64 | Dezelfde workflow bouwt DEB, RPM en AppImage. De [containerworkflow](../.github/workflows/docker-image.yml) gebruikt de DEB van de bijbehorende tag. | Een zelfstandig geïnstalleerde AppImage en Linux-pakkethandtekening hebben eigen acceptatie nodig. De container heeft een afzonderlijke Cosign-identiteit. |
| Browser/WASM | De [sitebuild](https://github.com/OpenAEC-Foundation/open-frame-studio/actions/runs/45397666213) op de releasecommit is geslaagd. | Workspace- en platformacceptatie staan bij [#9](https://github.com/OpenAEC-Foundation/open-frame-studio/issues/9). |

De SHA-256 van de Windows-systeeminstaller is `c2d7fcf72d67a8dd8562354450409766a686c203c46584e6b131bdb1dfb4a8ac`. Voor de gebruikersinstaller is dit `7dccd9f1f1f06d164485a3c95a0d0c37cd896f87cd87887388690c680c84d084`. De uitgepakte hoofd-EXE uit de gebruikersinstaller heeft SHA-256 `445bc72db7a3c9a2ff7ec889fea6904d72fe06536445f493646c3bed95e004b0`.

```powershell
$signature = Get-AuthenticodeSignature -LiteralPath $bestand
$signature | Select-Object Status, SignerCertificate, TimeStamperCertificate
Get-FileHash -LiteralPath $bestand -Algorithm SHA256
```

## Integratie in de vervolgbron

| Onderdeel | Implementatie en controle | Acceptatie buiten deze bron |
| --- | --- | --- |
| Rust en native SDK | De [C-ABI](../ofs-sdk/include/ofs_sdk.h) levert een indicatieve kostprijs voor één vastbeglaasd houten kozijn. De [externe Python-consument](../ofs-sdk/example.py) laadt de echte DLL of SO en controleert geldige en ongeldige maten. Op Windows gaf 1000 × 1200 mm € 280,42 en 2,61 arbeidsuur. De [SDK-handleiding](../ofs-sdk/README.md) beschrijft versiebeleid, geheugen, foutcodes en de bestaande licentievermeldingen. | Release-ZIP voor Windows x64 en Linux x64, met een apart getekende DLL voor Windows, vanaf de eerstvolgende release met deze bron. |
| HTTP-API en AWS | De [headless service](../ofs-cloud/src/service.rs) biedt health en een berekening met een bearer-token. De service is stateless. De projectproef met opslag in geheugen zit in een aparte startmodus. Het [Fargate-runbook](../packaging/aws/README.md) beschrijft configuratie en de rooktest. | Een geslaagde AWS-deployment met smoke-resultaat vereist een aangewezen AWS-account en runtime. |
| MCP en gebruikerswerkplek | De [stdio-server](../packaging/mcp/stdio.mjs) en het [interne containerendpoint](../packaging/docker/mcp-http.mjs) delen de frame-tools met de browser. Claude Code 2.1.331 maakte verbinding op Windows x64. De officiële Agent SDK vond vier tools en de MCP-client berekende 1,8 m² uit het voorbeeldbestand. [Clientconfiguratie en resultaten](../packaging/mcp/README.md) | De ingelogde praktijkproef op de 3BM-werkplek en de verbinding vanuit de Claude Desktop-GUI blijven open. |
| UI | De [Blueprint-tokens](../ui/src/styles/tokens.css) komen uit [Blueprint 2eb8a95](https://github.com/OpenAEC-Foundation/OpenAEC-Blueprint/tree/2eb8a958bbfe864dea81237e8890cf2405620b7d/web). Svelte tekent het eigen lint, de panelen, titelbalk en statusbalk. De [themastore](../ui/src/stores/ui.js) koppelt `blue` aan `blueprint` en `highContrast` aan `high-contrast-dark`. | De [Style Repo op dfdcd41](https://github.com/OpenAEC-Foundation/OpenAEC-style-book/tree/dfdcd415b26604081b1699560621d3e9ae7ea6bf/packages) levert onder meer `@openaec/design-tokens`, shell, ribbon en UI-primitives. Open Frame Studio distribueert de Blueprint CSS en implementeert zijn eigen Svelte-componenten. |

De desktopversie heeft handmatige updates. Voor updaterhandtekeningen, macOS-notarisatie en het bestaande verschil tussen de licentietekst en de pakketmetadata staan de eerdere bevindingen in de [releasecontrole](release-readiness.md).
