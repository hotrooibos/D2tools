import json
import logging
import os

from json.decoder import JSONDecodeError
from time import time

from . import constants as const
from . import utils



class Singleton:
    instance = None

    def __new__(cls, *args, **kwargs):
        if not isinstance(cls.instance, cls):
            cls.instance = object.__new__(cls, *args, **kwargs)

        return cls.instance



class Filter(Singleton):
    """JSON data class
    Loaded from JSON data file or created 
    with no data if file doesn't exists.
    """
    def read(self, json_file):
        """Read JSON data file and load it in memory as a
        Jdata object or, if file does not exists, make a new one
        """
        
        with open(file=json_file,
                    mode='r',
                    encoding='utf-8') as json_file:
            self.jdat = json.load(json_file)
                




    def write(self,
              jdat: dict=None,
              json_file: str=const.JSON_PATH):
        """Write 'jdat' dict into 'jsonf' JSON file
        """

        if not jdat:
            jdat = self.jdat

        with open(file=json_file,
                  mode='w',
                  encoding='utf-8') as jsonfile:
            json.dump(jdat, jsonfile, indent=4)

        self.read()