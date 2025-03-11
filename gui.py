from telethon import functions
import tkinter as tk
from tkinter import ttk, messagebox, filedialog
import yaml
import os
import asyncio
import random
from datetime import datetime
import csv
from telethon.errors import (
    FloodWaitError,
    ChatWriteForbiddenError,
    UsernameInvalidError, 
    UsernameNotOccupiedError
)
from tg_shill_bot import (
    TelegramBot,
    get_group_members,
    send_dm,
    Style,
    log_green,
    log_red
)
from spintax import SpinTax
import sys
import re

class AccountDialog(tk.Toplevel):
    def __init__(self, parent, api_id='', api_hash='', phone=''):
        super().__init__(parent)
        self.title("Add Account")
        self.result = None
        
        # Create form
        ttk.Label(self, text="API ID:").grid(row=0, column=0, padx=5, pady=5)
        self.api_id = ttk.Entry(self)
        self.api_id.grid(row=0, column=1, padx=5, pady=5)
        if api_id:
            self.api_id.insert(0, str(api_id))
        
        ttk.Label(self, text="API Hash:").grid(row=1, column=0, padx=5, pady=5)
        self.api_hash = ttk.Entry(self)
        self.api_hash.grid(row=1, column=1, padx=5, pady=5)
        if api_hash:
            self.api_hash.insert(0, api_hash)
        
        ttk.Label(self, text="Phone:").grid(row=2, column=0, padx=5, pady=5)
        self.phone = ttk.Entry(self)
        self.phone.grid(row=2, column=1, padx=5, pady=5)
        if phone:
            self.phone.insert(0, phone)
        
        ttk.Button(self, text="OK", command=self.ok).grid(row=3, column=0, padx=5, pady=5)
        ttk.Button(self, text="Cancel", command=self.cancel).grid(row=3, column=1, padx=5, pady=5)
        
        self.transient(parent)
        self.grab_set()
        parent.wait_window(self)

    def ok(self):
        try:
            api_id = int(self.api_id.get())
            api_hash = self.api_hash.get()
            phone = self.phone.get()
            if api_hash and phone:
                self.result = (api_id, api_hash, phone)
                self.destroy()
        except ValueError:
            messagebox.showerror("Error", "Invalid API ID")

    def cancel(self):
        self.destroy()


class CodeDialog(tk.Toplevel):
    def __init__(self, parent):
        super().__init__(parent)
        self.title("Enter Code")
        self.result = None
        
        ttk.Label(self, text="Enter the code sent to your phone:").grid(row=0, column=0, columnspan=2, padx=5, pady=5)
        self.code = ttk.Entry(self)
        self.code.grid(row=1, column=0, columnspan=2, padx=5, pady=5)
        
        ttk.Button(self, text="OK", command=self.ok).grid(row=2, column=0, padx=5, pady=5)
        ttk.Button(self, text="Cancel", command=self.cancel).grid(row=2, column=1, padx=5, pady=5)
        
        self.transient(parent)
        self.grab_set()
        parent.wait_window(self)

    def ok(self):
        code = self.code.get()
        if code:
            self.result = code
            self.destroy()

    def cancel(self):
        self.destroy()

class GroupsDialog(tk.Toplevel):
    def __init__(self, parent):
        super().__init__(parent)
        self.title("Save Groups")
        self.result = None
        
        ttk.Label(self, text="Enter group names (one per line):").grid(row=0, column=0, columnspan=2, padx=5, pady=5)
        
        self.text = tk.Text(self, height=10, width=40)
        self.text.grid(row=1, column=0, columnspan=2, padx=5, pady=5)
        
        ttk.Button(self, text="Save", command=self.save).grid(row=2, column=0, padx=5, pady=5)
        ttk.Button(self, text="Cancel", command=self.cancel).grid(row=2, column=1, padx=5, pady=5)
        
        self.transient(parent)
        self.grab_set()
        parent.wait_window(self)
        
    def save(self):
        self.result = self.text.get("1.0", tk.END).strip()
        self.destroy()
        
    def cancel(self):
        self.destroy()

class ReservationDialog(tk.Toplevel):
    def __init__(self, parent):
        super().__init__(parent)
        self.title("Add Reservation Message")
        self.result = None
        
        # Message Frame
        message_frame = ttk.LabelFrame(self, text="Message")
        message_frame.pack(fill='x', padx=5, pady=5)
        
        self.message_text = tk.Text(message_frame, height=6, width=50)
        self.message_text.pack(padx=5, pady=5)
        
        # Settings Frame
        settings_frame = ttk.LabelFrame(self, text="Reservation Settings")
        settings_frame.pack(fill='x', padx=5, pady=5)
        
        # Start Date
        date_frame = ttk.Frame(settings_frame)
        date_frame.pack(fill='x', padx=5, pady=2)
        ttk.Label(date_frame, text="Start Date:").pack(side='left')
        self.date_entry = ttk.Entry(date_frame)
        self.date_entry.pack(side='left', padx=5)
        self.date_entry.insert(0, datetime.now().strftime("%Y-%m-%d %H:%M"))
        
        # Message Count Limit
        count_frame = ttk.Frame(settings_frame)
        count_frame.pack(fill='x', padx=5, pady=2)
        ttk.Label(count_frame, text="Send After Messages:").pack(side='left')
        self.count_entry = ttk.Entry(count_frame)
        self.count_entry.pack(side='left', padx=5)
        self.count_entry.insert(0, "100")
        
        # Buttons
        button_frame = ttk.Frame(self)
        button_frame.pack(fill='x', padx=5, pady=5)
        ttk.Button(button_frame, text="Save", command=self.save).pack(side='left', padx=5)
        ttk.Button(button_frame, text="Cancel", command=self.cancel).pack(side='left')
        
    def save(self):
        try:
            message = self.message_text.get("1.0", tk.END).strip()
            start_date = datetime.strptime(self.date_entry.get(), "%Y-%m-%d %H:%M")
            count_limit = int(self.count_entry.get())
            
            if not message:
                raise ValueError("Message cannot be empty")
            
            self.result = {
                'message': message,
                'start_date': start_date,
                'count_limit': count_limit
            }
            self.destroy()
            
        except ValueError as e:
            messagebox.showerror("Error", str(e))
            
    def cancel(self):
        self.destroy()

class PasswordDialog(tk.Toplevel):
    def __init__(self, parent):
        super().__init__(parent)
        self.title("Two-Factor Authentication")
        self.result = None
        
        ttk.Label(self, text="Enter your 2FA password:").grid(row=0, column=0, columnspan=2, padx=5, pady=5)
        self.password = ttk.Entry(self, show="*")  # Hide password characters
        self.password.grid(row=1, column=0, columnspan=2, padx=5, pady=5)
        
        ttk.Button(self, text="OK", command=self.ok).grid(row=2, column=0, padx=5, pady=5)
        ttk.Button(self, text="Cancel", command=self.cancel).grid(row=2, column=1, padx=5, pady=5)
        
        self.transient(parent)
        self.grab_set()
        parent.wait_window(self)

    def ok(self):
        password = self.password.get()
        if password:
            self.result = password
            self.destroy()

    def cancel(self):
        self.destroy()

