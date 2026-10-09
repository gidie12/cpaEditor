

import xml.etree.ElementTree as Et

import tkinter as tk
from tkinter import ttk


def clear_tree(tree):
    if len(tree.get_children()) > 0:
        for item in tree.get_children():
            tree.delete(item)


class XMLEditor(tk.Frame):
    def __init__(self, master, logger, **kw):
        super().__init__(master, **kw)
        self.vsb = None
        self.delete_button = None
        self.add_button = None
        self.add_type_dropdown = None
        self.xml_entry_value = None
        self.tree_editor = None
        self.add_type_var = None
        self.xml_editor_entry = None
        self.xml_element_mapping = {}
        self.master = master
        self.logger = logger

    def reload(self):
        clear_tree(self.tree_editor)
        self.load()

    def load(self):
        self.xml_element_mapping = {}
        self.populate_tree(self.tree_editor, self.master.root)


    def create(self):

        # Create a Treeview widget for displaying the XML tree
        self.tree_editor = ttk.Treeview(self, columns="Value", selectmode="browse")
        self.tree_editor.heading("#0", text="Element")
        self.tree_editor.heading("Value", text="Value")
        self.vsb = ttk.Scrollbar(self, orient="vertical", command=self.tree_editor.yview)
        self.tree_editor.configure(yscrollcommand=self.vsb.set)
        self.vsb.pack(side=tk.RIGHT, fill=tk.Y)
        self.tree_editor.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)
        # Define style for attribute tags
        style = ttk.Style()
        style.configure("Treeview.Heading", font=("Helvetica", 10, "bold"))
        style.configure("attribute.Treeview.Item", font=("Helvetica", 10, "normal"))

        # Create a tag for attribute items
        self.tree_editor.tag_configure("attribute", font=("Helvetica", 10, "normal"))

        # Create entry fields for adding new items
        # Small requested width + expand: the fields scale with the window
        self.xml_editor_entry = tk.Entry(self, width=10)
        self.xml_editor_entry.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=5, pady=5)
        self.xml_entry_value = tk.Entry(self, width=10)
        self.xml_entry_value.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=5, pady=5)

        # Create a dropdown for selecting attribute or element
        self.add_type_var = tk.StringVar()
        self.add_type_var.set("Element")
        self.add_type_dropdown = ttk.Combobox(self, textvariable=self.add_type_var, values=["Element", "Attribute"])
        self.add_type_dropdown.pack(side=tk.LEFT, padx=5, pady=5)

        # Create buttons for adding and deleting items
        self.add_button = tk.Button(self, text="Add", command=self.add_item)
        self.add_button.pack(side=tk.LEFT, padx=5, pady=5)
        self.delete_button = tk.Button(self, text="Delete", command=self.delete_item)
        self.delete_button.pack(side=tk.LEFT, padx=5, pady=5)

        # Create buttons for editing and saving values

        self.tree_editor.bind("<Double-1>", self.edit_cell)  # Bind double-click event to the Treeview


        # Bind the Treeview widget to the right-click menu
        if self.master.host == 'mac':
            self.tree_editor.bind("<Button-2>", self.show_menu)
        else:
            self.tree_editor.bind("<Button-3>", self.show_menu)

    def show_menu(self, event):
        selected_item = self.tree_editor.selection()
        type_len = len(self.tree_editor.item(selected_item, 'tag'))
        text_len = len(self.tree_editor.item(selected_item, 'text'))
        type = ''
        text = ''
        if type_len > 0:
            type = self.tree_editor.item(selected_item, 'tag')[0]
        if text_len > 0:
            text = self.tree_editor.item(selected_item, 'text')

        if (selected_item and 'attrib' in type) or (selected_item and text != ''):
            menu = tk.Menu(self, tearoff=0)
            menu.add_command(label="Copy Value", command=self.copy_value)
            menu.post(event.x_root, event.y_root)

    def populate_tree(self, tree, parent=None, parent_id="", tag_indices=None):
        if parent is not None:
            tag = parent.tag
            if '}' in tag:
                _, tag = tag.split('}')

            if tag_indices is None:
                tag_indices = {}

            if tag not in tag_indices:
                tag_indices[tag] = []
            tag_indices[tag].append(tree.index(parent_id))

            item_id = tree.insert(parent_id, "end", text=tag, open=True)
            self.xml_element_mapping[item_id] = parent  # Map Treeview item ID to XML element

            # Display root attributes
            for attr_name, attr_value in parent.attrib.items():
                attr_item_id = tree.insert(item_id, "end", text=attr_name, values=(attr_value,), tags=("attribute",))
                self.xml_element_mapping[attr_item_id] = parent  # Map Treeview attribute item ID to XML element

            if parent.text and parent.text.strip():
                tree.item(item_id, values=(parent.text.strip(),))

            for child in parent:
                self.populate_tree(tree, child, parent_id=item_id, tag_indices=tag_indices)

    def add_item(self):
        item = self.tree_editor.selection()[0]
        if item:
            tag_or_attr = self.xml_editor_entry.get()
            value = self.xml_entry_value.get()
            xml_element = self.xml_element_mapping.get(item)

            if self.add_type_var.get() == "Attribute":
                self.tree_editor.insert(item, "end", text=tag_or_attr, values=(value,), tags=("attribute",))

                self.add_xml_object('Attribute', tag_or_attr, xml_element, value)
                self.reload()
            else:
                self.tree_editor.insert(item, "end", text=tag_or_attr, values=(value,))
                self.add_xml_object('Element', tag_or_attr, xml_element, value)
                self.reload()


            self.xml_editor_entry.delete(0, tk.END)
            self.xml_entry_value.delete(0, tk.END)


    def delete_item(self):
        item = self.tree_editor.selection()[0]
        if item:
            name = self.tree_editor.item(item, 'text')
            xml_element = self.xml_element_mapping.get(item)
            type = 'Attribute' if 'attribute' in self.tree_editor.item(item, 'tags') else 'Element'
            # Update the XML tree object
            self.delete_xml_object(type,name, xml_element, item)
            self.tree_editor.delete(item)

    def copy_value(self):
        selected_item = self.tree_editor.selection()
        if selected_item:
            k = self.tree_editor.item(selected_item)
            type = ''
            tag_value = ''
            value = ''
            if len(self.tree_editor.item(selected_item, 'tag')) > 0:
                type = self.tree_editor.item(selected_item, 'tag')[0]
            if len(self.tree_editor.item(selected_item, 'values')) > 0:
                value = self.tree_editor.item(selected_item, "values")[0]
            if len(self.tree_editor.item(selected_item, "text")) > 0:
                tag_value = self.tree_editor.item(selected_item, "text")

            clipboard_value = f"{tag_value} {value}"
            self.xml_entry_value.delete(0, tk.END)
            self.xml_entry_value.insert(0, value)
            self.xml_editor_entry.delete(0, tk.END)
            self.xml_editor_entry.insert(0, tag_value)

            self.clipboard_clear()
            self.clipboard_append(clipboard_value)
            self.update()

    def edit_cell(self, event):
        item = self.tree_editor.selection()[0]
        column = self.tree_editor.identify_column(event.x)

        if column == "#1":  # Assuming the "Value" column is the second column
            x, y, width, height = self.tree_editor.bbox(item, column)
            entry = tk.Entry(self.tree_editor, validate="key")
            entry.place(x=x, y=y, width=width, height=height)

            values = self.tree_editor.item(item, 'values')
            if values:
                entry.insert(0, values[0])
            entry.focus_set()

            def validate_edit(P):
                new_value = entry.get()
                if new_value:
                    old_values = self.tree_editor.item(item, 'values')
                    name = self.tree_editor.item(item, 'text')
                    new_values = (new_value,)
                    xml_element = self.xml_element_mapping.get(item)
                    type = 'Attribute' if 'attribute' in self.tree_editor.item(item, 'tags') else 'Element'
                    self.change_xml_object(type, name, xml_element, new_value)
                    self.tree_editor.item(item, values=new_values)
                entry.destroy()

            entry.bind("<Return>", validate_edit)
            entry.bind("<FocusOut>", validate_edit)
            entry.bind("<Escape>", lambda e: entry.destroy())

    def change_xml_object(self, type, name, xml_element, new_value):
        if type == "Attribute":
            xml_element.attrib[name] = new_value
            x = xml_element
        elif type == "Element":
            xml_element.text = new_value

    def delete_xml_object(self, type, name, xml_element, item_id):
        if type == "Attribute":
            del xml_element.attrib[name]
        elif type == "Element":
            parent_id = self.tree_editor.parent(item_id)
            parent_element = self.xml_element_mapping.get(parent_id)
            if parent_element is not None:
                parent_element.remove(xml_element)
        del self.xml_element_mapping[item_id]

    def add_xml_object(self, type, name, xml_element, value):
        if type == "Attribute":
            xml_element.attrib[name] = value
        elif type == "Element":
            name = '{' + self.master.namespace_uri + '}' + name
            new_element = Et.Element(f"{name}")
            new_element.text = value

            parent = list(xml_element)
            target_elements = xml_element.findall(name)

            if target_elements is not None and len(target_elements) > 0:
                xml_element.insert(parent.index(target_elements[-1]) + 1, new_element)
            else:
                xml_element.append(new_element)
