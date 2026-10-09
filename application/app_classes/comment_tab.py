import logging
# import xml.etree.ElementTree as Et
from lxml import etree as Et

import tkinter as tk
from application.helper_classes.cpaCreator import CPACreator
from application.helper_classes.cpaParser import CPAParser
from tkinter import ttk

from application.helper_classes.resizeApplication import update_entry_width


class Comment(tk.Frame):
    """
    Initialize the Comment tab.

    Args:
        master (tk.Tk): The master Tkinter window.
        logger (logging.Logger): The logger instance.
        **kw: Additional keyword arguments.
    """

    def __init__(self, master, logger, **kw):
        super().__init__(master, **kw)
        self.label_namespace = None
        self.selected_index = None
        self.comments_objects = None
        self.listbox = None
        self.entry_fields = []
        self.attribute_language = None
        self.delete_comment_button = None
        self.update_button = None
        self.new_comment_button = None
        self.comment_entry = None
        self.comments_in_file_list = None
        self.cpa_dict_entries = None
        self.master = master
        self.logger = logger
        self.xml_element_mapping = {}

    def on_select(self,event):
        """
        Handle the selection of an item in the listbox.

        Args:
            event (tk.Event): The event object.
        """
        selected_indices = self.listbox.curselection()
        if selected_indices:
            self.selected_index = selected_indices[0]
            self.comment_entry.delete(0, tk.END)
            self.comment_entry.insert(0, self.listbox.get(self.selected_index))

    def reload(self):
        """
           Reload the comments from the XML file.
           """
        self.load()

    def load(self):
        """
               Load the comments from the XML file and populate the listbox.

               Tests:
                   - loads_comments_correctly
        """
        update_entry_width(self.entry_fields, self.master.winfo_screenwidth())
        # Get all comments from the XML file
        cpa_parser_functions = CPAParser(self.master.root)
        self.comments_objects = cpa_parser_functions.get_comments()

        self.listbox.delete(0, tk.END)
        self.xml_element_mapping = {}
        # The selection points into the previous list, it must not survive a (re)load
        if self.selected_index is not None:
            self.comment_entry.delete(0, tk.END)
        self.selected_index = None
        
        x = 0
        for comment in self.comments_objects:
            self.listbox.insert(tk.END, comment.text)
            self.xml_element_mapping[x] = comment
            x = x + 1

    def focus_entry(self,event):
        """
                Set focus to the comment entry field.

                Args:
                    event (tk.Event): The event object.
                """
        self.comment_entry.focus_set()

    def delete_item(self):
        """
        Delete the selected comment from the listbox and XML file.

        Tests:
            - deletes_comment_correctly
            - handles_no_selection_on_delete
        """
        if self.selected_index is not None:
            deleted_index = self.selected_index
            delete_item = {'deleteXMLElement': [self.xml_element_mapping[deleted_index]]}
            CPACreator(self.master.root, self.logger).apply_changes(delete_item)
            # reorganize the mapping
            del self.xml_element_mapping[deleted_index]
            new_xml_element_mapping = {}
            for key in self.xml_element_mapping:
                if key > deleted_index:
                    # new key is the old key - 1
                    new_xml_element_mapping[key - 1] = self.xml_element_mapping[key]
                else:
                    new_xml_element_mapping[key] = self.xml_element_mapping[key]
            self.xml_element_mapping = new_xml_element_mapping

            self.listbox.delete(deleted_index)

            if self.listbox.size() > 0:
                new_index = min(deleted_index, self.listbox.size() - 1)
                self.listbox.select_set(new_index)
                self.on_select(None)  # Update entry field with new selection


    def update_item(self):
        """
              Update the selected comment in the listbox and XML file.

              Tests:
                  - updates_comment_correctly
                  - handles_no_selection_on_update
              """
        self.logger.debug(f"Comment selected: index {self.selected_index}")
        if self.selected_index is not None and self.comment_entry.get():
            self.listbox.delete(self.selected_index)
            self.listbox.insert(self.selected_index, self.comment_entry.get())
            change_entry = {'changeXMLElement': [self.comment_entry.get(), self.xml_element_mapping[self.selected_index]]}
            cpa_creator = CPACreator(self.master.root, self.logger)
            cpa_creator.apply_changes(change_entry)

            
    def add_item(self):
        """
              Add a new comment to the listbox and XML file.

              Tests:
                  - adds_comment_correctly
                  - handles_empty_comment_entry
              """
        if len(self.comment_entry.get()) > 0:
            self.listbox.insert(tk.END,self.comment_entry.get())

            comment_object = Et.element = Et.Element('{http://www.oasis-open.org/committees/ebxml-cppa/schema/cpp-cpa-2_0.xsd}Comment')
            comment_object.text = self.comment_entry.get()
            comment_object.set('{http://www.w3.org/XML/1998/namespace}type', 'nl-NL')
            element = {'addXMLElement': [comment_object, self.master.root]}
            CPACreator(self.master.root, self.logger).apply_changes(element)
            self.xml_element_mapping[self.listbox.size() - 1] = element
            self.comment_entry.delete(0, tk.END)

    def create(self):
        """
        Create the UI elements for the Comment tab.
        """
        self.listbox = tk.Listbox(self, selectmode=tk.SINGLE, width=200)

        self.label_namespace = tk.Label(self, text="Comments :")
        self.label_namespace.grid(sticky="w", row=0, column=0, columnspan=6, padx=5, pady=5)

        self.listbox.grid(row=1, column=1, rowspan=4, columnspan=8, sticky="nsew")
        self.listbox.bind("<<ListboxSelect>>", self.on_select)
        self.listbox.bind("<ButtonRelease-1>", self.focus_entry)  # Set focus to entry after clicking
        self.comment_entry = tk.Entry(self, width=50)
        self.entry_fields.append(self.comment_entry)

        self.attribute_language = ttk.Combobox(self, values=["nl-NL", "en-us"], width=5)
        # self.attribute_language.bind("<<ComboboxSelected>>", self.on_select(self, self.attribute_language))

        self.comment_entry.grid(row=5, column=1, columnspan=6, sticky="nesw")
        self.attribute_language.grid(row=5, column=7, columnspan=2, sticky="nesw")

        self.update_button = tk.Button(self, text="Update", command=self.update_item, width=10)
        self.update_button.grid(row=6, column=3, sticky="w", columnspan=2, padx=5, pady=5)

        self.delete_comment_button = tk.Button(self, text="Delete", command=self.delete_item, width=10)
        self.delete_comment_button.grid(row=6, column=5, sticky="w",columnspan=2, padx=5, pady=5)

        self.new_comment_button = tk.Button(self, text="New", command=self.add_item, width=10)
        self.new_comment_button.grid(row=6, column=1, sticky="w",columnspan=2, padx=5, pady=5)

        # Let the listbox and entry columns scale with the window
        for column in range(1, 9):
            self.grid_columnconfigure(column, weight=1)

