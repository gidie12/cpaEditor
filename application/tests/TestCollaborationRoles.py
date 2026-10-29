import os
import unittest

import xmlschema
from lxml import etree

from application.helper_classes.certificateExpiry import CPA_NAMESPACE
from application.helper_classes.collaborationRoles import (
    RECEIVE, SEND, DEFAULT_CHARACTERISTICS, add_action, add_collaboration_role, build_action,
    build_collaboration_role, cpa, delete_action, delete_collaboration_role, describe_action,
    describe_collaboration_role, find_action_by_binding_id, find_counterpart_collaboration_role, get_party_infos,
    insert_action, insert_indented, make_unique_id, make_valid_id, suggest_binding_id, suggest_channel_id,
    suggest_characteristics, update_action, validate_action)
from application.helper_classes.directories import TESTS_DIR
from application.helper_classes.validateSchema import DEFAULT_XSD

FULL_CPA_P = os.path.join(TESTS_DIR, 'resources', 'cpas', 'full_cpa_p.xml')

CHARACTERISTICS = ('tns:isAuthenticated="persistent" tns:isAuthorizationRequired="true" '
                   'tns:isConfidential="transient" tns:timeToPerform="PT2H"')

CPA = f"""<tns:CollaborationProtocolAgreement xmlns:tns="{CPA_NAMESPACE}" xmlns:xlink="http://www.w3.org/1999/xlink">
    <tns:PartyInfo tns:partyName="A" tns:defaultMshChannelId="A_default">
        <tns:PartyId>A_ID</tns:PartyId>
        <tns:PartyRef xlink:href="https://a.example.com"/>
        <tns:CollaborationRole>
            <tns:ProcessSpecification tns:name="Process" tns:version="1.0" xlink:href="spec"/>
            <tns:Role tns:name="Sender" xlink:href="spec"/>
            <tns:ServiceBinding>
                <tns:Service tns:type="urn:test">Order</tns:Service>
                <tns:CanSend>
                    <tns:ThisPartyActionBinding tns:id="A_S1_S_Place_PV_1" tns:action="Place" tns:packageId="pkg">
                        <tns:BusinessTransactionCharacteristics {CHARACTERISTICS}/>
                        <tns:ChannelId>A_reliable</tns:ChannelId>
                    </tns:ThisPartyActionBinding>
                    <tns:OtherPartyActionBinding>
                        B_S1_R_Place_PV_1
                    </tns:OtherPartyActionBinding>
                </tns:CanSend>
                <tns:CanReceive>
                    <tns:ThisPartyActionBinding tns:id="A_S1_R_Confirm_PV_1" tns:action="Confirm" tns:packageId="pkg">
                        <tns:BusinessTransactionCharacteristics {CHARACTERISTICS}/>
                        <tns:ChannelId>A_default</tns:ChannelId>
                    </tns:ThisPartyActionBinding>
                    <tns:OtherPartyActionBinding>B_S1_S_Confirm_PV_1</tns:OtherPartyActionBinding>
                </tns:CanReceive>
            </tns:ServiceBinding>
        </tns:CollaborationRole>
        <tns:CollaborationRole>
            <tns:ProcessSpecification tns:name="Process" tns:version="1.0" xlink:href="spec"/>
            <tns:Role tns:name="Sender" xlink:href="spec"/>
            <tns:ServiceBinding>
                <tns:Service tns:type="urn:test">Invoice</tns:Service>
            </tns:ServiceBinding>
        </tns:CollaborationRole>
        <tns:DeliveryChannel tns:channelId="A_default"/>
        <tns:DeliveryChannel tns:channelId="A_reliable"/>
    </tns:PartyInfo>
    <tns:PartyInfo tns:partyName="B">
        <tns:PartyId>B_ID</tns:PartyId>
        <tns:CollaborationRole>
            <tns:ProcessSpecification tns:name="Process" tns:version="1.0" xlink:href="spec"/>
            <tns:Role tns:name="Receiver" xlink:href="spec"/>
            <tns:ServiceBinding>
                <tns:Service tns:type="urn:test">OrderService</tns:Service>
                <tns:CanSend>
                    <tns:ThisPartyActionBinding tns:id="B_S1_S_Confirm_PV_1" tns:action="Confirm" tns:packageId="pkg">
                        <tns:BusinessTransactionCharacteristics/>
                        <tns:ChannelId>B_default</tns:ChannelId>
                    </tns:ThisPartyActionBinding>
                    <tns:OtherPartyActionBinding>A_S1_R_Confirm_PV_1</tns:OtherPartyActionBinding>
                </tns:CanSend>
                <tns:CanReceive>
                    <tns:ThisPartyActionBinding tns:id="B_S1_R_Place_PV_1" tns:action="Place" tns:packageId="pkg">
                        <tns:BusinessTransactionCharacteristics/>
                        <tns:ChannelId>B_default</tns:ChannelId>
                    </tns:ThisPartyActionBinding>
                    <tns:OtherPartyActionBinding>A_S1_S_Place_PV_1</tns:OtherPartyActionBinding>
                </tns:CanReceive>
            </tns:ServiceBinding>
        </tns:CollaborationRole>
        <tns:CollaborationRole>
            <tns:ProcessSpecification tns:name="Process" tns:version="1.0" xlink:href="spec"/>
            <tns:Role tns:name="Receiver" xlink:href="spec"/>
            <tns:ServiceBinding>
                <tns:Service tns:type="urn:test">Invoice</tns:Service>
            </tns:ServiceBinding>
        </tns:CollaborationRole>
        <tns:DeliveryChannel tns:channelId="B_default"/>
    </tns:PartyInfo>
    <tns:Packaging tns:id="pkg"/>
    <tns:Packaging tns:id="other_pkg"/>
</tns:CollaborationProtocolAgreement>"""


