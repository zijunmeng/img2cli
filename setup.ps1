# img2cli Windows Setup Script

Write-Host "=== img2cli Windows Auto Setup ===" -ForegroundColor Green

# 1. Check for Rust/Cargo
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Rust/Cargo is not installed on your system." -ForegroundColor Yellow
    Write-Host "Downloading Rustup installer..." -ForegroundColor Cyan
    $rustupUrl = "https://win.rustup.rs/x86_64"
    $rustupPath = "$env:TEMP\rustup-init.exe"
    Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath
    Write-Host "Running Rustup installer... Please accept the default options (press 1)." -ForegroundColor Cyan
    Start-Process -FilePath $rustupPath -Wait
    
    # Reload environment path
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","User") + ";" + [System.Environment]::GetEnvironmentVariable("Path","Machine")
    if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host "Rust installation not detected after install. Please restart PowerShell and run this script again." -ForegroundColor Red
        Exit
    }
} else {
    Write-Host "Rust/Cargo is already installed!" -ForegroundColor Green
}

# 2. Build the project
Write-Host "Compiling img2cli..." -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to compile img2cli." -ForegroundColor Red
    Exit
}
Write-Host "Compilation successful!" -ForegroundColor Green

# 3. Setup configuration directory and file
$configDir = "$env:USERPROFILE\.config\img2cli"
if (!(Test-Path $configDir)) {
    New-Item -ItemType Directory -Path $configDir | Out-Null
}

$configFile = "$configDir\config.toml"
$reconfigure = $true
if (Test-Path $configFile) {
    $choice = (Read-Host "config.toml already exists. Do you want to reconfigure it? (y/n) [default: n]").Trim().ToLower()
    if ($choice -ne 'y') {
        $reconfigure = $false
    }
}

