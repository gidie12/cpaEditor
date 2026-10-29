import uuid
import tkinter as tk
from tkinter import ttk

from application.helper_classes.collaborationRoles import (
    CHARACTERISTICS, DIRECTION_LABELS, RECEIVE, SEND, add_action, add_collaboration_role, cpa,
    describe_collaboration_role, find_counterpart_collaboration_role, get_channel_ids, get_other_party_info,
    get_package_ids, get_party_info_of, get_party_name, opposite_direction, suggest_binding_id,
    suggest_channel_id, suggest_characteristics)


def describe_role_choice(collaboration_role):
    """The text of a CollaborationRole in a list: service, role and process."""
    role = describe_collaboration_role(collaboration_role)
    return f"{role['service_name']} (Role {role['role_name']}, Process {role['process_name']} {role['process_version']})"


class CharacteristicsFrame(tk.LabelFrame):
    """Fields for the attributes of BusinessTransactionCharacteristics. An empty value leaves the attribute out."""

    def __init__(self, master, **kw):
        super().__init__(master, text="BusinessTransactionCharacteristics", **kw)
        self.variables = {}
        self.fields = {}
        self.enabled_states = {}
        for row, (name, allowed, _) in enumerate(CHARACTERISTICS):
            tk.Label(self, text=name).grid(row=row, column=0, sticky="w", padx=5, pady=2)
            variable = tk.StringVar()
            field = ttk.Combobox(self, textvariable=variable, width=28,
                                 values=('',) + allowed if allowed else ('PT1H', 'PT2H', 'PT24H'),
                                 state="readonly" if allowed else "normal")
            field.grid(row=row, column=1, sticky="we", padx=5, pady=2)
            self.variables[name] = variable
            self.fields[name] = field
            self.enabled_states[name] = "readonly" if allowed else "normal"
        self.columnconfigure(1, weight=1)

    def set(self, characteristics):
        for name, variable in self.variables.items():
            variable.set(characteristics.get(name, ''))

    def get(self):
        return {name: variable.get() for name, variable in self.variables.items()}

    def set_enabled(self, enabled):
        for name, field in self.fields.items():
            field.config(state=self.enabled_states[name] if enabled else "disabled")


class SuggestedEntry:
    """
    An Entry whose value follows a suggestion until the user types something else in it.

    Test Functions:
        - test_binding_id_follows_action_until_edited
    """

    def __init__(self, variable):
        self.variable = variable
        self.suggestion = None

    def suggest(self, value):
        if self.suggestion is None or self.variable.get() == self.suggestion:
            self.variable.set(value)
        self.suggestion = value


