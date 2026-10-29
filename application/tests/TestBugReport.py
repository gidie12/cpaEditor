import unittest
from unittest.mock import patch
import urllib.parse
import tkinter as tk

from application import CpaEditorApp as cpa_editor_app
from application.helper_classes import bugReport
from application.helper_classes.bugReport import bug_report_url, system_description
from application.version import __version__


class TestBugReport(unittest.TestCase):

    def test_bug_report_url(self):
        """Test: bug_report_url"""
        url = bug_report_url('1.0.5+nightly.20261029.abc1234', 'Windows 11')
        address, query = url.split('?')
        self.assertEqual(address, 'https://github.com/gidie12/cpaEditor/issues/new')
        self.assertEqual(urllib.parse.parse_qs(query), {'template': ['bug_report.yml'],
                                                        'version': ['1.0.5+nightly.20261029.abc1234'],
                                                        'os': ['Windows 11']})

    def test_system_description(self):
        """Test: system_description"""
        with patch.object(bugReport.platform, 'system', return_value='Darwin'), \
                patch.object(bugReport.platform, 'mac_ver', return_value=('15.6', ('', '', ''), 'arm64')):
            self.assertEqual(system_description(), 'macOS 15.6')
        with patch.object(bugReport.platform, 'system', return_value='Windows'), \
                patch.object(bugReport.platform, 'release', return_value='11'):
            self.assertEqual(system_description(), 'Windows 11')

    def test_help_menu_reports_bug(self):
        """Test: help_menu_reports_bug"""
        with patch.object(tk.Tk, 'mainloop', lambda self: None):
            app = cpa_editor_app.CpaEditorApp()
        try:
            menu_bar = app.nametowidget(app.cget('menu'))
            help_menu = app.nametowidget(menu_bar.entrycget(0, 'menu'))
            with patch.object(cpa_editor_app.webbrowser, 'open') as open_browser:
                help_menu.invoke(help_menu.index("Report a bug"))
            url = open_browser.call_args.args[0]
            self.assertTrue(url.startswith('https://github.com/gidie12/cpaEditor/issues/new?template=bug_report.yml'))
            self.assertIn(urllib.parse.quote_plus(__version__), url)
            self.assertIn("do not attach the CPA", app.log_output.get('1.0', tk.END))
        finally:
            for handler in list(cpa_editor_app.logger.handlers):
                cpa_editor_app.logger.removeHandler(handler)
            app.destroy()


if __name__ == '__main__':
    unittest.main()