if ($reconfigure) {
    Write-Host "`n--- Interactive Configuration ---" -ForegroundColor Cyan
    
    # Parse ~/.ssh/config for existing hosts
    $sshConfigPath = "$env:USERPROFILE\.ssh\config"
    $hosts = @()
    if (Test-Path $sshConfigPath) {
        Write-Host "Reading local SSH configurations from $sshConfigPath..." -ForegroundColor Cyan
        $currentHost = $null
        Get-Content $sshConfigPath | ForEach-Object {
            $line = $_.Trim()
            # Ignore comments and empty lines
            if ($line -ne "" -and !$line.StartsWith("#")) {
                if ($line -match "^Host\s+(.+)$") {
                    if ($currentHost -and $currentHost.Name -ne "*") {
                        $hosts += $currentHost
                    }
                    $currentHost = [PSCustomObject]@{
                        Name = $Matches[1].Trim()
                        HostName = ""
                        User = ""
                        Port = 22
                    }
                }
                elseif ($currentHost) {
                    if ($line -match "^HostName\s+(.+)$") {
                        $currentHost.HostName = $Matches[1].Trim()
                    }
                    elseif ($line -match "^User\s+(.+)$") {
                        $currentHost.User = $Matches[1].Trim()
                    }
                    elseif ($line -match "^Port\s+(\d+)$") {
                        $currentHost.Port = [int]$Matches[1].Trim()
                    }
                }
            }
        }
        if ($currentHost -and $currentHost.Name -ne "*") {
            $hosts += $currentHost
        }
    }
    
    $sshEnabled = $false
    $sshHost = ""
    $sshPort = "22"
    $sshUser = ""
    $sshRemoteDir = ""
    $sshTargetsList = @()
    
    if ($hosts.Count -gt 0) {
        Write-Host "`nDetected the following SSH hosts in your local SSH config:" -ForegroundColor Green
        for ($i = 0; $i -lt $hosts.Count; $i++) {
            $h = $hosts[$i]
            Write-Host "  [$($i + 1)] $($h.Name) (HostName: $($h.HostName), User: $($h.User), Port: $($h.Port))"
        }
        Write-Host "  [$($hosts.Count + 1)] Enter a custom host manually..."
        
        $choiceIndex = -1
        while ($choiceIndex -lt 1 -or $choiceIndex -gt ($hosts.Count + 1)) {
            $inputChoice = (Read-Host "Select a default host for SSH remote upload (1-$($hosts.Count + 1))").Trim()
            if ($inputChoice -match "^\d+$") {
                $choiceIndex = [int]$inputChoice
            }
        }
        
        if ($choiceIndex -le $hosts.Count) {
            $selected = $hosts[$choiceIndex - 1]
            $sshEnabled = $true
            # Use the Host alias directly so that ~/.ssh/config parameters (e.g. IdentityFile) are resolved natively
            $sshHost = $selected.Name
            $sshPort = $selected.Port.ToString()
            $sshUser = $selected.User
            Write-Host "Selected default SSH host: $sshHost" -ForegroundColor Green
        } else {
            $sshEnabled = (Read-Host "Do you want to enable SSH remote upload? (y/n) [default: n]").Trim().ToLower() -eq 'y'
            if ($sshEnabled) {
                $sshHost = (Read-Host "Enter remote SSH Host (e.g., 172.16.190.96)").Trim()
                while ($sshHost -eq "") {
                    $sshHost = (Read-Host "SSH Host is required. Enter remote SSH Host").Trim()
                }
                $sshPort = (Read-Host "Enter remote SSH Port [default: 22]").Trim()
                if ($sshPort -eq "") { $sshPort = "22" }
                $sshUser = (Read-Host "Enter remote SSH Username").Trim()
                while ($sshUser -eq "") {
                    $sshUser = (Read-Host "SSH Username is required. Enter remote SSH Username").Trim()
                }
            }
        }
        
        if ($sshEnabled) {
            # Ask if they want to configure multiple servers
            $multiConfig = (Read-Host "Do you want to configure multiple remote servers for automatic routing? (y/n) [default: y]").Trim().ToLower()
            if ($multiConfig -ne 'n') {
                Write-Host "`nSelect which hosts to add to the automatic routing targets (separate by commas, e.g. 1,3 or type 'all'):" -ForegroundColor Cyan
                $targetsInput = (Read-Host "Targets").Trim()
                $selectedTargets = @()
                if ($targetsInput -eq "all") {
                    $selectedTargets = $hosts
                } else {
                    $indices = $targetsInput.Split(",")
                    foreach ($idxStr in $indices) {
                        if ($idxStr.Trim() -match "^\d+$") {
                            $idx = [int]$idxStr.Trim()
                            if ($idx -ge 1 -and $idx -le $hosts.Count) {
                                $selectedTargets += $hosts[$idx - 1]
                            }
                        }
                    }
                }
                
                if ($selectedTargets.Count -gt 0) {
                    Write-Host "Configured $($selectedTargets.Count) automatic routing targets." -ForegroundColor Green
                    $sshTargetsList = $selectedTargets
                }
            }
        }
    } else {
        $sshEnabled = (Read-Host "Do you want to enable SSH remote upload? (y/n) [default: n]").Trim().ToLower() -eq 'y'
        if ($sshEnabled) {
            $sshHost = (Read-Host "Enter remote SSH Host (e.g., 172.16.190.96)").Trim()
            while ($sshHost -eq "") {
                $sshHost = (Read-Host "SSH Host is required. Enter remote SSH Host").Trim()
            }
            $sshPort = (Read-Host "Enter remote SSH Port [default: 22]").Trim()
            if ($sshPort -eq "") { $sshPort = "22" }
            $sshUser = (Read-Host "Enter remote SSH Username").Trim()
            while ($sshUser -eq "") {
                $sshUser = (Read-Host "SSH Username is required. Enter remote SSH Username").Trim()
            }
        }
    }
    
    if ($sshEnabled) {
        $resolvedUser = if ($sshUser -ne "") { $sshUser } else { "your_username" }
        $defaultDir = "/s1/SHARE/$resolvedUser/tmp/img2cli"
        $tempRemoteDir = (Read-Host "Enter remote directory for saving images [default: $defaultDir]").Trim()
        if ($tempRemoteDir -ne "") {
            $sshRemoteDir = $tempRemoteDir
        } else {
            $sshRemoteDir = $defaultDir
        }
        
        if ($sshRemoteDir -notlike "/s1/SHARE/*" -and $sshRemoteDir -notlike "C:*" -and $sshRemoteDir -notlike "\\*") {
            Write-Host "`nTip: If you are using a shared cluster environment, make sure '$sshRemoteDir' is inside a network-shared directory (e.g. starts with /s1/SHARE/) so that AI Agents running on other nodes can access the uploaded images." -ForegroundColor Yellow
        }
        
        # Ask to verify/authorize SSH keys
        $verifyKeys = (Read-Host "Do you want to verify/authorize SSH keys for passwordless login on the remote servers? (y/n) [default: y]").Trim().ToLower()
        if ($verifyKeys -ne 'n') {
            $sshFolder = "$env:USERPROFILE\.ssh"
            $pubKeyPath = "$sshFolder\id_ed25519.pub"
            $privKeyPath = "$sshFolder\id_ed25519"
            
            if (!(Test-Path $pubKeyPath)) {
                Write-Host "No local SSH key pair found. Generating a new passwordless SSH key pair..." -ForegroundColor Yellow
                if (!(Test-Path $sshFolder)) {
                    New-Item -ItemType Directory -Path $sshFolder | Out-Null
                }
                ssh-keygen -t ed25519 -N '""' -f $privKeyPath -q
            }
            
            $pubKeyContent = Get-Content $pubKeyPath
            $remoteCommand = "mkdir -p ~/.ssh && chmod 700 ~/.ssh && echo '$pubKeyContent' >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys"
            
            # Authorize default host
            Write-Host "Authorizing SSH key on $sshHost..." -ForegroundColor Cyan
            ssh $sshHost $remoteCommand
            ssh $sshHost "mkdir -p $sshRemoteDir"
            
            # Authorize other targets
            foreach ($t in $sshTargetsList) {
                if ($t.Name -eq $sshHost) {
                    continue
                }
                Write-Host "Authorizing SSH key on $($t.Name)..." -ForegroundColor Cyan
                ssh $($t.Name) $remoteCommand
                ssh $($t.Name) "mkdir -p $sshRemoteDir"
            }
        }
    }
    
    Write-Host "`n--- Local Configuration ---" -ForegroundColor Cyan
    $localSaveDirInput = (Read-Host "Enter local directory to save temporary screenshots [default: %TEMP%\img2cli]").Trim()
    if ($localSaveDirInput -eq "") {
        $localSaveDir = "$env:TEMP\img2cli"
    } else {
        $localSaveDir = $localSaveDirInput
    }

    if (!(Test-Path $localSaveDir)) {
        New-Item -ItemType Directory -Path $localSaveDir | Out-Null
    }
    
    $escapedSaveDir = $localSaveDir.Replace('\', '\\')
    
    $configContent = @"
# img2cli configuration file
save_dir = "$escapedSaveDir"
output_format = "markdown"
compress_quality = 80
max_dimension = 1024
workspace_aware = false

[ssh]
enabled = $(if ($sshEnabled) { "true" } else { "false" })
host = "$sshHost"
port = $(if ($sshEnabled) { $sshPort } else { "22" })
username = $(if ($sshUser -ne "") { '"{0}"' -f $sshUser } else { '""' })
remote_dir = "$sshRemoteDir"
"@

    if ($sshTargetsList.Count -gt 0) {
        $configContent += "`n"
        foreach ($t in $sshTargetsList) {
            $tHost = $t.Name
            $tPort = $t.Port
            $tUser = $t.User
            $configContent += @"

[[ssh_targets]]
enabled = true
match_pattern = "$tHost"
host = "$tHost"
port = $tPort
username = "$tUser"
remote_dir = "$sshRemoteDir"
"@
        }
    }

    Set-Content -Path $configFile -Value $configContent
    Write-Host "`nCreated config.toml at $configFile" -ForegroundColor Green
} else {
    Write-Host "Keeping existing config.toml at $configFile." -ForegroundColor Yellow
}

# 4. Create Background Runner script
$vbsFile = "run_hidden.vbs"
$currentPath = $pwd.Path
$vbsContent = @"
Set WshShell = CreateObject("WScript.Shell")
WshShell.Run """$currentPath\target\release\img2cli.exe"" run", 0, false
"@
Set-Content -Path $vbsFile -Value $vbsContent
Write-Host "Created background runner script: $vbsFile in the current directory!" -ForegroundColor Green

Write-Host "`n=== Setup Complete! ===" -ForegroundColor Green
Write-Host "1. To run img2cli in the background, double click the following file in this folder:" -ForegroundColor Cyan
Write-Host "   $vbsFile" -ForegroundColor Yellow
Write-Host "2. Copy a screenshot, select your SSH terminal, and paste the path (Ctrl+V, Shift+Insert, or Right-Click)." -ForegroundColor Cyan
