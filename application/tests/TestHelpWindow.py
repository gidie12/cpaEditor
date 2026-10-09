import unittest
from unittest.mock import patch
import tkinter as tk

from application import CpaEditorApp as cpa_editor_app
from application.app_classes.help_window import (HelpWindow, HOW_TO_USE_FILE_EN, HOW_TO_USE_FILE_NL, README_FILE,
                                                 insert_markdown)


class TestInsertMarkdown(unittest.TestCase):

    def setUp(self):
        self.master = tk.Tk()
        self.text = tk.Text(self.master)

    def tearDown(self):
        self.master.destroy()

    def tagged(self, tag):
        ranges = self.text.tag_ranges(tag)
        return [self.text.get(start, end) for start, end in zip(ranges[::2], ranges[1::2])]

    def test_insert_markdown_formats_headings_and_inline_markup(self):
        """Test: insert_markdown_formats_headings_and_inline_markup"""
        insert_markdown(self.text, "# Title\n## Heading\n- Click **Load CPA** or run `main.py`")
        self.assertEqual(self.text.get('1.0', tk.END).strip(),
                         "Title\nHeading\n• Click Load CPA or run main.py")
        self.assertEqual(self.tagged('title'), ["Title\n"])
        self.assertEqual(self.tagged('heading'), ["Heading\n"])
        self.assertEqual(self.tagged('bold'), ["Load CPA"])
        self.assertEqual(self.tagged('code'), ["main.py"])

    def test_insert_markdown_keeps_code_block_literal(self):
        """Test: insert_markdown_keeps_code_block_literal"""
        insert_markdown(self.text, "```bash\n# not a **heading**\n```\nafter")
        self.assertEqual(self.text.get('1.0', tk.END).strip(), "# not a **heading**\nafter")
        self.assertEqual(self.tagged('code'), ["# not a **heading**\n"])
        self.assertEqual(self.tagged('bold'), [])


class TestHelpWindow(unittest.TestCase):

    def setUp(self):
        with patch.object(tk.Tk, 'mainloop', lambda self: None):
            self.app = cpa_editor_app.CpaEditorApp()

    def tearDown(self):
        # The app logs into its own Text widget, which no longer exists after destroy
        for handler in list(cpa_editor_app.logger.handlers):
            cpa_editor_app.logger.removeHandler(handler)
        self.app.destroy()

    def test_help_menu_opens_help_files(self):
        """Test: help_menu_opens_help_files"""
        menu_bar = self.app.nametowidget(self.app.cget('menu'))
        help_menu = self.app.nametowidget(menu_bar.entrycget(0, 'menu'))
        self.assertEqual(menu_bar.entrycget(0, 'label'), "Help")
        expected = [("How to use (English)", HOW_TO_USE_FILE_EN, "How to use the CPA Editor"),
                    ("Handleiding (Nederlands)", HOW_TO_USE_FILE_NL, "Handleiding CPA Editor"),
                    ("Readme", README_FILE, "Set end date from certificates")]
        for index, (label, help_file, text) in enumerate(expected):
            self.assertEqual(help_menu.entrycget(index, 'label'), label)
            help_menu.invoke(index)
            help_window = self.app.help_windows[help_file]
            self.assertEqual(help_window.title(), label)
            self.assertIn(text, help_window.text.get('1.0', tk.END))
            self.assertEqual(help_window.text.cget('state'), 'disabled')

    def test_how_to_use_guides_have_same_structure(self):
        """Test: how_to_use_guides_have_same_structure"""
        def structure(help_file):
            with open(help_file, encoding='utf-8') as file:
                lines = file.read().splitlines()
            # Dutch has one extra paragraph about the English button names
            return [line.split(' ')[0] for line in lines if line.startswith(('#', '- '))]
        self.assertEqual(structure(HOW_TO_USE_FILE_EN), structure(HOW_TO_USE_FILE_NL))

    def test_show_help_reuses_open_window(self):
        """Test: show_help_reuses_open_window"""
        self.app.show_help(README_FILE, "Readme")
        first_window = self.app.help_windows[README_FILE]
        self.app.show_help(README_FILE, "Readme")
        self.assertIs(self.app.help_windows[README_FILE], first_window)
        first_window.destroy()
        self.app.show_help(README_FILE, "Readme")
        self.assertIsNot(self.app.help_windows[README_FILE], first_window)

    def test_help_window_without_help_file(self):
        """Test: help_window_without_help_file"""
        window = HelpWindow(self.app, help_file='does_not_exist.md')
        self.assertIn("Help file not found", window.text.get('1.0', tk.END))


if __name__ == '__main__':
    unittest.main()
