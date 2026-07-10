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
if (!(Test-Path $configFile)) {
    Write-Host "`n--- Interactive Configuration ---" -ForegroundColor Cyan
    $sshEnabled = (Read-Host "Do you want to enable SSH remote upload? (y/n) [default: n]").Trim().ToLower() -eq 'y'
    
    $sshHost = ""
    $sshUser = ""
    $sshRemoteDir = "/tmp/img2cli"
    
    if ($sshEnabled) {
        $sshHost = (Read-Host "Enter remote SSH Host (e.g., 172.16.190.96)").Trim()
        while ($sshHost -eq "") {
            $sshHost = (Read-Host "SSH Host is required. Enter remote SSH Host").Trim()
        }
        $sshPort = (Read-Host "Enter remote SSH Port [default: 22]").Trim()
        if ($sshPort -eq "") {
            $sshPort = "22"
        }
        $sshUser = (Read-Host "Enter remote SSH Username").Trim()
        while ($sshUser -eq "") {
            $sshUser = (Read-Host "SSH Username is required. Enter remote SSH Username").Trim()
        }
        
        # Detect or generate SSH keys locally on Windows
        $sshFolder = "$env:USERPROFILE\.ssh"
        $pubKeyPath = "$sshFolder\id_ed25519.pub"
        $privKeyPath = "$sshFolder\id_ed25519"
        
        if (!(Test-Path $pubKeyPath)) {
            Write-Host "No local SSH key pair found. Generating a new passwordless SSH key pair..." -ForegroundColor Yellow
            if (!(Test-Path $sshFolder)) {
                New-Item -ItemType Directory -Path $sshFolder | Out-Null
            }
            # Generate ed25519 key without passphrase
            ssh-keygen -t ed25519 -N '""' -f $privKeyPath -q
        }
        
        # Offer to automatically copy public key to remote server for passwordless login
        $copyKey = (Read-Host "Do you want to authorize this SSH key on the remote server for passwordless login? (y/n) [default: y]").Trim().ToLower()
        if ($copyKey -ne 'n') {
            Write-Host "Copying public key to remote server... You will be prompted for your remote password ONCE." -ForegroundColor Cyan
            $pubKeyContent = Get-Content $pubKeyPath
            
            # Commands to append public key to authorized_keys on remote Linux host
            $remoteCommand = "mkdir -p ~/.ssh && chmod 700 ~/.ssh && echo '$pubKeyContent' >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys"
            
            $sshTarget = "$sshUser@$sshHost"
            
            # Run SSH to execute the authorization command. This prompts for password once.
            ssh -p $sshPort $sshTarget $remoteCommand
            
            if ($LASTEXITCODE -eq 0) {
                Write-Host "SSH Key successfully authorized on the remote server! Passwordless login is now active." -ForegroundColor Green
            } else {
                Write-Host "Failed to authorize SSH Key. Please verify your password and SSH connections." -ForegroundColor Red
            }
        }
        
        $tempRemoteDir = (Read-Host "Enter remote directory for saving images [default: /tmp/img2cli]").Trim()
        if ($tempRemoteDir -ne "") {
            $sshRemoteDir = $tempRemoteDir
        }

        # Automatically create the remote directory
        Write-Host "Creating remote directory $sshRemoteDir on the server..." -ForegroundColor Cyan
        $sshTarget = "$sshUser@$sshHost"
        ssh -p $sshPort $sshTarget "mkdir -p $sshRemoteDir"
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Remote directory verified/created successfully!" -ForegroundColor Green
        } else {
            Write-Host "Warning: Could not verify or create remote directory. Please ensure it exists." -ForegroundColor Yellow
        }
    }
    
    Write-Host "`n--- Local Configuration ---" -ForegroundColor Cyan
    $localSaveDirInput = (Read-Host "Enter local directory to save temporary screenshots [default: %TEMP%\img2cli]").Trim()
    if ($localSaveDirInput -eq "") {
        $localSaveDir = "$env:TEMP\img2cli"
    } else {
        $localSaveDir = $localSaveDirInput
    }

    # Create the local directory if it doesn't exist
    if (!(Test-Path $localSaveDir)) {
        New-Item -ItemType Directory -Path $localSaveDir | Out-Null
    }
    
    $escapedSaveDir = $localSaveDir.Replace('\', '\\')
    
    $configContent = @"
# img2cli configuration file
save_dir = "$escapedSaveDir"
hotkey = "ctrl+shift+v"
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
    Set-Content -Path $configFile -Value $configContent
    Write-Host "Created config.toml at $configFile" -ForegroundColor Green
} else {
    Write-Host "config.toml already exists at $configFile." -ForegroundColor Yellow
}

# 4. Create Background Runner script in the current directory (Project Root)
# Placing it in the current directory makes it extremely easy for the user to find and run!
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
Write-Host "2. Copy a screenshot, select your SSH terminal, and press Ctrl+Shift+V." -ForegroundColor Cyan
