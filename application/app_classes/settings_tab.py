
import tkinter as tk
from tkinter import ttk
import xml.etree.ElementTree as Et


class Setting(tk.Frame):
    def __init__(self, master, logger, **kw):
        super().__init__(master, **kw)
        self.label_screen_size = None
        self.dropdown_screen_size = None
        self.apply_button = None
        self.screen_width = self.master.screen_width
        self.screen_height = self.master.screen_height
        self.bg_color = (230, 230, 230)
        self.ship_speed = 1.5
        self.bullet_speed = 1.0
        self.bullet_width = 3
        self.bullet_height = 15
        self.bullet_color = (60, 60, 60)
        self.bullets_allowed = 3
        self.fleet_drop_speed = 10
        self.fleet_direction = 1
        self.ship_limit = 3
        self.speedup_scale = 1.1
        self.score_scale = 1.5
        self.initialize_dynamic_settings()
        self.logger = logger

    def initialize_dynamic_settings(self):
        self.ship_speed = 1.5
        self.bullet_speed = 3.0
        self.alien_speed = 1.0
        self.fleet_direction = 1
        self.alien_points = 50

    def increase_speed(self):
        self.ship_speed *= self.speedup_scale
        self.bullet_speed *= self.speedup_scale
        self.alien_speed *= self.speedup_scale
        self.alien_points = int(self.alien_points * self.score_scale)

    def create(self):
        # show all settings in the settings tab
        self.label_screen_size = tk.Label(self, text="Screen size:")
        self.label_screen_size.grid(sticky="w", row=0, column=0, padx=5, pady=5)
        self.dropdown_screen_size = ttk.Combobox(self, values=['1920x1080', '1280x720', '800x600'])
        self.dropdown_screen_size.grid(sticky="w", row=1, column=0, padx=5, pady=5)
        self.apply_button = tk.Button(self, text="Apply", command=self.save)
        self.apply_button.grid(sticky='w', row=2, column=0, columnspan=3, padx=5, pady=5)


    def save(self):
        self.screen_width = int(self.dropdown_screen_size.get().split('x')[0])
        self.screen_height = int(self.dropdown_screen_size.get().split('x')[1])
        self.master.screen_width = self.screen_width
        self.master.geometry(f"{self.dropdown_screen_size.get()}")
        # change the width of the tab control depending on the screen width
        self.master.tab_control.config(width=self.screen_width - 300, height=self.screen_height - 300)
        # change the entry width in the general tab

        self.master.reload_tabs['general'](None)
        self.master.reload_tabs['transport'](None)
        self.master.reload_tabs['other'](None)
        self.master.reload_tabs['validator'](None)
        self.master.reload_tabs['settings'](None)
        self.master.reload_tabs['certificates'](None)
        self.master.reload_tabs['xml_editor'](None)




        # apply te changes and reload the application





        # self.initialize_dynamic_settings()

    def load(self):
        # set default screen size to 1920x1080

        self.dropdown_screen_size.set(f"1920x1080")
        # self.dropdown_screen_size.set(self.master.winfo_geometry().split('+')[0])
        self.initialize_dynamic_settings()

    def reload(self):
        self.entry_screen_width.delete(0, tk.END)
        self.entry_screen_width.insert(0, self.screen_width)
        self.entry_screen_height.delete(0, tk.END)
        self.entry_screen_height.insert(0, self.screen_height)


