import uuid
import tkinter as tk
from tkinter import ttk
from tkinter import simpledialog
from application.helper_classes.cpaParser import CPAParser

class Transport(tk.Frame):
    def __init__(self, master, logger, **kw):
        super().__init__(master, **kw)
        self.vsb = None
        self.options_combobox = None
        self.save_button = None
        self.field_label = None
        self.attribute_value_entry = None
        self.menu = None
        self.security_id_partner_b = None
        self.security_id_partner_a = None
        self.cert_ids_partner_a = None
        self.cert_ids_partner_b = None
        self.xml_element_mapping = {}  # Map Treeview items to XML elements
        self.partner_a = None
        self.partner_b = None
        self.transport_items_partner_a = None
        self.transport_items_partner_b = None
        self.tree_editor = None  # Treeview widget
        self.master = master
        self.logger = logger
        self.cpa_namespace = 'http://www.oasis-open.org/committees/ebxml-cppa/schema/cpp-cpa-2_0.xsd'

        # Keep track of the selected item for editing
        self.selected_item = None
        self.selected_field_name = None  # To store the name of the field being edited
        self.selected_cert_ids = []  # Store certId values for selection

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
        self.transport_items_partner_a = cpa_parser.get_transport_items_partner_a()
        self.transport_items_partner_b = cpa_parser.get_transport_items_partner_b()
        self.cert_ids_partner_a = cpa_parser.get_cert_ids_partner_a()
        self.cert_ids_partner_b = cpa_parser.get_cert_ids_partner_b()
        self.security_id_partner_a = cpa_parser.get_security_ids_partner_a()
        self.security_id_partner_b = cpa_parser.get_security_ids_partner_b()

        self.tree_editor.delete(*self.tree_editor.get_children())  # Clear existing Treeview items

        # Populate Treeview with data for Partner A
        if self.transport_items_partner_a:
            for item in self.transport_items_partner_a:
                self.populate_tree(self.tree_editor, parent=item, header_value=f'Partner {self.partner_a}',
                                   headers=True, color_background="blue", parent_tag=f"{self.partner_a}")

        # Populate Treeview with data for Partner B
        if self.transport_items_partner_b:
            for item in self.transport_items_partner_b:
                self.populate_tree(self.tree_editor, parent=item, header_value=f'Partner {self.partner_b}',
                                   headers=True, color_background="red", parent_tag=f"{self.partner_b}")

    def clear_tree(self):
        """Clear the Treeview of all items."""
        self.tree_editor.delete(*self.tree_editor.get_children())

    def populate_tree(self, tree, parent=None, parent_id="", tag_indices=None, header_value=None, headers=False,
                      color_background="", parent_tag=""):
        """Recursively populate the Treeview with XML elements and attributes."""
        if headers:
            # Insert a header for each transport partner (A or B)
            self.insert_header(tree, header_value, color_background)

        if parent is not None:
            tag = self.clean_tag_name(parent.tag)  # Clean tag name (removes namespace)
            tag_indices = self.update_tag_indices(tag_indices, tag, parent_id)

            item_id = tree.insert(parent_id, "end", text=tag, open=True, tags=(parent_tag,))
            self.xml_element_mapping[item_id] = parent  # Map item ID to XML element

            # Insert attributes of the transport element into the tree
            self.insert_attributes(tree, parent, item_id, parent_tag)

            # Insert text values, if available
            self.insert_text(tree, parent, item_id)

            # Recursively insert child elements
            for child in parent:
                self.populate_tree(tree, child, parent_id=item_id, tag_indices=tag_indices, parent_tag=(parent_tag,))

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

    def insert_attributes(self, tree, parent, item_id, parent_tag=""):
        """Insert attributes of the transport role element."""
        for attr_name, attr_value in parent.attrib.items():
            # Clean the attribute name to remove namespaces
            cleaned_attr_name = self.clean_attr_name(attr_name)

            # Insert the cleaned attribute into the Treeview
            attr_item_id = tree.insert(item_id, "end", text=cleaned_attr_name, values=(attr_value,),
                                       tags=("attribute",parent_tag,))
            self.xml_element_mapping[attr_item_id] = parent  # Map attribute item ID to XML element

    def insert_text(self, tree, parent, item_id):
        """Insert text value of the transport role element, if it exists."""
        if parent.text and parent.text.strip():
            tree.item(item_id, values=(parent.text.strip(),))

    def save_changes(self):
        """Save the changes made in the entry field to the XML element and Treeview."""
        if self.selected_item:
            # Retrieve the updated value from the text entry
            updated_value = self.attribute_value_entry.get()

            # If it's a certId, save the selected certId from the ComboBox
            if self.selected_field_name == "certId" or self.selected_field_name == "securityId":
                updated_value = self.options_combobox.get()

            # Get the corresponding XML element
            xml_element = self.xml_element_mapping.get(self.selected_item)

            if xml_element is not None:
                type = 'Attribute' if 'attribute' in self.tree_editor.item(self.selected_item, 'tags') else 'Element'
                name = '{' + self.cpa_namespace + '}' + self.selected_field_name
                self.change_xml_object(type, name, xml_element, updated_value)
                # self.tree_editor.item(self.selected_item, values=new_values)
                self.tree_editor.item(self.selected_item, values=(updated_value,))

    def change_xml_object(self, type, name, xml_element, new_value):
        """Change the value of an XML object based on the type (attribute or element)."""
        if type == 'Attribute':
            xml_element.attrib[name] = new_value
        else:
            xml_element.text = new_value

    def on_treeview_select(self, event):
        """Handle the event when an item (attribute) is selected."""
        selected_item = self.tree_editor.selection()
        if selected_item:
            # Retrieve the selected attribute value
            item = selected_item[0]
            values = self.tree_editor.item(item, "values")

            tags = self.tree_editor.item(item, "tags")

            # If it is an attribute, fill the value into the Entry
            if values:
                self.selected_item = item  # Save the selected item for later updates
                self.selected_field_name = self.tree_editor.item(item, "text")  # Save the field name (tag or attribute)

                # Update the label to show which field is being edited
                self.field_label.config(text=f"Editing: {self.selected_field_name}")

                partner_name = self.get_partner_name_form_tags()

                if self.selected_field_name in ["certId", "securityId"]:
                    if partner_name:
                        if self.selected_field_name == "certId":
                            self.load_cert_ids(partner_name)
                        else:
                            self.load_security_ids(partner_name)
                        self.options_combobox.config(state="readonly")
                        self.attribute_value_entry.config(state="disabled")
                        self.attribute_value_entry.delete(0, tk.END)
                        if values[0] in self.options_combobox['values']:
                            self.options_combobox.set(values[0])
                        else:
                            self.options_combobox.set(self.options_combobox['values'][0])
                else:
                    # disable the combobox if it's not a certId or securityId
                    self.options_combobox.config(state="disabled")
                    self.options_combobox.set("No options")  # Reset selection
                    self.attribute_value_entry.config(state="normal")
                    self.attribute_value_entry.delete(0, tk.END)
                    self.attribute_value_entry.insert(0, values[0])  # Fill the value in the entry box

    def load_security_ids(self, partner_name):
        """Load securityId values for a given partner."""
        # Simulating fetching securityId values from the XML or other data source
        security_ids = self.fetch_security_ids_for_partner(partner_name)

        # Update the certId combo box with available certIds
        if security_ids:
            self.options_combobox['values'] = security_ids
            self.options_combobox.pack(pady=5)
        else:
            self.options_combobox.set("No securityIds available")  # Default message

    def get_partner_name_form_tags(self):
        """Determine which partner's certIds should be loaded based on the selection."""
        # If we're editing a certId, we need to know which partner to fetch certId values for
        selected_item = self.tree_editor.selection()[0]
        parent_item = self.tree_editor.parent(selected_item)

        tags = self.tree_editor.item(selected_item, "tags")

        if self.partner_a in tags:
            return self.partner_a
        elif self.partner_b in tags:
            return self.partner_b
        else:
            self.logger.error("Could not determine partner name for certId")

    def load_cert_ids(self, partner_name):
        """Load certId values for a given partner."""
        # Simulating fetching certId values from the XML or other data source
        cert_ids = self.fetch_cert_ids_for_partner(partner_name)

        # Update the certId combo box with available certIds
        if cert_ids:
            self.options_combobox['values'] = cert_ids
            self.options_combobox.pack(pady=5)
        else:
            self.options_combobox.set("No certIds available")  # Default message

    def fetch_cert_ids_for_partner(self, partner_name):
        """Fetch the available certIds from the XML for the specified partner."""
        return self.cert_ids_partner_a if partner_name == self.partner_a else self.cert_ids_partner_b

    def fetch_security_ids_for_partner(self, partner_name):
        """Fetch the available securityIds from the XML for the specified partner."""
        return self.security_id_partner_a if partner_name == self.partner_a else self.security_id_partner_b

    def popup(self, event):
        """Display a right-click context menu for treeview actions."""
        self.menu = tk.Menu(self, tearoff=0)
        self.menu.add_command(label="Clone", command=self.clone_element)
        self.menu.post(event.x_root, event.y_root)

    def clone_element(self):
        """Clone the selected XML element and insert it into the XML tree."""
        item = self.tree_editor.selection()[0]
        if item:
            xml_element = self.xml_element_mapping.get(item)
            element = self.master.root.find(".//" + xml_element.tag)
            parent = element.getparent()
            parent.insert(parent.index(xml_element) + 1, xml_element)
            self.reload()

    def add_transport_item(self):
        """Generate new transport item."""
        new_row = len(self.tree_editor.get_children())
        label_transport_id = tk.Label(self, text=f"Transport ID {new_row}:")
        entry_transport_id = tk.Entry(self)
        label_transport_id.pack(padx=5, pady=5)
        entry_transport_id.pack(padx=5, pady=5)

        label_transport_type = tk.Label(self, text=f"Transport Type {new_row}:")
        entry_transport_type = tk.Entry(self)
        label_transport_type.pack( padx=5, pady=5)
        entry_transport_type.pack(padx=5, pady=5)

        self.logger.debug(f"New transport item added: ID {new_row}, Type {new_row}")

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

        # Bind the Treeview select event to fill the value in the entry field
        self.tree_editor.bind("<<TreeviewSelect>>", self.on_treeview_select)
        # Text field to show the attribute value when clicked

        # Label to display which field is being edited
        self.field_label = tk.Label(self, text="Select a field to edit")
        self.field_label.pack(side=tk.LEFT, padx=5, pady=5)

        self.attribute_value_entry = tk.Entry(self, width=40)
        self.attribute_value_entry.pack(side=tk.LEFT,after=self.field_label, padx=5, pady=5)
        self.attribute_value_entry.config(state="disabled")
        # self.entry_party_id_partner_a.grid(sticky="w", row=2, column=1, padx=5, pady=5)


        # ComboBox to display certId options for selection
        self.options_combobox = ttk.Combobox(self, state="disabled", width=40)
        self.options_combobox.set("No options")  # Reset selection
        self.options_combobox.pack(side=tk.LEFT, padx=5, pady=5)

        # Add a button to save changes
        self.save_button = tk.Button(self, text="Save Changes", command=self.save_changes)
        self.save_button.pack(side=tk.LEFT, padx=5, pady=5)

        # Add a button to generate new transport items
        self.add_transport_button = tk.Button(self, text="Add Transport", command=self.add_transport_item)
        self.add_transport_button.pack(side=tk.LEFT, padx=5, pady=5)