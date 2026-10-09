import logging

import tkinter as tk
from datetime import datetime, timezone
from functools import partial

from application.helper_classes.certificateExpiry import CPA_DATE_FORMAT, get_certificate_expiries
from application.helper_classes.cpaCreator import CPACreator
from application.helper_classes.cpaDates import describe_cpa_date
from application.helper_classes.cpaParser import CPAParser
from application.helper_classes.resizeApplication import update_entry_width


class General(tk.Frame):
    def __init__(self, master, logger, **kw):
        super().__init__(master, **kw)
        self.entry_fields = []
        self.entry_start_date = None
        self.label_start_date = None
        self.all_entry_fields = None
        self.label_end_date = None
        self.all_dropdown_fields = None
        self.entry_end_date = None
        self.button_end_date_from_certificates = None
        self.button_start_date_now = None
        self.label_start_date_times = None
        self.label_end_date_times = None
        self.entry_party_id_type_partner_b = None
        self.label_party_id_type_partner_b = None
        self.entry_party_id_type_partner_a = None
        self.label_party_id_type_partner_a = None
        self.entry_party_id_partner_b = None
        self.label_party_id_partner_b = None
        self.entry_party_id_partner_a = None
        self.label_party_id_partner_a = None
        self.label_partner_name_partner_b = None
        self.entry_partner_name_partner_b = None
        self.entry_partner_name_partner_a = None
        self.label_partner_name_partner_a = None
        self.label_cpa_status = None
        self.entry_cpa_status = None
        self.entry_cpa_id = None
        self.label_cpa_id = None
        self.cpa_dict_entries = None
        self.mods_list = []
        self.master = master
        self.cpa_status_options = [
        "proposed",
        "agreed",
        "signed",
        ]
        self.cpa_status_value = tk.StringVar(self)
        self.logger = logger


    def create(self):
        """
        Creates the GUI elements for the General tab.

        This method initializes and arranges the various input fields and labels
        for the General tab, including CPA ID, status, party names, party IDs,
        party ID types, and start and end dates. It also binds the necessary
        events to these fields.

        Tests:
        - test_creates_all_fields_correctly
        - test_handles_empty_fields_gracefully
        """
        # CPAID
        self.label_cpa_id = tk.Label(self, text="CPAId:")
        self.label_cpa_id.grid(sticky="w", row=0, column=0, padx=5, pady=5)
        self.entry_cpa_id = tk.Entry(self)
        self.entry_fields.append(self.entry_cpa_id)
        self.entry_cpa_id.grid(sticky="we", row=0, column=1, padx=5, pady=5)

        self.label_cpa_status = tk.Label(self, text="Status:")
        # self.entry_cpa_status = tk.Entry(self, width=25)
        self.entry_cpa_status = tk.OptionMenu(self, self.cpa_status_value,*self.cpa_status_options)
        self.label_cpa_status.grid(sticky="w", row=0, column=2, padx=5, pady=5)
        self.entry_cpa_status.grid(sticky="w", row=0, column=3, padx=5, pady=5)


        self.label_partner_name_partner_a = tk.Label(self, text="Party Name Partner A:")
        self.entry_partner_name_partner_a = tk.Entry(self)
        self.entry_fields.append(self.entry_partner_name_partner_a)
        self.label_partner_name_partner_a.grid(sticky="w", row=1, column=0, padx=5, pady=5)
        self.entry_partner_name_partner_a.grid(sticky="we",row=1, column=1, padx=5, pady=5)

        self.label_partner_name_partner_b = tk.Label(self, text="Party Name Partner B:")
        self.entry_partner_name_partner_b = tk.Entry(self)
        self.entry_fields.append(self.entry_partner_name_partner_b)
        self.label_partner_name_partner_b.grid(sticky="w", row=1, column=2, padx=5, pady=5)
        self.entry_partner_name_partner_b.grid(sticky="we", row=1, column=3, padx=5, pady=5)

        self.label_party_id_partner_a = tk.Label(self, text="Party Id Partner A:")
        self.entry_party_id_partner_a = tk.Entry(self)
        self.entry_fields.append(self.entry_party_id_partner_a)
        self.label_party_id_partner_a.grid(sticky="w", row=2, column=0, padx=5, pady=5)
        self.entry_party_id_partner_a.grid(sticky="we", row=2, column=1, padx=5, pady=5)

        self.label_party_id_partner_b = tk.Label(self, text="Party Id Partner B:")
        self.entry_party_id_partner_b = tk.Entry(self)
        self.entry_fields.append(self.entry_party_id_partner_b)
        self.label_party_id_partner_b.grid(sticky="w", row=2, column=2, padx=5, pady=5)
        self.entry_party_id_partner_b.grid(sticky="we", row=2, column=3, padx=5, pady=5)

        self.label_party_id_type_partner_a = tk.Label(self, text="Party Id Type Partner A:")
        self.entry_party_id_type_partner_a = tk.Entry(self)
        self.entry_fields.append(self.entry_party_id_type_partner_a)
        self.label_party_id_type_partner_a.grid(sticky="w", row=3, column=0, padx=5, pady=5)
        self.entry_party_id_type_partner_a.grid(sticky="we", row=3, column=1, padx=5, pady=5)

        self.label_party_id_type_partner_b = tk.Label(self, text="Party Id Type Partner B:")
        self.entry_party_id_type_partner_b = tk.Entry(self)
        self.entry_fields.append(self.entry_party_id_type_partner_b)
        self.label_party_id_type_partner_b.grid(sticky="w", row=3, column=2, padx=5, pady=5)
        self.entry_party_id_type_partner_b.grid(sticky="we", row=3, column=3, padx=5, pady=5)

        self.label_start_date = tk.Label(self, text="Start date:")
        # resize based on screen size
        self.entry_start_date = tk.Entry(self)
        self.entry_fields.append(self.entry_start_date)
        self.label_start_date.grid(sticky="w", row=4, column=0, padx=5, pady=5)
        self.entry_start_date.grid(sticky="we", row=4, column=1, padx=5, pady=5)

        self.label_end_date = tk.Label(self, text="End date:")
        self.entry_end_date = tk.Entry(self)
        self.entry_fields.append(self.entry_end_date)
        self.label_end_date.grid(sticky="w", row=4, column=2, padx=5, pady=5)
        self.entry_end_date.grid(sticky="we", row=4, column=3, padx=5, pady=5)

        # Below each date: the same moment in Zulu time and in the timezone set on this computer
        self.label_start_date_times = tk.Label(self, text="", anchor="w")
        self.label_start_date_times.grid(sticky="we", row=5, column=1, padx=5)
        self.label_end_date_times = tk.Label(self, text="", anchor="w")
        self.label_end_date_times.grid(sticky="we", row=5, column=3, padx=5)
        for entry in (self.entry_start_date, self.entry_end_date):
            entry.bind("<KeyRelease>", self.update_date_labels)

        # The date buttons sit below the field they fill
        self.button_start_date_now = tk.Button(self, text="Set to now", command=self.set_start_date_to_now)
        self.button_start_date_now.grid(sticky="w", row=6, column=1, padx=5, pady=5)

        self.button_end_date_from_certificates = tk.Button(self, text="Set from certificates",
                                                           command=self.set_end_date_from_certificates)
        self.button_end_date_from_certificates.grid(sticky="w", row=6, column=3, padx=5, pady=5)

        # Entry columns share the available width, so the fields scale with the window
        self.grid_columnconfigure(1, weight=1, uniform="entries")
        self.grid_columnconfigure(3, weight=1, uniform="entries")

        self.bind_fields()
        update_entry_width(self.entry_fields, self.master.winfo_width())

    def bind_fields(self):
        self.all_entry_fields = {'CPAId': self.entry_cpa_id,
                           'partyNamePartnerA': self.entry_partner_name_partner_a,
                           'partyNamePartnerB': self.entry_partner_name_partner_b,
                           'partyIdPartnerA': self.entry_party_id_partner_a,
                           'partyIdPartnerB': self.entry_party_id_partner_b,
                           'partyIdTypePartnerA': self.entry_party_id_type_partner_a,
                           'partyIdTypePartnerB': self.entry_party_id_type_partner_b,
                           'cpaStartDate': self.entry_start_date,
                           'cpaEndDate': self.entry_end_date
                                 }
        self.all_dropdown_fields = {'cpaStatus': [self.cpa_status_value, self.entry_cpa_status]}

        for field_name, dropdown in self.all_dropdown_fields.items():
            dropdown_var, dropdown = dropdown
            dropdown_callback = partial(self.on_dropdown_change, field_name, dropdown_var)
            self.bind_event_to_widget(dropdown, "<Configure>", dropdown_callback)

        # Bind an event to the entry fields

        for field_name, entry in self.all_entry_fields.items():
            callback = partial(self.on_field_change, field_name)
            self.bind_event_to_entry(entry, callback)

    def on_dropdown_change(self, field_name, dropdown, event):
        # Now you have access to the dropdown widget
        selected_value = dropdown.get()  # Get the selected value
        # Do something with the selected value
        if self.master.xml_tree:
            change_entry = {field_name: selected_value}
            cpa_creator = CPACreator(self.master.root, self.logger)
            cpa_creator.apply_changes(change_entry)
            self.logger.debug(f"Dropdown changed {field_name} New selected value: {selected_value}")

    def on_field_change(self, field_name, event):
        # Now you have access to the field name
        entry = event.widget
        if self.master.xml_tree:
            change_entry = {field_name: entry.get()}
            try:
                cpa_creator = CPACreator(self.master.root, self.logger)
                cpa_creator.apply_changes(change_entry)
                self.logger.debug(f"Field '{field_name}' changed: New value: {entry.get()}")
            except Exception as e:
                self.logger.error(f"Error while changing element: {e}")

    def update_date_labels(self, event=None):
        """
        Shows the start and end date in Zulu time (UTC) and in local time below their fields.

        Tests:
        - test_update_date_labels_shows_zulu_and_local
        """
        self.label_start_date_times.config(text=describe_cpa_date(self.entry_start_date.get()))
        self.label_end_date_times.config(text=describe_cpa_date(self.entry_end_date.get()))

    def set_start_date_to_now(self):
        """
        Sets the start date of the CPA to the current date and time (UTC).

        Tests:
        - test_set_start_date_to_now
        - test_set_start_date_to_now_without_cpa
        """
        if self.master.root is None:
            self.logger.error("No CPA loaded")
            return
        start_date = datetime.now(timezone.utc).strftime(CPA_DATE_FORMAT)
        CPACreator(self.master.root, self.logger).apply_changes({'cpaStartDate': start_date})
        self.entry_start_date.delete(0, tk.END)
        self.entry_start_date.insert(0, start_date)
        self.update_date_labels()
        self.logger.info(f"Start date set to {start_date} (current date and time, UTC)")

    def set_end_date_from_certificates(self):
        """
        Sets the end date of the CPA to the expiry date of the certificate that expires first.

        All certificates in the CPA are taken into account: leaf, intermediate and root.
        With debug messages enabled, every certificate is listed in order of expiry.

        Tests:
        - test_set_end_date_from_certificates_uses_first_expiring
        - test_set_end_date_from_certificates_lists_certificates_in_debug
        - test_set_end_date_from_certificates_without_cpa
        - test_set_end_date_from_certificates_without_certificates
        """
        if self.master.root is None:
            self.logger.error("No CPA loaded")
            return
        expiries = get_certificate_expiries(self.master.root, self.logger)
        if not expiries:
            self.logger.error("No certificates found in the CPA, end date not changed")
            return
        for position, expiry in enumerate(expiries, start=1):
            self.logger.debug(f"Certificate {position}/{len(expiries)}: {expiry['common_name']} expires "
                              f"{expiry['not_after'].strftime(CPA_DATE_FORMAT)} "
                              f"({expiry['type']}, certId {expiry['cert_id']})")
        first_expiring = expiries[0]
        end_date = first_expiring['not_after'].strftime(CPA_DATE_FORMAT)
        CPACreator(self.master.root, self.logger).apply_changes({'cpaEndDate': end_date})
        self.entry_end_date.delete(0, tk.END)
        self.entry_end_date.insert(0, end_date)
        self.update_date_labels()
        self.logger.info(f"End date set to {end_date}: expiry of {first_expiring['type']} certificate "
                         f"'{first_expiring['subject']}' (certId {first_expiring['cert_id']})")
        if first_expiring['not_after'] < datetime.now(timezone.utc):
            self.logger.warning("This certificate has already expired, the end date is in the past")

    def bind_event_to_entry(self, entry, callback):
        entry.bind("<FocusOut>", lambda event, callback=callback: callback(event))

    def bind_event_to_widget(self, widget, event, callback):
        widget.bind(event, lambda event, callback=callback: callback(event))

    def reload(self):
        self.load()

    def load(self):
        update_entry_width(self.entry_fields, self.master.winfo_width())
        # Create an instance of the CPAParser class
        cpa_parser_functions = CPAParser(self.master.root)

        cpa_id = cpa_parser_functions.get_cpa_id()
        status = cpa_parser_functions.get_cpa_status()
        party_id_partner_a = cpa_parser_functions.get_party_id_partner_a()
        party_id_partner_b = cpa_parser_functions.get_party_id_partner_b()
        partner_name_partner_a = cpa_parser_functions.get_party_name_partner_a()
        partner_name_partner_b = cpa_parser_functions.get_party_name_partner_b()
        start_date = cpa_parser_functions.get_cpa_start_date()
        end_date = cpa_parser_functions.get_cpa_end_date()
        party_id_type_partner_a = cpa_parser_functions.get_party_ids_type_partner_a()
        party_id_type_partner_b = cpa_parser_functions.get_party_ids_type_partner_b()

        # Populate the GUI fields with the extracted data
        self.entry_cpa_id.delete(0, tk.END)
        self.entry_cpa_id.insert(0, cpa_id)

        self.cpa_status_value.set(status)

        self.entry_party_id_partner_a.delete(0, tk.END)
        self.entry_party_id_partner_a.insert(0, party_id_partner_a)

        self.entry_party_id_partner_b.delete(0, tk.END)
        self.entry_party_id_partner_b.insert(0, party_id_partner_b)

        self.entry_start_date.delete(0, tk.END)
        self.entry_start_date.insert(0, start_date)

        self.entry_end_date.delete(0, tk.END)
        self.entry_end_date.insert(0, end_date)

        self.entry_partner_name_partner_a.delete(0, tk.END)
        self.entry_partner_name_partner_a.insert(0, partner_name_partner_a)

        self.entry_partner_name_partner_b.delete(0, tk.END)
        self.entry_partner_name_partner_b.insert(0, partner_name_partner_b)

        self.entry_party_id_type_partner_a.delete(0, tk.END)
        self.entry_party_id_type_partner_a.insert(0, party_id_type_partner_a)

        self.entry_party_id_type_partner_b.delete(0, tk.END)
        self.entry_party_id_type_partner_b.insert(0, party_id_type_partner_b)

        self.update_date_labels()
