import uuid
import tkinter as tk
from tkinter import ttk
from application.helper_classes.cpaParser import CPAParser


class CollaborationRole(tk.Frame):
    def __init__(self, master, logger, **kw):
        super().__init__(master, **kw)
        self.menu = None
        self.xml_element_mapping = {}  # Map Treeview items to XML elements
        self.partner_a = None
        self.partner_b = None
        self.collaboration_items_partner_b = None
        self.collaboration_items_partner_a = None
        self.tree_editor = None  # Treeview widget
        self.master = master
        self.logger = logger

    def reload(self):
        """Reload the transport roles and data."""
        self.load()

    def load(self):
        """Load transport items for both partners and populate the Treeview."""
        self.xml_element_mapping = {}
        cpa_parser = CPAParser(self.master.root)

        # Get transport item data for both partners
        self.partner_a = cpa_parser.get_party_name_partner_a()
        self.partner_b = cpa_parser.get_party_name_partner_b()
        self.collaboration_items_partner_a = cpa_parser.get_collaboration_roles_partner_a()
        self.collaboration_items_partner_b = cpa_parser.get_collaboration_roles_partner_b()

        self.tree_editor.delete(*self.tree_editor.get_children())  # Clear existing Treeview items

        # Populate Treeview with data for Partner A
        if self.collaboration_items_partner_a:
            for item in self.collaboration_items_partner_a:
                self.populate_tree(self.tree_editor, parent=item, header_value=f'Partner {self.partner_a}',
                                   headers=True, color_background="blue")

        # Populate Treeview with data for Partner B
        if self.collaboration_items_partner_b:
            for item in self.collaboration_items_partner_b:
                self.populate_tree(self.tree_editor, parent=item, header_value=f'Partner {self.partner_b}',
                                   headers=True, color_background="red")

    def clear_tree(self):
        """Clear the Treeview of all items."""
        self.tree_editor.delete(*self.tree_editor.get_children())

    def populate_tree(self, tree, parent=None, parent_id="", tag_indices=None, header_value=None, headers=False,
                      color_background=""):
        """Recursively populate the Treeview with XML elements and attributes."""
        if headers:
            # Insert a header for each transport partner (A or B)
            self.insert_header(tree, header_value, color_background)

        if parent is not None:
            tag = self.clean_tag_name(parent.tag)  # Clean tag name (removes namespace)
            tag_indices = self.update_tag_indices(tag_indices, tag, parent_id)

            item_id = tree.insert(parent_id, "end", text=tag, open=True)
            self.xml_element_mapping[item_id] = parent  # Map item ID to XML element

            # Insert attributes of the transport element into the tree
            self.insert_attributes(tree, parent, item_id)

            # Insert text values, if available
            self.insert_text(tree, parent, item_id)

            # Recursively insert child elements
            for child in parent:
                self.populate_tree(tree, child, parent_id=item_id, tag_indices=tag_indices)

    def insert_header(self, tree, header_value, color_background):
        """Insert a header row in the Treeview."""
        header_tag = f"header_color_{uuid.uuid4()}"
        item_id = tree.insert("", "end", text=header_value, open=True, tags=(f"{header_tag}",))
        tree.tag_configure(f"{header_tag}", background=color_background)
        self.xml_element_mapping[item_id] = None

    def clean_tag_name(self, tag):
        """Remove namespace from the tag name."""
        if '}' in tag:
            tag = tag.split('}', 1)[1]  # Extract the actual tag name after the '}'
        return tag

    def clean_attr_name(self, attr_name):
        """Remove namespace from the attribute name."""
        if '}' in attr_name:
            attr_name = attr_name.split('}', 1)[1]  # Extract the actual attribute name after the '}'
        return attr_name

    def update_tag_indices(self, tag_indices, tag, parent_id):
        """Update tag indices to manage XML hierarchy."""
        if tag_indices is None:
            tag_indices = {}

        if tag not in tag_indices:
            tag_indices[tag] = []
        tag_indices[tag].append(self.tree_editor.index(parent_id))
        return tag_indices

    def insert_attributes(self, tree, parent, item_id):
        """Insert attributes of the transport role element."""
        for attr_name, attr_value in parent.attrib.items():
            # Clean the attribute name to remove namespaces
            cleaned_attr_name = self.clean_attr_name(attr_name)

            # Insert the cleaned attribute into the Treeview
            attr_item_id = tree.insert(item_id, "end", text=cleaned_attr_name, values=(attr_value,),
                                       tags=("attribute",))
            self.xml_element_mapping[attr_item_id] = parent  # Map attribute item ID to XML element

    def insert_text(self, tree, parent, item_id):
        """Insert text value of the transport role element, if it exists."""
        if parent.text and parent.text.strip():
            tree.item(item_id, values=(parent.text.strip(),))

    def clone_element(self):
        """Clone the selected XML element and insert it into the XML tree."""
        item = self.tree_editor.selection()[0]
        if item:
            xml_element = self.xml_element_mapping.get(item)
            element = self.master.root.find(".//" + xml_element.tag)
            parent = element.getparent()
            parent.insert(parent.index(xml_element) + 1, xml_element)
            self.reload()

    def popup(self, event):
        """Display a right-click context menu for treeview actions."""
        self.menu = tk.Menu(self, tearoff=0)
        self.menu.add_command(label="Clone", command=self.clone_element)
        self.menu.post(event.x_root, event.y_root)

    def create(self):
        """Create the Treeview widget for displaying transport role data."""
        self.tree_editor = ttk.Treeview(self, columns=("Partner", "Value"), selectmode="browse")
        self.tree_editor.heading("#0", text="Element")
        self.tree_editor.heading("Value", text="Value")
        self.vsb = ttk.Scrollbar(self, orient="vertical", command=self.tree_editor.yview)
        self.tree_editor.configure(yscrollcommand=self.vsb.set)
        self.vsb.pack(side=tk.RIGHT, fill=tk.Y)
        self.tree_editor.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)

        # Bind right-click context menu to Treeview
        if self.master.host == 'mac':
            self.tree_editor.bind("<Button-2>", self.popup)
        else:
            self.tree_editor.bind("<Button-3>", self.popup)