class CollaborationRolesTestCase(unittest.TestCase):

    def setUp(self):
        self.root = etree.fromstring(CPA)
        self.party_a, self.party_b = get_party_infos(self.root)
        self.order_a, self.invoice_a = self.party_a.findall(cpa('CollaborationRole'))
        self.order_b, self.invoice_b = self.party_b.findall(cpa('CollaborationRole'))

    def action(self, binding_id):
        return find_action_by_binding_id(self.root, binding_id)

    def binding_ids(self, collaboration_role):
        return [binding.get(cpa('id')) for binding in collaboration_role.iter(cpa('ThisPartyActionBinding'))]


class TestDescribe(CollaborationRolesTestCase):

    def test_describe_collaboration_role(self):
        """Test: describe_collaboration_role"""
        self.assertEqual(describe_collaboration_role(self.order_a), {
            'process_name': 'Process', 'process_version': '1.0', 'role_name': 'Sender',
            'service_name': 'Order', 'service_type': 'urn:test'})

    def test_describe_action(self):
        """Test: describe_action strips the OtherPartyActionBinding and drops namespaces"""
        action = describe_action(self.action('A_S1_S_Place_PV_1'))
        self.assertEqual(action['direction'], SEND)
        self.assertEqual(action['action'], 'Place')
        self.assertEqual(action['package_id'], 'pkg')
        self.assertEqual(action['channel_ids'], ['A_reliable'])
        self.assertEqual(action['other_party_binding'], 'B_S1_R_Place_PV_1')
        self.assertEqual(action['characteristics'], {'isAuthenticated': 'persistent',
                                                     'isAuthorizationRequired': 'true',
                                                     'isConfidential': 'transient', 'timeToPerform': 'PT2H'})

    def test_find_action_by_binding_id(self):
        """Test: find_action_by_binding_id"""
        self.assertEqual(self.action('B_S1_S_Confirm_PV_1').tag, cpa(SEND))
        self.assertIsNone(find_action_by_binding_id(self.party_a, 'B_S1_S_Confirm_PV_1'))
        self.assertIsNone(find_action_by_binding_id(self.root, ''))


