[CmdletBinding(DefaultParameterSetName = 'Candidate')]
param(
    [Parameter(ParameterSetName = 'Candidate')][switch]$UnsignedDebugCandidate,
    [Parameter(Mandatory, ParameterSetName = 'Published')][string]$PublishedInstaller,
    [Parameter(Mandatory, ParameterSetName = 'Published')][string]$PublishedTag
)

$ErrorActionPreference = 'Stop'

function Assert-SignedFile([string]$Path) {
    $signature = Get-AuthenticodeSignature -LiteralPath $Path
    if ($signature.Status -ne 'Valid' -or -not $signature.TimeStamperCertificate) {
        throw "Expected a valid timestamped signature on $Path, got $($signature.Status)"
    }
    Write-Output "$Path signed by $($signature.SignerCertificate.Subject) ($($signature.SignerCertificate.Thumbprint)), timestamped by $($signature.TimeStamperCertificate.Subject)"
}

# Check the actual installed frontend. A hidden top-level HWND is not returned
# by Process.MainWindowHandle. Use the test process's isolated WebView2 profile
# and its temporary loopback debugging socket to read the rendered document.
function Test-InstalledEditor([string]$Installed, [string]$ProfileDir) {
    $listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
    $listener.Start()
    $port = $listener.LocalEndpoint.Port
    $listener.Stop()
    $settings = Join-Path ([Environment]::GetFolderPath('ApplicationData')) 'org.openaec.open-frame-studio/settings.json'
    $firstLaunch = -not (Test-Path -LiteralPath $settings)
    $start = [DateTime]::UtcNow
    $app = Start-Process -FilePath $Installed -PassThru -WindowStyle Hidden -Environment @{
        WEBVIEW2_USER_DATA_FOLDER = $ProfileDir
        WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$port --remote-debugging-address=127.0.0.1 --disable-gpu"
    }
    try {
        $deadline = [DateTime]::UtcNow.AddSeconds(60)
        $page = $null
        while (-not $page -and [DateTime]::UtcNow -lt $deadline) {
            if ($app.HasExited) { throw "Installed application exited during startup ($($app.ExitCode))" }
            try {
                $pages = Invoke-RestMethod -NoProxy -Uri "http://127.0.0.1:$port/json/list" -TimeoutSec 2
                $page = $pages | Where-Object { $_.type -eq 'page' -and $_.url -match '^https?://tauri\.localhost(:\d+)?(/|$)' } | Select-Object -First 1
            } catch { Write-Verbose "Waiting for WebView2" }
            if (-not $page) { Start-Sleep -Milliseconds 400 }
        }
        if (-not $page) {
            # Some hosted Windows images restrict the WebView debugging interface.
            # Check the first real IPC request from the installed frontend and
            # a renderer belonging to this exact application on that runner.
            $processes = @(Get-CimInstance Win32_Process -Filter "Name='msedgewebview2.exe'")
            $browsers = @($processes | Where-Object ParentProcessId -eq $app.Id)
            $renderers = @($processes | Where-Object {
                $_.ParentProcessId -in $browsers.ProcessId -and $_.CommandLine -match '--type=renderer'
            })
            if ($firstLaunch -and (Test-Path -LiteralPath $settings) -and $renderers.Count -gt 0) {
                $file = Get-Item -LiteralPath $settings
                $prefs = Get-Content -Raw -LiteralPath $file.FullName | ConvertFrom-Json
                if ($file.LastWriteTimeUtc -ge $start -and $prefs.locale -and $prefs.theme -and -not $app.HasExited) {
                    Write-Output "Installed WebView renderer $($renderers[0].ProcessId) initialized the frontend and loaded its native desktop preferences."
                    return
                }
            }
            $targets = @($pages | ForEach-Object { $_.url }) -join ', '
            throw "Installed WebView did not initialize. First launch: $firstLaunch. Preferences created: $(Test-Path -LiteralPath $settings). Browser processes: $($browsers.Count). Renderers: $($renderers.Count). Targets: $targets."
        }
        @'
const socket = new WebSocket(process.argv[2]);
const timer = setTimeout(() => { console.error("Installed editor did not render"); process.exit(1); }, 30000);
let poll;
let id = 0;
socket.onopen = () => {
  const expression = 'Boolean(window.__TAURI_INTERNALS__ && document.querySelector("#tab-editor") && document.querySelector("#workspace-panel"))';
  poll = setInterval(() => socket.send(JSON.stringify({ id: ++id, method: "Runtime.evaluate", params: { expression, returnByValue: true } })), 300);
};
socket.onerror = () => { console.error("Installed WebView connection failed"); process.exit(1); };
socket.onmessage = ({ data }) => {
  if (JSON.parse(data).result?.result?.value !== true) return;
  clearTimeout(timer);
  clearInterval(poll);
  console.log("Installed application rendered the editor and workspace panel");
  socket.close();
};
'@ | node --experimental-websocket --input-type=module - $page.webSocketDebuggerUrl
        if ($LASTEXITCODE -ne 0) { throw "Installed editor smoke failed" }
    } finally {
        if (-not $app.HasExited) { $app.Kill($true); $app.WaitForExit(10000) | Out-Null }
    }
}

