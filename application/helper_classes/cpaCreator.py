import xml.etree.ElementTree as ET
from lxml import etree as lxml_etree

from MarkupPy.markup import element


class CPACreator():
    def __init__(self, root, logger):
        self.root = root
        self.namespace = 'http://www.oasis-open.org/committees/ebxml-cppa/schema/cpp-cpa-2_0.xsd'
        self.nsmap = {"tns": "'http://www.oasis-open.org/committees/ebxml-cppa/schema/cpp-cpa-2_0.xsd"}
        self.namespace_party_info = '{' + self.namespace + '}' + 'PartyInfo'
        self.party_info_partner_a = self.root.findall(f'./{self.namespace_party_info}')[0]
        self.party_info_partner_b = self.root.findall(f'./{self.namespace_party_info}')[1]
        self.key_mapping = {
            'CPAId': lambda v: self.change_cpa_id(v),
            'partyNamePartnerA': lambda v: self.change_party_name(v, 'A'),
            'partyNamePartnerB': lambda v: self.change_party_name(v, 'B'),
            'partyIdPartnerA': lambda v: self.change_party_id(v, 'A'),
            'partyIdPartnerB': lambda v: self.change_party_id(v, 'B'),
            'partyIdTypePartnerA': lambda v: self.change_party_id_type(v, 'A'),
            'partyIdTypePartnerB': lambda v: self.change_party_id_type(v, 'B'),
            'cpaStatus': lambda v: self.change_status(v),
            'cpaStartDate': lambda v: self.change_start_date(v),
            'cpaEndDate': lambda v: self.change_end_date(v),
            'changeXMLElement': lambda v: self.changeXMLElement(v[0], v[1]),
            'deleteXMLElement': lambda v: self.deleteXMLElement(v[0]),
            'addXMLElement': lambda v: self.addXMLElement(v[0], v[1])
        }
        self.logger = logger

    def changeXMLElement(self, value, element_object):
        try:
            element_object.text = value
        except Exception as e:
            self.logger.error(f'Error while changing element: {e}')


    def change_comment(self, value):
        comment = ET.Element('{http://www.oasis-open.org/committees/ebxml-cppa/schema/cpp-cpa-2_0.xsd}Comment')
        comment.text = value

    def change_start_date(self, value):
        namespace_element_name = '{' + self.namespace + '}' + 'Start'

        try:
            self.root.find(f'./{namespace_element_name}').text = value
        except Exception as e:
            print(e)
    def change_end_date(self, value):
        namespace_element_name = '{' + self.namespace + '}' + 'End'

        try:
            self.root.find(f'./{namespace_element_name}').text = value
        except Exception as e:
            print(e)

    def change_party_id_type(self, value, partner):
        namespace_element_name_party_id = '{' + self.namespace + '}' + 'PartyId'
        namespace_element_name_party_id_type = '{' + self.namespace + '}' + 'type'

        try:
            partner_info = {
                'A': self.party_info_partner_a,
                'B': self.party_info_partner_b
            }
            partner_info[partner].find(f'./{namespace_element_name_party_id}').attrib[namespace_element_name_party_id_type] = value
        except Exception as e:
            print(e)

    def change_status(self, value):
        namespaced_attribute_name = '{' + self.namespace + '}' + 'value'
        namespaced_element_name = '{' + self.namespace + '}' + 'Status'
        try:
            self.root.find(f"./{namespaced_element_name}").attrib[namespaced_attribute_name] = value
        except Exception as e:
            print(e)

    def change_cpa_id(self, value):
        try:
            self.root.attrib[f'{{{self.namespace}}}cpaid'] = value
        except KeyError as e:
            print(f"Namespace '{self.namespace}' not found in the XML document.")
        except Exception as e:
            print(e)

    def change_party_name(self, value, partner):
        namespace_element_party_name = '{' + self.namespace + '}' + 'partyName'
        try:
            partner_info = {
                'A': self.party_info_partner_a,
                'B': self.party_info_partner_b
            }
            partner_info[partner].attrib[namespace_element_party_name] = value
        except Exception as e:
            print(e)
    def change_party_id(self, value, partner):
        namespace_element_name_party_id = '{' + self.namespace + '}' + 'PartyId'
        try:
            partner_info = {
                'A': self.party_info_partner_a,
                'B': self.party_info_partner_b
            }
            partner_info[partner].find(f'./{namespace_element_name_party_id}').text = value
        except Exception as e:
            print(e)

    # def create_comment(self):
    #     self.root.co

    def change_certificate(self, value):
        partner_info = {
            'A': self.party_info_partner_a,
            'B': self.party_info_partner_b
        }
        # partner_info[partner].find(f'./{namespace_element_name_party_id}').text = value
    def apply_changes(self, cpa_dict_entries):
        for key, value in cpa_dict_entries.items():

            func = self.key_mapping.get(key)
            if func:
                func(value)
            else:
                print(f"Unsupported key: {key}")

    def deleteXMLElement(self, element_object):
        try:
            print(f'Deleting element: {element_object}')
            self.root.remove(element_object)
        except Exception as e:
            self.logger.error(f'Error while deleting element: {e}')

    def addXMLElement(self, element, parent):
        try:
            parent.append(element)
        except Exception as e:
            self.logger.error(f'Error while adding element: {e}')



