import os
import markdown
import re

def convert_md_to_html(directory, readme_path):
    md_files = [f for f in os.listdir(directory) if f.endswith('.md')]

    # Ensure Readme.md is included
    if os.path.exists(readme_path):
        md_files.append(readme_path)

    # Create html/elements directory if it doesn't exist
    documentation_dir = os.path.dirname(os.path.dirname(os.path.dirname(directory)))
    html_directory = os.path.join(documentation_dir, 'html', 'cpp-cpa', 'elements')
    if not os.path.exists(html_directory):
        os.makedirs(html_directory)

    for md_file in md_files:
        if md_file == readme_path:
            md_path = readme_path
            html_file = 'Readme.html'
            html_path = os.path.join(documentation_dir, 'html', 'cpp-cpa', html_file)
        else:
            md_path = os.path.join(directory, md_file)
            html_file = md_file.replace('.md', '.html')
            html_path = os.path.join(html_directory, html_file)

        with open(md_path, 'r') as file:
            md_content = file.read()

        # Replace .md links with .html links
        md_content = re.sub(r'\(([^)]+)\.md\)', r'(\1.html)', md_content)

        # Convert Markdown to HTML with fenced code blocks
        html_content = markdown.markdown(md_content, extensions=['fenced_code'])

        with open(html_path, 'w') as file:
            file.write(html_content)
        print(f"Converted {md_file} to {html_file}")

if __name__ == "__main__":
    directory = 'application/documentation/sources/markdown/cpp-cpa/elements'
    readme_path = 'application/documentation/sources/markdown/cpp-cpa/Readme.md'
    convert_md_to_html(directory, readme_path)