class ServiceActionDialog(tk.Toplevel):
    """
    Dialog to add a Service action (CanSend or CanReceive) to a CollaborationRole, by default with the mirrored
    action at the other partner.

    Test Functions:
        - test_dialog_prefills_suggestions
        - test_binding_id_follows_action_until_edited
        - test_submit_adds_action_and_mirror
        - test_submit_without_mirror
        - test_submit_shows_error
        - test_mirror_disabled_without_other_role
    """

    def __init__(self, master, root, collaboration_role, on_done=None, direction=SEND):
        super().__init__(master)
        self.root = root
        self.collaboration_role = collaboration_role
        self.on_done = on_done
        self.result = None
        self.party_info = get_party_info_of(collaboration_role)
        self.other_party_info = get_other_party_info(root, self.party_info)
        self.other_roles = self.other_party_info.findall(cpa('CollaborationRole')) \
            if self.other_party_info is not None else []

        self.title("Add Service action")
        self.transient(master)
        self.resizable(True, False)

        self.direction = tk.StringVar(value=direction)
        self.action_name = tk.StringVar()
        self.binding_id = tk.StringVar()
        self.package_id = tk.StringVar()
        self.channel_id = tk.StringVar()
        self.mirror_enabled = tk.BooleanVar(value=bool(self.other_roles))
        self.mirror_role = tk.StringVar()
        self.mirror_binding_id = tk.StringVar()
        self.mirror_channel_id = tk.StringVar()
        self.error_text = tk.StringVar()
        self.binding_id_entry = SuggestedEntry(self.binding_id)
        self.mirror_binding_id_entry = SuggestedEntry(self.mirror_binding_id)

        self.create_widgets()
        self.prefill()
        self.direction.trace_add('write', lambda *args: self.update_suggestions(direction_changed=True))
        self.action_name.trace_add('write', lambda *args: self.update_suggestions())
        self.mirror_role.trace_add('write', lambda *args: self.update_suggestions(direction_changed=True))
        self.mirror_enabled.trace_add('write', lambda *args: self.update_mirror_state())
        self.bind("<Escape>", lambda event: self.destroy())

    def create_widgets(self):
        form = tk.Frame(self, padx=10, pady=10)
        form.pack(fill=tk.BOTH, expand=True)
        form.columnconfigure(1, weight=1)

        rows = [("Partner", get_party_name(self.party_info)), ("Service", describe_role_choice(self.collaboration_role))]
        for row, (label, value) in enumerate(rows):
            tk.Label(form, text=label).grid(row=row, column=0, sticky="w", pady=2)
            tk.Label(form, text=value, anchor="w").grid(row=row, column=1, sticky="we", pady=2)

        tk.Label(form, text="Direction").grid(row=2, column=0, sticky="w", pady=2)
        directions = tk.Frame(form)
        directions.grid(row=2, column=1, sticky="w")
        for direction in (SEND, RECEIVE):
            ttk.Radiobutton(directions, text=f"{DIRECTION_LABELS[direction]} ({direction})", value=direction,
                            variable=self.direction).pack(side=tk.LEFT, padx=(0, 10))

        tk.Label(form, text="Action").grid(row=3, column=0, sticky="w", pady=2)
        self.action_entry = ttk.Entry(form, textvariable=self.action_name)
        self.action_entry.grid(row=3, column=1, sticky="we", pady=2)
        tk.Label(form, text="ThisPartyActionBinding id").grid(row=4, column=0, sticky="w", pady=2)
        ttk.Entry(form, textvariable=self.binding_id).grid(row=4, column=1, sticky="we", pady=2)
        tk.Label(form, text="Package").grid(row=5, column=0, sticky="w", pady=2)
        self.package_combobox = ttk.Combobox(form, textvariable=self.package_id, state="readonly",
                                             values=get_package_ids(self.root))
        self.package_combobox.grid(row=5, column=1, sticky="we", pady=2)
        tk.Label(form, text="Channel").grid(row=6, column=0, sticky="w", pady=2)
        self.channel_combobox = ttk.Combobox(form, textvariable=self.channel_id, state="readonly",
                                             values=get_channel_ids(self.party_info))
        self.channel_combobox.grid(row=6, column=1, sticky="we", pady=2)

        self.characteristics = CharacteristicsFrame(form, padx=5, pady=5)
        self.characteristics.grid(row=7, column=0, columnspan=2, sticky="we", pady=5)

        other_name = get_party_name(self.other_party_info) if self.other_party_info is not None else ''
        mirror_frame = tk.LabelFrame(form, text="Mirrored action", padx=5, pady=5)
        mirror_frame.grid(row=8, column=0, columnspan=2, sticky="we", pady=5)
        mirror_frame.columnconfigure(1, weight=1)
        self.mirror_checkbutton = ttk.Checkbutton(
            mirror_frame, variable=self.mirror_enabled,
            text=f"Also add the opposite action at partner {other_name}; both OtherPartyActionBindings refer to each other")
        self.mirror_checkbutton.grid(row=0, column=0, columnspan=2, sticky="w")
        tk.Label(mirror_frame, text="Service").grid(row=1, column=0, sticky="w", pady=2)
        self.mirror_role_combobox = ttk.Combobox(mirror_frame, textvariable=self.mirror_role, state="readonly",
                                                 values=[describe_role_choice(role) for role in self.other_roles])
        self.mirror_role_combobox.grid(row=1, column=1, sticky="we", pady=2)
        tk.Label(mirror_frame, text="ThisPartyActionBinding id").grid(row=2, column=0, sticky="w", pady=2)
        self.mirror_binding_id_field = ttk.Entry(mirror_frame, textvariable=self.mirror_binding_id)
        self.mirror_binding_id_field.grid(row=2, column=1, sticky="we", pady=2)
        tk.Label(mirror_frame, text="Channel").grid(row=3, column=0, sticky="w", pady=2)
        self.mirror_channel_combobox = ttk.Combobox(
            mirror_frame, textvariable=self.mirror_channel_id, state="readonly",
            values=get_channel_ids(self.other_party_info) if self.other_party_info is not None else [])
        self.mirror_channel_combobox.grid(row=3, column=1, sticky="we", pady=2)
        if not self.other_roles:
            self.mirror_checkbutton.config(state="disabled")

        tk.Label(form, textvariable=self.error_text, fg="red", anchor="w", justify="left").grid(
            row=9, column=0, columnspan=2, sticky="we")
        buttons = tk.Frame(form)
        buttons.grid(row=10, column=0, columnspan=2, pady=(5, 0))
        ttk.Button(buttons, text="Add", command=self.submit).pack(side=tk.LEFT, padx=5)
        ttk.Button(buttons, text="Cancel", command=self.destroy).pack(side=tk.LEFT, padx=5)

    def prefill(self):
        package_ids = get_package_ids(self.root)
        default_package = self.party_info.get(cpa('defaultMshPackageId'))
        self.package_id.set(default_package if default_package in package_ids else (package_ids[0] if package_ids else ''))
        self.characteristics.set(suggest_characteristics(self.collaboration_role))
        counterpart = find_counterpart_collaboration_role(self.root, self.collaboration_role)
        if counterpart is None and self.other_roles:
            counterpart = self.other_roles[0]
        if counterpart is not None:
            self.mirror_role.set(describe_role_choice(counterpart))
        self.update_suggestions(direction_changed=True)
        self.update_mirror_state()
        self.action_entry.focus_set()

    def selected_mirror_role(self):
        choices = [describe_role_choice(role) for role in self.other_roles]
        if self.mirror_role.get() in choices:
            return self.other_roles[choices.index(self.mirror_role.get())]
        return None

    def update_suggestions(self, direction_changed=False):
        direction = self.direction.get()
        action_name = self.action_name.get().strip() or 'Action'
        self.binding_id_entry.suggest(suggest_binding_id(self.root, self.collaboration_role, direction, action_name))
        if direction_changed:
            self.channel_id.set(suggest_channel_id(self.collaboration_role, direction))
        mirror_role = self.selected_mirror_role()
        if mirror_role is not None:
            self.mirror_binding_id_entry.suggest(suggest_binding_id(
                self.root, mirror_role, opposite_direction(direction), action_name, reserved=[self.binding_id.get()]))
            if direction_changed:
                self.mirror_channel_id.set(suggest_channel_id(mirror_role, opposite_direction(direction)))

    def update_mirror_state(self):
        state = "normal" if self.mirror_enabled.get() else "disabled"
        self.mirror_binding_id_field.config(state=state)
        for combobox in (self.mirror_role_combobox, self.mirror_channel_combobox):
            combobox.config(state="readonly" if self.mirror_enabled.get() else "disabled")

    def submit(self):
        """Adds the action(s) to the CPA; on a problem the dialog stays open with the message."""
        mirror = None
        if self.mirror_enabled.get():
            mirror_role = self.selected_mirror_role()
            if mirror_role is None:
                self.error_text.set("Choose the service of the mirrored action")
                return None
            mirror = {'collaboration_role': mirror_role, 'binding_id': self.mirror_binding_id.get().strip(),
                      'channel_id': self.mirror_channel_id.get()}
        try:
            self.result = add_action(self.root, self.collaboration_role, self.direction.get(), self.action_name.get(),
                                     self.package_id.get(), self.channel_id.get(), self.characteristics.get(),
                                     self.binding_id.get().strip(), mirror)
        except ValueError as e:
            self.error_text.set(str(e))
            return None
        self.destroy()
        if self.on_done:
            self.on_done(self.result)
        return self.result


