#!/usr/bin/env python3

import json
import os
import requests
from time import time, strftime, localtime
import zipfile


def zip_directory(folder_path, zip_file):
    for folder_name, subfolders, filenames in os.walk(folder_path):
        for filename in filenames:
            # Create complete filepath of file in directory
            file_path = os.path.join(folder_name, filename)
            # Add file to zip
            zip_file.write(file_path, arcname=filename)


time_api = 'http://worldtimeapi.org/api/timezone/Europe/Paris'
response = requests.get(time_api)

# Get time from API, or fallback with local time
if response.status_code == 200:
    data = response.json()
    now_epochtime = data["unixtime"]
else:
    print("Error:", response.status_code)
    now_epochtime = int(time())

now_datetime = strftime('%Y%m%d_%H%M%S', localtime(now_epochtime))
sav_dir = "/mnt/c/Users/tda/Saved Games/Diablo II Resurrected/mods/Tdafilter/"
bak_dir = f"/mnt/d/Backups/D2R/{now_datetime}_d2r_bak.zip"

# Create new zip file
zip_file = zipfile.ZipFile(bak_dir, 'w',
                           compression=zipfile.ZIP_LZMA,
                           compresslevel=9)

# Backup / make zip
zip_directory(sav_dir, zip_file)
zip_file.close()

# File size in KB
bak_size = os.path.getsize(bak_dir) / 1024
bak_size = round(bak_size, 2)

print(f"Created {bak_dir} - size {bak_size} KB")