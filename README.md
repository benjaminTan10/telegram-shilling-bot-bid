perfect bot
contact me on telegram @blockchainDeveloper_Ben

# Telegram DM Bot - Project Overview

## Core Features

### 1. Multi-Account Management
- Support for multiple Telegram accounts
- Session management for persistent logins
- Easy account switching and management
- Distributed message sending across accounts to avoid rate limits

### 2. Member Scraping
- Scrape members from Telegram groups
- Filters out bots and users without usernames
- Collects detailed user information:
  - Username
  - User ID
  - First/Last Name
  - Phone (if available)
  - Account status (verified, premium, etc.)
- Saves data to CSV for future use

### 3. Message Distribution System

#### Direct Messaging (DM)
- **Individual Messages**: Each target receives one personalized message
- **Message Customization**:
  - SpinTax support for message variations
  - Media attachment support (images, videos, etc.)
- **Smart Delays**:
  - Configurable intervals between messages (e.g., 60-80 seconds)
  - Random delay generation to appear more natural
- **Distribution**:
  - Messages split among multiple accounts
  - Parallel sending for efficiency

#### Group Messaging
- Send messages directly to groups
- Automatic group joining if needed
- Configurable delays between group messages
- Support for media attachments

### 4. Advanced Message Features

#### SpinTax System
yaml
Example: "{Hello|Hi|Hey} {there|friend}!"
Generates variations like:
"Hello there!"
"Hi friend!"
"Hey there!"

- Creates unique message variations for each recipient
- Reduces spam detection
- Makes messages appear more natural

#### Media Support
- Images
- Videos
- Audio files
- Documents

### 5. Progress Tracking

#### CSV Management
- Tracks all message attempts
- Records success/failure status
- Maintains member database
- Allows for campaign resumption

#### Real-time Logging
- Live progress updates
- Error reporting
- Success confirmations

### 6. Safety Features

#### Rate Limiting
- Smart delays between messages
- Account rotation
- Flood control protection

#### Error Handling
- Automatic retry system
- Error logging
- Session persistence

## Technical Specifications

### Configuration
``
api_id: YOUR_API_ID
api_hash: "YOUR_API_HASH"
splay: 7 # Base delay
messages:
template_one: |
{Hello|Hi} {there|friend}!
raid:
CHANNEL_NAME:
message_type: template_one
wait_interval: 300
``

### File Structure
- `members.csv`: Scraped member database
- `groups.csv`: Target groups list
- `settings.yml`: Bot configuration
- `sessions/`: Telegram session files

## Usage Flow

1. **Setup**
   - Configure API credentials
   - Add Telegram accounts
   - Set message templates

2. **Member Collection**
   - Scrape from groups
   - Import from CSV
   - Filter and verify members

3. **Message Configuration**
   - Create message template
   - Set up SpinTax variations
   - Attach media (optional)
   - Configure sending intervals

4. **Distribution**
   - Select target members
   - Choose sending method
   - Start distribution
   - Monitor progress

5. **Tracking**
   - View live logs
   - Check CSV for status
   - Handle any errors
   - Resume incomplete campaigns

## Safety Guidelines

1. **Account Protection**
   - Use reasonable delays
   - Rotate between accounts
   - Monitor for warnings

2. **Best Practices**
   - Start with small batches
   - Use message variations
   - Maintain natural sending patterns
   - Regular session management

## Limitations

- Telegram API restrictions
- Rate limiting considerations
- Account safety measures
- Platform-specific constraints

## Support

- Error logging system
- CSV export for troubleshooting
- Session management tools


# Telegram DM Bot

A Telegram bot for sending direct messages.

## Prerequisites

- Python 3.8 or higher
- Virtual environment (recommended)

## Required Files

The following files must be included for the bot to work:

- `launch.py` - Main entry point
- `tg_shill_bot.py` - Core bot functionality 
- `spintax.py` - Message templating
- `settings.yml` - Bot configuration
- `requirements.txt` - Python dependencies

## Setup

1. Clone this repository with all required files
2. Create and activate a virtual environment:

   ```bash
   # Windows
   python -m venv .venv
   .venv\Scripts\activate

   # Linux/Mac
   python -m venv .venv
   source .venv/bin/activate
   ```

3. Install required packages:
   ```bash
   pip install -r requirements.txt
   ```

## Running the Bot

1. Configure your settings in `settings.yml`
2. Run the bot:
   ```bash
   python launch.py
   ```

## Building Executable

To create a single-file executable with no console window:

1. Install PyInstaller:
   ```bash
   pip install pyinstaller
   ```

2. Build the executable:
   ```bash
   # Windows
   pyinstaller telegram_dm_bot.spec --clean --noconfirm

   # Linux/Mac 
   pyinstaller telegram_dm_bot.spec --clean --noconfirm
   ```

The executable will be created in the `dist` folder. You can distribute this single file to run the bot on any compatible system without requiring Python installation.

Note: Make sure all required files (`launch.py`, `tg_shill_bot.py`, `spintax.py`, `settings.yml`) are included in the same directory as the executable.


 pyinstaller telegram_dm_bot.spec --clean


