import sys
from gui import launch_gui
import os
import csv

def ensure_directories():
    """Create required directories"""
    directories = ['./sessions', './data']
    for directory in directories:
        os.makedirs(directory, exist_ok=True)

def ensure_data_files():
    """Create required data files if they don't exist"""
    os.makedirs('./data', exist_ok=True)
    
    # Create groups.csv if it doesn't exist
    groups_csv = os.path.join('./data', 'groups.csv')
    if not os.path.exists(groups_csv):
        with open(groups_csv, 'w', newline='', encoding='utf-8') as f:
            writer = csv.writer(f)
            writer.writerow(['group_name'])

if __name__ == "__main__":
    # Create required directories
    ensure_directories()
    
    # Create required data files
    ensure_data_files()
    
    # Launch GUI
    launch_gui() 