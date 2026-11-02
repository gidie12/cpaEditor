# One colour per partner, in the order of the PartyInfo elements of the CPA (partner A first), so a partner has the
# same colour on every tab
PARTNER_COLORS = ('#cfe2ff', '#f8d7da')
# Dark text: the light backgrounds must also be readable in dark mode
PARTNER_TEXT_COLOR = 'black'


def partner_color(index):
    """
    Returns the background colour of the partner at this position (0 for partner A).

    Test Functions:
        - test_partner_color
    """
    return PARTNER_COLORS[index % len(PARTNER_COLORS)]


def configure_partner_tag(tree, tag, index):
    """Gives the rows of a Treeview with this tag the colours of the partner at this position."""
    tree.tag_configure(tag, background=partner_color(index), foreground=PARTNER_TEXT_COLOR)
