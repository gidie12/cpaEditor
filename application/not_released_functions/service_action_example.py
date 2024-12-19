import tkinter as tk
import json

def add_to_list():
    service = service_entry.get()
    action = action_entry.get()
    if service and action:
        item = {"Service": service, "Action": action}
        item_list.insert(tk.END, json.dumps(item))
        service_entry.delete(0, tk.END)
        action_entry.delete(0, tk.END)

def delete_item():
    selected_index = item_list.curselection()
    if selected_index:
        deleted_index = selected_index[0]
        item_list.delete(deleted_index)

        if item_list.size() > 0:
            new_index = min(deleted_index, item_list.size() - 1)
            item_list.select_set(new_index)
            # on_select(None)  # Update entry field with new selection


def print_list_items():
    for item in item_list.get(0, tk.END):
        g = json.loads(item)
        print(g['Service'])

# Create the main application window
root = tk.Tk()
root.title("Service and Action List")

# Create GUI elements
service_label = tk.Label(root, text="Service:")
service_label.pack(pady=5)

service_entry = tk.Entry(root)
service_entry.pack(pady=5)

action_label = tk.Label(root, text="Action:")
action_label.pack(pady=5)

action_entry = tk.Entry(root)
action_entry.pack(pady=5)

add_button = tk.Button(root, text="Add to List", command=add_to_list)
add_button.pack(pady=10)

print_button = tk.Button(root, text="Print List Items", command=print_list_items)
print_button.pack(pady=5)

item_list = tk.Listbox(root, height=5, width=40)
item_list.pack(padx=10, pady=5)

delete_button = tk.Button(root, text="Delete", command=delete_item)
delete_button.pack()

root.mainloop()
