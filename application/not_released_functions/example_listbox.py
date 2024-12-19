import tkinter as tk

def on_select(event):
    selected_index = listbox.curselection()
    if selected_index:
        entry.delete(0, tk.END)
        entry.insert(0, listbox.get(selected_index[0]))

def focus_entry(event):
    entry.focus_set()

def delete_item():
    selected_index = listbox.curselection()
    if selected_index:
        deleted_index = selected_index[0]
        listbox.delete(deleted_index)

        if listbox.size() > 0:
            new_index = min(deleted_index, listbox.size() - 1)
            listbox.select_set(new_index)
            on_select(None)  # Update entry field with new selection

def update_item():
    selected_index = listbox.curselection()
    if selected_index and entry.get():
        listbox.delete(selected_index[0])
        listbox.insert(selected_index[0], entry.get())

root = tk.Tk()
root.title("Editable List")

listbox = tk.Listbox(root, selectmode=tk.SINGLE)
for i in range(10):
    listbox.insert(tk.END, f"Item {i}")

listbox.grid(row=0, column=0, rowspan=4, sticky="nsew")
listbox.bind("<<ListboxSelect>>", on_select)
listbox.bind("<ButtonRelease-1>", focus_entry)  # Set focus to entry after clicking

entry = tk.Entry(root)
entry.grid(row=0, column=1, sticky="ew")

delete_button = tk.Button(root, text="Delete", command=delete_item)
delete_button.grid(row=2, column=1, sticky="ew")

update_button = tk.Button(root, text="Update", command=update_item)
update_button.grid(row=3, column=1, sticky="ew")

root.grid_rowconfigure(0, weight=1)
root.grid_columnconfigure(0, weight=1)

root.mainloop()
