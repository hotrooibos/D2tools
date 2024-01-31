#!/usr/bin/env python3
import json
import os
import tomllib
import sys

from .main import Filter as ft


def read_json(self, json_file):
    """Read JSON data file
    """

    with open(file=json_file,
                mode='r',
                encoding='utf-8') as json_file:
        return json.load(json_file)


# Argument check
if len(sys.argv) > 2:
    print("Too much arguments\nExample: py d2lootfilter lootfilter_endgame.toml")
    exit(1)

if len(sys.argv) < 1:
    print("Argument missing, please specify a lootfilter config file to apply\nExample: py d2lootfilter lootfilter_endgame.toml")
    exit(1)

# Get argument given in command line
arg = str(sys.argv[1]).lower()
script_dir = os.path.abspath( os.path.dirname( __file__ ) )

# Load configuration from file
with open(f"{script_dir}/{arg}", "rb") as f:
    conf = tomllib.load(f)

print(conf)

# Read D2 json files
# TODO ft_profile = read_json(template2.5/data/global/ui/layouts/_profilehd.json)
# TODO ft_modifiers = read_json(template2.5/data/loca/hd/lng/strings/item-modifiers.json)
# TODO ft_affixes = read_json(template2.5/data/loca/hd/lng/strings/item-nameaffixes.json)
# TODO ft_names = read_json(template2.5/data/loca/hd/lng/strings/item-names.json)
# TODO ft_runes = read_json(template2.5/data/loca/hd/lng/strings/item-runes.json)
