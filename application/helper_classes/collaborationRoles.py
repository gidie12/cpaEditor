import re

from lxml import etree as lxml_etree

from application.helper_classes.certificateExpiry import CPA_NAMESPACE

XLINK_NAMESPACE = 'http://www.w3.org/1999/xlink'

SEND = 'CanSend'
RECEIVE = 'CanReceive'
DIRECTION_LABELS = {SEND: 'Send', RECEIVE: 'Receive'}

PERSISTENCE_LEVELS = ('none', 'transient', 'persistent', 'transient-and-persistent')
BOOLEAN_VALUES = ('true', 'false')

# The attributes of BusinessTransactionCharacteristics with their allowed values (None: free text) and the
# default value of a new action. The order is the order in which the attributes are shown and written.
CHARACTERISTICS = (
    ('isAuthenticated', PERSISTENCE_LEVELS, 'transient'),
    ('isAuthorizationRequired', BOOLEAN_VALUES, 'true'),
    ('isIntelligibleCheckRequired', BOOLEAN_VALUES, 'false'),
    ('isNonRepudiationReceiptRequired', BOOLEAN_VALUES, 'false'),
    ('isNonRepudiationRequired', BOOLEAN_VALUES, 'false'),
    ('isConfidential', PERSISTENCE_LEVELS, 'transient'),
    ('isTamperProof', PERSISTENCE_LEVELS, 'transient'),
    ('timeToPerform', None, 'PT1H'),
)
DEFAULT_CHARACTERISTICS = {name: default for name, _, default in CHARACTERISTICS}


def cpa(name):
    """Returns the name with the namespace of the CPA, as lxml uses it for tags and attributes."""
    return '{' + CPA_NAMESPACE + '}' + name


def xlink(name):
    """Returns the name with the xlink namespace."""
    return '{' + XLINK_NAMESPACE + '}' + name


def text_of(element):
    """Returns the text of an element without surrounding whitespace, or an empty string."""
    if element is None or element.text is None:
        return ''
    return element.text.strip()


def get_party_infos(root):
    """Returns the PartyInfo elements of the CPA."""
    return root.findall(cpa('PartyInfo'))


def get_party_name(party_info):
    """Returns the partyName of a PartyInfo element."""
    return party_info.get(cpa('partyName'), '')


def get_other_party_info(root, party_info):
    """Returns the PartyInfo of the other partner, or None when the CPA has no other partner."""
    for other in get_party_infos(root):
        if other is not party_info:
            return other
    return None


def get_channel_ids(party_info):
    """Returns the channelIds of the DeliveryChannels of a partner, in the order of the CPA."""
    return [channel.get(cpa('channelId')) for channel in party_info.findall(cpa('DeliveryChannel'))
            if channel.get(cpa('channelId'))]


def get_package_ids(root):
    """Returns the ids of the Packaging elements of the CPA, in the order of the CPA."""
    return [package.get(cpa('id')) for package in root.findall(cpa('Packaging')) if package.get(cpa('id'))]


def get_used_ids(root):
    """Returns all values of id attributes in the CPA: a new id must not be one of them (type ID)."""
    return {element.get(cpa('id')) for element in root.iter()
            if isinstance(element.tag, str) and element.get(cpa('id'))}


def get_party_info_of(element):
    """Returns the PartyInfo element that contains the element, or None."""
    ancestor = element
    while ancestor is not None and ancestor.tag != cpa('PartyInfo'):
        ancestor = ancestor.getparent()
    return ancestor


def get_collaboration_role_of(element):
    """Returns the CollaborationRole element that contains the element, or None."""
    ancestor = element
    while ancestor is not None and ancestor.tag != cpa('CollaborationRole'):
        ancestor = ancestor.getparent()
    return ancestor


