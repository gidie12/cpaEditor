def get_parent(target_element):
    return target_element.getparent()


class CPAParser():
    def __init__(self, root):
        self.security_details_party_b = None
        self.security_details_party_a = None
        self.party_info_b_collaboration_roles = None
        self.party_info_a_collaboration_roles = None
        self.certificate_party_b_elements = None
        self.certificate_party_a_elements = None
        self.transport_items_list = None
        self.comments_in_file = []
        self.party_ids_type_list = []
        self.root = root
        self.cpa_namespace = 'http://www.oasis-open.org/committees/ebxml-cppa/schema/cpp-cpa-2_0.xsd'
        self.ds_namespace = 'http://www.w3.org/2000/09/xmldsig#'
        self.nsmap = {"tns": "'http://www.oasis-open.org/committees/ebxml-cppa/schema/cpp-cpa-2_0.xsd"}
        self.party_ids_list = []
        self.party_names_list = []
        self.start_date = ""
        self.end_date = ""
        self.cpa_status = ""
        self.partner_a_key_info = []
        self.partner_b_key_info = []

        # Haal partners op uit xml en plaats in partners_list
        self.get_partners()
        party_info = '{' + self.cpa_namespace + '}' + 'PartyInfo'
        self.party_info_list = self.root.findall(f'./{party_info}')

        self.party_info_a = self.party_info_list[0]
        self.party_info_b = self.party_info_list[1]
        self.collaboration_roles()

        self.transport_items()
        # self.partners_key_info()
        self.certificate_info()
        self.security_details()

    def get_certificate_party_b_elements(self):
        return self.certificate_party_b_elements

    def get_certificate_party_a_elements(self):
        return self.certificate_party_a_elements

    def get_cert_ids_partner_a(self):
        namespaced_attribute_name = '{' + self.cpa_namespace + '}' + 'certId'
        cert_ids = []
        for item in self.certificate_party_a_elements:
            cert_ids.append(item.attrib[namespaced_attribute_name])
        return cert_ids

    def get_cert_ids_partner_b(self):
        namespaced_attribute_name = '{' + self.cpa_namespace + '}' + 'certId'
        cert_ids = []
        for item in self.certificate_party_b_elements:
            cert_ids.append(item.attrib[namespaced_attribute_name])
        return cert_ids

    def get_security_ids_partner_a(self):
        namespaced_attribute_name = '{' + self.cpa_namespace + '}' + 'securityId'
        security_ids = []
        for item in self.security_details_party_a:
            security_ids.append(item.attrib[namespaced_attribute_name])
        return security_ids

    def get_security_ids_partner_b(self):
        namespaced_attribute_name = '{' + self.cpa_namespace + '}' + 'securityId'
        security_ids = []
        for item in self.security_details_party_b:
            security_ids.append(item.attrib[namespaced_attribute_name])
        return security_ids

    def get_cpa_id(self):
        namespaced_attribute_name = '{' + self.cpa_namespace + '}' + 'cpaid'
        try:
            cpa_id_item = self.root.attrib[namespaced_attribute_name]

            return cpa_id_item

        except Exception as e:
            print(e)

    def get_cpa_status(self):
        namespaced_attribute_name = '{' + self.cpa_namespace + '}' + 'value'
        namespaced_element_name = '{' + self.cpa_namespace + '}' + 'Status'
        try:
            self.cpa_status = self.root.find(f"./{namespaced_element_name}").attrib[namespaced_attribute_name]
            return self.cpa_status
        except (AttributeError, KeyError) as e:
            print(f"CPA Status not found. Error: {e}")
            return None

    def get_party_id_partner_a(self):
        if len(self.party_ids_list) == 2:
            return self.party_ids_list[0]
        else:
            print("No party id found for partner A")
            return None

    def get_party_id_partner_b(self):
        if len(self.party_ids_list) == 2:
            return self.party_ids_list[1]
        else:
            print("No party id found for partner B")
            return None

    def get_party_name_partner_a(self):
        if len(self.party_names_list) == 2:
            return self.party_names_list[0]
        else:
            print("No party name found for partner A")
            return None

    def get_party_name_partner_b(self):
        if len(self.party_names_list) == 2:
            return self.party_names_list[1]
        else:
            print("No party name found for partner B")
            return None

    def get_party_ids_type_partner_a(self):
        if len(self.party_ids_type_list) == 2:
            return self.party_ids_type_list[0]
        else:
            print("No party id type found for partner A")
            return None

    def get_party_ids_type_partner_b(self):
        if len(self.party_ids_type_list) == 2:
            return self.party_ids_type_list[1]
        else:
            print("No party id type found for partner B")
            return None

    def get_partners(self):
        namespaced_attribute_name = '{' + self.cpa_namespace + '}' + 'value'
        namespaced_element_name = '{' + self.cpa_namespace + '}' + 'PartyInfo'
        namespaced_element_name_party_id = '{' + self.cpa_namespace + '}' + 'PartyId'
        namespaced_element_name_party_name = '{' + self.cpa_namespace + '}' + 'partyName'
        namespaced_element_name_party_id_type = '{' + self.cpa_namespace + '}' + 'type'

        try:
            for item in self.root:
                if item.tag == namespaced_element_name:
                    # print(item.find(f'./{namespaced_element_name2}').text)
                    # for element in item:
                    #     element.tag ==
                    self.party_ids_list.append(item.find(f'./{namespaced_element_name_party_id}').text)
                    self.party_ids_type_list.append(item.find(f'./{namespaced_element_name_party_id}').attrib[
                                                        namespaced_element_name_party_id_type])
                    self.party_names_list.append(item.attrib[namespaced_element_name_party_name])
        except Exception as e:
            print(e)

    def get_cpa_start_date(self):
        namespaced_attribute_name = '{' + self.cpa_namespace + '}' + 'value'
        namespaced_element_name = '{' + self.cpa_namespace + '}' + 'Start'
        try:
            for item in self.root:
                if item.tag == namespaced_element_name:
                    self.start_date = item.text
                    return self.start_date
        except Exception as e:
            print(e)
        return None

    def get_cpa_end_date(self):
        namespaced_element_name = '{' + self.cpa_namespace + '}' + 'End'
        try:
            for item in self.root:
                if item.tag == namespaced_element_name:
                    self.end_date = item.text
                    return self.end_date
        except Exception as e:
            print(e)
        return None

    def get_comments(self):
        namespace_element_name = '{' + self.cpa_namespace + '}' + 'Comment'

        comments_xml = self.root.findall(f'./{namespace_element_name}')
        # for comment in comments_xml:
        #     self.comments_in_file.append(comment.text)

        return comments_xml

    def get_namespace_and_short_name(self):
        if self.root.tag.startswith('{'):
            namespace_uri, _, short_name = self.root.tag[1:].partition('}')
        else:
            namespace_uri = ''
            short_name = self.root.tag
        return namespace_uri, short_name

    def get_collaboration_roles_partner_a(self):
        return self.party_info_a_collaboration_roles

    def get_collaboration_roles_partner_b(self):
        return self.party_info_b_collaboration_roles

    def collaboration_roles(self):
        namespace_element_name = '{' + self.cpa_namespace + '}' + 'CollaborationRole'
        self.party_info_a_collaboration_roles = self.party_info_a.findall(f'./{namespace_element_name}')
        self.party_info_b_collaboration_roles = self.party_info_b.findall(f'./{namespace_element_name}')

    def certificate_info(self):
        namespace_element_certificate = '{' + self.cpa_namespace + '}' + 'Certificate'
        self.certificate_party_a_elements = self.party_info_a.findall(f'./{namespace_element_certificate}')
        self.certificate_party_b_elements = self.party_info_b.findall(f'./{namespace_element_certificate}')

    def security_details(self):
        # Get security details for both partners
        namespace_element_security_details = '{' + self.cpa_namespace + '}' + 'SecurityDetails'
        self.security_details_party_a = self.party_info_a.findall(f'./{namespace_element_security_details}')
        self.security_details_party_b = self.party_info_b.findall(f'./{namespace_element_security_details}')

    def get_transport_items_partner_a(self):
        return self.party_info_a_transport

    def get_transport_items_partner_b(self):
        return self.party_info_b_transport

    def get_transport_items(self):
        return self.transport_items_list

    def transport_items(self):
        namespace_attribute_name = '{' + self.cpa_namespace + '}' + 'partyName'
        namespace_element_transport = '{' + self.cpa_namespace + '}' + 'Transport'
        namespace_element_transport_sender = '{' + self.cpa_namespace + '}' + 'TransportSender'
        namespace_element_transport_receiver = '{' + self.cpa_namespace + '}' + 'TransportReceiver'
        namespace_element_transport_protocol = '{' + self.cpa_namespace + '}' + 'TransportProtocol'
        namespace_element_endpoint = '{' + self.cpa_namespace + '}' + 'Endpoint'
        namespace_element_client_certificate_ref = '{' + self.cpa_namespace + '}' + 'ClientCertificateRef'
        namespace_element_transport_id = '{' + self.cpa_namespace + '}' + 'transportId'

        namespace_element_uri = '{' + self.cpa_namespace + '}' + 'uri'
        self.partner_a_transport_items = []
        self.partner_b_transport_items = []
        self.party_info_a_transport = self.party_info_a.findall(f'.//{namespace_element_transport}')
        self.party_info_b_transport = self.party_info_b.findall(f'.//{namespace_element_transport}')
        self.transport_ids = []

        for item in self.party_info_a_transport:
            self.partner_a_transport_items.append(item)
            self.transport_ids.append(item.attrib[f'{namespace_element_transport_id}'])
        for item in self.party_info_b_transport:
            self.partner_b_transport_items.append(item)
            self.transport_ids.append(item.attrib[f'{namespace_element_transport_id}'])

        # for item in transport_items:
        #     namespace_element_id_name = '{' + self.namespace + '}' + 'Id'
        #     id = item.attrib[namespace_element_id_name]
        #
        #     namespace_element_transport_sender_name = '{' + self.namespace + '}' + 'TransportSender'
        #     namespace_element_transport_receiver_name = '{' + self.namespace + '}' + 'TransportReceiver'
        #
        #     namespace_element_transport_protocol_name = '{' + self.namespace + '}' + 'TransportProtocol'
        #     namespace_element_transport_endpoint = '{' + self.namespace + '}' + 'Endpoint'
        #     namespace_element_transport_endpoint_uri = '{' + self.namespace + '}' + 'uri'
        #
        #     transport_sender = item.find(f'./{namespace_element_transport_sender_name}')[0]
        #     transport_sender_transport_protocol = item.find(
        #         f'./{namespace_element_transport_sender_name}/{namespace_element_transport_protocol_name}')
        #     transport_receiver = item.find(f'./{namespace_element_transport_receiver_name}')[0]
        #     transport_receiver_transport_protocol = \
        #     item.find(f'./{namespace_element_transport_receiver_name}/{namespace_element_transport_protocol_name}')[0]
        #
        #
        #     transport_receiver_endpoint = \
        #     item.find(f'./{namespace_element_transport_receiver_name}/{namespace_element_transport_protocol_name}')[0]
        #     transport_receiver_endpoint_uri = item.find(
        #         f'./{namespace_element_transport_receiver_name}/{namespace_element_transport_protocol_name}').attrib[
        #         namespace_element_transport_endpoint_uri]
        #
        #     record = {'id': id, 'xml_object': item, 'transport_sender': transport_sender,
        #               'transport_receiver': transport_receiver,
        #               'transport_sender_transport_protocol': transport_sender_transport_protocol,
        #               'transport_receiver_transport_protocol': transport_receiver_transport_protocol,
        #               'transport_receiver_endpoint':transport_receiver_endpoint, 'transport_receiver_endpoint_uri':
        #                   transport_receiver_endpoint_uri
        #               }
        #     print(record)
