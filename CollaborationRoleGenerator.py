import os
import tkinter as tk
from tkinter import ttk, filedialog, messagebox
from jinja2 import Template
import uuid  # Import for UUID generation

from application.helper_classes.directories import TEMPLATES_DIR


class CollaborationRoleApp(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("Collaboration Role Generator")
        self.geometry("1000x800")
        self.create_widgets()

        self.can_sends = []
        self.can_receives = []

        # Load the Jinja2 template from the file
        self.load_template()

    def load_template(self):
        """Load the Jinja2 template from an external file."""
        try:
            with open(os.path.join(f"{TEMPLATES_DIR}", "collaboration_role","collaboration_role_template.j2"), "r") as template_file:
                self.jinja2_template = template_file.read()
        except FileNotFoundError:
            messagebox.showerror("Error", "Template file not found. Please ensure 'collaboration_role_template.j2' is in the correct directory.")
            self.jinja2_template = ""  # Fallback to an empty string if file is not found

    def create_widgets(self):
        # Process and Role fields
        self.inputs = {}
        input_labels = [
            "Process Name", "Process Version", "Process HREF", "Process UUID",
            "Role Name", "Role HREF", "Service Type", "Service Name"
        ]
        for i, label in enumerate(input_labels):
            tk.Label(self, text=label).grid(row=i, column=0, sticky="w", padx=5, pady=5)
            self.inputs[label] = tk.Entry(self, width=50)
            self.inputs[label].grid(row=i, column=1, padx=5, pady=5)

        # Add Generate UUID button
        self.generate_uuid_button = tk.Button(self, text="Generate UUID", command=self.generate_uuid)
        self.generate_uuid_button.grid(row=3, column=2, padx=5, pady=5)

        # CanSend and CanReceive Sections
        self.can_send_frame = tk.LabelFrame(self, text="CanSend Items")
        self.can_send_frame.grid(row=8, column=0, columnspan=2, padx=10, pady=10, sticky="nsew")
        self.add_can_send_button = tk.Button(self.can_send_frame, text="Add CanSend", command=self.add_can_send)
        self.add_can_send_button.pack(pady=5)

        self.can_receive_frame = tk.LabelFrame(self, text="CanReceive Items")
        self.can_receive_frame.grid(row=9, column=0, columnspan=2, padx=10, pady=10, sticky="nsew")
        self.add_can_receive_button = tk.Button(self.can_receive_frame, text="Add CanReceive", command=self.add_can_receive)
        self.add_can_receive_button.pack(pady=5)

        # Generate and Save Buttons
        self.generate_button = tk.Button(self, text="Generate XML", command=self.generate_xml)
        self.generate_button.grid(row=10, column=0, columnspan=2, pady=10)

        self.output_box = tk.Text(self, wrap="word", height=15)
        self.output_box.grid(row=11, column=0, columnspan=2, padx=5, pady=10)

        self.save_button = tk.Button(self, text="Save XML", command=self.save_xml)
        self.save_button.grid(row=12, column=0, columnspan=2, pady=10)

    def generate_uuid(self):
        """Generate a unique UUID and populate the 'Process UUID' field."""
        unique_id = str(uuid.uuid4())
        self.inputs["Process UUID"].delete(0, tk.END)
        self.inputs["Process UUID"].insert(0, unique_id)

    def add_can_send(self):
        """Add a CanSend item."""
        frame = tk.Frame(self.can_send_frame)
        frame.pack(pady=5, fill="x")

        action_binding_id = tk.Entry(frame, width=20)
        action_binding_id.grid(row=0, column=0, padx=5, pady=5)
        action_binding_id.insert(0, "Action Binding ID")

        action = tk.Entry(frame, width=20)
        action.grid(row=0, column=1, padx=5, pady=5)
        action.insert(0, "Action")

        package_id = tk.Entry(frame, width=20)
        package_id.grid(row=0, column=2, padx=5, pady=5)
        package_id.insert(0, "Package ID")

        channel_id = tk.Entry(frame, width=20)
        channel_id.grid(row=0, column=3, padx=5, pady=5)
        channel_id.insert(0, "Channel ID")

        other_party_binding = tk.Entry(frame, width=20)
        other_party_binding.grid(row=0, column=4, padx=5, pady=5)
        other_party_binding.insert(0, "Other Party Action Binding")

        self.can_sends.append({
            "action_binding_id": action_binding_id,
            "action": action,
            "package_id": package_id,
            "channel_id": channel_id,
            "other_party_action_binding": other_party_binding,
        })

    def add_can_receive(self):
        """Add a CanReceive item."""
        frame = tk.Frame(self.can_receive_frame)
        frame.pack(pady=5, fill="x")

        action_binding_id = tk.Entry(frame, width=20)
        action_binding_id.grid(row=0, column=0, padx=5, pady=5)
        action_binding_id.insert(0, "Action Binding ID")

        action = tk.Entry(frame, width=20)
        action.grid(row=0, column=1, padx=5, pady=5)
        action.insert(0, "Action")

        package_id = tk.Entry(frame, width=20)
        package_id.grid(row=0, column=2, padx=5, pady=5)
        package_id.insert(0, "Package ID")

        channel_id = tk.Entry(frame, width=20)
        channel_id.grid(row=0, column=3, padx=5, pady=5)
        channel_id.insert(0, "Channel ID")

        other_party_binding = tk.Entry(frame, width=20)
        other_party_binding.grid(row=0, column=4, padx=5, pady=5)
        other_party_binding.insert(0, "Other Party Action Binding")

        self.can_receives.append({
            "action_binding_id": action_binding_id,
            "action": action,
            "package_id": package_id,
            "channel_id": channel_id,
            "other_party_action_binding": other_party_binding,
        })

    def generate_xml(self):
        """Generate the XML from the form data."""
        if not self.jinja2_template:
            messagebox.showerror("Error", "Template is missing. Please ensure the template file is loaded correctly.")
            return

        template = Template(self.jinja2_template)
        data = {
            "process_name": self.inputs["Process Name"].get(),
            "process_version": self.inputs["Process Version"].get(),
            "process_href": self.inputs["Process HREF"].get(),
            "process_uuid": self.inputs["Process UUID"].get(),
            "role_name": self.inputs["Role Name"].get(),
            "role_href": self.inputs["Role HREF"].get(),
            "service_type": self.inputs["Service Type"].get(),
            "service_name": self.inputs["Service Name"].get(),
            "can_sends": [{
                "action_binding_id": cs["action_binding_id"].get(),
                "action": cs["action"].get(),
                "package_id": cs["package_id"].get(),
                "channel_id": cs["channel_id"].get(),
                "other_party_action_binding": cs["other_party_action_binding"].get(),
                "transaction_characteristics": {
                    "is_authenticated": "transient",
                    "is_authorization_required": "true",
                    "is_intelligible_check_required": "false",
                    "is_non_repudiation_receipt_required": "false",
                    "is_non_repudiation_required": "false",
                    "is_confidential": "transient",
                    "is_tamper_proof": "transient",
                    "time_to_perform": "PT1H"
                }
            } for cs in self.can_sends],
            "can_receives": [{
                "action_binding_id": cr["action_binding_id"].get(),
                "action": cr["action"].get(),
                "package_id": cr["package_id"].get(),
                "channel_id": cr["channel_id"].get(),
                "other_party_action_binding": cr["other_party_action_binding"].get(),
                "transaction_characteristics": {
                    "is_authenticated": "transient",
                    "is_authorization_required": "true",
                    "is_intelligible_check_required": "false",
                    "is_non_repudiation_receipt_required": "false",
                    "is_non_repudiation_required": "false",
                    "is_confidential": "transient",
                    "is_tamper_proof": "transient",
                    "time_to_perform": "PT1H"
                }
            } for cr in self.can_receives],
        }

        xml_content = template.render(data)
        self.output_box.delete("1.0", tk.END)
        self.output_box.insert(tk.END, xml_content)

    def save_xml(self):
        """Save the generated XML to a file."""
        file_path = filedialog.asksaveasfilename(defaultextension=".xml", filetypes=[("XML files", "*.xml")])
        if file_path:
            with open(file_path, "w") as file:
                file.write(self.output_box.get("1.0", tk.END))
            messagebox.showinfo("Success", "XML saved successfully!")

if __name__ == "__main__":
    app = CollaborationRoleApp()
    app.mainloop()