class TestValidate(CollaborationRolesTestCase):

    def test_validate_action_without_problems(self):
        """Test: validate_action_without_problems"""
        self.assertEqual(validate_action(self.root, self.action('A_S1_S_Place_PV_1')), [])
        self.assertEqual(validate_action(self.root, self.action('B_S1_R_Place_PV_1')), [])

    def test_validate_action_unknown_channel_and_package(self):
        """Test: validate_action_unknown_channel_and_package"""
        action = self.action('A_S1_S_Place_PV_1')
        binding = action.find(cpa('ThisPartyActionBinding'))
        binding.find(cpa('ChannelId')).text = 'B_default'
        binding.set(cpa('packageId'), 'missing')
        self.assertEqual(validate_action(self.root, action), [
            "ChannelId 'B_default' is not a DeliveryChannel of this partner",
            "packageId 'missing' is not a Packaging of the CPA"])

    def test_validate_action_missing_other_party_binding(self):
        """Test: validate_action_missing_other_party_binding"""
        action = self.action('A_S1_S_Place_PV_1')
        action.remove(action.find(cpa('OtherPartyActionBinding')))
        self.assertEqual(validate_action(self.root, action), ["No OtherPartyActionBinding"])

    def test_validate_action_other_party_binding_not_found(self):
        """Test: an OtherPartyActionBinding of the own partner is not accepted"""
        action = self.action('A_S1_S_Place_PV_1')
        action.find(cpa('OtherPartyActionBinding')).text = 'A_S1_R_Confirm_PV_1'
        self.assertEqual(validate_action(self.root, action),
                         ["OtherPartyActionBinding 'A_S1_R_Confirm_PV_1' is not an action of the other partner"])

    def test_validate_action_other_party_binding_same_direction(self):
        """Test: validate_action_other_party_binding_same_direction"""
        action = self.action('A_S1_S_Place_PV_1')
        action.find(cpa('OtherPartyActionBinding')).text = 'B_S1_S_Confirm_PV_1'
        self.assertEqual(validate_action(self.root, action), [
            "OtherPartyActionBinding 'B_S1_S_Confirm_PV_1' is also a Send action",
            "OtherPartyActionBinding 'B_S1_S_Confirm_PV_1' does not refer back to 'A_S1_S_Place_PV_1'"])

    def test_validate_action_other_party_binding_does_not_refer_back(self):
        """Test: validate_action_other_party_binding_does_not_refer_back"""
        self.action('B_S1_R_Place_PV_1').find(cpa('OtherPartyActionBinding')).text = 'something_else'
        self.assertEqual(validate_action(self.root, self.action('A_S1_S_Place_PV_1')),
                         ["OtherPartyActionBinding 'B_S1_R_Place_PV_1' does not refer back to 'A_S1_S_Place_PV_1'"])


class TestCounterpart(CollaborationRolesTestCase):

    def test_find_counterpart_collaboration_role_by_links(self):
        """Test: the links are followed, although the service names differ"""
        self.assertIs(find_counterpart_collaboration_role(self.root, self.order_a), self.order_b)
        self.assertIs(find_counterpart_collaboration_role(self.root, self.order_b), self.order_a)

    def test_find_counterpart_collaboration_role_by_service(self):
        """Test: without actions the service name and type are compared"""
        self.assertIs(find_counterpart_collaboration_role(self.root, self.invoice_a), self.invoice_b)

    def test_find_counterpart_collaboration_role_not_found(self):
        """Test: find_counterpart_collaboration_role_not_found"""
        self.invoice_b.find(f"{cpa('ServiceBinding')}/{cpa('Service')}").text = 'Other'
        self.assertIsNone(find_counterpart_collaboration_role(self.root, self.invoice_a))


