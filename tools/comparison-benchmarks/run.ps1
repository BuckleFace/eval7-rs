[CmdletBinding()]
param(
    [string]$OutputRoot,
    [string]$AffinityMask,
    [switch]$ValidateOnly
)

$ErrorActionPreference = 'Stop'
$ToolRoot = $PSScriptRoot
$RepoRoot = (Resolve-Path (Join-Path $ToolRoot '..\..')).Path
$Manifest = Join-Path $ToolRoot 'Cargo.toml'
if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $RepoRoot 'benchmark-results'
}

$Timestamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$OutputDirectory = Join-Path $OutputRoot $Timestamp
$RawDirectory = Join-Path $OutputDirectory 'raw'
New-Item -ItemType Directory -Force -Path $RawDirectory | Out-Null
$Failures = [System.Collections.Generic.List[string]]::new()

function Invoke-Logged {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$LogPath,
        [Parameter(Mandatory)][scriptblock]$Command
    )
    $PreviousErrorAction = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    & $Command 2>&1 | ForEach-Object { $_.ToString() } | Tee-Object -FilePath $LogPath
    $ExitCode = $LASTEXITCODE
    $ErrorActionPreference = $PreviousErrorAction
    if ($ExitCode -ne 0) {
        $script:Failures.Add("$Name failed with exit code $ExitCode")
        throw "$Name failed; see $LogPath"
    }
}

