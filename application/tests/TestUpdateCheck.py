import io
import socket
import ssl
import time
import unittest
from unittest.mock import patch
import urllib.error
import tkinter as tk

from application import CpaEditorApp as cpa_editor_app
from application.helper_classes import updateCheck
from application.helper_classes.updateCheck import (LATEST_RELEASE_DOWNLOAD_URL, LATEST_RELEASE_PAGE_URL,
                                                    check_for_update, describe_network_error, get_latest_version,
                                                    is_newer, parse_version)
from application.version import __version__


class TestUpdateCheck(unittest.TestCase):

    def test_parse_version(self):
        """Test: parse_version"""
        self.assertEqual(parse_version('v1.0.2'), (1, 0, 2))
        self.assertEqual(parse_version('1.10'), (1, 10))
        self.assertEqual(parse_version(__version__), parse_version('v' + __version__))

    def test_parse_version_invalid(self):
        """Test: parse_version_invalid"""
        for version in ('', 'latest', '1.0-beta', 'v1..2'):
            with self.assertRaises(ValueError):
                parse_version(version)

    def test_is_newer(self):
        """Test: is_newer"""
        self.assertTrue(is_newer('v1.0.10', '1.0.9'))
        self.assertTrue(is_newer('v2.0', '1.9.9'))
        self.assertFalse(is_newer('v1.1', '1.1.0'))
        self.assertFalse(is_newer('v1.0.2', '1.1.0'))

    def test_get_latest_version_reads_tag(self):
        """Test: get_latest_version_reads_tag"""
        with patch.object(updateCheck.urllib.request, 'urlopen',
                          return_value=io.BytesIO(b'{"tag_name": "v1.2.3", "html_url": "https://example.com"}')) as urlopen:
            self.assertEqual(get_latest_version(timeout=3), 'v1.2.3')
        request = urlopen.call_args.args[0]
        self.assertEqual(request.full_url, 'https://api.github.com/repos/gidie12/cpaEditor/releases/latest')
        self.assertIsNone(request.data)
        self.assertEqual(urlopen.call_args.kwargs['timeout'], 3)
        context = urlopen.call_args.kwargs['context']
        self.assertEqual(context.verify_mode, ssl.CERT_REQUIRED)
        self.assertTrue(context.check_hostname)

    def test_check_for_update_newer_version(self):
        """Test: check_for_update_newer_version"""
        with patch.object(updateCheck, 'get_latest_version', return_value='v1.2.0'):
            self.assertEqual(check_for_update('1.1.0'), {'current': '1.1.0', 'latest': '1.2.0', 'update_available': True})

    def test_check_for_update_up_to_date(self):
        """Test: check_for_update_up_to_date"""
        with patch.object(updateCheck, 'get_latest_version', return_value='v1.1.0'):
            self.assertFalse(check_for_update('1.1.0')['update_available'])

    def test_check_for_update_reports_error(self):
        """Test: check_for_update_reports_error"""
        with patch.object(updateCheck, 'get_latest_version', side_effect=OSError("no network")):
            self.assertEqual(check_for_update('1.1.0'), {'current': '1.1.0', 'error': 'no network'})
        with patch.object(updateCheck, 'get_latest_version', return_value='nightly'):
            self.assertIn('error', check_for_update('1.1.0'))

    def test_get_latest_version_falls_back_to_windows_internet_settings(self):
        """Test: get_latest_version_falls_back_to_windows_internet_settings"""
        unresolved = urllib.error.URLError(socket.gaierror(11001, 'getaddrinfo failed'))
        rate_limited = urllib.error.HTTPError(updateCheck.LATEST_RELEASE_API_URL, 403, 'rate limit', None, None)
        proxy_login = urllib.error.HTTPError(updateCheck.LATEST_RELEASE_API_URL, 407, 'proxy login', None, None)
        with patch.object(updateCheck, 'fetch_with_windows_internet_settings',
                          return_value=b'{"tag_name": "v1.2.3"}') as fallback:
            with patch.object(updateCheck.sys, 'platform', 'win32'):
                for error in (unresolved, proxy_login):
                    with patch.object(updateCheck.urllib.request, 'urlopen', side_effect=error):
                        self.assertEqual(get_latest_version(timeout=3), 'v1.2.3')
                self.assertEqual(fallback.call_args.args[0], updateCheck.LATEST_RELEASE_API_URL)
                self.assertEqual(fallback.call_args.args[2], 3)
                # An answer of GitHub itself is not retried
                fallback.reset_mock()
                with patch.object(updateCheck.urllib.request, 'urlopen', side_effect=rate_limited):
                    self.assertRaises(urllib.error.HTTPError, get_latest_version)
                fallback.assert_not_called()
                # When Windows cannot get out either, both errors are reported
                fallback.side_effect = OSError("no route")
                with patch.object(updateCheck.urllib.request, 'urlopen', side_effect=unresolved):
                    error_message = check_for_update('1.1.0')['error']
                self.assertIn('HTTPS_PROXY', error_message)
                self.assertIn('no route', error_message)
            # Other systems have no such settings
            fallback.reset_mock()
            with patch.object(updateCheck.sys, 'platform', 'darwin'):
                with patch.object(updateCheck.urllib.request, 'urlopen', side_effect=unresolved):
                    self.assertRaises(urllib.error.URLError, get_latest_version)
            fallback.assert_not_called()

    def test_describe_network_error(self):
        """Test: describe_network_error"""
        unresolved = urllib.error.URLError(socket.gaierror(11001, 'getaddrinfo failed'))
        message = describe_network_error(unresolved)
        self.assertIn('getaddrinfo failed', message)
        self.assertIn('HTTPS_PROXY', message)
        self.assertIn(LATEST_RELEASE_PAGE_URL, message)
        with patch.object(updateCheck, 'get_latest_version', side_effect=unresolved):
            self.assertEqual(check_for_update('1.1.0')['error'], message)
        refused = urllib.error.URLError(ConnectionRefusedError(61, 'Connection refused'))
        self.assertEqual(describe_network_error(refused), str(refused))