class CollaborationRoleDialog(tk.Toplevel):
    """
    Dialog to add a CollaborationRole (one Service) to a partner, by default also at the other partner.

    Test Functions:
        - test_role_dialog_prefills_from_existing_role
        - test_role_dialog_adds_role_at_both_partners
        - test_role_dialog_shows_error
        - test_role_dialog_generate_uuid
    """

    FIELDS = (('process_name', "Process name"), ('process_version', "Process version"),
              ('process_href', "Process href"), ('process_uuid', "Process uuid"),
              ('role_name', "Role name"), ('role_href', "Role href"),
              ('service_name', "Service name"), ('service_type', "Service type"))

    def __init__(self, master, root, party_info, on_done=None):
        super().__init__(master)
        self.root = root
        self.party_info = party_info
        self.on_done = on_done
        self.result = None
        self.other_party_info = get_other_party_info(root, party_info)
        self.title("Add Collaboration Role")
        self.transient(master)
        self.resizable(True, False)
        self.variables = {name: tk.StringVar() for name, _ in self.FIELDS}
        self.mirror_enabled = tk.BooleanVar(value=self.other_party_info is not None)
        self.mirror_role_name = tk.StringVar()
        self.error_text = tk.StringVar()
        self.create_widgets()
        self.prefill()
        self.bind("<Escape>", lambda event: self.destroy())

    def create_widgets(self):
        form = tk.Frame(self, padx=10, pady=10)
        form.pack(fill=tk.BOTH, expand=True)
        form.columnconfigure(1, weight=1)
        tk.Label(form, text="Partner").grid(row=0, column=0, sticky="w", pady=2)
        tk.Label(form, text=get_party_name(self.party_info), anchor="w").grid(row=0, column=1, sticky="we")
        for row, (name, label) in enumerate(self.FIELDS, start=1):
            tk.Label(form, text=label).grid(row=row, column=0, sticky="w", pady=2)
            ttk.Entry(form, textvariable=self.variables[name], width=60).grid(row=row, column=1, sticky="we", pady=2)
            if name == 'process_uuid':
                ttk.Button(form, text="Generate UUID", command=self.generate_uuid).grid(row=row, column=2, padx=5)
        row = len(self.FIELDS) + 1
        other_name = get_party_name(self.other_party_info) if self.other_party_info is not None else ''
        mirror_frame = tk.LabelFrame(form, text="Other partner", padx=5, pady=5)
        mirror_frame.grid(row=row, column=0, columnspan=3, sticky="we", pady=5)
        mirror_frame.columnconfigure(1, weight=1)
        checkbutton = ttk.Checkbutton(mirror_frame, variable=self.mirror_enabled,
                                      text=f"Also add this service at partner {other_name}")
        checkbutton.grid(row=0, column=0, columnspan=2, sticky="w")
        tk.Label(mirror_frame, text="Role name").grid(row=1, column=0, sticky="w", pady=2)
        ttk.Entry(mirror_frame, textvariable=self.mirror_role_name).grid(row=1, column=1, sticky="we", pady=2)
        if self.other_party_info is None:
            checkbutton.config(state="disabled")
        tk.Label(form, textvariable=self.error_text, fg="red", anchor="w").grid(row=row + 1, column=0, columnspan=3,
                                                                              sticky="we")
        buttons = tk.Frame(form)
        buttons.grid(row=row + 2, column=0, columnspan=3, pady=(5, 0))
        ttk.Button(buttons, text="Add", command=self.submit).pack(side=tk.LEFT, padx=5)
        ttk.Button(buttons, text="Cancel", command=self.destroy).pack(side=tk.LEFT, padx=5)

    def prefill(self):
        """Process, role and service type of an existing CollaborationRole of the partner are a good start."""
        existing = self.party_info.find(cpa('CollaborationRole'))
        if existing is not None:
            process = existing.find(cpa('ProcessSpecification'))
            role = existing.find(cpa('Role'))
            service = existing.find(f"{cpa('ServiceBinding')}/{cpa('Service')}")
            xlink_href = '{http://www.w3.org/1999/xlink}href'
            values = {
                'process_name': process.get(cpa('name'), '') if process is not None else '',
                'process_version': process.get(cpa('version'), '') if process is not None else '',
                'process_href': process.get(xlink_href, '') if process is not None else '',
                'role_name': role.get(cpa('name'), '') if role is not None else '',
                'role_href': role.get(xlink_href, '') if role is not None else '',
                'service_type': service.get(cpa('type'), '') if service is not None else '',
            }
            for name, value in values.items():
                self.variables[name].set(value)
        if self.other_party_info is not None:
            other_role = self.other_party_info.find(f"{cpa('CollaborationRole')}/{cpa('Role')}")
            if other_role is not None:
                self.mirror_role_name.set(other_role.get(cpa('name'), ''))

    def generate_uuid(self):
        self.variables['process_uuid'].set(f"urn:uuid:{uuid.uuid4()}")

    def submit(self):
        values = {name: variable.get() for name, variable in self.variables.items()}
        if self.mirror_enabled.get() and not self.mirror_role_name.get().strip():
            self.error_text.set("Role name of the other partner is required")
            return None
        try:
            role = add_collaboration_role(self.party_info, **values)
        except ValueError as e:
            self.error_text.set(str(e))
            return None
        mirror_role = None
        if self.mirror_enabled.get():
            values['role_name'] = self.mirror_role_name.get()
            mirror_role = add_collaboration_role(self.other_party_info, **values)
        self.result = (role, mirror_role)
        self.destroy()
        if self.on_done:
            self.on_done(self.result)
        return self.result
