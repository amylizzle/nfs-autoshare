#!/usr/bin/env python3

import os
import gi
import socket
from typing import List
from gi.repository import Nautilus, GObject


class NFSAutoShareProvider(GObject.GObject, Nautilus.MenuProvider):
    def get_file_items(self, files: List[Nautilus.FileInfo]) -> List[Nautilus.MenuItem]:

        top_menuitem = Nautilus.MenuItem(
                name="NFSAutoShareProvider::ShareItem",
                label="Share over network",
                tip="",
                icon="",
            )

        return [top_menuitem]

    def get_background_items(self, current_folder: Nautilus.FileInfo) -> List[Nautilus.MenuItem]:
        menuitem = Nautilus.MenuItem(
                name="NFSAutoShareProvider::ShareItem",
                label="Share over network",
                tip="",
                icon="",
            )

        return [
            menuitem,
        ]
    


