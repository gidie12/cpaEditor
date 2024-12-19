import os

# directories.py

# Dynamically determine the base directory for the project
BASE_DIR = os.path.dirname((os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

# Directory for application source code
APP_DIR = os.path.join(BASE_DIR, 'application')

# Directory for test cases
TESTS_DIR = os.path.join(APP_DIR, 'tests')

# Directory for helper classes
HELPER_CLASSES_DIR = os.path.join(APP_DIR, 'helper_classes')

# Directory for app classes
APP_CLASSES_DIR = os.path.join(APP_DIR, 'app_classes')

# Directory for resources (if any)
RESOURCES_DIR = os.path.join(BASE_DIR, 'resources')

# Directory for configuration files (if any)
CONFIG_DIR = os.path.join(BASE_DIR, 'config')

# Directory for templates (if any)
TEMPLATES_DIR = os.path.join(APP_DIR, 'templates')

# Documentation directories
DOCUMENTATION_DIR = os.path.join(APP_DIR, 'documentation')