class TestSuggestions(CollaborationRolesTestCase):

    def test_make_valid_id(self):
        """Test: make_valid_id"""
        self.assertEqual(make_valid_id('Order Service/v1'), 'Order_Service_v1')
        self.assertEqual(make_valid_id('1abc'), '_1abc')
        self.assertEqual(make_valid_id(''), '_')
        self.assertEqual(make_valid_id('A_b-1.0'), 'A_b-1.0')

    def test_make_unique_id(self):
        """Test: make_unique_id"""
        self.assertEqual(make_unique_id(self.root, 'new'), 'new')
        self.assertEqual(make_unique_id(self.root, 'pkg'), 'pkg_2')
        self.assertEqual(make_unique_id(self.root, 'new', reserved=['new', 'new_2']), 'new_3')

    def test_suggest_binding_id_follows_example(self):
        """Test: the id of an action with the same direction is the example"""
        self.assertEqual(suggest_binding_id(self.root, self.order_a, SEND, 'Cancel'), 'A_S1_S_Cancel_PV_1')
        self.assertEqual(suggest_binding_id(self.root, self.order_b, RECEIVE, 'Cancel'), 'B_S1_R_Cancel_PV_1')

    def test_suggest_binding_id_without_example(self):
        """Test: suggest_binding_id_without_example"""
        self.assertEqual(suggest_binding_id(self.root, self.invoice_a, SEND, 'Send Invoice'),
                         'A_Invoice_S_Send_Invoice')

    def test_suggest_binding_id_is_unique(self):
        """Test: suggest_binding_id_is_unique"""
        self.assertEqual(suggest_binding_id(self.root, self.order_a, SEND, 'Place'), 'A_S1_S_Place_PV_1_2')
        self.assertEqual(suggest_binding_id(self.root, self.order_a, SEND, 'Cancel', reserved=['A_S1_S_Cancel_PV_1']),
                         'A_S1_S_Cancel_PV_1_2')

    def test_suggest_channel_id(self):
        """Test: suggest_channel_id"""
        self.assertEqual(suggest_channel_id(self.order_a, SEND), 'A_reliable')
        self.assertEqual(suggest_channel_id(self.invoice_a, SEND), 'A_default')
        self.assertEqual(suggest_channel_id(self.invoice_b, SEND), 'B_default')

    def test_suggest_characteristics(self):
        """Test: suggest_characteristics"""
        self.assertEqual(suggest_characteristics(self.invoice_a), DEFAULT_CHARACTERISTICS)
        characteristics = suggest_characteristics(self.order_a)
        self.assertEqual(characteristics['isAuthenticated'], 'persistent')
        self.assertEqual(characteristics['timeToPerform'], 'PT2H')
        self.assertEqual(characteristics['isTamperProof'], 'transient')