class ScrapingDialog(tk.Toplevel):
    def __init__(self, parent):
        super().__init__(parent)
        self.title("Scrape Group Members")
        self.result = None
        
        # Make dialog modal
        self.transient(parent)
        self.grab_set()
        
        # Source Frame
        source_frame = ttk.LabelFrame(self, text="Group Source")
        source_frame.pack(fill='x', padx=5, pady=5)
        
        self.source_var = tk.StringVar(value="input")
        ttk.Radiobutton(source_frame, text="Get Group from CSV (data/groups.csv)", 
                       value="csv", variable=self.source_var, 
                       command=self.toggle_input).pack(anchor=tk.W, padx=5)
        ttk.Radiobutton(source_frame, text="Get Group from Input", 
                       value="input", variable=self.source_var, 
                       command=self.toggle_input).pack(anchor=tk.W, padx=5)
        
        # Group Input
        self.input_frame = ttk.Frame(source_frame)
        self.input_frame.pack(fill='x', padx=5, pady=5)
        ttk.Label(self.input_frame, text="Group URL:").pack(side='left')
        self.group_entry = ttk.Entry(self.input_frame, width=40)
        self.group_entry.pack(side='left', padx=5)
        
        # Scraping Options Frame
        options_frame = ttk.LabelFrame(self, text="Scraping Options")
        options_frame.pack(fill='x', padx=5, pady=5)
        
        self.scrape_type = tk.StringVar(value="all")
        ttk.Radiobutton(options_frame, text="Scrape All Users", value="all", 
                       variable=self.scrape_type).pack(anchor=tk.W, padx=5)
        ttk.Radiobutton(options_frame, text="Scrape Admins Only", value="admin", 
                       variable=self.scrape_type).pack(anchor=tk.W, padx=5)
        
        # Buttons
        button_frame = ttk.Frame(self)
        button_frame.pack(fill='x', padx=5, pady=5)
        ttk.Button(button_frame, text="Start Scraping", command=self.start).pack(side='left', padx=5)
        ttk.Button(button_frame, text="Cancel", command=self.cancel).pack(side='left')
        
        # Center the dialog
        self.geometry("+%d+%d" % (parent.winfo_rootx() + 50,
                                 parent.winfo_rooty() + 50))
        
        # Wait for window
        self.wait_window()
    
    def toggle_input(self):
        if self.source_var.get() == "input":
            self.group_entry.config(state='normal')
        else:
            self.group_entry.config(state='disabled')
    
    def start(self):
        if self.source_var.get() == "input" and not self.group_entry.get().strip():
            messagebox.showerror("Error", "Please enter a group URL")
            return
            
        self.result = {
            'source': self.source_var.get(),
            'group_url': self.group_entry.get() if self.source_var.get() == "input" else None,
            'scrape_type': self.scrape_type.get()
        }
        self.destroy()
    
    def cancel(self):
        self.result = None
        self.destroy()