class TestUpdateCheckInApp(unittest.TestCase):

    def setUp(self):
        with patch.object(tk.Tk, 'mainloop', lambda self: None):
            self.app = cpa_editor_app.CpaEditorApp()
        # These tests run the event loop: keep the real check at startup (network) out of them
        self.app.after_cancel(self.app.startup_update_check)

    def tearDown(self):
        # The app logs into its own Text widget, which no longer exists after destroy
        for handler in list(cpa_editor_app.logger.handlers):
            cpa_editor_app.logger.removeHandler(handler)
        self.app.destroy()

    def last_log_line(self):
        return self.app.log_output.get('1.0', tk.END).strip().splitlines()[-1]

    def test_title_shows_version(self):
        """Test: title_shows_version"""
        self.assertEqual(self.app.title(), f"CPA Editor v{__version__}")

    def answer_dialog(self, button_text):
        """Presses a button of the update popup as soon as it is shown."""
        def press():
            for frame in self.app.update_dialog.winfo_children():
                for widget in frame.winfo_children():
                    if widget.cget('text') == button_text:
                        widget.invoke()
                        return
        self.app.after(50, press)

    def test_help_menu_checks_for_updates(self):
        """Test: help_menu_checks_for_updates"""
        menu_bar = self.app.nametowidget(self.app.cget('menu'))
        help_menu = self.app.nametowidget(menu_bar.entrycget(0, 'menu'))
        self.assertEqual(help_menu.entrycget(tk.END, 'label'), "Check for updates")
        result = {'current': __version__, 'latest': __version__, 'update_available': False}
        with patch.object(cpa_editor_app, 'check_for_update', return_value=result) as check:
            help_menu.invoke(tk.END)
            self.assertEqual(self.app.update_check_results.get(timeout=5), (False, result))
        check.assert_called_once_with(__version__)

    def test_check_for_updates_at_startup(self):
        """Test: check_for_updates_at_startup"""
        result = {'current': '1.1.0', 'latest': '1.2.0', 'update_available': True}
        with patch.object(tk.Tk, 'mainloop', lambda self: None), \
                patch.object(cpa_editor_app.CpaEditorApp, 'check_for_updates') as check:
            app = cpa_editor_app.CpaEditorApp()
            deadline = time.time() + 5
            while not check.called and time.time() < deadline:
                app.update()
            app.destroy()
        check.assert_called_once_with(automatic=True)

        with patch.object(cpa_editor_app, 'check_for_update', return_value=result), \
                patch.object(self.app, 'show_update_result') as show:
            self.app.check_for_updates(automatic=True)
            deadline = time.time() + 5
            while not show.called and time.time() < deadline:
                self.app.update()
        show.assert_called_once_with(result, True)
        self.assertNotIn("Checking for updates", self.app.log_output.get('1.0', tk.END))

    def test_show_update_result_download(self):
        """Test: show_update_result_download"""
        result = {'current': '1.1.0', 'latest': '1.2.0', 'update_available': True}
        for host, url in (('windows', LATEST_RELEASE_DOWNLOAD_URL), ('mac', LATEST_RELEASE_PAGE_URL)):
            self.app.host = host
            self.answer_dialog("Download")
            with patch.object(cpa_editor_app.webbrowser, 'open') as open_browser:
                self.app.show_update_result(result, automatic=True)
            open_browser.assert_called_once_with(url)
        self.assertIn("Version v1.2.0 is available, this is version v1.1.0", self.last_log_line())

    def test_show_update_result_skip(self):
        """Test: show_update_result_skip"""
        self.answer_dialog("Skip")
        with patch.object(cpa_editor_app.webbrowser, 'open') as open_browser:
            self.app.show_update_result({'current': '1.1.0', 'latest': '1.2.0', 'update_available': True})
        open_browser.assert_not_called()
        self.assertFalse(self.app.update_dialog.winfo_exists())

    def test_ask_download_shows_versions(self):
        """Test: ask_download_shows_versions"""
        texts = []
        def read_and_close():
            dialog = self.app.update_dialog
            texts.extend(widget.cget('text') for widget in dialog.winfo_children() if isinstance(widget, tk.Label))
            self.assertTrue(dialog.bind("<Escape>"))
            # Closing the popup with the window button counts as Skip
            dialog.tk.call(dialog.protocol("WM_DELETE_WINDOW"))
        self.app.after(50, read_and_close)
        self.assertFalse(self.app.ask_download({'current': '1.1.0', 'latest': '1.2.0', 'update_available': True}))
        self.assertEqual(texts, ["Version v1.2.0 of the CPA Editor is available.\nYou are using version v1.1.0."])

    def test_show_update_result_up_to_date_and_error(self):
        """Test: show_update_result_up_to_date_and_error"""
        with patch.object(self.app, 'ask_download') as ask:
            self.app.show_update_result({'current': '1.1.0', 'latest': '1.1.0', 'update_available': False})
            self.assertIn("You are using the latest version (v1.1.0)", self.last_log_line())
            self.app.show_update_result({'current': '1.1.0', 'error': 'no network'})
            self.assertIn("ERROR - Could not check for updates: no network", self.last_log_line())
        ask.assert_not_called()

    def test_show_update_result_automatic_is_quiet(self):
        """Test: show_update_result_automatic_is_quiet"""
        with patch.object(self.app, 'ask_download') as ask:
            self.app.show_update_result({'current': '1.1.0', 'latest': '1.1.0', 'update_available': False}, automatic=True)
            self.app.show_update_result({'current': '1.1.0', 'error': 'no network'}, automatic=True)
        ask.assert_not_called()
        # Debug messages are off by default, so nothing is shown
        self.assertEqual(self.app.log_output.get('1.0', tk.END).strip(), '')

if __name__ == '__main__':
    unittest.main()
