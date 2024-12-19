
import tkinter as tk
from tkinter import Menu
import webbrowser
import os

# Function to open HTML file in the default web browser
def open_html_file(file_path):
    webbrowser.open_new_tab(f'file://{os.path.abspath(file_path)}')

# Create the main application window
root = tk.Tk()
root.title("Documentation Menu Example")

# Create a menu bar
menu_bar = Menu(root)
root.config(menu=menu_bar)

# Create the Help menu
help_menu = Menu(menu_bar, tearoff=0)
menu_bar.add_cascade(label="Help", menu=help_menu)

# Add Documentation directly to Help menu
help_menu.add_command(label="Documentation", command=lambda: open_html_file('application/documentation/html/readme.html'))

# Run the application
root.mainloop()