def describe_collaboration_role(collaboration_role):
    """
    Collects the values of a CollaborationRole that identify it.

    Args:
        collaboration_role (lxml.etree.Element): A CollaborationRole element.

    Returns:
        dict: process_name, process_version, role_name, service_name and service_type. A missing value is an
              empty string.

    Test Functions:
        - test_describe_collaboration_role
    """
    process = collaboration_role.find(cpa('ProcessSpecification'))
    role = collaboration_role.find(cpa('Role'))
    service = collaboration_role.find(f"{cpa('ServiceBinding')}/{cpa('Service')}")
    return {
        'process_name': process.get(cpa('name'), '') if process is not None else '',
        'process_version': process.get(cpa('version'), '') if process is not None else '',
        'role_name': role.get(cpa('name'), '') if role is not None else '',
        'service_name': text_of(service),
        'service_type': service.get(cpa('type'), '') if service is not None else '',
    }


def describe_action(action_element):
    """
    Collects the values of a CanSend or CanReceive element.

    Args:
        action_element (lxml.etree.Element): A CanSend or CanReceive element.

    Returns:
        dict: element, direction (SEND or RECEIVE), id, action, package_id, channel_ids (list),
              other_party_binding (empty string when missing) and characteristics (attribute name -> value of
              BusinessTransactionCharacteristics, without namespace).

    Test Functions:
        - test_describe_action
    """
    binding = action_element.find(cpa('ThisPartyActionBinding'))
    if binding is None:
        binding = lxml_etree.Element(cpa('ThisPartyActionBinding'))
    characteristics = binding.find(cpa('BusinessTransactionCharacteristics'))
    return {
        'element': action_element,
        'direction': lxml_etree.QName(action_element).localname,
        'id': binding.get(cpa('id'), ''),
        'action': binding.get(cpa('action'), ''),
        'package_id': binding.get(cpa('packageId'), ''),
        'channel_ids': [text_of(channel) for channel in binding.findall(cpa('ChannelId'))],
        'other_party_binding': text_of(action_element.find(cpa('OtherPartyActionBinding'))),
        'characteristics': {lxml_etree.QName(name).localname: value
                            for name, value in characteristics.attrib.items()} if characteristics is not None else {},
    }


def get_action_elements(parent):
    """Returns the CanSend and CanReceive children of a ServiceBinding (or of a CanSend/CanReceive)."""
    return [child for child in parent if child.tag in (cpa(SEND), cpa(RECEIVE))]


def get_service_binding(collaboration_role):
    """Returns the ServiceBinding of a CollaborationRole, or None."""
    return collaboration_role.find(cpa('ServiceBinding'))


def find_action_by_binding_id(container, binding_id):
    """
    Finds the CanSend or CanReceive whose ThisPartyActionBinding has the id.

    Args:
        container (lxml.etree.Element): The element to search in, for example a PartyInfo or the root.
        binding_id (str): The id of the ThisPartyActionBinding.

    Returns:
        lxml.etree.Element: The CanSend or CanReceive element, or None.

    Test Functions:
        - test_find_action_by_binding_id
    """
    if not binding_id:
        return None
    for binding in container.iter(cpa('ThisPartyActionBinding')):
        if binding.get(cpa('id')) == binding_id:
            return binding.getparent()
    return None


def opposite_direction(direction):
    """Returns RECEIVE for SEND and SEND for RECEIVE."""
    return RECEIVE if direction == SEND else SEND