class ShillBotGUI:
    def __init__(self, root):
        self.root = root
        self.root.title("Telegram DM Bot")
        self.root.geometry("800x700")
        
        # Create main frame first
        self.main_frame = ttk.Frame(root, padding="5")
        self.main_frame.grid(row=0, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))
        
        # Default settings
        self.default_api_id = 22991900
        self.default_api_hash = '95059a6145cb244308fa8ef38b987088'
        self.default_phone = '+17402171358'
        
        # Set longer intervals to prevent spam
        self.default_interval = "3600-4200"  # 60-70 minutes
        self.group_join_delay = 3600  # 60 minutes between group joins
        
        # Initialize rate limiter with stricter limits
        self.rate_limiter = RateLimiter()
        self.rate_limiter.add_delay('send_message', 3600)  # 1 hour between messages
        self.rate_limiter.add_delay('join_group', 3600)   # 1 hour between joins
        
        # Initialize variables
        self.bots = []
        self.scraped_members = []
        self.is_running = False
        self.loop = asyncio.new_event_loop()
        asyncio.set_event_loop(self.loop)
        self.is_logged_in = False
        self.media_path = None
        self.csv_file_path = None
        self.is_sending = False
        
        # Setup UI
        self.setup_header_section()
        self.setup_settings_section()
        self.setup_message_section()
        self.setup_log_section()
        self.setup_footer_section()
        
        # Ensure data directory exists
        self.ensure_data_dir()

    def ensure_data_dir(self):
        """Ensure data directory exists"""
        self.data_dir = os.path.join(os.path.dirname(__file__), 'data')
        if not os.path.exists(self.data_dir):
            os.makedirs(self.data_dir)

    def setup_header_section(self):
        # Account info frame
        header_frame = ttk.Frame(self.main_frame)
        header_frame.grid(row=0, column=0, columnspan=3, sticky=(tk.W, tk.E), pady=5)
        
        # Left side - Account info
        account_frame = ttk.Frame(header_frame)
        account_frame.grid(row=0, column=0, sticky=tk.W)
        
        ttk.Label(account_frame, text="Signed in").grid(row=0, column=0, sticky=tk.W)
        self.account_display = ttk.Entry(account_frame, width=30, state='readonly')
        self.account_display.grid(row=0, column=1, padx=5)
        self.account_display.insert(0, "Not logged in") 
        
        # Right side - Buttons
        self.button_frame = ttk.Frame(header_frame)
        self.button_frame.grid(row=0, column=1, sticky=tk.E)
        
        # Add Login and Load Session buttons side by side
        self.login_button = ttk.Button(self.button_frame, text="Login", command=self.start_login)
        self.login_button.grid(row=0, column=0, padx=2)
        
        self.load_session_button = ttk.Button(self.button_frame, text="Load Session", command=self.load_session)
        self.load_session_button.grid(row=0, column=1, padx=2)
        
        self.session_buttons = []  # Store session-related buttons
        self.create_session_buttons()
        self.update_button_states()

    def create_session_buttons(self):
        self.logout_button = ttk.Button(self.button_frame, text="Log out", command=self.logout)
        self.add_account_button = ttk.Button(self.button_frame, text="Add Account", command=self.add_account)
        self.save_session_button = ttk.Button(self.button_frame, text="Save Session", command=self.save_session)
        self.load_session_button = ttk.Button(self.button_frame, text="Load Session", command=self.load_session)
        
        self.session_buttons = [
            self.logout_button,
            self.add_account_button,
            self.save_session_button,
            self.load_session_button
        ]

    def update_button_states(self):
        # Check if open_csv_button exists before trying to access it
        if hasattr(self, 'open_csv_button'):
            self.open_csv_button.config(state='disabled')
        
        if self.is_logged_in:
            # Hide login and load session buttons
            self.login_button.grid_remove()
            self.load_session_button.grid_remove()
            
            # Show session management buttons
            for i, button in enumerate(self.session_buttons):
                button.grid(row=0, column=i, padx=2)
        else:
            # Show login and load session buttons
            self.login_button.grid(row=0, column=0, padx=2)
            self.load_session_button.grid(row=0, column=1, padx=2)
            
            # Hide session management buttons
            for button in self.session_buttons:
                button.grid_remove()
            
            # Reset account display
            self.account_display.delete(0, tk.END)
            self.account_display.insert(0, "Not logged in")

    def start_login(self):
        """Modified to pre-fill values"""
        dialog = AccountDialog(self.root, 
                             api_id=self.default_api_id,
                             api_hash=self.default_api_hash,
                             phone=self.default_phone)
        if dialog.result:
            self.loop.run_until_complete(self.process_login(dialog.result))

    async def process_login(self, credentials):
        """Process login with credentials and handle 2FA"""
        try:
            api_id, api_hash, phone = credentials
            bot = TelegramBot()
            success, message = await bot.connect(api_id, api_hash, phone)
            
            if not success:
                if message == "Code requested":
                    code_dialog = CodeDialog(self.root)
                    if code_dialog.result:
                        success, message = await bot.sign_in(phone, code_dialog.result)
                        
                        # Handle 2FA if needed
                        if not success and "Two-steps verification" in str(message):
                            password_dialog = PasswordDialog(self.root)
                            if password_dialog.result:
                                success, message = await bot.check_2fa(password_dialog.result)
                
                if not success:
                    if "PHONE_CODE_INVALID" in str(message):
                        messagebox.showerror("Error", "Invalid code. Please try again.")
                        return
                    elif "PASSWORD_HASH_INVALID" in str(message):
                        messagebox.showerror("Error", "Invalid 2FA password. Please try again.")
                        return
                    else:
                        messagebox.showerror("Error", str(message))
                        return
            
            if success:
                self.bots.append(bot)
                self.is_logged_in = True
                self.update_account_display()
                self.update_button_states()
                self.log_message(f"Successfully logged in with {phone}")
            else:
                messagebox.showerror("Error", message)
                
        except Exception as e:
            messagebox.showerror("Error", f"Login failed: {str(e)}")

    def setup_settings_section(self):
        # Settings frame
        settings_frame = ttk.Frame(self.main_frame)
        settings_frame.grid(row=1, column=0, columnspan=3, sticky=(tk.W, tk.E), pady=5)
        
        # Left side - Group settings
        left_frame = ttk.Frame(settings_frame)
        left_frame.grid(row=0, column=0, sticky=tk.W)
        
        ttk.Label(left_frame, text="Group to DM (@groupName):").grid(row=0, column=0, sticky=tk.W)
        self.group_entry = ttk.Entry(left_frame, width=40)
        self.group_entry.grid(row=0, column=1, sticky=(tk.W, tk.E), pady=2)
        
        # Center - Time settings
        center_frame = ttk.Frame(settings_frame)
        center_frame.grid(row=0, column=1, padx=20, sticky=tk.W)
        
        ttk.Label(center_frame, text="Time interval (sec):").grid(row=0, column=0, sticky=tk.W)
        self.interval_entry = ttk.Entry(center_frame, width=10)
        self.interval_entry.grid(row=0, column=1, sticky=tk.W, pady=2)
        self.interval_entry.insert(0, self.default_interval)
        
        ttk.Label(center_frame, text="Max DM Per Account:").grid(row=1, column=0, sticky=tk.W)
        self.max_dm_entry = ttk.Entry(center_frame, width=10)
        self.max_dm_entry.grid(row=1, column=1, sticky=tk.W, pady=2)
        self.max_dm_entry.insert(0, "50")
        
        # Right side - Checkboxes
        right_frame = ttk.Frame(settings_frame)
        right_frame.grid(row=0, column=2, sticky=tk.E)
        
        self.use_proxy_var = tk.BooleanVar()
        ttk.Checkbutton(right_frame, text="Use Proxy", variable=self.use_proxy_var).grid(row=0, column=0)
        
        self.use_spintax_var = tk.BooleanVar()
        ttk.Checkbutton(right_frame, text="Use SpinTax", variable=self.use_spintax_var).grid(row=1, column=0)

    def setup_message_section(self):
        # Message frame
        message_frame = ttk.LabelFrame(self.main_frame, text="Message")
        message_frame.grid(row=2, column=0, columnspan=3, sticky=(tk.W, tk.E), pady=5)
        
        # Single message text area
        self.message_text = tk.Text(
            message_frame, 
                height=6,
                width=90,
                wrap=tk.WORD,
                undo=True
            )
        self.message_text.pack(fill='both', expand=True, padx=5, pady=5)
        
        # Media label
        self.media_label = ttk.Label(message_frame, text="No media attached")
        self.media_label.pack(padx=5, pady=2)
        
        # Button frame
        button_frame = ttk.Frame(message_frame)
        button_frame.pack(fill='x', padx=5, pady=2)
        
        # Add buttons
        ttk.Button(button_frame, text="Attach Media", command=self.attach_media).pack(side='left', padx=2)
        ttk.Button(button_frame, text="Send Message", command=self.send_message).pack(side='left', padx=2)
        ttk.Button(button_frame, text="Scrape Group", command=self.show_scraping_dialog).pack(side='left', padx=2)

    def setup_log_section(self):
        """Setup log section with clear button"""
        # Log frame
        log_frame = ttk.LabelFrame(self.main_frame, text="Log")
        log_frame.grid(row=3, column=0, columnspan=3, sticky=(tk.W, tk.E, tk.N, tk.S), pady=5)
        
        # Add clear button at the top
        ttk.Button(
            log_frame, 
            text="Clear Log", 
            command=self.clear_log
        ).grid(row=0, column=0, sticky=tk.W, padx=5, pady=2)
        
        # Log text widget
        self.log_text = tk.Text(log_frame, height=10, width=90)
        self.log_text.grid(row=1, column=0, sticky=(tk.W, tk.E, tk.N, tk.S), padx=5, pady=5)
        
        # Add scrollbar
        scrollbar = ttk.Scrollbar(log_frame, orient=tk.VERTICAL, command=self.log_text.yview)
        scrollbar.grid(row=1, column=1, sticky=(tk.N, tk.S))
        self.log_text['yscrollcommand'] = scrollbar.set

    def clear_log(self):
        """Clear the log text widget"""
        self.log_text.delete(1.0, tk.END)
        self.log_message("Log cleared")

    def setup_footer_section(self):
        # Footer frame
        footer_frame = ttk.Frame(self.main_frame)
        footer_frame.grid(row=4, column=0, columnspan=3, sticky=(tk.W, tk.E), pady=5)
        
        # Footer text
        ttk.Label(
            footer_frame, 
            text="This bot is created by @blockchainDeveloper_Ben on Telegram",
            foreground='gray'
        ).grid(row=0, column=0, sticky=tk.W)

    def log_message(self, message):
        """Log message to GUI"""
        try:
            timestamp = datetime.now().strftime("%H:%M:%S")
            log_text = f"[{timestamp}] {message}\n"
            
            # Add to log text widget
            self.log_text.insert(tk.END, log_text)
            self.log_text.see(tk.END)  # Scroll to bottom
            
            # Update GUI
            self.root.update_idletasks()
            
        except Exception as e:
            print(f"GUI logging error: {str(e)}")

    async def add_account_async(self):
        """Add new Telegram account"""
        try:
            # Create login dialog
            dialog = AccountDialog(self.root)
            if dialog.result:
                api_id, api_hash, phone = dialog.result
                
                bot = TelegramBot()
                success, message = await bot.connect(api_id, api_hash, phone)
                
                if success:
                    self.bots.append(bot)
                    self.update_account_display()
                    self.log_message(f"Added account {phone}")
                else:
                    # Show code entry dialog
                    code_dialog = CodeDialog(self.root)
                    if code_dialog.result:
                        success, message = await bot.sign_in(phone, code_dialog.result)
                        if success:
                            self.bots.append(bot)
                            self.update_account_display()
                            self.log_message(f"Added account {phone}")
                        else:
                            messagebox.showerror("Error", message)
                
        except Exception as e:
            messagebox.showerror("Error", f"Failed to add account: {str(e)}")

    def add_account(self):
        """Wrapper for add_account_async"""
        self.loop.run_until_complete(self.add_account_async())

    def save_members_to_csv(self, members, filepath):
        """Save members to CSV file"""
        try:
            # Ensure data directory exists
            os.makedirs(os.path.dirname(filepath), exist_ok=True)
            
            # Add timestamp to filename
            filename = os.path.splitext(filepath)[0]
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filepath = f"{filename}_{timestamp}.csv"
            
            self.log_message(f"Saving members to {filepath}")
            
            with open(filepath, 'w', newline='', encoding='utf-8') as f:
                if members:
                    writer = csv.DictWriter(f, fieldnames=members[0].keys())
                    writer.writeheader()
                    writer.writerows(members)
                    self.log_message(f"Successfully saved {len(members)} members")
                else:
                    self.log_message("No members to save")
                
        except Exception as e:
            self.log_message(f"Error saving members to CSV: {str(e)}")
            raise

    async def scrape_members_async(self):
        """Enhanced member scraping with progress updates"""
        try:
            group = self.group_entry.get()
            if not group:
                raise ValueError("Please enter a group name")
            
            if not self.bots:
                raise ValueError("No accounts logged in")
            
            # Use first account to scrape
            bot = self.bots[0]
            
            # Update GUI to show progress
            self.log_message("Starting member scraping...")
            self.root.update()
            
            # Get members with enhanced info
            members = await get_group_members(bot.client, group)
            
            if members:
                self.scraped_members = members
                
                # Save to single CSV file in data directory
                filename = os.path.join('./data', 'members.csv')
                
                fieldnames = [
                    'username', 'id', 'first_name', 'last_name', 
                    'phone', 'admin', 'last_seen', 'lang_code',
                    'group_name', 'scrape_date', 'send_status'
                ]
                
                with open(filename, 'w', newline='', encoding='utf-8') as f:
                    writer = csv.DictWriter(f, fieldnames=fieldnames)
                    writer.writeheader()
                    writer.writerows(members)
                
                self.log_message(f"Saved {len(members)} members to {filename}")
                self.csv_file_path = filename
                self.open_csv_button.config(state='normal')
                
            else:
                raise ValueError("No valid members found in group")
                
        except Exception as e:
            messagebox.showerror("Error", str(e))

    def scrape_members(self):
        """Wrapper for scrape_members_async"""
        self.loop.run_until_complete(self.scrape_members_async())
        
        # Save scraped members to CSV
        if self.scraped_members:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            group_name = self.group_entry.get().replace('@', '')
            filename = f'scraped_members_{group_name}_{timestamp}.csv'
            
            try:
                with open(filename, 'w', newline='', encoding='utf-8') as f:
                    writer = csv.DictWriter(f, fieldnames=['id', 'username', 'phone'])
                    writer.writeheader()
                    writer.writerows(self.scraped_members)
                self.log_message(f"Saved {len(self.scraped_members)} members to {filename}")
            except Exception as e:
                messagebox.showerror("Error", f"Failed to save members to CSV: {str(e)}")

    def send_message(self):
        """Send message to members from CSV"""
        if self.is_running:
            messagebox.showwarning("Warning", "DM operation already in progress")
            return
        
        try:
            # Check if members.csv exists
            members_file = os.path.join('./data', 'members.csv')
            if not os.path.exists(members_file):
                raise ValueError("No members found. Please scrape members first.")
            
            self.log_message("Reading members from CSV file...")
            
            # Read members from CSV
            members = []
            with open(members_file, 'r', encoding='utf-8') as f:
                reader = csv.DictReader(f)
                members = [row for row in reader]
            
            if not members:
                raise ValueError("No members found in members.csv")
            
            self.log_message(f"Found {len(members)} members in CSV")
            
            # Get message from text area
            message = self.message_text.get("1.0", tk.END).strip()
            if not message:
                raise ValueError("Please enter a message")
            
            # Start sending messages
            self.is_running = True
            self.loop.run_until_complete(self.send_message_async(message, members))
            
        except Exception as e:
            self.log_message(f"Error: {str(e)}")
            messagebox.showerror("Error", str(e))
            self.is_running = False

    async def send_message_async(self, message, members):
        """Send messages to members with rate limiting"""
        try:
            if not self.bots:
                raise ValueError("Please login first")
            
            total_sent = 0
            failed = 0
            retry_queue = []  # Store failed attempts for retry
            
            self.log_message(f"Starting to send messages to {len(members)} members")
            
            # Track last send time for each bot
            bot_last_send = {bot.phone: 0 for bot in self.bots}
            
            for member in members:
                if not self.is_running:
                    self.log_message("Operation cancelled")
                    break
                    
                # Try each bot until message is sent
                message_sent = False
                for bot in self.bots:
                    try:
                        current_time = datetime.now().timestamp()
                        time_since_last_send = current_time - bot_last_send[bot.phone]
                        
                        # Check if bot needs to wait
                        if time_since_last_send < 3600:  # 1 hour in seconds
                            wait_time = 3600 - time_since_last_send
                            self.log_message(f"Bot {bot.phone} needs to wait {int(wait_time)} seconds")
                            continue
                        
                        success = await self.send_dm_safely(bot, member, message)
                        
                        if success:
                            total_sent += 1
                            bot_last_send[bot.phone] = current_time
                            self.log_message(f"Message sent to {member['username']} using {bot.phone}")
                            message_sent = True
                            break  # Exit bot loop on success
                            
                    except Exception as e:
                        self.log_message(f"Error with bot {bot.phone}: {str(e)}")
                        continue
                    
                if not message_sent:
                    failed += 1
                    retry_queue.append(member)
                    self.log_message(f"Failed to send message to {member['username']}, will retry in 1 hour")
                    
                # Progress update
                if (total_sent + failed) % 5 == 0:
                    self.log_message(f"Progress: {total_sent}/{len(members)} messages sent, {failed} failed")
            
            # Handle retries after 1 hour
            if retry_queue:
                self.log_message(f"Waiting 1 hour before retrying {len(retry_queue)} failed messages...")
                await asyncio.sleep(3600)  # Wait 1 hour
                
                # Attempt retries
                for member in retry_queue:
                    if not self.is_running:
                        break
                        
                    message_sent = False
                    for bot in self.bots:
                        try:
                            success = await self.send_dm_safely(bot, member, message)
                            if success:
                                total_sent += 1
                                failed -= 1
                                self.log_message(f"Retry successful: Message sent to {member['username']}")
                                message_sent = True
                                break  # Exit bot loop on success
                                
                        except Exception as e:
                            self.log_message(f"Retry error with bot {bot.phone}: {str(e)}")
                            continue
                        
                    if not message_sent:
                        self.log_message(f"Final retry failed for {member['username']}")
            
            self.log_message(f"Completed: {total_sent} messages sent, {failed} failed")
            
        except Exception as e:
            self.log_message(f"Error in send_message_async: {str(e)}")
        finally:
            self.is_running = False

    def logout(self):
        try:
            for bot in self.bots:
                self.loop.run_until_complete(bot.disconnect())
            self.bots = []
            self.is_logged_in = False
            self.update_account_display()
            self.update_button_states()
            self.log_message("Logged out successfully")
            messagebox.showinfo("Success", "Logged out successfully")
        except Exception as e:
            messagebox.showerror("Error", f"Failed to logout: {str(e)}")

    def save_session(self):
        try:
            settings = {
                'accounts': [],
                'raid': {}
            }
            
            # Save account info
            for bot in self.bots:
                if bot.client and bot.is_connected:
                    settings['accounts'].append({
                        'phone': bot.client.phone,
                        'api_id': bot.client.api_id,
                        'api_hash': bot.client.api_hash
                    })
            
            # Save raid settings
            group = self.group_entry.get()
            if group:
                settings['raid'][group] = {
                    'time_interval': self.interval_entry.get(),
                    'max_dm': self.max_dm_entry.get(),
                    'use_proxy': self.use_proxy_var.get(),
                    'use_spintax': self.use_spintax_var.get()
                }
            
            filename = filedialog.asksaveasfilename(
                defaultextension=".yml",
                filetypes=[("YAML files", "*.yml"), ("All files", "*.*")]
            )
            if filename:
                with open(filename, 'w', encoding='utf8') as f:
                    yaml.dump(settings, f, allow_unicode=True)
                self.log_message(f"Settings saved to {filename}")
            
        except Exception as e:
            messagebox.showerror("Error", f"Failed to save settings: {str(e)}")

    def load_session(self):
        """Load session from file"""
        try:
            filenames = filedialog.askopenfilenames(
                title="Select Session File(s)",
                filetypes=[("Telethon Session", "*.session")],
                initialdir="./sessions"
            )
            
            if not filenames:
                return
            
            # Load settings
            with open('settings.yml', 'r', encoding='utf8') as f:
                settings = yaml.safe_load(f)
            
            api_id = settings.get('api_id')
            api_hash = settings.get('api_hash')
            
            if not api_id or not api_hash:
                raise ValueError("API credentials not found in settings.yml")
            
            # Process each selected session file
            for filename in filenames:
                # Extract phone number from filename
                phone = os.path.splitext(os.path.basename(filename))[0]
                
                # Create new bot instance
                bot = TelegramBot()
                
                # Connect using session file
                success, message = self.loop.run_until_complete(
                    bot.connect(api_id, api_hash, phone)
                )
                
                if success:
                    self.bots.append(bot)
                    self.is_logged_in = True
                    self.log_message(f"Successfully loaded session for {phone}")
                else:
                    self.log_message(f"Failed to load session for {phone}: {message}")
            
            # Update display and buttons
            if self.bots:
                self.update_account_display()
                self.update_button_states()
                messagebox.showinfo("Success", f"Loaded {len(self.bots)} account(s)")
            
        except Exception as e:
            messagebox.showerror("Error", f"Failed to load session file(s): {str(e)}")

    def attach_media(self):
        try:
            filename = filedialog.askopenfilename(
                title="Select Media",
                filetypes=(
                    ("All Media", "*.png;*.jpg;*.jpeg;*.gif;*.mp4;*.mp3;*.webm;*.wav;*.avi"),
                    ("Images", "*.png;*.jpg;*.jpeg;*.gif"),
                    ("Videos", "*.mp4;*.webm;*.avi"),
                    ("Audio", "*.mp3;*.wav"),
                    ("All files", "*.*")
                )
            )
            
            if filename:
                # Verify file exists and is accessible
                if not os.path.exists(filename):
                    raise FileNotFoundError("Selected file does not exist")
                
                # Store media path
                self.media_path = filename
                
                # Update media label
                file_name = os.path.basename(filename)
                self.media_label.config(text=f"Media attached: {file_name}")
                
                self.log_message(f"Media attached: {file_name}")
                
        except Exception as e:
            self.media_path = None
            self.media_label.config(text="No media attached")
            messagebox.showerror("Error", f"Failed to attach media: {str(e)}")

    def update_account_display(self):
        """Update the account display with current accounts"""
        self.account_display.config(state='normal')
        self.account_display.delete(0, tk.END)
        
        if self.bots:
            accounts = [bot.phone for bot in self.bots]
            display_text = f"{len(accounts)} account(s): {', '.join(accounts)}"
            if len(display_text) > 30:
                display_text = f"{len(accounts)} account(s)"
            self.account_display.insert(0, display_text)
        else:
            self.account_display.insert(0, "Not logged in")
        
        self.account_display.config(state='readonly')

    def load_initial_settings(self):
        """Load initial settings from settings.yml"""
        try:
            with open('settings.yml', 'r', encoding='utf8') as f:
                settings = yaml.safe_load(f)
                self.default_api_id = settings.get('api_id', '')
                self.default_api_hash = settings.get('api_hash', '')
                self.default_app_name = settings.get('app_short_name', '')
                self.default_phone = settings.get('phone_number', '')
        except Exception as e:
            self.log_message(f"Warning: Could not load settings.yml: {str(e)}")
            self.default_api_id = ''
            self.default_api_hash = ''
            self.default_app_name = ''
            self.default_phone = ''

    def open_csv_file(self):
        """Open the CSV file with default application"""
        if self.csv_file_path and os.path.exists(self.csv_file_path):
            if sys.platform == 'win32':
                os.startfile(self.csv_file_path)
            elif sys.platform == 'darwin':  # macOS
                subprocess.run(['open', self.csv_file_path])
            else:  # Linux
                subprocess.run(['xdg-open', self.csv_file_path])
        else:
            messagebox.showerror("Error", "No CSV file available")

    def save_groups(self):
        """Open dialog to save group names to CSV"""
        dialog = GroupsDialog(self.root)
        if dialog.result:
            try:
                groups = dialog.result.split('\n')
                groups = [g.strip() for g in groups if g.strip()]
                
                with open('./data/groups.csv', 'w', newline='', encoding='utf-8') as f:
                    writer = csv.writer(f)
                    writer.writerow(['group_name'])
                    for group in groups:
                        writer.writerow([group])
                
                self.log_message(f"Saved {len(groups)} groups to groups.csv")
                messagebox.showinfo("Success", f"Saved {len(groups)} groups")
            except Exception as e:
                messagebox.showerror("Error", f"Failed to save groups: {str(e)}")

    async def fetch_from_csv_async(self):
        """Fetch members from all groups in groups.csv"""
        try:
            if not self.bots:
                raise ValueError("Please login first")
            
            if not os.path.exists('./data/groups.csv'):
                raise ValueError("No groups.csv file found. Please save groups first.")
            
            # Read groups from CSV
            groups = []
            with open('./data/groups.csv', 'r', encoding='utf-8') as f:
                reader = csv.DictReader(f)
                groups = [row['group_name'] for row in reader]
            
            if not groups:
                raise ValueError("No groups found in groups.csv")
            
            # Use first account to fetch
            bot = self.bots[0]
            all_members = []
            
            for group in groups:
                self.log_message(f"Fetching members from {group}...")
                members = await get_group_members(bot.client, group)
                if members:
                    all_members.extend(members)
                    self.log_message(f"Found {len(members)} members in {group}")
            
            if all_members:
                self.scraped_members = all_members
                self.csv_file_path = self.save_members_to_csv(all_members)
                self.log_message(f"Total members fetched: {len(all_members)}")
                self.open_csv_button.config(state='normal')
                messagebox.showinfo("Success", f"Fetched {len(all_members)} members from {len(groups)} groups")
            else:
                raise ValueError("No members found in any group")
            
        except Exception as e:
            messagebox.showerror("Error", str(e))

    def fetch_from_csv(self):
        """Wrapper for fetch_from_csv_async"""
        self.loop.run_until_complete(self.fetch_from_csv_async())

    async def send_to_group_async(self):
        """Send message to groups directly"""
        try:
            if not self.bots:
                raise ValueError("Please login first")
            
            if not os.path.exists('./data/groups.csv'):
                raise ValueError("No groups.csv file found. Please save groups first.")
            
            # Get messages from all tabs
            messages = []
            for text_widget in self.message_texts:
                msg = text_widget.get("1.0", tk.END).strip()
                if msg:  # Only add non-empty messages
                    messages.append(msg)
            
            if not messages:
                raise ValueError("Please enter at least one message")

            # Read groups from CSV
            groups = []
            with open('./data/groups.csv', 'r', encoding='utf-8') as f:
                reader = csv.DictReader(f)
                groups = [row['group_name'] for row in reader]
            
            if not groups:
                raise ValueError("No groups found in groups.csv")
            
            # Confirm operation
            if not messagebox.askyesno(
                "Confirm", 
                f"Send messages to {len(groups)} groups?"
            ):
                return
            
            # Use first account to send
            bot = self.bots[0]
            
            for group in groups:
                try:
                    self.log_message(f"Sending message to {group}...")
                    
                    # Join group if not already joined
                    try:
                        entity = await bot.client.get_entity(group)
                        await bot.client(functions.channels.JoinChannelRequest(channel=entity))
                        await asyncio.sleep(2)  # Small delay after joining
                    except Exception as e:
                        self.log_message(f"Error joining {group}: {str(e)}")
                        continue
                    
                    # Send random message from available messages
                    message = random.choice(messages)
                    
                    # Try sending with media first if attached
                    if self.media_path:
                        try:
                            await bot.client.send_message(entity, message, file=self.media_path)
                            self.log_message(f"Successfully sent message with media to {group}")
                        except Exception as media_error:
                            # If media sending fails, try sending text only
                            self.log_message(f"Failed to send media to {group}, trying text only: {str(media_error)}")
                            try:
                                await bot.client.send_message(entity, message)
                                self.log_message(f"Successfully sent text-only message to {group}")
                            except Exception as text_error:
                                self.log_message(f"Failed to send text message to {group}: {str(text_error)}")
                                continue
                    else:
                        # Text only message
                        await bot.client.send_message(entity, message)
                        self.log_message(f"Successfully sent message to {group}")

                    # Random delay between sends (20-25 minutes)
                    delay = random.randint(3600, 3700)
                    self.log_message(f"Waiting {delay} seconds before next group...")
                    await asyncio.sleep(delay)

                except Exception as e:
                    self.log_message(f"Error processing {group}: {str(e)}")
                    continue

            messagebox.showinfo("Success", "Completed sending messages to groups")

        except Exception as e:
            messagebox.showerror("Error", str(e))

    def save_valid_group(self, group_name):
        """Save successful group to CSV"""
        try:
            filepath = os.path.join(self.data_dir, "validgroupname.csv")
            
            # Create file with header if it doesn't exist
            if not os.path.exists(filepath):
                with open(filepath, 'w', newline='', encoding='utf-8') as f:
                    writer = csv.writer(f)
                    writer.writerow(['group_name'])
            
            # Check if group already exists
            existing_groups = set()
            with open(filepath, 'r', encoding='utf-8') as f:
                reader = csv.DictReader(f)
                existing_groups = {row['group_name'] for row in reader}
            
            # Only append if group not already saved
            if group_name not in existing_groups:
                with open(filepath, 'a', newline='', encoding='utf-8') as f:
                    writer = csv.writer(f)
                    writer.writerow([group_name])
            
        except Exception as e:
            self.log_message(f"Error saving valid group: {str(e)}")

    async def send_group_message(self, bot, group, message):
        """Enhanced group message sending with better rate limit handling"""
        try:
            if not bot.client:
                return False

            try:
                entity = await bot.client.get_entity(group)
                
                # Try to send message
                await bot.client.send_message(
                    entity,
                    message,
                    file=self.media_path if hasattr(self, 'media_path') else None
                )
                
                return True

            except FloodWaitError as e:
                self.log_message(f"Rate limit hit for {bot.phone} - waiting {e.seconds} seconds")
                await asyncio.sleep(e.seconds)
                return False
                
            except ChatWriteForbiddenError:
                self.log_message(f"No permission to write in {group}")
                return False
                
            except Exception as e:
                self.log_message(f"Error sending to {group}: {str(e)}")
                return False

        except Exception as e:
            self.log_message(f"Error in send_group_message: {str(e)}")
            return False

    def is_valid_group_name(self, group_name):
        """Validate group name format"""
        try:
            # Remove @ if present
            group_name = group_name.strip().lstrip('@')
            
            # Extract username from URL if it's a link
            if 't.me/' in group_name:
                group_name = group_name.split('t.me/')[-1].split('/')[0]
            
            # Basic validation
            if not re.match(r'^[a-zA-Z]\w{3,}$', group_name):
                self.log_message(f"Invalid group name format: {group_name}")
                return None
            
            return group_name
        except Exception as e:
            self.log_message(f"Error cleaning group name {group_name}: {str(e)}")
            return None

    def distribute_groups_evenly(self, groups, num_bots):
        """Distribute groups evenly among bots"""
        chunks = []
        chunk_size = len(groups) // num_bots
        remainder = len(groups) % num_bots
        
        start = 0
        for i in range(num_bots):
            # Add one extra item to some chunks if there's a remainder
            end = start + chunk_size + (1 if i < remainder else 0)
            chunks.append(groups[start:end])
            start = end
        
        return chunks

    def save_valid_groups(self, valid_groups):
        """Save valid group names to CSV"""
        try:
            with open('./data/validgroupname.csv', 'w', newline='', encoding='utf-8') as f:
                writer = csv.writer(f)
                writer.writerow(['group_name'])
                for group in valid_groups:
                    writer.writerow([group])
        except Exception as e:
            self.log_message(f"Error saving valid groups: {str(e)}")

    async def process_group_message(self, bot, group, message, assignment, valid_groups):
        """Process sending message to a single group"""
        try:
            # Increment attempt counter
            assignment['attempts'][group] = assignment['attempts'].get(group, 0) + 1
            
            # Random delay between 18-22 minutes
            delay = random.randint(1080, 1320)
            
            # Try to join group first
            joined = await self.join_group_safely(bot, group)
            if joined:
                # Try to send message
                success = await self.send_group_message(bot, group, message)
                if success:
                    assignment['progress'][group] = datetime.now()
                    valid_groups.add(group)
                    self.log_message_to_db(
                        bot, 
                        group,
                        message,
                        "success"
                    )
                
            # Wait for delay regardless of success
            await asyncio.sleep(delay)
            
        except Exception as e:
            self.log_error(bot, group, str(e))

    async def send_to_saved_members_async(self):
        """Send messages to saved members with strict rate limiting"""
        try:
            if not self.bots:
                raise ValueError("Please login first")
            
            message = self.message_text.get("1.0", tk.END).strip()
            if not message:
                raise ValueError("Please enter a message")

            # Read members from CSV
            if not os.path.exists('./data/members.csv'):
                raise ValueError("members.csv not found")
            
            members = []
            with open('./data/members.csv', 'r', encoding='utf-8') as f:
                reader = csv.DictReader(f)
                members = [row for row in reader if row['send_status'].upper() == 'PENDING']

            if not members:
                raise ValueError("No pending members to message")

            # Distribute members among bots
            assignments = []
            for i, member in enumerate(members):
                bot_index = i % len(self.bots)
                assignments.append({
                    'bot': self.bots[bot_index],
                    'member': member
                })

            # Start sending messages
            self.is_sending = True
            for assignment in assignments:
                if not self.is_sending:
                    break
                
                try:
                    # Send message without retries
                    success = await self.send_dm_safely(
                        assignment['bot'],
                        assignment['member'],
                        message
                    )
                    
                    # Update status regardless of success
                    new_status = 'Sent' if success else 'Failed'
                    self.update_member_status(assignment['member']['id'], new_status)
                    
                    # Log the attempt
                    status = "success" if success else "failed"
                    self.log_message_to_db(
                        assignment['bot'],
                        assignment['member']['id'],
                        message,
                        status
                    )
                    
                    # Wait 1 hour between messages regardless of success
                    await asyncio.sleep(3600)  # 1 hour delay
                    
                except Exception as e:
                    self.log_error(assignment['bot'], assignment['member']['id'], str(e))
                    # Still wait 1 hour even if there was an error
                    await asyncio.sleep(3600)
                    continue

        except Exception as e:
            messagebox.showerror("Error", str(e))
            self.is_sending = False

    async def send_dm_safely(self, bot, member, message):
        """Send DM with strict rate limiting and no retries"""
        try:
            if not bot.client:
                return False

            user_id = int(member['id'])
            username = member.get('username', 'Not defined')
            
            self.log_message(f"Attempting to send message to {username} using {bot.phone}")
            
            try:
                # Try to get user entity (only one attempt)
                try:
                    if username and username != "Not defined":
                        user = await bot.client.get_entity(username)
                    else:
                        user = await bot.client.get_entity(user_id)
                except ValueError:
                    self.log_message(f"Could not find user {username}")
                    return False
                
                # Send message (only one attempt)
                await bot.client.send_message(
                    user,
                    message,
                    file=self.media_path if self.media_path else None
                )
                
                self.log_message(f"Successfully sent message to {username}")
                return True
                
            except FloodWaitError as e:
                self.log_message(f"Rate limit hit for {bot.phone}, waiting {e.seconds} seconds")
                await asyncio.sleep(e.seconds)
                return False
                
            except Exception as e:
                self.log_message(f"Error sending message to {username}: {str(e)}")
                return False
                
        except Exception as e:
            self.log_message(f"Error in send_dm_safely: {str(e)}")
            return False

    def send_to_saved_members(self):
        """Wrapper for send_to_saved_members_async"""
        self.loop.run_until_complete(self.send_to_saved_members_async())

    def cancel_sending(self):
        """Cancel ongoing message sending"""
        self.is_sending = False
        self.log_message("Cancelling message sending...")
        self.cancel_send_button.config(state='disabled')
        self.send_saved_button.config(state='normal')
        
        # Close analytics window
        if self.analytics_window:
            self.analytics_window.destroy()
            self.analytics_window = None

    def handle_text_input(self, event):
        """Handle text input including emoji"""
        try:
            # Allow the event to process normally
            return True
        except Exception as e:
            self.log_message(f"Text input error: {str(e)}")
            return True  # Still allow the input

    def handle_paste(self, event):
        """Handle paste events including emoji"""
        try:
            # Get the widget that triggered the event
            widget = event.widget
            clipboard = self.root.clipboard_get()
            widget.insert(tk.INSERT, clipboard)
            return 'break'  # Prevent default paste behavior
        except Exception as e:
            self.log_message(f"Paste error: {str(e)}")
            return True  # Fall back to default paste behavior

    async def join_group_safely(self, bot, group_name):
        """Safely join a group with proper error handling and delay"""
        try:
            # Check if already joined
            try:
                entity = await bot.client.get_entity(group_name)
                if hasattr(entity, 'username'):
                    return True
            except Exception:
                pass

            # Attempt to join
            await bot.client(functions.channels.JoinChannelRequest(channel=group_name))
            
            # Log success
            self.db.log_bot_activity({
                'bot_id': bot.bot_id,
                'activity_type': 'group_join',
                'group_name': group_name,
                'status': 'success',
                'timestamp': datetime.now(timezone.utc)
            })
            
            # Standard delay after successful join
            await asyncio.sleep(self.group_join_delay)
            return True
            
        except FloodWaitError as e:
            wait_time = e.seconds
            self.log_message(f"Rate limit hit for {bot.phone} - waiting {wait_time} seconds")
            # Store the wait time for this bot
            bot.next_action_time = datetime.now(timezone.utc) + timedelta(seconds=wait_time)
            # Skip to next bot instead of waiting
            return False
            
        except Exception as e:
            self.log_message(f"Error joining {group_name}: {str(e)}")
            return False

    async def distribute_groups(self):
        """Distribute groups evenly among bots"""
        try:
            if not os.path.exists('./data/groups.csv'):
                raise ValueError("No groups.csv file found")

            # Clear existing assignments
            for bot in self.bots:
                bot.clear_assignments()

            # Read groups
            groups = []
            with open('./data/groups.csv', 'r', encoding='utf-8') as f:
                reader = csv.DictReader(f)
                groups = [row['group_name'] for row in reader]

            if not groups:
                raise ValueError("No groups found in groups.csv")

            # Distribute groups evenly among bots
            for i, group in enumerate(groups):
                bot_index = i % len(self.bots)
                self.bots[bot_index].assign_group(group)

            return True

        except Exception as e:
            self.log_message(f"Error distributing groups: {str(e)}")
            return False

    async def check_and_send_reservation_messages(self):
        """Check and send any pending reservation messages"""
        try:
            current_time = datetime.now()
            
            for msg in self.reserved_messages:
                if current_time >= msg['start_date'] and \
                   self.messages_sent >= msg['count_limit']:
                    
                    # Send reservation message through all available bots
                    tasks = []
                    for bot in self.bots:
                        if bot.is_connected:
                            tasks.append(self.send_reservation_message(bot, msg['message']))
                    
                    if tasks:
                        await asyncio.gather(*tasks)
                        
                    # Remove sent reservation message
                    self.reserved_messages.remove(msg)
                    self.save_reserved_messages()
                    
        except Exception as e:
            self.log_message(f"Error handling reservation messages: {str(e)}")

    async def send_reservation_message(self, bot, message):
        """Send a reservation message with proper error handling"""
        try:
            # Add random delay between 1-5 minutes
            delay = random.randint(3000, 3600)
            await asyncio.sleep(delay)
            
            # Send message
            if hasattr(self, 'current_entity'):
                await bot.client.send_message(
                    self.current_entity,
                    message,
                    file=self.media_path if hasattr(self, 'media_path') else None
                )
                
                # Log successful reservation message
                self.log_message_to_db(bot, "reservation", message, "success")
                
        except Exception as e:
            self.log_message(f"Error sending reservation message: {str(e)}")
            self.log_message_to_db(bot, "reservation", message, "failed")

    def send_to_group(self):
        """Wrapper for send_to_group_async"""
        self.loop.run_until_complete(self.send_to_group_async())

    def clean_group_name(self, group):
        """Clean and validate group name"""
        try:
            # Remove @ symbol if present
            group = group.strip().lstrip('@')
            
            # Extract username from URL if it's a link
            if 't.me/' in group:
                group = group.split('t.me/')[-1].split('/')[0]
            
            # Basic validation
            if not re.match(r'^[a-zA-Z]\w{3,}$', group):
                self.log_message(f"Invalid group name format: {group}")
                return None
            
            return group
        except Exception as e:
            self.log_message(f"Error cleaning group name {group}: {str(e)}")
            return None

    async def load_groups(self):
        """Load and validate groups from CSV"""
        valid_groups = []
        try:
            if not os.path.exists('./data/groups.csv'):
                raise ValueError("groups.csv not found")
            
            with open('./data/groups.csv', 'r', encoding='utf-8') as f:
                reader = csv.DictReader(f)
                for row in reader:
                    group_name = self.clean_group_name(row['group_name'])
                    if group_name:
                        valid_groups.append(group_name)
                    
            # Write valid groups back to CSV
            if valid_groups:
                with open('./data/groups.csv', 'w', newline='', encoding='utf-8') as f:
                    writer = csv.writer(f)
                    writer.writerow(['group_name'])
                    for group in valid_groups:
                        writer.writerow([group])
                    
            return valid_groups
        except Exception as e:
            self.log_message(f"Error loading groups: {str(e)}")
            return []

    def log_error(self, bot, group, error_msg):
        """Log error message to GUI and database"""
        try:
            # Log to GUI
            error_text = f"Error for {bot.phone} in {group}: {error_msg}"
            self.log_message(error_text)
            
            # Log to database if available
            try:
                if hasattr(self, 'db'):
                    self.db.log_bot_activity({
                        'bot_id': bot.phone,
                        'activity_type': 'error',
                        'group_name': group,
                        'status': 'failed',
                        'error': error_msg,
                        'timestamp': datetime.now(timezone.utc)
                    })
            except Exception as db_err:
                self.log_message(f"Database logging error: {str(db_err)}")
                
        except Exception as e:
            print(f"Logging error: {str(e)}")

    def log_message_to_db(self, bot, group, message, status):
        """Log message activity to database"""
        try:
            if hasattr(self, 'db'):
                self.db.log_message({
                    'bot_id': bot.phone,
                    'group_id': group,
                    'message_text': message,
                    'status': status,
                    'timestamp': datetime.now(timezone.utc)
                })
        except Exception as e:
            self.log_message(f"Database logging error: {str(e)}")

    def view_analytics(self):
        """Placeholder for analytics view"""
        messagebox.showinfo("Info", "Analytics feature has been removed")

    def update_button_states(self):
        """Update button states based on login status"""
        if self.is_logged_in:
            self.login_button.grid_remove()
            self.load_session_button.grid_remove()
            
            # Show session management buttons
            for i, button in enumerate(self.session_buttons):
                button.grid(row=0, column=i, padx=2)
        else:
            # Show login and load session buttons
            self.login_button.grid(row=0, column=0, padx=2)
            self.load_session_button.grid(row=0, column=1, padx=2)
            
            # Hide session management buttons
            for button in self.session_buttons:
                button.grid_remove()

    def show_scraping_dialog(self):
        """Show dialog for group scraping"""
        if not self.bots:
            messagebox.showerror("Error", "Please login first")
            return
        
        self.log_message("Opening scraping dialog...")
        dialog = ScrapingDialog(self.root)
        
        # Store dialog result before it's destroyed
        result = dialog.result
        
        if result:
            self.log_message(f"Dialog result: {result}")
            self.log_message(f"Source: {result['source']}")
            if result['source'] == 'input':
                self.log_message(f"Group URL: {result['group_url']}")
            self.log_message(f"Scrape type: {result['scrape_type']}")
            
            # Start scraping with the obtained result
            self.start_scraping(result)
        else:
            self.log_message("Scraping cancelled by user")

    def start_scraping(self, options):
        """Start scraping based on dialog options"""
        self.log_message("Starting start_scraping function... function called")
        try:
            self.log_message("Starting scraping process...")
            
            if not self.bots:
                raise ValueError("Please login first")
            
            if options['source'] == 'csv':
                csv_path = os.path.join('./data', 'groups.csv')
                self.log_message(f"Reading groups from {csv_path}")
                
                if not os.path.exists(csv_path):
                    raise FileNotFoundError(f"Groups CSV file not found: {csv_path}")
                
                with open(csv_path, 'r', encoding='utf-8') as f:
                    reader = csv.DictReader(f)
                    groups = [row['group_name'] for row in reader if row['group_name'].strip()]
                    
                self.log_message(f"Found {len(groups)} groups in CSV")
            else:
                group_url = options['group_url']
                if not group_url:
                    raise ValueError("Please enter a group URL")
                groups = [group_url]
                self.log_message(f"Using input group: {groups[0]}")
            
            # Process each group
            total_members = []
            for group in groups:
                self.log_message(f"\nProcessing group: {group}")
                try:
                    members = self.loop.run_until_complete(
                        self.scrape_group(group, options['scrape_type'])
                    )
                    if members:
                        total_members.extend(members)
                        self.log_message(f"Successfully added {len(members)} members from {group}")
                    else:
                        self.log_message(f"No members found in {group}")
                except Exception as e:
                    self.log_message(f"Error processing group {group}: {str(e)}")
            
            # Save all members to CSV
            if total_members:
                output_path = os.path.join('./data', 'members.csv')
                self.save_members_to_csv(total_members, output_path)
                self.log_message(f"\nTotal members scraped: {len(total_members)}")
                messagebox.showinfo("Success", f"Scraped {len(total_members)} members total")
            else:
                self.log_message("\nNo members were scraped")
                messagebox.showwarning("Warning", "No members were found to save")
            
        except Exception as e:
            error_msg = f"Error during scraping: {str(e)}"
            self.log_message(error_msg)
            messagebox.showerror("Error", error_msg)

    async def scrape_group(self, group_url, scrape_type):
        """Scrape members from a group"""
        self.log_message("Starting scrape_group function...")
        try:
            if not self.bots:
                raise ValueError("No bot available")
            
            bot = self.bots[0]  # Use first available bot
            
            # Clean up group URL
            group = group_url.split('/')[-1].replace('@', '')
            self.log_message(f"\nStarting to scrape group: {group}")
            
            # Get group entity
            self.log_message("Attempting to get group entity...")
            entity = await bot.client.get_entity(group)
            self.log_message(f"Successfully found group: {entity.title}")
            
            # Get admin list if needed
            admins = set()
            if scrape_type == 'admin':
                self.log_message("Getting admin list...")
                admin_participants = await bot.client.get_participants(
                    entity, 
                    filter=ChannelParticipantsAdmins
                )
                admins = {admin.id for admin in admin_participants}
                self.log_message(f"Found {len(admins)} admins")
            
            # Start scraping members
            self.log_message("Starting to scrape members...")
            members = []
            
            try:
                # Use get_participants instead of GetParticipantsRequest
                participants = await bot.client.get_participants(entity)
                
                self.log_message(f"Processing {len(participants)} members...")
                batch_members = 0
                batch_bots = 0
                
                for user in participants:
                    # Skip bots
                    if user.bot or (user.username and user.username.endswith('_bot')):
                        batch_bots += 1
                        continue
                    
                    # Get last seen status
                    last_seen = "Never"
                    if hasattr(user.status, 'was_online'):
                        last_seen = user.status.was_online.strftime("%Y-%m-%d %H:%M:%S")
                    elif hasattr(user, 'status'):
                        last_seen = str(user.status)
                    
                    # Only add if admin filter matches
                    if scrape_type == 'admin' and user.id not in admins:
                        continue
                    
                    member = {
                        'username': user.username or "Not defined",
                        'id': user.id,
                        'first_name': user.first_name or "Not defined",
                        'last_name': user.last_name or "Not defined",
                        'phone': getattr(user, 'phone', "Not defined"),
                        'admin': user.id in admins,
                        'last_seen': last_seen,
                        'lang_code': getattr(user, 'lang_code', "Not defined"),
                        'group_name': group,
                        'scrape_date': datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
                        'send_status': 'Pending'
                    }
                    
                    members.append(member)
                    batch_members += 1
                
                self.log_message(f"Batch results: {batch_members} members added, {batch_bots} bots skipped")
                
            except FloodWaitError as e:
                wait_time = e.seconds
                self.log_message(f"Rate limit hit, waiting {wait_time} seconds")
                await asyncio.sleep(wait_time)
            except Exception as e:
                self.log_message(f"Error during batch scraping: {str(e)}")
                await asyncio.sleep(5)  # Wait before retrying
            
            self.log_message(f"\nScraping completed for {group}")
            self.log_message(f"Total members scraped: {len(members)}")
            return members
            
        except Exception as e:
            error_msg = f"Error scraping group {group_url}: {str(e)}"
            self.log_message(error_msg)
            return []

    def save_members_to_csv(self, members, filepath):
        """Save members to CSV file"""
        try:
            # Ensure data directory exists
            os.makedirs(os.path.dirname(filepath), exist_ok=True)
            
            # Add timestamp to filename
            filename = os.path.splitext(filepath)[0]
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filepath = f"{filename}_{timestamp}.csv"
            
            self.log_message(f"Saving members to {filepath}")
            
            with open(filepath, 'w', newline='', encoding='utf-8') as f:
                if members:
                    writer = csv.DictWriter(f, fieldnames=members[0].keys())
                    writer.writeheader()
                    writer.writerows(members)
                    self.log_message(f"Successfully saved {len(members)} members")
                else:
                    self.log_message("No members to save")
                
        except Exception as e:
            self.log_message(f"Error saving members to CSV: {str(e)}")
            raise

