param([switch]$UnsignedDebugCandidate)

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
    $app = Start-Process -FilePath $Installed -PassThru -WindowStyle Hidden -Environment @{
        WEBVIEW2_USER_DATA_FOLDER = $ProfileDir
        WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=0 --remote-debugging-address=127.0.0.1'
    }
    try {
        $portFile = Join-Path $ProfileDir 'EBWebView/DevToolsActivePort'
        $deadline = [DateTime]::UtcNow.AddSeconds(60)
        $page = $null
        while (-not $page -and [DateTime]::UtcNow -lt $deadline) {
            if ($app.HasExited) { throw "Installed application exited during startup ($($app.ExitCode))" }
            if (Test-Path -LiteralPath $portFile) {
                $port = Get-Content -LiteralPath $portFile -TotalCount 1
                try {
                    $pages = Invoke-RestMethod -NoProxy -Uri "http://127.0.0.1:$port/json/list" -TimeoutSec 2
                    $page = $pages | Where-Object { $_.type -eq 'page' -and $_.url -match '^https?://tauri\.localhost/' } | Select-Object -First 1
                } catch { Write-Verbose "Waiting for WebView2" }
            }
            if (-not $page) { Start-Sleep -Milliseconds 400 }
        }
        if (-not $page) { throw "Installed WebView did not load the bundled application" }
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

$buildMode = if ($UnsignedDebugCandidate) { 'debug' } else { 'release' }
$targetDir = Join-Path $PWD "target/$buildMode"
$installers = @(Get-ChildItem -Path "$targetDir/bundle/nsis/*-setup.exe" -File)
if ($installers.Count -ne 1) { throw "Expected one NSIS installer, got $($installers.Count)" }
$expected = Join-Path $targetDir 'open-frame-studio.exe'
$installDir = Join-Path $env:RUNNER_TEMP ("ofs-install-" + [guid]::NewGuid().ToString("N"))
if (-not $UnsignedDebugCandidate) {
    Assert-SignedFile $installers[0].FullName
    Assert-SignedFile $expected
    Assert-SignedFile (Join-Path $targetDir 'ofs_sdk.dll')
}
# NSIS requires /D= to be the last switch and unquoted, including spaces.
$setup = Start-Process -FilePath $installers[0].FullName -ArgumentList "/S /D=$installDir" -PassThru -WindowStyle Hidden
if (-not $setup.WaitForExit(120000)) { $setup.Kill($true); throw "Installer timed out" }
if ($setup.ExitCode -ne 0) { throw "Installer returned $($setup.ExitCode)" }
$installed = Join-Path $installDir 'open-frame-studio.exe'
if (-not $UnsignedDebugCandidate) { Assert-SignedFile $installed }
$installedHash = (Get-FileHash -LiteralPath $installed -Algorithm SHA256).Hash
if ($installedHash -ne (Get-FileHash -LiteralPath $expected -Algorithm SHA256).Hash) {
    throw "Installed binary differs from the application provided to the bundler"
}
Write-Output "Installed executable SHA-256 $installedHash matches the build"
Test-InstalledEditor $installed ($installDir + '-webview')