def validate_action(root, action_element):
    """
    Checks the references of a CanSend or CanReceive.

    The ChannelId must be a DeliveryChannel of the same partner, the packageId a Packaging of the CPA and the
    OtherPartyActionBinding the opposite action of the other partner, which refers back.

    Args:
        root (lxml.etree.Element): The root element of the CPA.
        action_element (lxml.etree.Element): A CanSend or CanReceive element.

    Returns:
        list: Descriptions of the problems; empty when the action is correct.

    Test Functions:
        - test_validate_action_without_problems
        - test_validate_action_unknown_channel_and_package
        - test_validate_action_missing_other_party_binding
        - test_validate_action_other_party_binding_not_found
        - test_validate_action_other_party_binding_same_direction
        - test_validate_action_other_party_binding_does_not_refer_back
    """
    action = describe_action(action_element)
    party_info = get_party_info_of(action_element)
    problems = []

    channel_ids = get_channel_ids(party_info) if party_info is not None else []
    for channel_id in action['channel_ids']:
        if channel_id not in channel_ids:
            problems.append(f"ChannelId '{channel_id}' is not a DeliveryChannel of this partner")
    if action['package_id'] not in get_package_ids(root):
        problems.append(f"packageId '{action['package_id']}' is not a Packaging of the CPA")

    other_binding = action['other_party_binding']
    if not other_binding:
        problems.append("No OtherPartyActionBinding")
        return problems
    other_party_info = get_other_party_info(root, party_info) if party_info is not None else None
    counterpart = find_action_by_binding_id(other_party_info, other_binding) if other_party_info is not None else None
    if counterpart is None:
        problems.append(f"OtherPartyActionBinding '{other_binding}' is not an action of the other partner")
        return problems
    counterpart_action = describe_action(counterpart)
    if counterpart_action['direction'] != opposite_direction(action['direction']):
        problems.append(f"OtherPartyActionBinding '{other_binding}' is also a "
                        f"{DIRECTION_LABELS[action['direction']]} action")
    if counterpart_action['other_party_binding'] != action['id']:
        problems.append(f"OtherPartyActionBinding '{other_binding}' does not refer back to '{action['id']}'")
    return problems


def find_counterpart_collaboration_role(root, collaboration_role):
    """
    Finds the CollaborationRole of the other partner that belongs to a CollaborationRole.

    First the OtherPartyActionBindings of its actions are followed. Without links the CollaborationRole of the
    other partner with the same Service (name and type) is used.

    Args:
        root (lxml.etree.Element): The root element of the CPA.
        collaboration_role (lxml.etree.Element): A CollaborationRole element.

    Returns:
        lxml.etree.Element: The CollaborationRole of the other partner, or None.

    Test Functions:
        - test_find_counterpart_collaboration_role_by_links
        - test_find_counterpart_collaboration_role_by_service
        - test_find_counterpart_collaboration_role_not_found
    """
    party_info = get_party_info_of(collaboration_role)
    other_party_info = get_other_party_info(root, party_info) if party_info is not None else None
    if other_party_info is None:
        return None
    for binding in collaboration_role.iter(cpa('ThisPartyActionBinding')):
        counterpart = find_action_by_binding_id(other_party_info,
                                                describe_action(binding.getparent())['other_party_binding'])
        if counterpart is not None:
            return get_collaboration_role_of(counterpart)
    service = describe_collaboration_role(collaboration_role)
    for other_role in other_party_info.findall(cpa('CollaborationRole')):
        other_service = describe_collaboration_role(other_role)
        if (other_service['service_name'], other_service['service_type']) == \
                (service['service_name'], service['service_type']):
            return other_role
    return None


def make_valid_id(value):
    """
    Turns a text into a valid XML id (NCName): other characters become '_' and a leading digit, '.' or '-'
    gets a '_' in front.

    Test Functions:
        - test_make_valid_id
    """
    valid = re.sub(r'[^A-Za-z0-9_.-]', '_', value.strip())
    if not valid or not (valid[0].isalpha() or valid[0] == '_'):
        valid = '_' + valid
    return valid


def make_unique_id(root, candidate, reserved=()):
    """
    Adds _2, _3, ... to the id until no element of the CPA and no reserved id uses it.

    Test Functions:
        - test_make_unique_id
    """
    used = get_used_ids(root) | set(reserved)
    unique, number = candidate, 2
    while unique in used:
        unique = f"{candidate}_{number}"
        number += 1
    return unique