class TestBuildAndInsert(CollaborationRolesTestCase):

    def test_build_action(self):
        """Test: build_action"""
        element = build_action(RECEIVE, 'id1', 'Do', 'pkg', 'A_default',
                               {'isAuthenticated': 'none', 'timeToPerform': ''}, 'other1')
        action = describe_action(element)
        self.assertEqual((action['direction'], action['id'], action['action'], action['package_id']),
                         (RECEIVE, 'id1', 'Do', 'pkg'))
        self.assertEqual(action['channel_ids'], ['A_default'])
        self.assertEqual(action['other_party_binding'], 'other1')
        self.assertEqual(action['characteristics'], {'isAuthenticated': 'none'})
        self.assertIsNone(build_action(SEND, 'id2', 'Do', 'pkg', 'c', {}).find(cpa('OtherPartyActionBinding')))

    def test_insert_action_keeps_schema_order(self):
        """Test: a CanSend goes after the last CanSend, a CanReceive at the end"""
        insert_action(self.order_a, build_action(SEND, 'new_send', 'X', 'pkg', 'A_default', {}))
        insert_action(self.order_a, build_action(RECEIVE, 'new_receive', 'Y', 'pkg', 'A_default', {}))
        service_binding = self.order_a.find(cpa('ServiceBinding'))
        self.assertEqual([etree.QName(child).localname for child in service_binding],
                         ['Service', SEND, SEND, RECEIVE, RECEIVE])
        self.assertEqual(self.binding_ids(self.order_a),
                         ['A_S1_S_Place_PV_1', 'new_send', 'A_S1_R_Confirm_PV_1', 'new_receive'])

    def test_insert_indented_matches_siblings(self):
        """Test: the new element is indented like the existing ones"""
        insert_action(self.invoice_a, build_action(SEND, 'new_send', 'X', 'pkg', 'A_default', {}))
        text = etree.tostring(self.invoice_a, encoding='unicode')
        self.assertIn('\n                <tns:Service', text)
        self.assertIn('\n                <tns:CanSend>\n                    <tns:ThisPartyActionBinding', text)
        self.assertIn('\n                        <tns:ChannelId>A_default</tns:ChannelId>', text)
        self.assertIn('</tns:CanSend>\n            </tns:ServiceBinding>', text)

    def test_insert_indented_without_whitespace(self):
        """Test: a parent without indentation gets the element without extra whitespace"""
        parent = etree.fromstring('<a><b/></a>')
        insert_indented(parent, 1, etree.fromstring('<c><d/></c>'))
        self.assertEqual(etree.tostring(parent, encoding='unicode'), '<a><b/><c><d/></c></a>')


