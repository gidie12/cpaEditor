
import tkinter as tk

from application.helper_classes.validateSchema import xmlValidation

class Validator(tk.Frame):
    def __init__(self, master, logger, **kw):
        super().__init__(master, **kw)
        self.master = master
        self.logger = logger

    def create(self):

        self.label_validation_errors = tk.Label(self, text="Validation Errors:")
        self.label_validation_errors.grid(sticky="w", row=0, column=0, padx=5, pady=5)

        self.validation_errors = tk.Text(self, height=30, wrap="word")  # Adjust height as needed
        self.validation_errors.insert('1.0','Geen validatie uitgevoerd')
        self.validation_errors.grid(sticky="wsne", row=0, column=1, padx=5, pady=5)

        # Create vertical scrollbar for the Text widget
        vsb = tk.Scrollbar(self, orient="vertical", command=self.validation_errors.yview)
        vsb.grid(row=0, column=2, sticky="ns")
        self.validation_errors.config(yscrollcommand=vsb.set)

        self.validate_button = tk.Button(self, text="Validate CPA", command=self.validate_schema)
        self.validate_button.grid(sticky='w', row=1, column=0, columnspan=3, padx=5, pady=5)

        # Configure grid weights for resizing
        self.grid_rowconfigure(0, weight=1)
        self.grid_columnconfigure(1, weight=1)
    def validate_schema(self):
        xml_object_validation = xmlValidation(root=self.master.root, is_xml_object=True)
        if xml_object_validation.validate() is True:
            self.validation_errors.delete('1.0', tk.END)
            self.validation_errors.insert('1.0', 'Geen fouten gevonden')
        else:
            self.validation_errors.delete('1.0', tk.END)
            self.validation_errors.insert('1.0', xml_object_validation.get_validation_errors())
            # # Check if the text is near the end of the visible area
            # last_visible_line = self.validation_errors.index("@0,%d" % (self.validation_errors.winfo_height()))
            # if self.validation_errors.compare("end-1c", ">=", last_visible_line):
            #     self.validation_errors.see("end")
