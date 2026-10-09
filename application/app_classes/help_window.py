import os
import re
import sys
import tkinter as tk
from tkinter import ttk
from tkinter import font as tkfont

# In a PyInstaller build the bundled data files live in sys._MEIPASS instead of the project root
BUNDLE_DIR = getattr(sys, '_MEIPASS', os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
README_FILE = os.path.join(BUNDLE_DIR, 'README.md')
HOW_TO_USE_FILE_EN = os.path.join(BUNDLE_DIR, 'docs', 'how-to-use.en.md')
HOW_TO_USE_FILE_NL = os.path.join(BUNDLE_DIR, 'docs', 'how-to-use.nl.md')

# **bold** and `code` inside a line
INLINE_MARKUP = re.compile(r'(\*\*.+?\*\*|`.+?`)')


def insert_markdown(text_widget, markdown_text):
    """
    Inserts Markdown into a Text widget using the tags 'title', 'heading', 'bold' and 'code'.

    Only the Markdown used by the help files is supported: headings, fenced code blocks, lists, **bold** and `code`.

    Test Functions:
        - test_insert_markdown_formats_headings_and_inline_markup
        - test_insert_markdown_keeps_code_block_literal
    """
    in_code_block = False
    for line in markdown_text.splitlines():
        if line.startswith('```'):
            in_code_block = not in_code_block
        elif in_code_block:
            text_widget.insert(tk.END, line + '\n', 'code')
        elif line.startswith('#'):
            level = len(line) - len(line.lstrip('#'))
            text_widget.insert(tk.END, line.lstrip('#').strip() + '\n', 'title' if level == 1 else 'heading')
        else:
            if line.startswith('- '):
                line = '• ' + line[2:]
            for part in INLINE_MARKUP.split(line):
                if part.startswith('**') and part.endswith('**') and len(part) > 4:
                    text_widget.insert(tk.END, part[2:-2], 'bold')
                elif part.startswith('`') and part.endswith('`') and len(part) > 2:
                    text_widget.insert(tk.END, part[1:-1], 'code')
                else:
                    text_widget.insert(tk.END, part)
            text_widget.insert(tk.END, '\n')


class HelpWindow(tk.Toplevel):
    """Read-only window that shows a Markdown help file of the application."""

    def __init__(self, master, help_file, title="CPA Editor Help", **kw):
        super().__init__(master, **kw)
        self.title(title)
        self.geometry("900x650")

        self.text = tk.Text(self, wrap="word", padx=15, pady=10)
        vsb = ttk.Scrollbar(self, orient="vertical", command=self.text.yview)
        self.text.configure(yscrollcommand=vsb.set)
        vsb.pack(side=tk.RIGHT, fill=tk.Y)
        self.text.pack(fill=tk.BOTH, expand=True)

        base_font = tkfont.nametofont("TkDefaultFont")
        size = base_font.cget("size")
        family = base_font.cget("family")
        self.text.configure(font=base_font)
        self.text.tag_configure('title', font=(family, size + 8, "bold"), spacing3=8)
        self.text.tag_configure('heading', font=(family, size + 3, "bold"), spacing1=10, spacing3=4)
        self.text.tag_configure('bold', font=(family, size, "bold"))
        self.text.tag_configure('code', font=tkfont.nametofont("TkFixedFont"))

        try:
            with open(help_file, encoding='utf-8') as file:
                insert_markdown(self.text, file.read())
        except OSError as e:
            self.text.insert(tk.END, f"Help file not found: {e}")
        self.text.configure(state="disabled")