class TestAddAction(CollaborationRolesTestCase):

    def test_add_action_without_mirror(self):
        """Test: add_action_without_mirror"""
        element, mirror_element = add_action(self.root, self.order_a, SEND, ' Cancel ', 'pkg', 'A_reliable',
                                             DEFAULT_CHARACTERISTICS, 'A_cancel')
        self.assertIsNone(mirror_element)
        action = describe_action(element)
        self.assertEqual((action['action'], action['other_party_binding']), ('Cancel', ''))
        self.assertIn('A_cancel', self.binding_ids(self.order_a))
        self.assertEqual(self.binding_ids(self.party_b), self.binding_ids(etree.fromstring(CPA)[1]))

    def test_add_action_with_mirror(self):
        """Test: the mirrored action has the opposite direction and both refer to each other"""
        element, mirror_element = add_action(
            self.root, self.order_a, SEND, 'Cancel', 'other_pkg', 'A_reliable', {'isAuthenticated': 'none'},
            'A_cancel', mirror={'collaboration_role': self.order_b, 'binding_id': 'B_cancel',
                                'channel_id': 'B_default'})
        mirror = describe_action(mirror_element)
        self.assertEqual(mirror['direction'], RECEIVE)
        self.assertEqual((mirror['id'], mirror['action'], mirror['package_id']), ('B_cancel', 'Cancel', 'other_pkg'))
        self.assertEqual(mirror['channel_ids'], ['B_default'])
        self.assertEqual(mirror['characteristics'], {'isAuthenticated': 'none'})
        self.assertEqual(mirror['other_party_binding'], 'A_cancel')
        self.assertEqual(describe_action(element)['other_party_binding'], 'B_cancel')
        self.assertEqual(validate_action(self.root, element), [])
        self.assertEqual(validate_action(self.root, mirror_element), [])

    def test_add_action_rejects_used_id(self):
        """Test: add_action_rejects_used_id"""
        with self.assertRaisesRegex(ValueError, "already used"):
            add_action(self.root, self.order_a, SEND, 'X', 'pkg', 'A_default', {}, 'A_S1_R_Confirm_PV_1')
        with self.assertRaisesRegex(ValueError, "already used"):
            add_action(self.root, self.order_a, SEND, 'X', 'pkg', 'A_default', {}, 'new',
                       mirror={'collaboration_role': self.order_b, 'binding_id': 'pkg', 'channel_id': 'B_default'})
        with self.assertRaisesRegex(ValueError, "different ids"):
            add_action(self.root, self.order_a, SEND, 'X', 'pkg', 'A_default', {}, 'new',
                       mirror={'collaboration_role': self.order_b, 'binding_id': 'new', 'channel_id': 'B_default'})
        with self.assertRaisesRegex(ValueError, "not a valid id"):
            add_action(self.root, self.order_a, SEND, 'X', 'pkg', 'A_default', {}, 'with space')
        self.assertEqual(len(self.order_a.find(cpa('ServiceBinding'))), 3)

    def test_add_action_rejects_missing_values(self):
        """Test: add_action_rejects_missing_values"""
        for action_name, package_id, channel_id, message in (('', 'pkg', 'c', "Action"), ('X', '', 'c', "Package"),
                                                             ('X', 'pkg', '', "Channel")):
            with self.assertRaisesRegex(ValueError, message):
                add_action(self.root, self.order_a, SEND, action_name, package_id, channel_id, {}, 'new')
        with self.assertRaisesRegex(ValueError, "mirrored"):
            add_action(self.root, self.order_a, SEND, 'X', 'pkg', 'A_default', {}, 'new',
                       mirror={'collaboration_role': self.order_b, 'binding_id': 'new_b', 'channel_id': ''})

    def test_add_action_result_is_valid_against_schema(self):
        """Test: a CPA that is valid stays valid after adding a mirrored action in both directions"""
        root = etree.parse(FULL_CPA_P).getroot()
        party_a, party_b = get_party_infos(root)
        role_a = party_a.find(cpa('CollaborationRole'))
        role_b = find_counterpart_collaboration_role(root, role_a)
        package_id = root.find(cpa('Packaging')).get(cpa('id'))
        for direction in (SEND, RECEIVE):
            binding_id = suggest_binding_id(root, role_a, direction, f'New{direction}')
            mirror_id = suggest_binding_id(root, role_b, RECEIVE if direction == SEND else SEND, f'New{direction}',
                                           reserved=[binding_id])
            add_action(root, role_a, direction, f'New{direction}', package_id,
                       suggest_channel_id(role_a, direction), suggest_characteristics(role_a), binding_id,
                       mirror={'collaboration_role': role_b, 'binding_id': mirror_id,
                               'channel_id': suggest_channel_id(role_b, RECEIVE if direction == SEND else SEND)})
        add_collaboration_role(party_a, 'P', '1', 'href', '', 'R', 'href', 'S', '')
        errors = list(xmlschema.XMLSchema(DEFAULT_XSD).iter_errors(root))
        self.assertEqual([str(error.reason) for error in errors], [])