def suggest_binding_id(root, collaboration_role, direction, action_name, reserved=()):
    """
    Suggests the id of the ThisPartyActionBinding of a new action.

    An action with the same direction in the same CollaborationRole whose id contains its action name is used as
    example: the action name in that id is replaced, so the naming convention of the CPA is kept. Without such
    an action the id is <partyName>_<service>_<S or R>_<action>.

    Args:
        root (lxml.etree.Element): The root element of the CPA.
        collaboration_role (lxml.etree.Element): The CollaborationRole that gets the action.
        direction (str): SEND or RECEIVE.
        action_name (str): The action of the new action.
        reserved (iterable): Ids that are not in the CPA yet but are taken, for example the mirrored action.

    Returns:
        str: A valid id that is not used in the CPA.

    Test Functions:
        - test_suggest_binding_id_follows_example
        - test_suggest_binding_id_without_example
        - test_suggest_binding_id_is_unique
    """
    service_binding = get_service_binding(collaboration_role)
    candidate = None
    for element in get_action_elements(service_binding) if service_binding is not None else []:
        example = describe_action(element)
        if example['direction'] == direction and example['action'] and example['action'] in example['id']:
            # Replace the last occurrence: the action name is near the end in the usual convention
            start = example['id'].rfind(example['action'])
            candidate = example['id'][:start] + action_name + example['id'][start + len(example['action']):]
            break
    if candidate is None:
        party_info = get_party_info_of(collaboration_role)
        party_name = get_party_name(party_info) if party_info is not None else ''
        service_name = describe_collaboration_role(collaboration_role)['service_name']
        letter = 'S' if direction == SEND else 'R'
        candidate = '_'.join(part for part in (party_name, service_name, letter, action_name) if part)
    return make_unique_id(root, make_valid_id(candidate), reserved)


def suggest_channel_id(collaboration_role, direction):
    """
    Suggests the ChannelId of a new action: the channel of another action with the same direction in the
    CollaborationRole, else the defaultMshChannelId of the partner, else its first DeliveryChannel.

    Test Functions:
        - test_suggest_channel_id
    """
    service_binding = get_service_binding(collaboration_role)
    for element in get_action_elements(service_binding) if service_binding is not None else []:
        example = describe_action(element)
        if example['direction'] == direction and example['channel_ids']:
            return example['channel_ids'][0]
    party_info = get_party_info_of(collaboration_role)
    if party_info is None:
        return ''
    default_channel = party_info.get(cpa('defaultMshChannelId'))
    channel_ids = get_channel_ids(party_info)
    if default_channel in channel_ids:
        return default_channel
    return channel_ids[0] if channel_ids else ''


def suggest_characteristics(collaboration_role):
    """
    Returns the BusinessTransactionCharacteristics for a new action: those of the first action in the
    CollaborationRole, completed with the defaults.

    Test Functions:
        - test_suggest_characteristics
    """
    characteristics = dict(DEFAULT_CHARACTERISTICS)
    service_binding = get_service_binding(collaboration_role)
    actions = get_action_elements(service_binding) if service_binding is not None else []
    if actions:
        characteristics.update(describe_action(actions[0])['characteristics'])
    return characteristics


