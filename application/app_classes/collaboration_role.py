import tkinter as tk
from tkinter import ttk, messagebox

from application.app_classes.service_action_dialog import (CharacteristicsFrame, CollaborationRoleDialog,
                                                           ServiceActionDialog)
from application.helper_classes.collaborationRoles import (
    DIRECTION_LABELS, RECEIVE, SEND, cpa, delete_action, delete_collaboration_role, describe_action,
    describe_collaboration_role, find_action_by_binding_id, get_action_elements, get_channel_ids,
    get_collaboration_role_of, get_other_party_info, get_package_ids, get_party_info_of, get_party_infos,
    get_party_name, get_service_binding, update_action, validate_action)

PARTNER_COLORS = ('#cfe2ff', '#f8d7da')
DIRECTION_SYMBOLS = {SEND: '▶', RECEIVE: '◀'}
EXPERIMENTAL_NOTICE = ("Experimental: adding, changing and deleting Service actions and Collaboration Roles is new. "
                       "Check the result on the CPA Validation tab before you save the CPA.")


class CollaborationRole(tk.Frame):
    """
    Tab with the collaboration roles of both partners: per partner the services with their Send and Receive
    actions. Actions can be added (with the mirrored action at the other partner), changed and deleted.

    Test Functions: see TestCollaborationRoleTab.
    """

    COLUMNS = (('action', "Action", 160), ('channel', "Channel", 260), ('package', "Package", 160),
               ('other_party', "OtherPartyActionBinding", 300), ('status', "Status", 140))

    def __init__(self, master, logger, **kw):
        super().__init__(master, **kw)
        self.menu = None
        self.xml_element_mapping = {}  # Treeview item -> ('partner' | 'role' | 'action', XML element)
        self.tree_editor = None  # Treeview widget
        self.filter_text = tk.StringVar()
        self.selected_action = None
        self.master = master
        self.logger = logger

    def reload(self):
        """Reload the collaboration roles, keeping the selection."""
        self.load()

    def load(self):
        """Fill the Treeview with the collaboration roles of both partners."""
        selected = self.selected_element()
        open_state = {self.xml_element_mapping[item][1]: self.tree_editor.item(item, 'open')
                      for item in self.xml_element_mapping if self.tree_editor.exists(item)}
        self.clear_tree()
        self.xml_element_mapping = {}
        if self.master.root is None:
            self.show_action(None)
            return
        search = self.filter_text.get().strip().lower()
        for index, party_info in enumerate(get_party_infos(self.master.root)):
            header_tag = f"partner_{index}"
            partner_item = self.tree_editor.insert("", "end", text=f"Partner {get_party_name(party_info)}",
                                                   open=open_state.get(party_info, True), tags=(header_tag,))
            self.tree_editor.tag_configure(header_tag, background=PARTNER_COLORS[index % len(PARTNER_COLORS)])
            self.xml_element_mapping[partner_item] = ('partner', party_info)
            for collaboration_role in party_info.findall(cpa('CollaborationRole')):
                self.insert_collaboration_role(partner_item, collaboration_role, search, open_state)

        restored = self.find_item(selected)
        if restored:
            self.tree_editor.selection_set(restored)
            self.tree_editor.see(restored)
        self.show_action(selected if restored and self.xml_element_mapping[restored][0] == 'action' else None)

    def insert_collaboration_role(self, partner_item, collaboration_role, search, open_state):
        """Insert a row for the service of a CollaborationRole with its actions below it."""
        role = describe_collaboration_role(collaboration_role)
        service_binding = get_service_binding(collaboration_role)
        actions = get_action_elements(service_binding) if service_binding is not None else []
        role_text = ' '.join(role.values()).lower()
        if search and search not in role_text and not any(self.action_matches(action, search) for action in actions):
            return
        service_type = f" [{role['service_type']}]" if role['service_type'] else ''
        role_item = self.tree_editor.insert(
            partner_item, "end", open=open_state.get(collaboration_role, True),
            text=f"Service {role['service_name']}{service_type}   Role {role['role_name']}   "
                 f"Process {role['process_name']} {role['process_version']}")
        self.xml_element_mapping[role_item] = ('role', collaboration_role)
        for action in actions:
            if not search or search in role_text or self.action_matches(action, search):
                self.insert_action(role_item, action)

    def action_matches(self, action_element, search):
        action = describe_action(action_element)
        return any(search in value.lower() for value in (action['action'], action['id'], action['other_party_binding']))

    def insert_action(self, parent_item, action_element):
        """Insert a row for a CanSend or CanReceive; nested actions become child rows."""
        action = describe_action(action_element)
        problems = validate_action(self.master.root, action_element)
        status = f"{len(problems)} problem(s)" if problems else "OK"
        item = self.tree_editor.insert(
            parent_item, "end", open=True, tags=('problem',) if problems else (),
            text=f"{DIRECTION_SYMBOLS[action['direction']]} {DIRECTION_LABELS[action['direction']]}",
            values=(action['action'], ', '.join(action['channel_ids']), action['package_id'],
                    action['other_party_binding'], status))
        self.xml_element_mapping[item] = ('action', action_element)
        for nested in get_action_elements(action_element):
            self.insert_action(item, nested)

    def clear_tree(self):
        """Clear the Treeview of all items."""
        self.tree_editor.delete(*self.tree_editor.get_children())

    def find_item(self, element):
        """Returns the Treeview item of an XML element, or None."""
        if element is None:
            return None
        for item, (_, mapped) in self.xml_element_mapping.items():
            if mapped is element:
                return item
        return None

    def selected_element(self, kind=None):
        """Returns the XML element of the selected row (of the given kind), or None."""
        selection = self.tree_editor.selection() if self.tree_editor is not None else ()
        if not selection or selection[0] not in self.xml_element_mapping:
            return None
        selected_kind, element = self.xml_element_mapping[selection[0]]
        return element if kind is None or kind == selected_kind else None

    def selected_collaboration_role(self):
        """The CollaborationRole of the selected row: the role itself or the role of the selected action."""
        element = self.selected_element()
        if element is None or element.tag == cpa('PartyInfo'):
            return None
        return get_collaboration_role_of(element)

    def selected_party_info(self):
        element = self.selected_element()
        return get_party_info_of(element) if element is not None else None

    def on_treeview_select(self, event=None):
        self.show_action(self.selected_element('action'))

    def show_action(self, action_element):
        """Show the selected action in the detail panel, or empty the panel."""
        self.selected_action = action_element
        state = "normal" if action_element is not None else "disabled"
        if action_element is None:
            self.detail_title.config(text="Select an action to edit")
            for variable in (self.binding_id, self.action_name, self.package_id, self.channel_id):
                variable.set('')
            self.characteristics.set({})
            self.problems_text.set('')
        else:
            action = describe_action(action_element)
            party_info = get_party_info_of(action_element)
            self.detail_title.config(text=f"{DIRECTION_LABELS[action['direction']]} action of partner "
                                          f"{get_party_name(party_info)}")
            self.binding_id.set(action['id'])
            self.action_name.set(action['action'])
            self.package_id.set(action['package_id'])
            self.channel_id.set(action['channel_ids'][0] if action['channel_ids'] else '')
            self.package_combobox.config(values=get_package_ids(self.master.root))
            self.channel_combobox.config(values=get_channel_ids(party_info))
            self.characteristics.set(action['characteristics'])
            problems = validate_action(self.master.root, action_element)
            self.problems_text.set('\n'.join(problems) if problems else "No problems found")
        self.action_entry.config(state=state)
        for combobox in (self.package_combobox, self.channel_combobox):
            combobox.config(state="readonly" if action_element is not None else "disabled")
        self.characteristics.set_enabled(action_element is not None)
        self.save_button.config(state=state)
        self.problems_label.config(fg="red" if self.problems_text.get() not in ('', "No problems found") else "green")

    def save_changes(self):
        """Write the values of the detail panel to the selected action."""
        if self.selected_action is None:
            return
        try:
            update_action(self.selected_action, self.action_name.get(), self.package_id.get(), self.channel_id.get(),
                          self.characteristics.get())
        except ValueError as e:
            messagebox.showerror("Save Changes", str(e), parent=self)
            return
        self.logger.info(f"Action '{self.binding_id.get()}' changed")
        self.reload()

    def add_service_action(self):
        """Open the dialog to add a Service action to the selected CollaborationRole."""
        collaboration_role = self.selected_collaboration_role()
        if collaboration_role is None:
            messagebox.showinfo("Add Service action", "Select a service or an action first.", parent=self)
            return None
        return ServiceActionDialog(self, self.master.root, collaboration_role, on_done=self.on_action_added)

    def on_action_added(self, result):
        element, mirror_element = result
        added = [describe_action(element)['id']] + ([describe_action(mirror_element)['id']] if mirror_element is not None
                                                    else [])
        self.logger.info(f"Service action added: {', '.join(added)}")
        self.reload()
        item = self.find_item(element)
        if item:
            self.tree_editor.selection_set(item)
            self.tree_editor.see(item)

    def add_collaboration_role(self):
        """Open the dialog to add a CollaborationRole to the partner of the selected row."""
        party_info = self.selected_party_info()
        if party_info is None:
            party_infos = get_party_infos(self.master.root) if self.master.root is not None else []
            if not party_infos:
                return None
            party_info = party_infos[0]
        return CollaborationRoleDialog(self, self.master.root, party_info, on_done=self.on_collaboration_role_added)

    def on_collaboration_role_added(self, result):
        role, _ = result
        self.logger.info(f"Collaboration Role added: service {describe_collaboration_role(role)['service_name']}")
        self.reload()
        item = self.find_item(role)
        if item:
            self.tree_editor.selection_set(item)
            self.tree_editor.see(item)

    def delete_selected(self):
        """Delete the selected action or CollaborationRole after confirmation."""
        selection = self.tree_editor.selection()
        if not selection or selection[0] not in self.xml_element_mapping:
            return
        kind, element = self.xml_element_mapping[selection[0]]
        if kind == 'action':
            self.delete_action(element)
        elif kind == 'role':
            self.delete_collaboration_role(element)

    def delete_action(self, action_element):
        action = describe_action(action_element)
        party_info = get_party_info_of(action_element)
        other_party_info = get_other_party_info(self.master.root, party_info)
        counterpart = find_action_by_binding_id(other_party_info, action['other_party_binding']) \
            if other_party_info is not None else None
        if counterpart is not None:
            answer = messagebox.askyesnocancel(
                "Delete action",
                f"Delete action '{action['id']}'?\n\nYes: also delete the mirrored action "
                f"'{action['other_party_binding']}' of partner {get_party_name(other_party_info)}.\n"
                f"No: delete only this action.", parent=self)
            if answer is None:
                return
            delete_counterpart = answer
        else:
            if not messagebox.askyesno("Delete action", f"Delete action '{action['id']}'?", parent=self):
                return
            delete_counterpart = False
        deleted = delete_action(self.master.root, action_element, delete_counterpart)
        self.logger.info(f"Service action deleted: {', '.join(deleted)}")
        self.reload()

    def delete_collaboration_role(self, collaboration_role):
        role = describe_collaboration_role(collaboration_role)
        if not messagebox.askyesno("Delete Collaboration Role",
                                   f"Delete the Collaboration Role of service '{role['service_name']}' with all its "
                                   f"actions?\nActions of the other partner that refer to them are not deleted.",
                                   parent=self):
            return
        try:
            delete_collaboration_role(collaboration_role)
        except ValueError as e:
            messagebox.showerror("Delete Collaboration Role", str(e), parent=self)
            return
        self.logger.info(f"Collaboration Role deleted: service {role['service_name']}")
        self.reload()

    def popup(self, event):
        """Display a right-click context menu for the row under the mouse."""
        item = self.tree_editor.identify_row(event.y)
        if item:
            self.tree_editor.selection_set(item)
        kind = self.xml_element_mapping.get(item, (None, None))[0]
        self.menu = tk.Menu(self, tearoff=0)
        if kind in ('role', 'action'):
            self.menu.add_command(label="Add Service action", command=self.add_service_action)
        self.menu.add_command(label="Add Collaboration Role", command=self.add_collaboration_role)
        if kind == 'action':
            self.menu.add_command(label="Delete action", command=self.delete_selected)
        elif kind == 'role':
            self.menu.add_command(label="Delete Collaboration Role", command=self.delete_selected)
        self.menu.post(event.x_root, event.y_root)

    def create(self):
        """Create the toolbar, the Treeview and the detail panel."""
        self.experimental_label = tk.Label(
            self, text=EXPERIMENTAL_NOTICE, bg='#fff3cd', fg='#664d03', anchor="w", justify="left", padx=8, pady=4)
        self.experimental_label.pack(side=tk.TOP, fill=tk.X, padx=10, pady=(10, 0))
        toolbar = tk.Frame(self)
        toolbar.pack(side=tk.TOP, fill=tk.X, padx=10, pady=(10, 0))
        tk.Label(toolbar, text="Filter").pack(side=tk.LEFT)
        self.filter_entry = ttk.Entry(toolbar, textvariable=self.filter_text, width=30)
        self.filter_entry.pack(side=tk.LEFT, padx=5)
        self.filter_text.trace_add('write', lambda *args: self.reload() if self.master.root is not None else None)
        ttk.Button(toolbar, text="Add Service action", command=self.add_service_action).pack(side=tk.LEFT, padx=5)
        ttk.Button(toolbar, text="Add Collaboration Role", command=self.add_collaboration_role).pack(side=tk.LEFT,
                                                                                                    padx=5)
        ttk.Button(toolbar, text="Delete", command=self.delete_selected).pack(side=tk.LEFT, padx=5)

        panes = ttk.PanedWindow(self, orient=tk.HORIZONTAL)
        panes.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)

        tree_frame = tk.Frame(panes)
        self.tree_editor = ttk.Treeview(tree_frame, columns=[name for name, _, _ in self.COLUMNS],
                                        selectmode="browse")
        self.tree_editor.heading("#0", text="Partner / Service / Direction")
        self.tree_editor.column("#0", width=420, stretch=True)
        for name, heading, width in self.COLUMNS:
            self.tree_editor.heading(name, text=heading)
            self.tree_editor.column(name, width=width, stretch=True)
        self.tree_editor.tag_configure('problem', foreground='red')
        self.vsb = ttk.Scrollbar(tree_frame, orient="vertical", command=self.tree_editor.yview)
        self.tree_editor.configure(yscrollcommand=self.vsb.set)
        self.vsb.pack(side=tk.RIGHT, fill=tk.Y)
        self.tree_editor.pack(fill=tk.BOTH, expand=True)
        panes.add(tree_frame, weight=3)

        detail = tk.Frame(panes, padx=10)
        detail.columnconfigure(1, weight=1)
        self.detail_title = tk.Label(detail, text="Select an action to edit", font=('TkDefaultFont', 12, 'bold'))
        self.detail_title.grid(row=0, column=0, columnspan=2, sticky="w", pady=(0, 5))
        self.binding_id = tk.StringVar()
        self.action_name = tk.StringVar()
        self.package_id = tk.StringVar()
        self.channel_id = tk.StringVar()
        tk.Label(detail, text="ThisPartyActionBinding id").grid(row=1, column=0, sticky="w", pady=2)
        tk.Label(detail, textvariable=self.binding_id, anchor="w").grid(row=1, column=1, sticky="we", pady=2)
        tk.Label(detail, text="Action").grid(row=2, column=0, sticky="w", pady=2)
        self.action_entry = ttk.Entry(detail, textvariable=self.action_name)
        self.action_entry.grid(row=2, column=1, sticky="we", pady=2)
        tk.Label(detail, text="Package").grid(row=3, column=0, sticky="w", pady=2)
        self.package_combobox = ttk.Combobox(detail, textvariable=self.package_id, state="disabled")
        self.package_combobox.grid(row=3, column=1, sticky="we", pady=2)
        tk.Label(detail, text="Channel").grid(row=4, column=0, sticky="w", pady=2)
        self.channel_combobox = ttk.Combobox(detail, textvariable=self.channel_id, state="disabled")
        self.channel_combobox.grid(row=4, column=1, sticky="we", pady=2)
        self.characteristics = CharacteristicsFrame(detail, padx=5, pady=5)
        self.characteristics.grid(row=5, column=0, columnspan=2, sticky="we", pady=5)
        self.save_button = ttk.Button(detail, text="Save Changes", command=self.save_changes)
        self.save_button.grid(row=6, column=0, columnspan=2, pady=5)
        self.problems_text = tk.StringVar()
        self.problems_label = tk.Label(detail, textvariable=self.problems_text, justify="left", anchor="w",
                                       wraplength=380)
        self.problems_label.grid(row=7, column=0, columnspan=2, sticky="we")
        panes.add(detail, weight=1)
        self.show_action(None)

        # Bind right-click context menu to Treeview
        if self.master.host == 'mac':
            self.tree_editor.bind("<Button-2>", self.popup)
        else:
            self.tree_editor.bind("<Button-3>", self.popup)
        self.tree_editor.bind("<<TreeviewSelect>>", self.on_treeview_select)
        self.tree_editor.bind("<Delete>", lambda event: self.delete_selected())