class TestUpdateAndDelete(CollaborationRolesTestCase):

    def test_update_action(self):
        """Test: update_action"""
        element = self.action('A_S1_S_Place_PV_1')
        update_action(element, 'PlaceNew', 'other_pkg', 'A_default',
                      {'isAuthenticated': 'none', 'timeToPerform': '', 'isTamperProof': 'persistent'})
        action = describe_action(element)
        self.assertEqual((action['id'], action['action'], action['package_id']),
                         ('A_S1_S_Place_PV_1', 'PlaceNew', 'other_pkg'))
        self.assertEqual(action['channel_ids'], ['A_default'])
        self.assertEqual(action['characteristics'], {'isAuthenticated': 'none', 'isAuthorizationRequired': 'true',
                                                     'isConfidential': 'transient', 'isTamperProof': 'persistent'})

    def test_update_action_rejects_empty_action(self):
        """Test: update_action_rejects_empty_action"""
        with self.assertRaisesRegex(ValueError, "Action"):
            update_action(self.action('A_S1_S_Place_PV_1'), ' ', 'pkg', 'A_default', {})
        self.assertEqual(describe_action(self.action('A_S1_S_Place_PV_1'))['action'], 'Place')

    def test_delete_action_only(self):
        """Test: delete_action_only"""
        self.assertEqual(delete_action(self.root, self.action('A_S1_S_Place_PV_1')), ['A_S1_S_Place_PV_1'])
        self.assertEqual(self.binding_ids(self.order_a), ['A_S1_R_Confirm_PV_1'])
        self.assertIsNotNone(self.action('B_S1_R_Place_PV_1'))

    def test_delete_action_with_counterpart(self):
        """Test: the action in the OtherPartyActionBinding is deleted as well and the whitespace stays tidy"""
        self.assertEqual(delete_action(self.root, self.action('A_S1_R_Confirm_PV_1'), delete_counterpart=True),
                         ['A_S1_R_Confirm_PV_1', 'B_S1_S_Confirm_PV_1'])
        self.assertEqual(self.binding_ids(self.order_a), ['A_S1_S_Place_PV_1'])
        self.assertEqual(self.binding_ids(self.order_b), ['B_S1_R_Place_PV_1'])
        self.assertIn('</tns:CanSend>\n            </tns:ServiceBinding>',
                      etree.tostring(self.order_a, encoding='unicode'))


class TestCollaborationRoleElements(CollaborationRolesTestCase):

    def test_build_collaboration_role(self):
        """Test: build_collaboration_role"""
        role = build_collaboration_role('P', '2.0', 'phref', 'urn:uuid', 'R', 'rhref', 'S', 'urn:type')
        self.assertEqual(describe_collaboration_role(role), {
            'process_name': 'P', 'process_version': '2.0', 'role_name': 'R', 'service_name': 'S',
            'service_type': 'urn:type'})
        process = role.find(cpa('ProcessSpecification'))
        self.assertEqual(process.get(cpa('uuid')), 'urn:uuid')
        self.assertEqual(process.get('{http://www.w3.org/1999/xlink}href'), 'phref')
        self.assertIsNone(build_collaboration_role('P', '1', '', '', 'R', '', 'S', '').find(
            f"{cpa('ServiceBinding')}/{cpa('Service')}").get(cpa('type')))

    def test_add_collaboration_role(self):
        """Test: the new CollaborationRole comes after the last one, before the DeliveryChannels"""
        role = add_collaboration_role(self.party_a, ' P ', '1', '', '', 'R', '', 'New', '')
        self.assertEqual([etree.QName(child).localname for child in self.party_a],
                         ['PartyId', 'PartyRef', 'CollaborationRole', 'CollaborationRole', 'CollaborationRole',
                          'DeliveryChannel', 'DeliveryChannel'])
        self.assertIs(self.party_a[4], role)
        self.assertEqual(describe_collaboration_role(role)['process_name'], 'P')

    def test_add_collaboration_role_rejects_missing_values(self):
        """Test: add_collaboration_role_rejects_missing_values"""
        with self.assertRaisesRegex(ValueError, "Service name"):
            add_collaboration_role(self.party_a, 'P', '1', '', '', 'R', '', ' ', '')
        self.assertEqual(len(self.party_a.findall(cpa('CollaborationRole'))), 2)

    def test_delete_collaboration_role(self):
        """Test: delete_collaboration_role"""
        delete_collaboration_role(self.invoice_a)
        self.assertEqual(self.party_a.findall(cpa('CollaborationRole')), [self.order_a])

    def test_delete_last_collaboration_role_is_refused(self):
        """Test: delete_last_collaboration_role_is_refused"""
        delete_collaboration_role(self.invoice_a)
        with self.assertRaisesRegex(ValueError, "at least one"):
            delete_collaboration_role(self.order_a)
        self.assertEqual(self.party_a.findall(cpa('CollaborationRole')), [self.order_a])


if __name__ == '__main__':
    unittest.main()
