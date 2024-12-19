import tkinter as tk
from tkinter import ttk


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
root.title("Editable Treeview")

# Create a Treeview widget
tree = ttk.Treeview(root, columns=("Value"))
tree.heading("#0", text="Element")
tree.heading("Value", text="Value")
tree.pack(fill=tk.BOTH, expand=True)

# Insert sample data
item1 = tree.insert("", "end", text="Tag1", values=("Value1"))
item2 = tree.insert("", "end", text="Tag2", values=("Value2"))

tree.bind("<Double-1>", edit_cell)  # Bind double-click event to the Treeview

root.mainloop()