Push-Location $RepoRoot
$OriginalAffinity = $null
try {
    $Commit = (git rev-parse HEAD).Trim()
    $DirtyLines = @(git status --porcelain)
    $Dirty = $DirtyLines.Count -ne 0
    $Cpu = Get-CimInstance Win32_Processor | Select-Object Name, NumberOfCores, NumberOfLogicalProcessors, MaxClockSpeed
    $Os = Get-CimInstance Win32_OperatingSystem | Select-Object Caption, Version, BuildNumber, OSArchitecture
    $Rustc = (rustc -Vv) -join "`n"
    $Cargo = (cargo -V).Trim()
    $RustFlags = if ($env:RUSTFLAGS) { $env:RUSTFLAGS } else { '(unset; generic target defaults)' }

    if ($AffinityMask) {
        $MaskText = $AffinityMask -replace '^0x', ''
        $MaskValue = [Convert]::ToUInt64($MaskText, 16)
        if ($MaskValue -eq 0) {
            throw 'AffinityMask must select at least one logical processor.'
        }
        $CurrentProcess = Get-Process -Id $PID
        $OriginalAffinity = $CurrentProcess.ProcessorAffinity
        $CurrentProcess.ProcessorAffinity = [IntPtr]([Int64]$MaskValue)
    }

    $Metadata = @"
# Benchmark run metadata

- Timestamp: $(Get-Date -Format o)
- Git commit: $Commit
- Worktree dirty: $Dirty
- Affinity mask: $(if ($AffinityMask) { $AffinityMask } else { 'not set; Windows scheduler controls placement' })
- Build profiles: Criterion uses bench; probes use release; both set `codegen-units = 1` and `lto = "thin"`
- RUSTFLAGS: $RustFlags
- Criterion: warm-up 3 s, measurement 5 s, sample count 80, plots disabled by runner
- Cargo: $Cargo

## rustc

``````text
$Rustc
``````

## CPU

``````text
$($Cpu | Format-List | Out-String)
``````

## Operating system

``````text
$($Os | Format-List | Out-String)
``````

## Initial worktree status

``````text
$($DirtyLines -join "`n")
``````
"@
    Set-Content -LiteralPath (Join-Path $OutputDirectory 'metadata.md') -Value $Metadata -Encoding utf8
    cargo tree --manifest-path $Manifest --locked | Set-Content -LiteralPath (Join-Path $RawDirectory 'cargo-tree.txt') -Encoding utf8

    Invoke-Logged -Name 'correctness preflight' -LogPath (Join-Path $RawDirectory 'preflight.log') -Command {
        cargo run --release --locked --manifest-path $Manifest --bin preflight
    }

    if ($ValidateOnly) {
        Set-Content -LiteralPath (Join-Path $OutputDirectory 'VALIDATION_ONLY.txt') -Value 'Correctness preflight completed. No performance measurements were run.' -Encoding utf8
        Write-Host "Validation-only run completed: $OutputDirectory"
        return
    }

    $TargetDirectory = (cargo metadata --no-deps --format-version 1 --manifest-path $Manifest | ConvertFrom-Json).target_directory
    $TargetRoot = [IO.Path]::GetFullPath($TargetDirectory).TrimEnd('\')
    $CriterionDirectory = [IO.Path]::GetFullPath((Join-Path $TargetDirectory 'criterion'))
    if (!$CriterionDirectory.StartsWith($TargetRoot + '\', [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Resolved Criterion directory is outside the Cargo target directory.'
    }
    if (Test-Path -LiteralPath $CriterionDirectory) {
        Move-Item -LiteralPath $CriterionDirectory -Destination (Join-Path $RawDirectory 'criterion-preexisting')
    }

    Invoke-Logged -Name 'Criterion comparison suite' -LogPath (Join-Path $RawDirectory 'criterion.log') -Command {
        cargo bench --locked --manifest-path $Manifest --bench comparison -- --noplot
    }

    if (!(Test-Path -LiteralPath $CriterionDirectory)) {
        throw "Criterion did not create $CriterionDirectory"
    }
    Copy-Item -Recurse -LiteralPath $CriterionDirectory -Destination (Join-Path $RawDirectory 'criterion')

    Invoke-Logged -Name 'probe build' -LogPath (Join-Path $RawDirectory 'probe-build.log') -Command {
        cargo build --release --locked --manifest-path $Manifest --bin probe
    }
    $Probe = Join-Path $TargetDirectory 'release\probe.exe'
    $ProbeNames = @('noop', 'eval7-formula', 'eval7', 'rs_poker', 'aya_poker', 'poker')
    $InitializationPath = Join-Path $RawDirectory 'initialization.txt'
    foreach ($Name in $ProbeNames) {
        for ($Iteration = 1; $Iteration -le 20; $Iteration++) {
            try {
                "iteration=$Iteration $(& $Probe $Name 0 0)" | Add-Content -LiteralPath $InitializationPath -Encoding utf8
                if ($LASTEXITCODE -ne 0) { throw "exit code $LASTEXITCODE" }
            } catch {
                $Failures.Add("initialization probe $Name iteration $Iteration failed: $_")
            }
        }
    }

    $MemoryRows = @()
    foreach ($Name in $ProbeNames) {
        $Stdout = Join-Path $RawDirectory "memory-$Name.stdout.txt"
        $Stderr = Join-Path $RawDirectory "memory-$Name.stderr.txt"
        try {
            $Process = Start-Process -FilePath $Probe -ArgumentList @($Name, '3000', '20000') -PassThru -RedirectStandardOutput $Stdout -RedirectStandardError $Stderr
            $ReadyDeadline = (Get-Date).AddSeconds(30)
            while (!$Process.HasExited -and (!(Test-Path -LiteralPath $Stdout) -or (Get-Item -LiteralPath $Stdout).Length -eq 0)) {
                if ((Get-Date) -gt $ReadyDeadline) {
                    throw 'probe did not signal readiness within 30 seconds'
                }
                Start-Sleep -Milliseconds 50
                $Process.Refresh()
            }
            if ($Process.HasExited) {
                throw "probe exited before memory snapshot with exit code $($Process.ExitCode)"
            }
            $Snapshot = Get-Process -Id $Process.Id
            $MemoryRows += [pscustomobject]@{
                evaluator = $Name
                pid = $Process.Id
                working_set_bytes = $Snapshot.WorkingSet64
                private_bytes = $Snapshot.PrivateMemorySize64
                peak_working_set_bytes = $Snapshot.PeakWorkingSet64
            }
            $Process.WaitForExit()
            # Windows PowerShell can retain a stale Process object after WaitForExit(),
            # leaving ExitCode as $null until the object is refreshed.
            $Process.Refresh()
            if (!$Process.HasExited) { throw 'probe did not exit after WaitForExit returned' }
            $MemoryExitCode = $Process.ExitCode
            if ($null -eq $MemoryExitCode) { throw 'probe exit code was unavailable after refresh' }
            if ($MemoryExitCode -ne 0) { throw "exit code $MemoryExitCode" }
        } catch {
            $Failures.Add("memory probe $Name failed: $_")
        }
    }
    $MemoryRows | Export-Csv -NoTypeInformation -LiteralPath (Join-Path $RawDirectory 'memory.csv')

    $Rows = @()
    Get-ChildItem -Recurse -Filter estimates.json (Join-Path $RawDirectory 'criterion') |
        Where-Object { $_.Directory.Name -eq 'new' } |
        ForEach-Object {
        $Estimate = Get-Content -Raw -LiteralPath $_.FullName | ConvertFrom-Json
        $BenchmarkPath = Join-Path $_.DirectoryName 'benchmark.json'
        if (Test-Path -LiteralPath $BenchmarkPath) {
            $Benchmark = Get-Content -Raw -LiteralPath $BenchmarkPath | ConvertFrom-Json
            $Elements = if ($Benchmark.throughput -and $Benchmark.throughput.Elements) { [double]$Benchmark.throughput.Elements } else { 1.0 }
            $Rows += [pscustomobject]@{
                benchmark = $Benchmark.full_id
                ns_per_evaluation = $Estimate.mean.point_estimate / $Elements
                ci_lower_ns = $Estimate.mean.confidence_interval.lower_bound / $Elements
                ci_upper_ns = $Estimate.mean.confidence_interval.upper_bound / $Elements
                evaluations_per_second = 1.0e9 / ($Estimate.mean.point_estimate / $Elements)
            }
        }
    }
    $Rows = $Rows | Sort-Object benchmark
    $Rows | Export-Csv -NoTypeInformation -LiteralPath (Join-Path $OutputDirectory 'criterion-summary.csv')

    $Table = ($Rows | ForEach-Object {
        "| $($_.benchmark) | $([math]::Round($_.ns_per_evaluation, 3)) | $([math]::Round($_.ci_lower_ns, 3))-$([math]::Round($_.ci_upper_ns, 3)) | $([math]::Round($_.evaluations_per_second, 0)) |"
    }) -join "`n"
    $FailureText = if ($Failures.Count -eq 0) { 'None recorded.' } else { ($Failures | ForEach-Object { "- $_" }) -join "`n" }
    $Summary = @"
# Eval7 comparison benchmark summary

See metadata.md for the environment and raw/ for Criterion samples, console logs, initialization observations, and process-memory snapshots.

| Benchmark | Mean ns/evaluation | 95% CI (ns) | Evaluations/s |
|---|---:|---:|---:|
$Table

## Failures

$FailureText

Memory figures are Windows process-level observations, not allocator-exclusive byte counts. Compare them with the noop process and the exact static sizes printed by each probe. Initialization and first-use observations are stored raw because individual wall-clock samples are intentionally not presented as Criterion-quality estimates.
"@
    Set-Content -LiteralPath (Join-Path $OutputDirectory 'summary.md') -Value $Summary -Encoding utf8

    if ($Failures.Count -ne 0) {
        throw "$($Failures.Count) benchmark steps failed; see summary.md"
    }
    Write-Host "Benchmark suite completed: $OutputDirectory"
} catch {
    $Failures.Add($_.Exception.Message)
    $CompleteCriterionDestination = Join-Path $RawDirectory 'criterion'
    if ($CriterionDirectory -and
        (Test-Path -LiteralPath $CriterionDirectory) -and
        !(Test-Path -LiteralPath $CompleteCriterionDestination)) {
        $PartialDestination = Join-Path $RawDirectory 'criterion-partial'
        if (!(Test-Path -LiteralPath $PartialDestination)) {
            Copy-Item -Recurse -LiteralPath $CriterionDirectory -Destination $PartialDestination
        }
    }
    $Failures | Set-Content -LiteralPath (Join-Path $OutputDirectory 'failures.txt') -Encoding utf8
    throw
} finally {
    if ($null -ne $OriginalAffinity) {
        (Get-Process -Id $PID).ProcessorAffinity = $OriginalAffinity
    }
    Pop-Location
}
