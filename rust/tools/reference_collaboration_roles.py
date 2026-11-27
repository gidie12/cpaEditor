"""Writes what the Python CPA Editor saves after editing CollaborationRoles and their actions, so the Rust tests
(cpa-core/tests/collaboration_roles.rs) can replay the same operations and compare byte for byte.

Run from the rust directory with a Python that has the packages of requirements.txt:
    PYTHONPATH=.. python tools/reference_collaboration_roles.py

Every case loads a test CPA with the same safe parser as tools/make_reference.py, applies operations with the real
functions of application/helper_classes/collaborationRoles.py and writes the result to
cpa-core/tests/reference/collaboration_roles/<case>.xml. Keep the cases in step with the Rust test.
"""
import pathlib

from lxml import etree

from application.helper_classes.collaborationRoles import (
    RECEIVE, SEND, add_action, add_collaboration_role, cpa, delete_action, delete_collaboration_role,
    find_action_by_binding_id, find_counterpart_collaboration_role, get_party_infos, remove_element,
    suggest_binding_id, suggest_channel_id, suggest_characteristics, update_action)

CPAS = pathlib.Path('../application/tests/resources/cpas')
OUT = pathlib.Path('cpa-core/tests/reference/collaboration_roles')
PARSER = etree.XMLParser(resolve_entities=False, no_network=True, load_dtd=False)


def roles(root, party_index):
    return get_party_infos(root)[party_index].findall(cpa('CollaborationRole'))


def add_suggested(root, role, direction, action_name, mirror_role=None):
    """Adds an action with the suggested id, channel and characteristics, like the GUI does."""
    binding_id = suggest_binding_id(root, role, direction, action_name)
    mirror = None
    if mirror_role is not None:
        opposite = RECEIVE if direction == SEND else SEND
        mirror = {'collaboration_role': mirror_role,
                  'binding_id': suggest_binding_id(root, mirror_role, opposite, action_name, reserved=[binding_id]),
                  'channel_id': suggest_channel_id(mirror_role, opposite)}
    package_id = root.find(cpa('Packaging')).get(cpa('id'))
    return add_action(root, role, direction, action_name, package_id, suggest_channel_id(role, direction),
                      suggest_characteristics(role), binding_id, mirror=mirror)


def add_send_with_mirror(root):
    role = roles(root, 0)[0]
    add_suggested(root, role, SEND, 'NewAction', find_counterpart_collaboration_role(root, role))


def add_receive_with_mirror_tab_indent(root):
    role = roles(root, 0)[1]
    add_suggested(root, role, RECEIVE, 'NewAction', find_counterpart_collaboration_role(root, role))


def add_receive_after_existing(root):
    add_suggested(root, roles(root, 1)[0], RECEIVE, 'Extra Receive')


def add_receive_first_in_service_binding(root):
    party_info = get_party_infos(root)[0]
    role = add_collaboration_role(party_info, 'Process', '1.0', 'http://example.com/spec', '', 'Role',
                                  'http://example.com/spec', 'Empty', '')
    add_action(root, role, RECEIVE, 'First', 'defaultPackage_Profile', suggest_channel_id(role, RECEIVE),
               {'isAuthenticated': 'none', 'timeToPerform': 'PT5M'}, 'first_receive')


def add_send_after_comment(root):
    role = roles(root, 0)[0]
    service_binding = role.find(cpa('ServiceBinding'))
    service_binding.insert(1, etree.Comment(' note '))
    add_suggested(root, role, SEND, 'AfterComment')


def update_characteristics(root):
    update_action(find_action_by_binding_id(root, 'dummybroker-ontvangen'), ' Renamed ', 'defaultPackage_Profile',
                  'DC_T_Besteffort_S__R_PARTNERA',
                  {'isAuthenticated': 'persistent', 'timeToPerform': '', 'isTamperProof': ' none ',
                   'custom': 'x'})


def update_without_characteristics_and_channel(root):
    action = find_action_by_binding_id(root, 'dummybroker-verzonden')
    binding = action.find(cpa('ThisPartyActionBinding'))
    binding.remove(binding.find(cpa('BusinessTransactionCharacteristics')))
    binding.remove(binding.find(cpa('ChannelId')))
    update_action(action, 'Verzonden', 'defaultPackage_Profile', 'new_channel', {'isConfidential': 'none'})


def delete_action_only(root):
    delete_action(root, find_action_by_binding_id(root, 'BROKER_S1_BrokerX_Ketenpartner_S_ProcessAntwoord_PV_0.1'))


def delete_action_with_counterpart(root):
    delete_action(root, find_action_by_binding_id(root, 'BROKER_S1_Ketenpartner_BrokerX_R_ProcessVerzoek_PV_0.1'),
                  delete_counterpart=True)


def delete_last_and_only_child(root):
    role = roles(root, 0)[0]
    for binding_id in ('BROKER_S1_Ketenpartner_BrokerX_R_ProcessVerzoek_PV_0.1',
                       'BROKER_S1_BrokerX_Ketenpartner_S_ProcessAntwoord_PV_0.1'):
        delete_action(root, find_action_by_binding_id(root, binding_id))
    remove_element(role.find(cpa('ServiceBinding')).find(cpa('Service')))


def add_collaboration_role_with_uuid(root):
    add_collaboration_role(get_party_infos(root)[0], ' NewProcess ', '2.0', 'http://example.com/process',
                           'urn:uuid:1234', 'NewRole', 'http://example.com/role', 'NewService', 'urn:example:type')


def add_collaboration_role_mixed_indent(root):
    add_collaboration_role(get_party_infos(root)[0], 'P', '1', 'href', '', 'R', 'href', 'S', '')


def delete_collaboration_role_second(root):
    delete_collaboration_role(roles(root, 0)[1])


def delete_collaboration_role_first(root):
    delete_collaboration_role(roles(root, 1)[0])


CASES = [
    ('add_send_with_mirror', 'full_test_cpa.xml', add_send_with_mirror),
    ('add_receive_with_mirror_tab_indent', 'test_cpa.xml', add_receive_with_mirror_tab_indent),
    ('add_receive_after_existing', 'full_cpa_p.xml', add_receive_after_existing),
    ('add_receive_first_in_service_binding', 'full_cpa_p.xml', add_receive_first_in_service_binding),
    ('add_send_after_comment', 'full_test_cpa.xml', add_send_after_comment),
    ('update_characteristics', 'full_cpa_p.xml', update_characteristics),
    ('update_without_characteristics_and_channel', 'full_cpa_p.xml', update_without_characteristics_and_channel),
    ('delete_action_only', 'full_test_cpa.xml', delete_action_only),
    ('delete_action_with_counterpart', 'full_test_cpa.xml', delete_action_with_counterpart),
    ('delete_last_and_only_child', 'full_test_cpa.xml', delete_last_and_only_child),
    ('add_collaboration_role_with_uuid', 'full_cpa_p.xml', add_collaboration_role_with_uuid),
    ('add_collaboration_role_mixed_indent', 'test_cpa.xml', add_collaboration_role_mixed_indent),
    ('delete_collaboration_role_second', 'full_test_cpa.xml', delete_collaboration_role_second),
    ('delete_collaboration_role_first', 'full_test_cpa.xml', delete_collaboration_role_first),
]

if __name__ == '__main__':
    OUT.mkdir(parents=True, exist_ok=True)
    for name, source, operation in CASES:
        tree = etree.parse(str(CPAS / source), PARSER)
        operation(tree.getroot())
        (OUT / f'{name}.xml').write_bytes(etree.tostring(tree, encoding='UTF-8', xml_declaration=True))
        print(name)
