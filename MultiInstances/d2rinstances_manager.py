#!/usr/bin/env python3
import json
import os
import subprocess


script_dir = os.path.abspath( os.path.dirname( __file__ ) )

ahk_exe = r"C:\Users\tda\Apps\AutoHotkey2\AutoHotkey64.exe"
ahk_macro = r"C:\Users\tda\Git\hotrooibos\Game_macros\d2\d2_standard.ahk"

d2r_exe = r"C:\Program Files (x86)\Diablo II Resurrected\Diablo II Resurrected Launcher.exe"
d2r_args = "-mod Tdafilter_starter -txt"

# Mods (as in C:\Program Files (x86)\Diablo II Resurrected\mods) :
# - Tdafilter_starter
# - Tdafilter

d2r_alt_exe = r"C:\Program Files (x86)\Diablo II Resurrected_alt\Diablo II Resurrected Launcher.exe"
d2r_alt_args = "-mod Tdafilter_starter -txt"

d2r_alt2_exe = r"C:\Program Files (x86)\Diablo II Resurrected_alt2\Diablo II Resurrected Launcher.exe"
d2r_alt2_args = "-mod Tdafilter_starter -txt"

while (True) :
    print("0: AHK macro")
    print("1: D2R (toleda)")
    print("2: D2R alt (socca)")
    print("3: D2R alt2 (ferdi)")
    print("9: Exit all D2R")

    usrinput = input("Selection : ")
 
    # Read running D2R instance handles
    prog = subprocess.Popen(["runas",
                             "/noprofile",
                             "/user:Administrator",
                             f"{script_dir}/handle64.exe -accepteula -a -p D2R.exe > {script_dir}/d2r_handles.txt"],
                            stdin=subprocess.PIPE)
    prog.stdin.write('password')
    prog.communicate()


    # proc_id_populated = ""
    # handle_id_populated = ""

    # foreach($line in Get-Content $PSScriptRoot\d2r_handles.txt)
    # {
    #     $proc_id = $line | Select-String -Pattern '^D2R.exe pid\: (?<g1>.+) ' | %{$_.Matches.Groups[1].value}

    #     if ($proc_id)
    #     {
    #         $proc_id_populated = $proc_id
    #     }

    #     $handle_id = $line | Select-String -Pattern '^(?<g2>.+): Event.*DiabloII Check For Other Instances' | %{$_.Matches.Groups[1].value}

    #     if ($handle_id)
    #     {
    #         $handle_id_populated = $handle_id
    #         Write-Host "Closing" $proc_id_populated $handle_id_populated
    #         & "$PSScriptRoot\handle64.exe" -p $proc_id_populated -c $handle_id_populated -y

    #         Stop-Process -Name "Battle.net"
    #     }
    # }

    # switch (usrinput)
    # {
    #     0 {& "$ahk_exe" $ahk_macro; Break}
    #     1 {& "$d2r_exe" $d2r_args; Break}
    #     2 {& "$d2r_alt_exe" $d2r_alt_args; Break}
    #     3 {& "$d2r_alt2_exe" $d2r_alt2_args; Break}
    #     9 {Stop-Process -Name "D2R"; Break}
    # }

    # read-host "Press ENTER to continue..."
    # Clear-Host