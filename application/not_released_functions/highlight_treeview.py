import tkinter as tk
from tkinter import ttk


class ColoredTreeview(ttk.Treeview):
    def __init__(self, master=None, **kwargs):
        ttk.Treeview.__init__(self, master, **kwargs)
        self._row_colors = {}  # Dictionary to store row colors

    def tag_configure(self, tag_name, **kwargs):
        # Store row color information
        self._row_colors[tag_name] = kwargs.get('background', 'white')
        ttk.Treeview.tag_configure(self, tag_name, **kwargs)

    def _draw_row_background(self, event=None):
        for item in self.get_children():
            tag_name = self.item(item, 'tags')[0]
            color = self._row_colors.get(tag_name, 'white')
            self.item(item, tag_name, background=color)
        self.after(100, self._draw_row_background)


class YourApp:
    def __init__(self):
        self.root = tk.Tk()
        self.root.title("Change Background Color of Row Example")

        # Create a ColoredTreeview widget
        self.tree_editor = ColoredTreeview(self.root, columns=("Value",))
        self.tree_editor.heading("#0", text="Tag or Attribute")
        self.tree_editor.heading("Value", text="Value")

        self.tree_editor.pack(fill="both", expand=True)

        # Example data
        data = [
            {"tag_or_attr": "tag1", "value": "value1"},
            {"tag_or_attr": "tag2", "value": "value2"},
            {"tag_or_attr": "attr1", "value": "value3"},
        ]

        # Insert items into the Treeview and configure tags
        for item_data in data:
            item_id = self.tree_editor.insert("", "end", text=item_data["tag_or_attr"])
            self.tree_editor.set(item_id, "Value", item_data["value"])
            self.tree_editor.tag_configure(item_id, background="white")  # Set default background color

        # Button to change background color of the second row
        change_color_button = tk.Button(self.root, text="Change Color", command=self.change_row_color)
        change_color_button.pack()

    def change_row_color(self):
        # Get the item ID of the second row (index starts from 1)
        item_id = self.tree_editor.get_children()[1]

        # Configure the tag to change the background color
        self.tree_editor.tag_configure(item_id, background="yellow")
        self.tree_editor.tag_configure(item_id, foreground="black")  # Optionally, set text color

    def run(self):
        self.root.mainloop()


if __name__ == "__main__":
    app = YourApp()
    app.run()
