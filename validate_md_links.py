import os
import re

def validate_and_update_links(directory):
    md_files = [os.path.join(directory, f) for f in os.listdir(directory) if f.endswith('.md')]
    link_pattern = re.compile(r'\[([^\]]+)\]\(([^)]+)\)')

    for file_path in md_files:
        with open(file_path, 'r') as file:
            content = file.read()

        updated_content = link_pattern.sub(lambda match: validate_link(match, md_files), content)

        with open(file_path, 'w') as file:
            file.write(updated_content)

def validate_link(match, md_files):
    link_text, link_target = match.groups()
    if link_target.endswith('.md') and any(os.path.basename(f) == link_target for f in md_files):
        return f'[{link_text}]({link_target})'
    else:
        print(f"Invalid link found: {link_text} -> {link_target}")
        return match.group(0)

if __name__ == "__main__":
    directory = 'application/documentation/elements'
    validate_and_update_links(directory)