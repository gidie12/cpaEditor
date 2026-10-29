import re
import unittest
from unittest.mock import Mock
import tkinter as tk

from lxml import etree

from application.app_classes.service_action_dialog import CollaborationRoleDialog, ServiceActionDialog
from application.helper_classes.collaborationRoles import (
    RECEIVE, SEND, cpa, describe_action, describe_collaboration_role, find_action_by_binding_id, get_party_infos,
    validate_action)
from application.tests.TestCollaborationRoles import CPA


class DialogTestCase(unittest.TestCase):

    def setUp(self):
        self.master = tk.Tk()
        self.addCleanup(self.master.destroy)
        self.root = etree.fromstring(CPA)
        self.party_a, self.party_b = get_party_infos(self.root)
        self.order_a, self.invoice_a = self.party_a.findall(cpa('CollaborationRole'))
        self.order_b, self.invoice_b = self.party_b.findall(cpa('CollaborationRole'))


class TestServiceActionDialog(DialogTestCase):

    def test_dialog_prefills_suggestions(self):
        """Test: the dialog proposes package, channel, ids and the counterpart service"""
        dialog = ServiceActionDialog(self.master, self.root, self.order_a)
        dialog.action_name.set('Cancel')
        self.assertEqual(dialog.package_id.get(), 'pkg')
        self.assertEqual(dialog.channel_id.get(), 'A_reliable')
        self.assertEqual(dialog.binding_id.get(), 'A_S1_S_Cancel_PV_1')
        self.assertTrue(dialog.mirror_enabled.get())
        self.assertTrue(dialog.mirror_role.get().startswith('OrderService'))
        self.assertEqual(dialog.mirror_binding_id.get(), 'B_S1_R_Cancel_PV_1')
        self.assertEqual(dialog.mirror_channel_id.get(), 'B_default')
        self.assertEqual(dialog.characteristics.get()['isAuthenticated'], 'persistent')

        dialog.direction.set(RECEIVE)
        self.assertEqual(dialog.binding_id.get(), 'A_S1_R_Cancel_PV_1')
        self.assertEqual(dialog.channel_id.get(), 'A_default')
        self.assertEqual(dialog.mirror_binding_id.get(), 'B_S1_S_Cancel_PV_1')

    def test_binding_id_follows_action_until_edited(self):
        """Test: a typed id is kept when the action changes"""
        dialog = ServiceActionDialog(self.master, self.root, self.order_a)
        dialog.action_name.set('One')
        self.assertEqual(dialog.binding_id.get(), 'A_S1_S_One_PV_1')
        dialog.binding_id.set('my_own_id')
        dialog.action_name.set('Two')
        self.assertEqual(dialog.binding_id.get(), 'my_own_id')
        self.assertEqual(dialog.mirror_binding_id.get(), 'B_S1_R_Two_PV_1')

    def test_submit_adds_action_and_mirror(self):
        """Test: submit_adds_action_and_mirror"""
        on_done = Mock()
        dialog = ServiceActionDialog(self.master, self.root, self.order_a, on_done=on_done)
        dialog.action_name.set('Cancel')
        dialog.characteristics.variables['timeToPerform'].set('PT5M')
        element, mirror_element = dialog.submit()

        on_done.assert_called_once_with((element, mirror_element))
        self.assertFalse(dialog.winfo_exists())
        action, mirror = describe_action(element), describe_action(mirror_element)
        self.assertEqual((action['direction'], action['id'], action['other_party_binding']),
                         (SEND, 'A_S1_S_Cancel_PV_1', 'B_S1_R_Cancel_PV_1'))
        self.assertEqual((mirror['direction'], mirror['id'], mirror['other_party_binding']),
                         (RECEIVE, 'B_S1_R_Cancel_PV_1', 'A_S1_S_Cancel_PV_1'))
        self.assertEqual(action['characteristics']['timeToPerform'], 'PT5M')
        self.assertIs(mirror_element.getparent().getparent(), self.order_b)
        self.assertEqual(validate_action(self.root, element), [])

    def test_submit_without_mirror(self):
        """Test: submit_without_mirror"""
        dialog = ServiceActionDialog(self.master, self.root, self.order_a)
        dialog.action_name.set('Cancel')
        dialog.mirror_enabled.set(False)
        self.assertEqual(str(dialog.mirror_binding_id_field.cget('state')), 'disabled')
        element, mirror_element = dialog.submit()
        self.assertIsNone(mirror_element)
        self.assertIsNone(find_action_by_binding_id(self.root, 'B_S1_R_Cancel_PV_1'))
        self.assertEqual(describe_action(element)['other_party_binding'], '')

    def test_submit_shows_error(self):
        """Test: on an error the dialog stays open, shows the message and changes nothing"""
        before = etree.tostring(self.root)
        dialog = ServiceActionDialog(self.master, self.root, self.order_a)
        self.assertIsNone(dialog.submit())
        self.assertEqual(dialog.error_text.get(), "Action is required")
        dialog.action_name.set('Cancel')
        dialog.binding_id.set('A_S1_S_Place_PV_1')
        self.assertIsNone(dialog.submit())
        self.assertIn("already used", dialog.error_text.get())
        self.assertTrue(dialog.winfo_exists())
        self.assertEqual(etree.tostring(self.root), before)

    def test_mirror_disabled_without_other_role(self):
        """Test: without CollaborationRoles at the other partner no mirrored action is offered"""
        for role in (self.order_b, self.invoice_b):
            self.party_b.remove(role)
        dialog = ServiceActionDialog(self.master, self.root, self.order_a)
        self.assertFalse(dialog.mirror_enabled.get())
        self.assertEqual(str(dialog.mirror_checkbutton.cget('state')), 'disabled')
        dialog.action_name.set('Cancel')
        self.assertIsNone(dialog.submit()[1])