def insert_indented(parent, index, element):
    """
    Inserts the element at the index and indents it like its siblings, so the saved CPA stays readable.

    Test Functions:
        - test_insert_indented_matches_siblings
    """
    # The whitespace before the first child (for example "\n            ") and before the closing tag of the
    # parent (for example "\n        "): the difference is one level of indentation
    sibling_whitespace = parent.text if len(parent) and parent.text and not parent.text.strip() else None
    closing_whitespace = parent[-1].tail if len(parent) else None
    if sibling_whitespace is not None and closing_whitespace is not None and not closing_whitespace.strip():
        child_indent = sibling_whitespace.rsplit('\n', 1)[-1]
        parent_indent = closing_whitespace.rsplit('\n', 1)[-1]
        space = child_indent[len(parent_indent):]
        if space and child_indent.startswith(parent_indent) and len(child_indent) % len(space) == 0:
            lxml_etree.indent(element, space=space, level=len(child_indent) // len(space))
    else:
        sibling_whitespace = None
    parent.insert(index, element)
    if sibling_whitespace is None:
        return
    if index > 0:
        previous = parent[index - 1]
        element.tail = previous.tail
        previous.tail = sibling_whitespace
    else:
        element.tail = sibling_whitespace


def build_action(direction, binding_id, action_name, package_id, channel_id, characteristics,
                 other_party_binding=''):
    """
    Builds a CanSend or CanReceive element.

    Args:
        direction (str): SEND or RECEIVE.
        binding_id (str): The id of the ThisPartyActionBinding.
        action_name (str): The action.
        package_id (str): The id of the Packaging.
        channel_id (str): The channelId of the DeliveryChannel.
        characteristics (dict): The attributes of BusinessTransactionCharacteristics; empty values are left out.
        other_party_binding (str): The id of the ThisPartyActionBinding of the other partner; empty: left out.

    Returns:
        lxml.etree.Element: The new element, not yet in the CPA.

    Test Functions:
        - test_build_action
    """
    action_element = lxml_etree.Element(cpa(direction))
    binding = lxml_etree.SubElement(action_element, cpa('ThisPartyActionBinding'))
    binding.set(cpa('id'), binding_id)
    binding.set(cpa('action'), action_name)
    binding.set(cpa('packageId'), package_id)
    characteristics_element = lxml_etree.SubElement(binding, cpa('BusinessTransactionCharacteristics'))
    set_characteristics(characteristics_element, characteristics)
    lxml_etree.SubElement(binding, cpa('ChannelId')).text = channel_id
    if other_party_binding:
        lxml_etree.SubElement(action_element, cpa('OtherPartyActionBinding')).text = other_party_binding
    return action_element


def set_characteristics(characteristics_element, characteristics):
    """Writes the attributes of BusinessTransactionCharacteristics; an empty value removes the attribute."""
    known = [name for name, _, _ in CHARACTERISTICS]
    for name in known + [name for name in characteristics if name not in known]:
        if name not in characteristics:
            continue
        value = (characteristics[name] or '').strip()
        if value:
            characteristics_element.set(cpa(name), value)
        elif cpa(name) in characteristics_element.attrib:
            del characteristics_element.attrib[cpa(name)]


def insert_action(collaboration_role, action_element):
    """
    Adds a CanSend or CanReceive to the ServiceBinding of a CollaborationRole in the order of the schema:
    Service, then all CanSend, then all CanReceive.

    Test Functions:
        - test_insert_action_keeps_schema_order
    """
    service_binding = get_service_binding(collaboration_role)
    if service_binding is None:
        raise ValueError("The CollaborationRole has no ServiceBinding")
    index = 0
    for position, child in enumerate(service_binding):
        if not isinstance(child.tag, str):
            continue
        if child.tag == cpa('Service') or child.tag == cpa(SEND) or \
                (child.tag == cpa(RECEIVE) and action_element.tag == cpa(RECEIVE)):
            index = position + 1
    insert_indented(service_binding, index, action_element)


def add_action(root, collaboration_role, direction, action_name, package_id, channel_id, characteristics,
               binding_id, mirror=None):
    """
    Adds a Service action to a CollaborationRole and optionally the mirrored action at the other partner.

    The mirrored action has the opposite direction (a CanSend gets a CanReceive) and both
    OtherPartyActionBindings refer to each other.

    Args:
        root (lxml.etree.Element): The root element of the CPA.
        collaboration_role (lxml.etree.Element): The CollaborationRole that gets the action.
        direction (str): SEND or RECEIVE.
        action_name (str): The action.
        package_id (str): The id of the Packaging, used for both actions.
        channel_id (str): The channelId for the action.
        characteristics (dict): The BusinessTransactionCharacteristics, used for both actions.
        binding_id (str): The id of the ThisPartyActionBinding.
        mirror (dict): None, or collaboration_role, binding_id and channel_id of the mirrored action.

    Returns:
        tuple: The new element and the new mirrored element (None without mirror).

    Raises:
        ValueError: When a required value is missing or an id is already used.

    Test Functions:
        - test_add_action_without_mirror
        - test_add_action_with_mirror
        - test_add_action_rejects_used_id
        - test_add_action_rejects_missing_values
        - test_add_action_result_is_valid_against_schema
    """
    if not action_name.strip():
        raise ValueError("Action is required")
    if not package_id:
        raise ValueError("Package is required")
    if not channel_id:
        raise ValueError("Channel is required")
    used_ids = get_used_ids(root)
    ids = [binding_id] + ([mirror['binding_id']] if mirror else [])
    for new_id in ids:
        if not new_id or make_valid_id(new_id) != new_id:
            raise ValueError(f"'{new_id}' is not a valid id")
        if new_id in used_ids:
            raise ValueError(f"The id '{new_id}' is already used in the CPA")
    if mirror and mirror['binding_id'] == binding_id:
        raise ValueError("The action and the mirrored action need different ids")
    if mirror and not mirror.get('channel_id'):
        raise ValueError("Channel of the mirrored action is required")

    action_element = build_action(direction, binding_id, action_name.strip(), package_id, channel_id,
                                  characteristics, mirror['binding_id'] if mirror else '')
    insert_action(collaboration_role, action_element)
    mirror_element = None
    if mirror:
        mirror_element = build_action(opposite_direction(direction), mirror['binding_id'], action_name.strip(),
                                      package_id, mirror['channel_id'], characteristics, binding_id)
        insert_action(mirror['collaboration_role'], mirror_element)
    return action_element, mirror_element


def update_action(action_element, action_name, package_id, channel_id, characteristics):
    """
    Changes the action, packageId, first ChannelId and BusinessTransactionCharacteristics of a CanSend or
    CanReceive. The id stays the same, so the OtherPartyActionBinding of the other partner keeps working.

    Raises:
        ValueError: When action, package or channel is empty.

    Test Functions:
        - test_update_action
        - test_update_action_rejects_empty_action
    """
    if not action_name.strip():
        raise ValueError("Action is required")
    if not package_id:
        raise ValueError("Package is required")
    if not channel_id:
        raise ValueError("Channel is required")
    binding = action_element.find(cpa('ThisPartyActionBinding'))
    binding.set(cpa('action'), action_name.strip())
    binding.set(cpa('packageId'), package_id)
    channel = binding.find(cpa('ChannelId'))
    if channel is None:
        channel = lxml_etree.SubElement(binding, cpa('ChannelId'))
    channel.text = channel_id
    characteristics_element = binding.find(cpa('BusinessTransactionCharacteristics'))
    if characteristics_element is None:
        characteristics_element = lxml_etree.Element(cpa('BusinessTransactionCharacteristics'))
        binding.insert(0, characteristics_element)
    set_characteristics(characteristics_element, characteristics)


def remove_element(element):
    """Removes an element and keeps the whitespace of its siblings correct."""
    parent = element.getparent()
    previous = element.getprevious()
    if previous is not None:
        previous.tail = element.tail if element.getnext() is None else previous.tail
    elif element.getnext() is None:
        # Last child: the closing tag of the parent moves to the indentation of the parent
        parent.text = element.tail
    parent.remove(element)


def delete_action(root, action_element, delete_counterpart=False):
    """
    Deletes a CanSend or CanReceive and optionally the action of the other partner it refers to.

    Args:
        root (lxml.etree.Element): The root element of the CPA.
        action_element (lxml.etree.Element): The CanSend or CanReceive to delete.
        delete_counterpart (bool): Also delete the action in its OtherPartyActionBinding.

    Returns:
        list: The ids of the deleted ThisPartyActionBindings.

    Test Functions:
        - test_delete_action_only
        - test_delete_action_with_counterpart
    """
    action = describe_action(action_element)
    deleted = [action['id']]
    counterpart = None
    if delete_counterpart:
        party_info = get_party_info_of(action_element)
        other_party_info = get_other_party_info(root, party_info) if party_info is not None else None
        if other_party_info is not None:
            counterpart = find_action_by_binding_id(other_party_info, action['other_party_binding'])
    remove_element(action_element)
    if counterpart is not None:
        deleted.append(describe_action(counterpart)['id'])
        remove_element(counterpart)
    return deleted


def build_collaboration_role(process_name, process_version, process_href, process_uuid, role_name, role_href,
                             service_name, service_type):
    """
    Builds a CollaborationRole with a ProcessSpecification, Role and ServiceBinding without actions.

    Test Functions:
        - test_build_collaboration_role
    """
    collaboration_role = lxml_etree.Element(cpa('CollaborationRole'))
    process = lxml_etree.SubElement(collaboration_role, cpa('ProcessSpecification'))
    process.set(cpa('name'), process_name)
    process.set(cpa('version'), process_version)
    process.set(xlink('type'), 'simple')
    process.set(xlink('href'), process_href)
    if process_uuid:
        process.set(cpa('uuid'), process_uuid)
    role = lxml_etree.SubElement(collaboration_role, cpa('Role'))
    role.set(cpa('name'), role_name)
    role.set(xlink('type'), 'simple')
    role.set(xlink('href'), role_href)
    service_binding = lxml_etree.SubElement(collaboration_role, cpa('ServiceBinding'))
    service = lxml_etree.SubElement(service_binding, cpa('Service'))
    if service_type:
        service.set(cpa('type'), service_type)
    service.text = service_name
    return collaboration_role


def add_collaboration_role(party_info, process_name, process_version, process_href, process_uuid, role_name,
                           role_href, service_name, service_type):
    """
    Adds a CollaborationRole to a partner, after its last CollaborationRole (order of the schema).

    Raises:
        ValueError: When the process name, process version, role name or service name is empty.

    Returns:
        lxml.etree.Element: The new CollaborationRole.

    Test Functions:
        - test_add_collaboration_role
        - test_add_collaboration_role_rejects_missing_values
    """
    for label, value in (("Process name", process_name), ("Process version", process_version),
                         ("Role name", role_name), ("Service name", service_name)):
        if not value.strip():
            raise ValueError(f"{label} is required")
    collaboration_role = build_collaboration_role(process_name.strip(), process_version.strip(),
                                                  process_href.strip(), process_uuid.strip(), role_name.strip(),
                                                  role_href.strip(), service_name.strip(), service_type.strip())
    # Schema order of PartyInfo: PartyId, PartyRef, CollaborationRole, Certificate, ...
    index = 0
    for position, child in enumerate(party_info):
        if child.tag in (cpa('PartyId'), cpa('PartyRef'), cpa('CollaborationRole')):
            index = position + 1
    insert_indented(party_info, index, collaboration_role)
    return collaboration_role


def delete_collaboration_role(collaboration_role):
    """
    Deletes a CollaborationRole. A partner must keep at least one CollaborationRole (schema).

    Raises:
        ValueError: When it is the last CollaborationRole of the partner.

    Test Functions:
        - test_delete_collaboration_role
        - test_delete_last_collaboration_role_is_refused
    """
    party_info = collaboration_role.getparent()
    if len(party_info.findall(cpa('CollaborationRole'))) <= 1:
        raise ValueError("A partner needs at least one CollaborationRole")
    remove_element(collaboration_role)
