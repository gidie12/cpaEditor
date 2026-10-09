import unittest
from unittest.mock import Mock
from lxml import etree as lxml_etree
import tkinter as tk
from application.app_classes.certificates_tab import Certificates, create_sub_elements


class TestCreateSubElements(unittest.TestCase):

    def setUp(self):
        self.master = tk.Tk()
        self.addCleanup(self.master.destroy)
        self.logger = Mock()
        self.cert_tab = Certificates(self.master, self.logger)

    def test_create_sub_elements_creates_correct_elements(self):
        parent = lxml_etree.Element('Parent')
        elements = [
            ('Child1', 'Text1'),
            ('Child2', 'Text2')
        ]
        create_sub_elements(parent, elements)

        self.assertEqual(parent[0].tag, 'Child1')
        self.assertEqual(parent[0].text, 'Text1')
        self.assertEqual(parent[1].tag, 'Child2')
        self.assertEqual(parent[1].text, 'Text2')

    def test_create_sub_elements_adds_elements_to_parent(self):
        parent = lxml_etree.Element('Parent')
        elements = [
            ('Child1', 'Text1'),
            ('Child2', 'Text2')
        ]
        create_sub_elements(parent, elements)

        self.assertEqual(len(parent), 2)
        self.assertEqual(parent[0].tag, 'Child1')
        self.assertEqual(parent[1].tag, 'Child2')

if __name__ == '__main__':
    unittest.main()