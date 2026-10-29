import copy
import difflib
import re
import tkinter as tk
from tkinter import ttk
from tkinter import font as tkfont

import lxml.etree as Et


def cpa_text(root):
    """
    Returns the CPA as indented XML text, so a change shows as a change of lines and not of layout.

    Test Functions:
        - test_cpa_text_ignores_layout
    """
    indented = copy.deepcopy(root)
    Et.indent(indented)
    return Et.tostring(indented, encoding='unicode')


def cpa_diff(old_text, new_text):
    """
    Returns the unified diff lines between the loaded and the current CPA, or an empty list when they are equal.

    Test Functions:
        - test_cpa_diff_shows_changed_line
        - test_cpa_diff_empty_without_changes
    """
    return list(difflib.unified_diff(old_text.splitlines(), new_text.splitlines(),
                                     fromfile='Loaded CPA', tofile='Current CPA', lineterm=''))


# Words, and every other character on its own: a changed value is marked as a whole, not letter by letter
WORD = re.compile(r'\w+|\W')


def changed_parts(old_line, new_line):
    """
    Returns the (start, end) character ranges of the words that differ in a removed and an added line.

    Test Functions:
        - test_changed_parts_marks_whole_words
    """
    old_words = [match.span() for match in WORD.finditer(old_line)]
    new_words = [match.span() for match in WORD.finditer(new_line)]
    matcher = difflib.SequenceMatcher(None, [old_line[start:end] for start, end in old_words],
                                      [new_line[start:end] for start, end in new_words], autojunk=False)
    old_parts, new_parts = [], []
    for operation, old_start, old_end, new_start, new_end in matcher.get_opcodes():
        if operation == 'equal':
            continue
        if old_start < old_end:
            old_parts.append((old_words[old_start][0], old_words[old_end - 1][1]))
        if new_start < new_end:
            new_parts.append((new_words[new_start][0], new_words[new_end - 1][1]))
    return old_parts, new_parts


def hunk_line_number(hunk_header):
    """Returns the first line number of the current CPA in a hunk header such as '@@ -1,7 +1,7 @@'."""
    match = re.match(r'@@ -\d+(?:,\d+)? \+(\d+)', hunk_header)
    return int(match.group(1)) if match else None


class DiffWindow(tk.Toplevel):
    """Read-only window that shows the changes of the CPA since it was loaded."""

    def __init__(self, master, diff_lines, confirm=False, **kw):
        """With confirm, the window asks whether to save the CPA with these changes; see confirmed."""
        super().__init__(master, **kw)
        self.title("Save CPA with these changes?" if confirm else "Changes since the CPA was loaded")
        self.geometry("1100x700")
        self.confirmed = False
        self.save_button = None
        self.cancel_button = None
        if confirm:
            self.add_confirm_buttons()

        fixed_font = tkfont.nametofont("TkFixedFont").copy()
        fixed_font.configure(size=max(fixed_font.cget("size"), 13))
        heading_font = tkfont.nametofont("TkDefaultFont").copy()
        heading_font.configure(size=heading_font.cget("size") + 2, weight="bold")

        # Long XML lines are wrapped, so a change at the end of a line (an attribute of the root element) is visible
        self.text = tk.Text(self, wrap="char", padx=15, pady=10, font=fixed_font, spacing1=2, spacing3=2)
        vsb = ttk.Scrollbar(self, orient="vertical", command=self.text.yview)
        self.text.configure(yscrollcommand=vsb.set)
        vsb.pack(side=tk.RIGHT, fill=tk.Y)
        self.text.pack(fill=tk.BOTH, expand=True)

        # Dark text: the light backgrounds must also be readable in dark mode
        self.text.tag_configure('summary', font=tkfont.nametofont("TkDefaultFont"), spacing3=4)
        self.text.tag_configure('heading', font=heading_font, spacing1=14, spacing3=6)
        self.text.tag_configure('context', foreground='#888888')
        self.text.tag_configure('removed', background='#ffe3e1', foreground='black', lmargin2=30)
        self.text.tag_configure('added', background='#dcf5dc', foreground='black', lmargin2=30)
        self.text.tag_configure('removed_part', background='#ff9e97', foreground='black')
        self.text.tag_configure('added_part', background='#8fdc8f', foreground='black')
        self.text.tag_configure('context', lmargin2=30)
        # The changed characters are marked over the line colour
        self.text.tag_raise('removed_part')
        self.text.tag_raise('added_part')

        self.changed_lines = 0
        self.insert_hunks(diff_lines)
        self.text.insert('1.0', f"{self.changed_lines} {'line' if self.changed_lines == 1 else 'lines'} changed since "
                                "the CPA was loaded. Red: loaded CPA, green: current CPA; the changed words are "
                                "darker. Grey lines are unchanged.\n", 'summary')
        self.text.configure(state="disabled")

    def add_confirm_buttons(self):
        # Packed first, so the buttons stay visible when the window is made small
        buttons = tk.Frame(self)
        buttons.pack(side=tk.BOTTOM, fill=tk.X, padx=10, pady=10)
        self.cancel_button = ttk.Button(buttons, text="Cancel", command=lambda: self.close(False))
        self.cancel_button.pack(side=tk.RIGHT, padx=5)
        self.save_button = ttk.Button(buttons, text="Save...", command=lambda: self.close(True))
        self.save_button.pack(side=tk.RIGHT, padx=5)
        self.bind("<Escape>", lambda event: self.close(False))
        self.protocol("WM_DELETE_WINDOW", lambda: self.close(False))

    def close(self, confirmed):
        self.confirmed = confirmed
        self.destroy()

    def insert_hunks(self, diff_lines):
        removed, added = [], []
        for line in diff_lines + ['']:
            if line.startswith(('---', '+++')):
                continue
            if line.startswith('-'):
                removed.append(line[1:])
                continue
            if line.startswith('+'):
                added.append(line[1:])
                continue
            self.insert_change(removed, added)
            removed, added = [], []
            if line.startswith('@@'):
                number = hunk_line_number(line)
                self.text.insert(tk.END, f"Line {number}\n" if number else "Change\n", 'heading')
            elif line:
                self.text.insert(tk.END, f"  {line[1:]}\n", 'context')

    def insert_change(self, removed, added):
        self.changed_lines += max(len(removed), len(added))
        # A removed and an added line at the same place are one changed line: mark the characters that differ
        for index, line in enumerate(removed):
            old_parts = changed_parts(line, added[index])[0] if index < len(added) else []
            self.insert_line('- ', line, 'removed', old_parts)
        for index, line in enumerate(added):
            new_parts = changed_parts(removed[index], line)[1] if index < len(removed) else []
            self.insert_line('+ ', line, 'added', new_parts)

    def insert_line(self, sign, line, tag, parts):
        start = self.text.index(tk.END + '-1c')
        self.text.insert(tk.END, sign + line + '\n', tag)
        for part_start, part_end in parts:
            self.text.tag_add(tag + '_part', f"{start}+{len(sign) + part_start}c", f"{start}+{len(sign) + part_end}c")