class TestCollaborationRoleDialog(DialogTestCase):

    def test_role_dialog_prefills_from_existing_role(self):
        """Test: role_dialog_prefills_from_existing_role"""
        dialog = CollaborationRoleDialog(self.master, self.root, self.party_a)
        values = {name: variable.get() for name, variable in dialog.variables.items()}
        self.assertEqual(values, {'process_name': 'Process', 'process_version': '1.0', 'process_href': 'spec',
                                  'process_uuid': '', 'role_name': 'Sender', 'role_href': 'spec',
                                  'service_name': '', 'service_type': 'urn:test'})
        self.assertEqual(dialog.mirror_role_name.get(), 'Receiver')
        self.assertTrue(dialog.mirror_enabled.get())

    def test_role_dialog_adds_role_at_both_partners(self):
        """Test: role_dialog_adds_role_at_both_partners"""
        on_done = Mock()
        dialog = CollaborationRoleDialog(self.master, self.root, self.party_a, on_done=on_done)
        dialog.variables['service_name'].set('Delivery')
        role, mirror_role = dialog.submit()
        on_done.assert_called_once_with((role, mirror_role))
        self.assertEqual(describe_collaboration_role(role)['role_name'], 'Sender')
        self.assertEqual(describe_collaboration_role(mirror_role)['role_name'], 'Receiver')
        self.assertEqual(describe_collaboration_role(mirror_role)['service_name'], 'Delivery')
        self.assertIs(role.getparent(), self.party_a)
        self.assertIs(mirror_role.getparent(), self.party_b)

    def test_role_dialog_shows_error(self):
        """Test: role_dialog_shows_error"""
        dialog = CollaborationRoleDialog(self.master, self.root, self.party_a)
        self.assertIsNone(dialog.submit())
        self.assertEqual(dialog.error_text.get(), "Service name is required")
        dialog.variables['service_name'].set('Delivery')
        dialog.mirror_role_name.set('')
        self.assertIsNone(dialog.submit())
        self.assertEqual(dialog.error_text.get(), "Role name of the other partner is required")
        self.assertEqual(len(self.party_a.findall(cpa('CollaborationRole'))), 2)
        self.assertEqual(len(self.party_b.findall(cpa('CollaborationRole'))), 2)

    def test_role_dialog_generate_uuid(self):
        """Test: role_dialog_generate_uuid"""
        dialog = CollaborationRoleDialog(self.master, self.root, self.party_a)
        dialog.generate_uuid()
        self.assertRegex(dialog.variables['process_uuid'].get(), re.compile(r'^urn:uuid:[0-9a-f-]{36}$'))


if __name__ == '__main__':
    unittest.main()