class RateLimiter:
    def __init__(self):
        self.delays = {}
        self.last_request_time = {}
        self.base_delay = 3600  # Base delay of 1 hour
        self.max_retries = 1
        self.retry_count = {}
        self.error_counts = {}  # Track number of 429 errors
        
    def add_delay(self, operation, seconds):
        self.delays[operation] = max(seconds, self.base_delay)
        
    def can_proceed(self, operation):
        current_time = datetime.now()
        
        if operation not in self.last_request_time:
            self.last_request_time[operation] = current_time
            self.retry_count[operation] = 0
            return True
            
        elapsed = (current_time - self.last_request_time[operation]).total_seconds()
        required_delay = self.delays.get(operation, self.base_delay)
        
        # Exponential backoff if we've had errors
        if self.retry_count.get(operation, 0) > 0:
            required_delay *= (2 ** self.retry_count[operation])
        
        # Additional delay if we've seen too many 429s
        error_count = self.error_counts.get(operation, 0)
        if error_count > 3:
            required_delay *= 2
            
        return elapsed >= required_delay
        
    def handle_429(self, operation):
        """Handle rate limit error"""
        self.error_counts[operation] = self.error_counts.get(operation, 0) + 1
        self.retry_count[operation] = self.retry_count.get(operation, 0) + 1
        
        # Double the delay for this operation
        if operation in self.delays:
            self.delays[operation] *= 2
            
    def reset_counters(self, operation):
        """Reset counters after successful operation"""
        self.retry_count[operation] = 0
        self.error_counts[operation] = 0

def launch_gui():
    root = tk.Tk()
    app = ShillBotGUI(root)
    root.mainloop()

if __name__ == "__main__":
    launch_gui() 