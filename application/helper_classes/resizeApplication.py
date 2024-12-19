def update_entry_width(entry_fields, screen_width):
    """
    Updates the width of entry fields based on the screen width.

    Args:
        entry_fields (list): A list of tkinter Entry widgets.
        screen_width (int): The width of the screen.

    Returns:
        None

    Test Functions:
        - test_update_entry_width_sets_correct_width
        - test_update_entry_width_handles_no_entry_fields
    """
    entry_width = max(10, int(screen_width / 100))
    for entry in entry_fields:
        entry.config(width=entry_width)

def update_button_width(master, button_fields):
    screen_width = master.winfo_screenwidth()
    button_width = max(10, int(screen_width / 100))
    for button in button_fields:
        button.config(width=button_width)