import unittest
from unittest.mock import Mock, patch
import tkinter as tk

from lxml import etree

from application.app_classes.collaboration_role import CollaborationRole
from application.app_classes.service_action_dialog import CollaborationRoleDialog, ServiceActionDialog
from application.helper_classes.collaborationRoles import cpa, describe_action, find_action_by_binding_id
from application.tests.TestCollaborationRoles import CPA


class TestCollaborationRoleTab(unittest.TestCase):

    def setUp(self):
        self.master = tk.Tk()
        self.addCleanup(self.master.destroy)
        self.master.host = 'mac'
        self.master.root = etree.fromstring(CPA)
        self.logger = Mock()
        self.tab = CollaborationRole(self.master, self.logger)
        self.tab.create()
        self.tab.load()

    def tree(self):
        return self.tab.tree_editor

    def rows(self, parent=""):
        """Text and values of all rows below the parent, depth first."""
        rows = []
        for item in self.tree().get_children(parent):
            rows.append((self.tree().item(item, 'text'), tuple(self.tree().item(item, 'values'))))
            rows.extend(self.rows(item))
        return rows

    def select(self, element):
        item = self.tab.find_item(element)
        self.tree().selection_set(item)
        self.tab.on_treeview_select()
        return item

    def action(self, binding_id):
        return find_action_by_binding_id(self.master.root, binding_id)

    def role(self, partner_index, role_index):
        party_info = self.master.root.findall(cpa('PartyInfo'))[partner_index]
        return party_info.findall(cpa('CollaborationRole'))[role_index]

    def test_load_shows_partners_services_and_actions(self):
        """Test: the tree shows partner, service and one row per action with its values in the right columns"""
        rows = self.rows()
        self.assertEqual(rows[0], ("Partner A", ()))
        self.assertEqual(rows[1][0], "Service Order [urn:test]   Role Sender   Process Process 1.0")
        self.assertEqual(rows[2], ("▶ Send", ('Place', 'A_reliable', 'pkg', 'B_S1_R_Place_PV_1', 'OK')))
        self.assertEqual(rows[3], ("◀ Receive", ('Confirm', 'A_default', 'pkg', 'B_S1_S_Confirm_PV_1', 'OK')))
        self.assertEqual([text for text, _ in rows if text.startswith('Partner')], ["Partner A", "Partner B"])
        self.assertEqual(len(rows), 2 + 4 + 4)

    def test_load_marks_actions_with_problems(self):
        """Test: an action with a wrong channel is red and the detail panel names the problem"""
        self.action('B_S1_S_Confirm_PV_1').find(f"{cpa('ThisPartyActionBinding')}/{cpa('ChannelId')}").text = 'nope'
        self.tab.load()
        item = self.select(self.action('B_S1_S_Confirm_PV_1'))
        self.assertEqual(self.tree().item(item, 'tags'), ('problem',))
        self.assertEqual(self.tree().item(item, 'values')[4], '1 problem(s)')
        self.assertEqual(self.tab.problems_text.get(), "ChannelId 'nope' is not a DeliveryChannel of this partner")

    def test_filter_shows_matching_actions(self):
        """Test: filter_shows_matching_actions"""
        self.tab.filter_text.set('confirm')
        actions = [values[0] for text, values in self.rows() if values]
        self.assertEqual(actions, ['Confirm', 'Confirm'])
        self.tab.filter_text.set('invoice')
        self.assertEqual([text for text, _ in self.rows() if text.startswith('Service')],
                         ["Service Invoice [urn:test]   Role Sender   Process Process 1.0",
                          "Service Invoice [urn:test]   Role Receiver   Process Process 1.0"])
        self.tab.filter_text.set('')
        self.assertEqual(len(self.rows()), 10)

    def test_select_action_fills_detail_panel(self):
        """Test: select_action_fills_detail_panel"""
        self.select(self.action('A_S1_S_Place_PV_1'))
        self.assertEqual(self.tab.binding_id.get(), 'A_S1_S_Place_PV_1')
        self.assertEqual(self.tab.action_name.get(), 'Place')
        self.assertEqual(self.tab.channel_id.get(), 'A_reliable')
        self.assertEqual(tuple(self.tab.channel_combobox.cget('values')), ('A_default', 'A_reliable'))
        self.assertEqual(tuple(self.tab.package_combobox.cget('values')), ('pkg', 'other_pkg'))
        self.assertEqual(self.tab.characteristics.get()['isAuthenticated'], 'persistent')
        self.assertEqual(str(self.tab.save_button.cget('state')), 'normal')

        self.select(self.role(0, 0))
        self.assertEqual(self.tab.binding_id.get(), '')
        self.assertEqual(str(self.tab.action_entry.cget('state')), 'disabled')

    def test_save_changes_updates_action_and_keeps_selection(self):
        """Test: save_changes_updates_action_and_keeps_selection"""
        element = self.action('A_S1_S_Place_PV_1')
        self.select(element)
        self.tab.action_name.set('PlaceOrder')
        self.tab.channel_id.set('A_default')
        self.tab.characteristics.variables['isTamperProof'].set('persistent')
        self.tab.save_changes()
        action = describe_action(element)
        self.assertEqual((action['action'], action['channel_ids']), ('PlaceOrder', ['A_default']))
        self.assertEqual(action['characteristics']['isTamperProof'], 'persistent')
        self.assertEqual(self.tree().selection(), (self.tab.find_item(element),))
        self.assertEqual(self.tree().item(self.tab.find_item(element), 'values')[0], 'PlaceOrder')

    @patch('application.app_classes.collaboration_role.messagebox')
    def test_save_changes_shows_error(self, mock_messagebox):
        """Test: save_changes_shows_error"""
        self.select(self.action('A_S1_S_Place_PV_1'))
        self.tab.action_name.set('')
        self.tab.save_changes()
        mock_messagebox.showerror.assert_called_once()
        self.assertEqual(describe_action(self.action('A_S1_S_Place_PV_1'))['action'], 'Place')

    @patch('application.app_classes.collaboration_role.messagebox')
    def test_add_service_action_needs_selection(self, mock_messagebox):
        """Test: add_service_action_needs_selection"""
        self.assertIsNone(self.tab.add_service_action())
        mock_messagebox.showinfo.assert_called_once()

    def test_add_service_action_opens_dialog_and_reloads(self):
        """Test: the dialog is for the role of the selected action; after Add the new action is selected"""
        self.select(self.action('A_S1_S_Place_PV_1'))
        dialog = self.tab.add_service_action()
        self.assertIsInstance(dialog, ServiceActionDialog)
        self.assertIs(dialog.collaboration_role, self.role(0, 0))
        dialog.action_name.set('Cancel')
        element, mirror_element = dialog.submit()
        self.assertEqual(self.tree().selection(), (self.tab.find_item(element),))
        self.assertIsNotNone(self.tab.find_item(mirror_element))
        self.logger.info.assert_called_with("Service action added: A_S1_S_Cancel_PV_1, B_S1_R_Cancel_PV_1")

    def test_add_collaboration_role_opens_dialog_for_partner(self):
        """Test: add_collaboration_role_opens_dialog_for_partner"""
        self.select(self.action('B_S1_R_Place_PV_1'))
        dialog = self.tab.add_collaboration_role()
        self.assertIsInstance(dialog, CollaborationRoleDialog)
        self.assertIs(dialog.party_info, self.master.root.findall(cpa('PartyInfo'))[1])
        dialog.variables['service_name'].set('Delivery')
        role, _ = dialog.submit()
        self.assertEqual(self.tree().selection(), (self.tab.find_item(role),))

    @patch('application.app_classes.collaboration_role.messagebox')
    def test_delete_action_with_mirror(self, mock_messagebox):
        """Test: Yes deletes the action and its mirrored action"""
        mock_messagebox.askyesnocancel.return_value = True
        self.select(self.action('A_S1_S_Place_PV_1'))
        self.tab.delete_selected()
        self.assertIsNone(self.action('A_S1_S_Place_PV_1'))
        self.assertIsNone(self.action('B_S1_R_Place_PV_1'))
        self.assertEqual(len(self.rows()), 8)

    @patch('application.app_classes.collaboration_role.messagebox')
    def test_delete_action_only(self, mock_messagebox):
        """Test: No deletes only the selected action, Cancel deletes nothing"""
        mock_messagebox.askyesnocancel.return_value = None
        self.select(self.action('A_S1_S_Place_PV_1'))
        self.tab.delete_selected()
        self.assertIsNotNone(self.action('A_S1_S_Place_PV_1'))
        mock_messagebox.askyesnocancel.return_value = False
        self.tab.delete_selected()
        self.assertIsNone(self.action('A_S1_S_Place_PV_1'))
        self.assertIsNotNone(self.action('B_S1_R_Place_PV_1'))

    @patch('application.app_classes.collaboration_role.messagebox')
    def test_delete_collaboration_role(self, mock_messagebox):
        """Test: a role can be deleted, the last role of a partner not"""
        mock_messagebox.askyesno.return_value = True
        self.select(self.role(0, 1))
        self.tab.delete_selected()
        self.assertEqual(len(self.master.root.findall(f"{cpa('PartyInfo')}/{cpa('CollaborationRole')}")), 3)
        self.select(self.role(0, 0))
        self.tab.delete_selected()
        mock_messagebox.showerror.assert_called_once()
        self.assertEqual(len(self.master.root.findall(f"{cpa('PartyInfo')}/{cpa('CollaborationRole')}")), 3)

    def test_shows_experimental_notice(self):
        """Test: the tab says that it is experimental"""
        self.assertTrue(self.tab.experimental_label.cget('text').startswith("Experimental:"))

    def test_load_without_cpa(self):
        """Test: load_without_cpa"""
        self.master.root = None
        self.tab.load()
        self.assertEqual(self.tree().get_children(), ())


if __name__ == '__main__':
    unittest.main()
