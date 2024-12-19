import unittest
import os
import re
from application.helper_classes.directories import DOCUMENTATION_DIR

class TestMarkdownLinks(unittest.TestCase):
    def setUp(self):
        self.base_dir = os.path.join(DOCUMENTATION_DIR, 'sources', 'markdown', 'cpp-cpa')
        self.elements_dir = os.path.join(self.base_dir, 'elements')

        # Ensure the directory exists
        if not os.path.exists(self.elements_dir):
            os.makedirs(self.elements_dir)

        self.md_files = [os.path.join(self.elements_dir, f) for f in os.listdir(self.elements_dir) if f.endswith('.md')]
        self.md_files.append(os.path.join(self.base_dir, 'Readme.md'))

        self.html_files = [os.path.join(self.elements_dir, f) for f in os.listdir(self.elements_dir) if f.endswith('.html')]

    def test_markdown_links(self):
        link_pattern = re.compile(r'\[.*?\]\((.*?)\)')
        for md_file in self.md_files:
            with open(md_file, 'r') as file:
                content = file.read()
                links = link_pattern.findall(content)
                for link in links:
                    if link.startswith('http'):
                        continue  # Skip external links
                    link_path = os.path.normpath(os.path.join(os.path.dirname(md_file), link))
                    self.assertTrue(os.path.exists(link_path), f"Link {link} in {md_file} does not resolve")

    def test_html_links(self):
        link_pattern = re.compile(r'href="(.*?)"')
        for html_file in self.html_files:
            with open(html_file, 'r') as file:
                content = file.read()
                links = link_pattern.findall(content)
                for link in links:
                    if link.startswith('http'):
                        continue  # Skip external links
                    link_path = os.path.normpath(os.path.join(os.path.dirname(html_file), link))
                    self.assertTrue(os.path.exists(link_path), f"Link {link} in {html_file} does not resolve")

if __name__ == '__main__':
    unittest.main()