import logging
import queue
import threading
import time
import webbrowser

# import xml.etree.ElementTree as Et
import lxml.etree as Et

import tkinter as tk
from tkinter import ttk, filedialog

import platform


from application.app_classes.certificates_tab import Certificates
from application.app_classes.collaboration_role import CollaborationRole
from application.app_classes.general_tab import General
from application.app_classes.help_window import HelpWindow, HOW_TO_USE_FILE_EN, HOW_TO_USE_FILE_NL, README_FILE
from application.app_classes.transport_tab import Transport
from application.app_classes.validator_tab import Validator
from application.app_classes.xml_editor_tab import XMLEditor
from application.app_classes.comment_tab import Comment
from application.helper_classes.textHandler import TextHandler
from application.helper_classes.updateCheck import (LATEST_RELEASE_DOWNLOAD_URL, LATEST_RELEASE_PAGE_URL,
                                                    check_for_update)
from application.version import __version__
from application.app_classes.settings_tab import Setting
logger = logging.getLogger(__name__)

# CPA files come from external parties: never expand entities or load DTDs (XXE)
SAFE_XML_PARSER = Et.XMLParser(resolve_entities=False, no_network=True, load_dtd=False)

class CpaEditorApp(tk.Tk):
    def __init__(self):
        super().__init__()
        self.last_tab_change_time = 0
        self.debounce_delay = 0.3 # 300 milliseconds
        self.collaboration_role_tab = None
        self.setting_tab = None
        self.log_level = logging.INFO
        self.debug_enabled = tk.BooleanVar(value=False)
        self.debug_checkbutton = None
        self.help_windows = {}
        self.update_check_results = queue.Queue()
        self.update_dialog = None
        self.startup_update_check = None
        self.update_check_poll = None
        self.log_output = None
        self.certificates_tab = None
        self.validator_tab = None
        self.xml_editor_tab = None
        self.transport_tab = None
        self.comment_tab = None
        self.general_tab = None
        self.load_button = None
        self.save_button = None
        self.root = None
        self.xml_tree = None
        self.current_tab_index = 0
        self.comments_in_file_list = []
        self.width_tabs = 1650
        self.height_tabs = 800
        self.tab_control = tk.ttk.Notebook(self, width=self.width_tabs, height=self.height_tabs)
        self.tab_control.focus_set()  # Set focus to the Notebook widget initially
        self.title(f"CPA Editor v{__version__}")
        # Never open larger than the physical screen (margin for title bar / taskbar / dock)
        self.screen_width = min(1920, self.winfo_screenwidth())
        self.screen_height = min(1080, self.winfo_screenheight() - 100)
        self.geometry(f"{self.screen_width}x{self.screen_height}+0+0")
        self.minsize(800, 400)
        self.grid_propagate(False)  # Prevent automatic resizing of content_frame

        self.host = 'windows' if platform.system() == 'Windows' else 'mac' if platform.system() == 'Darwin' else ''
        # Only the notebook row/column absorbs size changes, so the log output and
        # the save/load buttons below it always stay visible
        self.grid_rowconfigure(0, weight=1)
        self.grid_columnconfigure(9, weight=1)
        self.tab_control.grid(row=0, column=0, rowspan=15, columnspan=10, sticky="nsew")
        self.root_window()
        self.create_menu()
        self.create_tabs()
        self.tab_control.bind("<<NotebookTabChanged>>", self.on_tab_change)
        self.reload_tabs = {
            'general': lambda v: self.general_tab.reload(),
            'collaboration_role': lambda v: self.collaboration_role_tab.reload(),
            'transport': lambda v: self.transport_tab.reload(),
            'other': lambda v: self.comment_tab.reload(),
            'validator': lambda v: self.validator_tab.reload(),
            'settings': lambda v: self.setting_tab.reload(),
            'certificates': lambda v: self.certificates_tab.reload(),
            'xml_editor': lambda v: self.xml_editor_tab.reload()
        }
        # self.run_function_every_minute()
        self.namespace_uri = ''
        self.validation_errors = ''
        # Look for a newer version once the window is shown
        self.startup_update_check = self.after(1000, lambda: self.check_for_updates(automatic=True))
        self.mainloop()



    def load_xml_file_dialog(self, event=None):
        # Open a file dialog to select a CPA file
        filename = filedialog.askopenfilename(filetypes=[('xml files', '.xml')])
        if filename:
            self.load_cpa_data(filename)
    
    def save_xml_file_dialog(self, event=None):
        if self.root is None:
            logger.error("No CPA loaded, nothing to save")
            return
        # Ask for the name only: the file is created in save_cpa_data, after the CPA is known to be writable
        filename = filedialog.asksaveasfilename(defaultextension=".xml", filetypes=[('xml files', '.xml')])
        if filename:
            self.save_cpa_data(filename)

    def root_window(self):
        self.log_output = tk.Text(self, height=5, wrap="word")
        self.log_output.grid(sticky="we", row=20, column=0, columnspan=10, padx=5, pady=5)

        text_handler = TextHandler(self.log_output)
        text_handler.setFormatter(logging.Formatter('%(asctime)s - %(levelname)s - %(message)s'))
        logger.addHandler(text_handler)
        logger.setLevel(self.log_level)

        self.save_button = ttk.Button(self, text="Save CPA", command=self.save_xml_file_dialog)
        self.save_button.grid(sticky='w', row=21, column=0, padx=5, pady=5)

        self.load_button = ttk.Button(self,text="Load CPA", command=self.load_xml_file_dialog)
        self.load_button.grid(sticky='w', row=21, column=1, padx=5, pady=5)

        self.debug_checkbutton = ttk.Checkbutton(self, text="Show debug messages", variable=self.debug_enabled,
                                                 command=self.toggle_debug_messages)
        self.debug_checkbutton.grid(sticky='w', row=21, column=2, padx=5, pady=5)

    def create_menu(self):
        menu_bar = tk.Menu(self)
        # The name 'help' makes Tk use the standard Help menu of the platform (macOS)
        help_menu = tk.Menu(menu_bar, name='help', tearoff=0)
        for label, help_file in (("How to use (English)", HOW_TO_USE_FILE_EN),
                                 ("Handleiding (Nederlands)", HOW_TO_USE_FILE_NL),
                                 ("Readme", README_FILE)):
            help_menu.add_command(label=label, command=lambda label=label, help_file=help_file: self.show_help(help_file, label))
        help_menu.add_separator()
        help_menu.add_command(label="Check for updates", command=self.check_for_updates)
        menu_bar.add_cascade(label="Help", menu=help_menu)
        self.config(menu=menu_bar)

    def check_for_updates(self, automatic=False):
        """
        Looks up the latest release in the background, so a slow network does not freeze the window.

        The automatic check at startup only shows something when a newer version exists.
        """
        if not automatic:
            logger.info("Checking for updates...")
        threading.Thread(target=lambda: self.update_check_results.put((automatic, check_for_update(__version__))),
                         daemon=True).start()
        self.update_check_poll = self.after(100, self.poll_update_check)

    def poll_update_check(self):
        # Tk may only be used from the main thread: the result is collected here instead of in the worker thread
        try:
            automatic, result = self.update_check_results.get_nowait()
        except queue.Empty:
            self.update_check_poll = self.after(100, self.poll_update_check)
            return
        self.show_update_result(result, automatic)

    def show_update_result(self, result, automatic=False):
        # At startup a failed check (no network) or an up-to-date version is not worth interrupting for
        quiet_log = logger.debug if automatic else None
        if 'error' in result:
            (quiet_log or logger.error)(f"Could not check for updates: {result['error']}")
        elif result['update_available']:
            logger.info(f"Version v{result['latest']} is available, this is version v{result['current']}")
            if self.ask_download(result):
                # The browser does the download: the application itself never downloads or runs a file
                webbrowser.open(LATEST_RELEASE_DOWNLOAD_URL if self.host == 'windows' else LATEST_RELEASE_PAGE_URL)
        else:
            (quiet_log or logger.info)(f"You are using the latest version (v{result['current']})")

    def destroy(self):
        # Timers of the update check must not fire after the window is gone
        for timer in (self.startup_update_check, self.update_check_poll):
            if timer is not None:
                self.after_cancel(timer)
        super().destroy()

    def ask_download(self, result):
        """Asks in a popup whether to download the newer version; returns True for Download and False for Skip."""
        dialog = tk.Toplevel(self)
        dialog.title("Update available")
        dialog.resizable(False, False)
        dialog.transient(self)
        answer = {'download': False}

        def close(download):
            answer['download'] = download
            dialog.destroy()

        tk.Label(dialog, justify="left", padx=20, pady=15,
                 text=f"Version v{result['latest']} of the CPA Editor is available.\n"
                      f"You are using version v{result['current']}.").pack()
        buttons = tk.Frame(dialog)
        buttons.pack(pady=(0, 15))
        download_button = ttk.Button(buttons, text="Download", command=lambda: close(True))
        download_button.pack(side=tk.LEFT, padx=5)
        ttk.Button(buttons, text="Skip", command=lambda: close(False)).pack(side=tk.LEFT, padx=5)
        dialog.bind("<Escape>", lambda event: close(False))
        dialog.protocol("WM_DELETE_WINDOW", lambda: close(False))
        download_button.focus_set()
        self.update_dialog = dialog
        dialog.grab_set()
        self.wait_window(dialog)
        return answer['download']

    def show_help(self, help_file, title):
        # Reuse the window of this help file when it is still open
        help_window = self.help_windows.get(help_file)
        if help_window is None or not help_window.winfo_exists():
            help_window = self.help_windows[help_file] = HelpWindow(self, help_file, title)
        help_window.lift()
        help_window.focus_set()

    def toggle_debug_messages(self):
        self.log_level = logging.DEBUG if self.debug_enabled.get() else logging.INFO
        logger.setLevel(self.log_level)

    def save_cpa_data(self, file_path):
        try:
            # Serialise the whole document (also comments before the root element) as UTF-8 bytes, so the
            # content matches the encoding in the XML declaration on every platform
            data = Et.tostring(self.root.getroottree(), encoding='UTF-8', xml_declaration=True)
            # Only open the file once there is something to write: a failed save must not empty an existing file
            with open(file_path, 'wb') as file:
                file.write(data)
            logger.info(f"CPA saved to {file_path}")
        except Exception as e:
            logger.error(f"Error while saving CPA: {e}")  # Display the error message in the error output field
    def load(self):
        self.log_output.delete('1.0', tk.END)  # Clear the error output field

    def load_cpa_data(self, file_path):

        try:
            # Parse the CPA file as an XML tree
            self.xml_tree = Et.parse(file_path, SAFE_XML_PARSER)
            self.root = self.xml_tree.getroot()

            # Get the namespace URI from the root element
            self.namespace_uri = self.root.tag.split('}')[0][1:]
            # Set the namespace prefix for the namespace URI
            Et.register_namespace("tns", self.namespace_uri)
            Et.register_namespace("xlink", "http://www.w3.org/1999/xlink")

            if len(self.root):
                # Load the data into the GUI fields
                self.load()
                self.general_tab.load()
                self.collaboration_role_tab.load()
                self.comment_tab.load()
                self.xml_editor_tab.load()
                self.transport_tab.load()
                self.certificates_tab.load()
                self.validator_tab.load()
                # self.setting_tab.load()

            logger.info("CPA data loaded successfully.")
        except Exception as e:
            # print row and column number of the error
            logger.error(f"Error while loading CPA data: {e}")

    def on_tab_change(self, event):
        """Reload the tab content when the user switches tabs."""
        current_time = time.time()
        if current_time - self.last_tab_change_time < self.debounce_delay:
            return
        self.last_tab_change_time = current_time
        self.tab_control.focus_set()
        # Get the current tab index
        self.current_tab_index = self.tab_control.index(self.tab_control.select())
        if self.root is not None:
            # Execute code based on the selected tab index
            if self.current_tab_index == 0:  # Example: General tab
                self.after(100, self.general_tab.reload)
                logger.debug("General tab selected")
            elif self.current_tab_index == 1:  # Example: Collaboration Role tab
                self.after(100, self.collaboration_role_tab.reload)
                logger.debug("Collaboration Role tab selected")
            elif self.current_tab_index == 2:  # Example: Transport tab
                self.after(100, self.transport_tab.reload)
                logger.debug("Transport tab selected")
            elif self.current_tab_index == 3:  # Example: Comment tab
                self.after(100, self.comment_tab.reload)
                logger.debug("Comment tab selected")
            elif self.current_tab_index == 4:  # Example: XML editor
                self.after(100, self.xml_editor_tab.reload)
                logger.debug("XML editor tab selected")
            elif self.current_tab_index == 5:  # Example: Certificates tab
                self.after(100, self.certificates_tab.reload)
                logger.debug("Certificates tab selected")
            elif self.current_tab_index == 6:  # Example: Validator tab
                logger.debug("Validator tab selected")
            elif self.current_tab_index == 7:  # Example: Setting tab
                logger.debug("Setting tab selected")
                # self.setting_tab.reload()

    def create_tabs(self):
        self.general_tab = General(self, logger)
        self.comment_tab = Comment(self, logger)
        self.transport_tab = Transport(self, logger)
        self.xml_editor_tab = XMLEditor(self, logger)
        self.validator_tab = Validator(self, logger)
        self.certificates_tab = Certificates(self, logger)
        self.collaboration_role_tab = CollaborationRole(self, logger)
        # self.setting_tab = Setting(self, logger)



        self.tab_control.add(self.general_tab, text="General")
        self.tab_control.add(self.collaboration_role_tab, text="Collaboration Role")
        self.tab_control.add(self.transport_tab, text="Transport")
        self.tab_control.add(self.comment_tab, text="Comment")
        self.tab_control.add(self.xml_editor_tab, text="XML editor")
        self.tab_control.add(self.certificates_tab, text="Certificates")
        self.tab_control.add(self.validator_tab, text="CPA Validation")
        # self.tab_control.add(self.setting_tab, text="Settings")

        self.general_tab.create()
        self.comment_tab.create()
        self.transport_tab.create()
        self.xml_editor_tab.create()
        self.validator_tab.create()
        self.certificates_tab.create()
        self.collaboration_role_tab.create()
        # self.setting_tab.create()
