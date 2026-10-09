import unittest
from unittest.mock import Mock
import tkinter as tk
from application.app_classes.certificates_tab import Certificates, replace_xml_object


class TestReplaceXmlObject(unittest.TestCase):
    def test_replace_xml_object_appends_new_element_when_xml_element_is_none(self):
        master = tk.Tk()
        self.addCleanup(master.destroy)

        parent_xml_element = Mock()
        new_element = Mock()
        cert_tab = Certificates(master, Mock())

        replace_xml_object(parent_xml_element, None, new_element)

        parent_xml_element.append.assert_called_once_with(new_element)


    def test_replace_xml_object_replaces_existing_element_with_new_element(self):
        master = tk.Tk()
        self.addCleanup(master.destroy)

        parent_xml_element = Mock()
        xml_element = Mock()
        new_element = Mock()
        cert_tab = Certificates(master, Mock())

        replace_xml_object(parent_xml_element, xml_element, new_element)

        parent_xml_element.remove.assert_called_once_with(xml_element)
        parent_xml_element.append.assert_called_once_with(new_element)


    def test_replace_xml_object_does_nothing_when_new_element_is_none(self):
        master = tk.Tk()
        self.addCleanup(master.destroy)

        parent_xml_element = Mock()
        xml_element = Mock()
        cert_tab = Certificates(master, Mock())

        replace_xml_object(parent_xml_element, xml_element, None)

        parent_xml_element.remove.assert_not_called()
        parent_xml_element.append.assert_not_called()


    def test_replace_xml_object_does_nothing_when_parent_xml_element_is_none(self):
        master = tk.Tk()
        self.addCleanup(master.destroy)

        xml_element = Mock()
        new_element = Mock()
        cert_tab = Certificates(master, Mock())

        replace_xml_object(None, xml_element, new_element)

        xml_element.remove.assert_not_called()
        xml_element.append.assert_not_called()