if ($PublishedInstaller) {
    $installer = Get-Item -LiteralPath $PublishedInstaller
    Assert-SignedFile $installer.FullName | Tee-Object -FilePath $env:GITHUB_STEP_SUMMARY -Append
} else {
    $buildMode = if ($UnsignedDebugCandidate) { 'debug' } else { 'release' }
    $targetDir = Join-Path $PWD "target/$buildMode"
    $installers = @(Get-ChildItem -Path "$targetDir/bundle/nsis/*-setup.exe" -File)
    if ($installers.Count -ne 1) { throw "Expected one NSIS installer, got $($installers.Count)" }
    $installer = $installers[0]
    $expected = Join-Path $targetDir 'open-frame-studio.exe'
    if (-not $UnsignedDebugCandidate) {
        Assert-SignedFile $installer.FullName
        Assert-SignedFile $expected
        Assert-SignedFile (Join-Path $targetDir 'ofs_sdk.dll')
    }
}
$installDir = Join-Path $env:RUNNER_TEMP ("ofs-install-" + [guid]::NewGuid().ToString("N"))
# NSIS requires /D= to be the last switch and unquoted, including spaces.
$setup = Start-Process -FilePath $installer.FullName -ArgumentList "/S /D=$installDir" -PassThru -WindowStyle Hidden
if (-not $setup.WaitForExit(120000)) { $setup.Kill($true); throw "Installer timed out" }
if ($setup.ExitCode -ne 0) { throw "Installer returned $($setup.ExitCode)" }
$installed = Join-Path $installDir 'open-frame-studio.exe'
if ($PublishedInstaller) {
    $wanted = $PublishedTag.Substring(1)
    $product = (Get-Item -LiteralPath $installed).VersionInfo.ProductVersion
    if ($product -notin @($wanted, "$wanted.0")) { throw "Installed version $product differs from $PublishedTag" }
    $signature = Get-AuthenticodeSignature -LiteralPath $installed
    $hash = (Get-FileHash -LiteralPath $installed -Algorithm SHA256).Hash
    "Installed EXE: $product, Authenticode $($signature.Status), SHA-256 $hash" >> $env:GITHUB_STEP_SUMMARY
    if ($PublishedTag -eq 'v2026.48.4' -and $signature.Status -eq 'NotSigned') {
        Write-Output "::warning::Published v2026.48.4 contains an unsigned application EXE (issue #6)."
    } else {
        Assert-SignedFile $installed | Tee-Object -FilePath $env:GITHUB_STEP_SUMMARY -Append
    }
} else {
    if (-not $UnsignedDebugCandidate) { Assert-SignedFile $installed }
    $installedHash = (Get-FileHash -LiteralPath $installed -Algorithm SHA256).Hash
    if ($installedHash -ne (Get-FileHash -LiteralPath $expected -Algorithm SHA256).Hash) {
        throw "Installed binary differs from the application provided to the bundler"
    }
    Write-Output "Installed executable SHA-256 $installedHash matches the build"
}
Test-InstalledEditor $installed (Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) ("ofs-smoke-" + [guid]::NewGuid().ToString('N')))
if ($PublishedInstaller) { 'The installed application started and initialized its WebView2 frontend. The step log records the observed result.' >> $env:GITHUB_STEP_SUMMARY }
