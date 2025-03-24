# Run multiples D2R clients

# This script closes Diablo 2 Resurrected following thread handles in order to run multiples D2R clients on the same computer :
#   - Event "DiabloII Check For Other Instances"
#   - Section "windows_shell_global_counters"

# Source : https://forums.d2jsp.org/topic.php?t=90563264&f=87

$ahk_exe = "C:\Users\tda\Apps\AutoHotkey2\AutoHotkey64.exe"
$ahk_macro = "C:\Users\tda\Git\hotrooibos\Game_macros\d2\d2_standard.ahk"

$d2r_exe = "C:\Program Files (x86)\Diablo II Resurrected\Diablo II Resurrected Launcher.exe"
$d2r_args = "-mod Tdafilter_starter -txt"

# Mods (as in C:\Program Files (x86)\Diablo II Resurrected\mods) :
# - Tdafilter_starter
# - Tdafilter

$d2r_alt_exe = "C:\Program Files (x86)\Diablo II Resurrected_alt\Diablo II Resurrected Launcher.exe"
$d2r_alt_args = "-mod Tdafilter_starter -txt"

$d2r_alt2_exe = "C:\Program Files (x86)\Diablo II Resurrected_alt2\Diablo II Resurrected Launcher.exe"
$d2r_alt2_args = "-mod Tdafilter_starter -txt"


while (1) {
    Write-Host "[0] AHK D2 macro"
    Write-Host "[1] Remote desktop port connection watch"
    Write-Host "[2] D2R (toleda)"
    Write-Host "[3] D2R (socca)"
    Write-Host "[4] D2R (ferdi)"
    Write-Host "[9] Kill all D2R"
    $input = Read-Host "Selection"
 
    function Remote-Watcher {
        $port = Read-Host "Port to watch"
        # Function to check if a specific port is in use
        function Is-PortInUse {
            param([int]$Port)
            $netstat = netstat -an | Select-String ":$Port"
            return $netstat -ne $null
        }

        # Loop until port is not in use
        Write-Host "Monitoring port $port..."

        while (Is-PortInUse -Port $port) {
            Start-Sleep -Seconds 5
        }

        # Once the port is no longer in use, check for TRUC.EXE process and kill it
        Write-Host "Port $port is no longer in use. Attempting to kill D2R..."
        
        $d2rProcess = Get-Process -Name "Notepad" -ErrorAction SilentlyContinue
        if ($d2rProcess) {
            Stop-Process -Name "Notepad"
            Write-Host "D2R has been terminated."
        } else {
            Write-Host "D2R is not running."
        }
    }

    if (!([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator"))
    {
        Start-Process powershell.exe "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`"" -Verb RunAs; exit
    }

    & "$PSScriptRoot\handle64.exe" -accepteula -a -p D2R.exe > $PSScriptRoot\d2r_handles.txt

    $proc_id_populated = ""
    $handle_id_populated = ""

    foreach($line in Get-Content $PSScriptRoot\d2r_handles.txt)
    {
        $proc_id = $line | Select-String -Pattern '^D2R.exe pid\: (?<g1>.+) ' | %{$_.Matches.Groups[1].value}

        if ($proc_id)
        {
            $proc_id_populated = $proc_id
        }

        $handle_id = $line | Select-String -Pattern '^(?<g2>.+): Event.*DiabloII Check For Other Instances' | %{$_.Matches.Groups[1].value}

        if ($handle_id)
        {
            $handle_id_populated = $handle_id
            Write-Host "Closing" $proc_id_populated $handle_id_populated
            & "$PSScriptRoot\handle64.exe" -p $proc_id_populated -c $handle_id_populated -y

            Stop-Process -Name "Battle.net"
        }
    }

    switch ($input)
    {
        0 {& "$ahk_exe" $ahk_macro; Break}
        1 {Remote-Watcher; Break}
        2 {& "$d2r_exe" $d2r_args; Break}
        3 {& "$d2r_alt_exe" $d2r_alt_args; Break}
        4 {& "$d2r_alt2_exe" $d2r_alt2_args; Break}
        9 {Stop-Process -Name "D2R"; Break}
    }

    # read-host "Press ENTER to continue..."
    # Clear-Host
}