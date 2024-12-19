import unittest
from unittest.mock import Mock, patch
import tkinter as tk
from application.app_classes.comment_tab import Comment

class TestCommentTab(unittest.TestCase):
    def setUp(self):
        self.master = tk.Tk()
        self.logger = Mock()
        self.comment_tab = Comment(self.master, self.logger)
        self.comment_tab.create()

    def loads_comments_correctly(self):
        mock_comment = Mock()
        mock_comment.text = "Test Comment"
        self.comment_tab.comments_objects = [mock_comment]
        self.comment_tab.load()
        self.assertEqual(self.comment_tab.listbox.size(), 1)
        self.assertEqual(self.comment_tab.listbox.get(0), "Test Comment")

    def adds_comment_correctly(self):
        self.comment_tab.comment_entry.insert(0, "New Comment")
        self.comment_tab.add_item()
        self.assertEqual(self.comment_tab.listbox.size(), 1)
        self.assertEqual(self.comment_tab.listbox.get(0), "New Comment")

    def updates_comment_correctly(self):
        self.comment_tab.comment_entry.insert(0, "Initial Comment")
        self.comment_tab.add_item()
        self.comment_tab.listbox.select_set(0)
        self.comment_tab.comment_entry.delete(0, tk.END)
        self.comment_tab.comment_entry.insert(0, "Updated Comment")
        self.comment_tab.update_item()
        self.assertEqual(self.comment_tab.listbox.get(0), "Updated Comment")

    def deletes_comment_correctly(self):
        self.comment_tab.comment_entry.insert(0, "Comment to Delete")
        self.comment_tab.add_item()
        self.comment_tab.listbox.select_set(0)
        self.comment_tab.delete_item()
        self.assertEqual(self.comment_tab.listbox.size(), 0)

    def handles_empty_comment_entry(self):
        self.comment_tab.comment_entry.insert(0, "")
        self.comment_tab.add_item()
        self.assertEqual(self.comment_tab.listbox.size(), 0)

    def handles_no_selection_on_update(self):
        self.comment_tab.comment_entry.insert(0, "Comment")
        self.comment_tab.add_item()
        self.comment_tab.comment_entry.delete(0, tk.END)
        self.comment_tab.comment_entry.insert(0, "Updated Comment")
        self.comment_tab.selected_index = None
        self.comment_tab.update_item()
        self.assertEqual(self.comment_tab.listbox.get(0), "Comment")

    def handles_no_selection_on_delete(self):
        self.comment_tab.comment_entry.insert(0, "Comment")
        self.comment_tab.add_item()
        self.comment_tab.selected_index = None
        self.comment_tab.delete_item()
        self.assertEqual(self.comment_tab.listbox.size(), 1)