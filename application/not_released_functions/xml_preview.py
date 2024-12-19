import sys
import tkinter as tk
from tkinter import ttk, filedialog
import xml.etree.ElementTree as ET



def load_xml():
    file_path = filedialog.askopenfilename(filetypes=[("XML files", "*.xml")])
    if file_path:
        with open(file_path, "r") as file:
            xml_content = file.read()
            try:
                xml_root = ET.fromstring(xml_content)
                populate_tree(tree, xml_root)
            except ET.ParseError:
                tree.delete(*tree.get_children())
                tree.insert("", "end", text="Invalid XML", values=("Invalid XML"))
                xml_tree = None


def populate_tree(tree, parent, parent_id=""):
    for idx, child in enumerate(parent):
        tag = child.tag
        if '}' in tag:
            _, tag = tag.split('}')
        item_id = tree.insert(parent_id, "end", text=tag, values=(child.text or "").strip(), open=True)
        for attr_name, attr_value in child.attrib.items():
            tree.insert(item_id, "end", text=attr_name, values=(attr_value,), tags=("attribute",))
        if list(child):
            populate_tree(tree, child, parent_id=item_id)


def add_item():
    selected_item = tree.selection()
    if selected_item:
        tag_or_attr = entry.get()
        value = entry_value.get()

        if add_type_var.get() == "Attribute":
            element_id = selected_item[0]
            tree.insert(element_id, "end", text=tag_or_attr, values=(value,), tags=("attribute",))

            # Update the XML tree object
            element = xml_tree.find(".//" + tag_or_attr)
            element.set(tag_or_attr, value)
        else:
            new_item_id = tree.insert(selected_item, "end", text=tag_or_attr, values=(value,))

            # Update the XML tree object
            parent_element_id = selected_item[0]
            parent_element = xml_tree.find(".//" + parent_element_id)
            new_element = ET.SubElement(parent_element, tag_or_attr)
            new_element.text = value

        entry.delete(0, tk.END)
        entry_value.delete(0, tk.END)


def delete_item():
    selected_item = tree.selection()
    if selected_item:
        element_id = selected_item[0]
        tree.delete(selected_item)

        # Update the XML tree object
        element = xml_tree.find(".//" + element_id)
        if element is not None:
            element.getparent().remove(element)

def show_menu(event):
    selected_item = tree.selection()
    type_len = len(tree.item(selected_item, 'tag'))
    text_len = len(tree.item(selected_item, 'text'))
    type = ''
    if type_len > 0:
        type = tree.item(selected_item, 'tag')[0]
    if text_len > 0:
        text = tree.item(selected_item, 'text')

    if (selected_item and 'attrib' in type) or (selected_item and text != ''):
        menu = tk.Menu(root, tearoff=0)
        menu.add_command(label="Copy Value", command=copy_value)
        menu.post(event.x_root, event.y_root)


def copy_value():
    selected_item = tree.selection()
    if selected_item:
        k = tree.item(selected_item)
        type = ''
        tag_value = ''
        value = ''
        if len(tree.item(selected_item, 'tag')) > 0 :
            type = tree.item(selected_item, 'tag')[0]
        if len(tree.item(selected_item, 'values')) > 0:
            value = tree.item(selected_item, "values")[0]
        if len(tree.item(selected_item, "text")) > 0:
            tag_value = tree.item(selected_item, "text")

        test_value = f"{tag_value} {value}"
        entry_value.delete(0, tk.END)
        entry_value.insert(0, value)
        entry.delete(0, tk.END)
        entry.insert(0, tag_value)

        root.clipboard_clear()
        root.clipboard_append(test_value)
        root.update()


def edit_cell(event):
    item = tree.selection()[0]
    column = tree.identify_column(event.x)

    if column == "#1":  # Assuming the "Value" column is the second column
        x, y, width, height = tree.bbox(item, column)
        entry = tk.Entry(tree, validate="key")
        entry.place(x=x, y=y, width=width, height=height)

        values = tree.item(item, 'values')
        if values:
            entry.insert(0, values[0])
        entry.focus_set()

        def validate_edit(P):
            new_value = entry.get()
            if new_value:
                old_values = tree.item(item, 'values')
                new_values = (new_value,)
                tree.item(item, values=new_values)
            entry.destroy()

        entry.bind("<Return>", validate_edit)
        entry.bind("<FocusOut>", validate_edit)
        entry.bind("<Escape>", lambda e: entry.destroy())


root = tk.Tk()
root.title("XML Viewer")

# Create a button to load and display XML file
load_file_button = tk.Button(root, text="Load XML File", command=load_xml)
load_file_button.pack()

# Create a Treeview widget for displaying the XML tree
tree = ttk.Treeview(root, columns=("Value"), selectmode="browse")
tree.heading("#0", text="Element")
tree.heading("Value", text="Value")
tree.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)

# Define style for attribute tags
style = ttk.Style()
style.configure("Treeview.Heading", font=("Helvetica", 10, "bold"))
style.configure("attribute.Treeview.Item", font=("Helvetica", 10, "normal"))

# Create a tag for attribute items
tree.tag_configure("attribute", font=("Helvetica", 10, "normal"))

# Create entry fields for adding new items
entry = tk.Entry(root)
entry.pack(side=tk.LEFT, padx=5, pady=5)
entry_value = tk.Entry(root)
entry_value.pack(side=tk.LEFT, padx=5, pady=5)

# Create a dropdown for selecting attribute or element
add_type_var = tk.StringVar()
add_type_var.set("Element")
add_type_dropdown = ttk.Combobox(root, textvariable=add_type_var, values=["Element", "Attribute"])
add_type_dropdown.pack(side=tk.LEFT, padx=5, pady=5)

# Create buttons for adding and deleting items
add_button = tk.Button(root, text="Add", command=add_item)
add_button.pack(side=tk.LEFT, padx=5, pady=5)
delete_button = tk.Button(root, text="Delete", command=delete_item)
delete_button.pack(side=tk.LEFT, padx=5, pady=5)

# Create buttons for editing and saving values


tree.bind("<Double-1>", edit_cell)  # Bind double-click event to the Treeview

# XML tree object
xml_tree = None

MACOS = False
if sys.platform == 'darwin':
    MACOS = True

# Bind the Treeview widget to the right-click menu
if MACOS:
    tree.bind("<Button-2>", show_menu)
else:
    tree.bind("<Button-3>", show_menu)

root.mainloop()
