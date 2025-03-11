# stdlib

import asyncio
import functools
import random
import sys
import traceback
from datetime import datetime
from enum import Enum
from pathlib import Path
from tkinter import messagebox
import os
import csv

# custom
import asyncstdlib
import jsonschema
import yaml
from telethon import TelegramClient, functions
from telethon.errors.rpcerrorlist import (
    ChatWriteForbiddenError,
    FloodWaitError,
    SlowModeWaitError,
    MediaCaptionTooLongError,
)
from telethon.tl.functions.channels import GetParticipantsRequest
from telethon.tl.types import ChannelParticipantsSearch, ChannelParticipantsAdmins
from typing import List, Dict

VERSION = "v0.24"


class Style(Enum):
    RED = "\033[31m"
    GREEN = "\033[32m"
    YELLOW = "\033[33m"
    CYAN = "\033[36m"
    RESET = "\033[0m"


def log_color(color, message):
    now = datetime.now()
    message = (
        color
        + "["
        + now.strftime("%H:%M:%S.%f")[:-3]
        + "] "
        + message
        + Style.RESET.value
    )
    print(message)
    return message


def log_green(message):
    return log_color(Style.GREEN.value, message)


def log_yellow(message):
    return log_color(Style.YELLOW.value, message)


def log_red(message):
    return log_color(Style.RED.value, message)


def load_settings():
    """Load settings from settings.yml file"""
    try:
        with open('settings.yml', 'r', encoding='utf8') as f:
            return yaml.safe_load(f)
    except Exception as e:
        log_red(f"Error loading settings: {str(e)}")
        return {"random_message": [], "raid": {}}


def random_messages():
    settings = load_settings()
    return settings["random_message"]


def random_message():
    rms = random_messages()
    return rms[random.randrange(len(rms))]


def format_random_message(rm1, rm2):
    settings = load_settings()
    tmp_rmf = settings["random_message_format"]
    rmf = eval(tmp_rmf.strip())
    return rmf(rm1, rm2)


def header():
    surfranch = f"""{Style.CYAN.value}
┏━━━┓━━━━━━━━┏━┓┏━━━┓━━━━━━━━━━━━━┏┓━━
┃┏━┓┃━━━━━━━━┃┏┛┃┏━┓┃━━━━━━━━━━━━━┃┃━━
┃┗━━┓┏┓┏┓┏━┓┏┛┗┓┃┗━┛┃┏━━┓━┏━┓━┏━━┓┃┗━┓
┗━━┓┃┃┃┃┃┃┏┛┗┓┏┛┃┏┓┏┛┗━┓┃━┃┏┓┓┃┏━┛┃┏┓┃
┃┗━┛┃┃┗┛┃┃┃━━┃┃━┃┃┃┗┓┃┗┛┗┓┃┃┃┃┃┗━┓┃┃┃┃
┗━━━┛┗━━┛┗┛━━┗┛━┗┛┗━┛┗━━━┛┗┛┗┛┗━━┛┗┛┗┛
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
━━ {VERSION} ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

"""
    print(surfranch)
    return surfranch


def channels_to_raid():
    settings = load_settings()
    return settings["raid"].keys()


@functools.lru_cache()
def splay_map():
    count = 1
    result = {}
    for channel in channels_to_raid():
        result[channel] = count * splay()
        count += 1
    return result


@functools.lru_cache()
def channel_splay(channel):
    channel_splay_map = splay_map()
    return channel_splay_map[channel]


@asyncstdlib.lru_cache()
async def get_entity(channel):
    return await CLIENT.get_input_entity(channel)


def channel_to_raid(channel):
    settings = load_settings()
    return settings["raid"][channel]


def channel_message(channel):
    settings = load_settings()
    messages = settings["messages"]
    message_type = channel_to_raid(channel)["message_type"]
    if isinstance(message_type, str):
        message_type = [message_type]
    return list(map(lambda mt: messages[mt], message_type))


def channel_wait_interval(channel):
    return channel_to_raid(channel).get("wait_interval", None)


def channel_increase_wait_interval(channel):
    return channel_to_raid(channel).get("increase_wait_interval", None)


def channel_image(channel):
    return channel_to_raid(channel).get("image", None)


def channel_total_messages(channel):
    return channel_to_raid(channel).get("total_messages", 999999999999)


def channel_map(channel):
    return {
        "name": channel,
        "splay": channel_splay(channel),
        "wait_interval": channel_wait_interval(channel),
        "increase_wait_interval": channel_increase_wait_interval(channel),
        "message": channel_message(channel),
        "last_message": -1,
        "image": channel_image(channel),
        "total_messages": channel_total_messages(channel),
        "count": 0,
        "is_connected": False,
    }


def increment_count(channel):
    channel["count"] += 1
    return channel


async def handle_message_floodwaiterror(error, channel):
    log_red(
        f"FloodWaitError invoked while sending a message to {channel['name']};"
        + f" Forcing a {error.seconds} second wait interval for all channels"
    )
    open_floodwaiterror()
    await asyncio.sleep(error.seconds)
    close_floodwaiterror()


def handle_slowmodewaiterror(error, channel):
    log_red(
        f"SlowModeWaitError invoked while sending a message to {channel['name']};"
        + f" Dynamically updating the channel's calculated wait interval to {error.seconds + 10}"
    )
    channel["calculated_wait_interval"] = error.seconds + 10
    return channel


def handle_mediacaptiontoolongerror(channel):
    log_red(
        f"MediaCaptionTooLongError invoked while sending a message to {channel['name']};"
        + " Abandoning sending all future messages"
    )
    channel["loop"] = False
    return channel


async def handle_chatwriteforbiddenerror(channel):
    one_hour = 60 * 60
    log_red(
        f"ChatWriteForbiddenError invoked while sending a message to {channel['name']};"
        + f" Forcing a {one_hour} second wait interval for this channel"
    )
    await asyncio.sleep(one_hour)


def handle_unknownerror(error):
    message = "Unknown error invoked while running bot; Abandoning all execution"
    if hasattr(error, "message"):
        message = message + f"\n{error.message}"
    log_red(message)
    traceback.print_exc()


def handle_unknownmessagingerror(error, channel):
    message = (
        f"Unknown error invoked while sending a message to {channel['name']};"
        + " Abandoning sending all future messages"
    )
    if hasattr(error, "message"):
        message = message + f"\n{error.message}"
    log_red(message)
    traceback.print_exc()
    channel["loop"] = False
    return channel


def open_floodwaiterror():
    STATE.update({"floodwaiterror_exists": True})


def close_floodwaiterror():
    STATE.update({"floodwaiterror_exists": False})


def floodwaiterror_exists():
    return STATE.get("floodwaiterror_exists", False)


def image_exists(channel):
    result = False
    if channel["image"]:
        path = Path(channel["image"])
        if path.is_file():
            result = True
        else:
            log_yellow(
                f">> Unable to locate {channel['name']}'s configured image {channel['image']};"
                + " Sending message without image"
            )
    return result


def randomize_message(channel, rm1=None, rm2=None):
    if not rm1:
        rm1 = random_message()
    if not rm2:
        rm2 = random_message()
    rm = format_random_message(rm1, rm2)
    return channel["message"][channel["last_message"]] + "\n" + rm


def next_message(channel):
    proposed_message = channel["last_message"] + 1
    possible_messages = len(channel["message"]) - 1
    if proposed_message > possible_messages:
        use_message = 0
    else:
        use_message = proposed_message
    channel["last_message"] = use_message
    return [randomize_message(channel), channel]


async def dispatch_message(message, channel):
    entity = await get_entity(channel["name"])
    channel = increment_count(channel)
    log_green(f"Sending message to {channel['name']} (#{channel['count']})")
    if image_exists(channel):
        await CLIENT.send_message(entity, message, file=channel["image"])
    else:
        await CLIENT.send_message(entity, message)
    return channel


async def send_message(channel):
    try:
        channel = await dispatch_message(*next_message(channel))
    except FloodWaitError as fwe:
        await handle_message_floodwaiterror(fwe, channel)
    except ChatWriteForbiddenError:
        await handle_chatwriteforbiddenerror(channel)
    except SlowModeWaitError as smwe:
        channel = handle_slowmodewaiterror(smwe, channel)
    except MediaCaptionTooLongError:
        channel = handle_mediacaptiontoolongerror(channel)
    except Exception as e:
        channel = handle_unknownmessagingerror(e, channel)
    return channel


async def send_single_message(channel):
    log_green(f"Raiding {channel['name']} once")
    await send_message(channel)


def calculate_wait_interval(channel):
    calculated_wait_interval = channel["wait_interval"] + channel["splay"]
    channel["calculated_wait_interval"] = calculated_wait_interval
    return channel


def recalculate_wait_interval(channel):
    if channel["loop"] and channel["increase_wait_interval"]:
        channel["calculated_wait_interval"] += channel["increase_wait_interval"]
        log_yellow(
            f">> Recalculated {channel['name']} wait interval to"
            + f" {channel['calculated_wait_interval']} seconds"
        )
    return channel


def resolve_total_messages(channel):
    if channel["count"] >= channel["total_messages"]:
        channel["loop"] = False
        channel["calculated_wait_interval"] = 1
        log_yellow(">> Allowed total messages reached; Stopping message loop")
    return channel


async def message_loop(channel):
    while channel["loop"]:
        if floodwaiterror_exists():
            log_yellow(
                f">> Skipped sending message to {channel['name']} due to active FloodWaitError"
            )
        else:
            channel = await send_message(channel)
            channel = recalculate_wait_interval(channel)
            channel = resolve_total_messages(channel)
        await asyncio.sleep(channel["calculated_wait_interval"])


async def send_looped_message(channel):
    channel = calculate_wait_interval(channel)
    channel["loop"] = True
    log_green(
        f"Raiding {channel['name']} every {channel['calculated_wait_interval']} seconds"
    )
    await message_loop(channel)


def message_once(channel):
    return not bool(channel["wait_interval"])


async def raid(channel):
    await asyncio.sleep(channel["splay"])

    if message_once(channel):
        await send_single_message(channel)
    else:
        await send_looped_message(channel)


async def handle_connection_floodwaiterror(error, channel):
    log_red(
        f"FloodWaitError invoked while connecting to {channel['name']};"
        + f" Forcing a {error.seconds} second wait interval for all channels"
    )
    open_floodwaiterror()
    await asyncio.sleep(error.seconds)
    close_floodwaiterror()


def handle_connectionerror(error, channel):
    message = (
        "Unknown error invoked while connecting to a channel;"
        + f" Abandoning sending messages to {channel['name']}"
    )
    if hasattr(error, "message"):
        message = message + f"\n{error.message}"
    log_red(message)


async def sleep_while_floodwaiterror_exists(channel):
    while floodwaiterror_exists():
        log_yellow(
            f">> Delaying connecting to {channel['name']} due to active FloodWaitError"
        )
        await asyncio.sleep(channel["splay"])


async def dispatch_connection(channel):
    await asyncio.sleep(channel["splay"])
    await sleep_while_floodwaiterror_exists(channel)
    log_green(f"Connecting to {channel['name']}")
    await CLIENT(functions.channels.JoinChannelRequest(channel=channel["name"]))
    channel["is_connected"] = True
    return channel


async def connect(channel):
    try:
        channel = await dispatch_connection(channel)
    except FloodWaitError as fwe:
        await handle_connection_floodwaiterror(fwe, channel)
    except Exception as e:
        handle_connectionerror(e, channel)
    return channel


async def do_raid(channels):
    tasks = [raid(channel) for channel in channels]
    await asyncio.gather(*tasks)


async def do_connect():
    tasks = [connect(channel_map(channel)) for channel in channels_to_raid()]
    channels = await asyncio.gather(*tasks)
    connected_channels = filter(lambda channel: channel["is_connected"], channels)
    return connected_channels


async def stop():
    try:
        await CLIENT.log_out()
    except Exception:
        pass


async def start():
    await CLIENT.start(phone_number())
    await asyncio.sleep(10)

    print("")
    log_green(f"Configured splay: {splay()} seconds")
    log_green(
        "Splay will be added to connection and user defined wait intervals"
        + " to avoid Telegram rate limiting"
    )
    channels = await do_connect()
    await do_raid(channels)


def validate_random_message_settings(settings):
    schema = {
        "type": "array",
        "minItems": 1,
        "items": {
            "type": "string",
        },
    }
    jsonschema.validate(settings, schema)


def handle_start_floodwaiterror(error):
    message = (
        "FloodWaitError invoked while communicating with Telegram during start;"
        + " Nothing can be done at this time; Abandoning all execution"
    )
    if hasattr(error, "message"):
        message = message + f"\n{error.message}"
    log_red(message)
    traceback.print_exc()


def splay():
    settings = load_settings()
    return settings["splay"]


def main():
    try:
        LOOP.run_until_complete(start())
        LOOP.run_until_complete(stop())
    except KeyboardInterrupt:
        LOOP.run_until_complete(stop())
        sys.exit(0)
    except FloodWaitError as start_fwe:
        error_msg = handle_start_floodwaiterror(start_fwe)
        LOOP.run_until_complete(stop())
        if messagebox:
            messagebox.showerror("Error", error_msg)
    except Exception as start_error:
        error_msg = handle_unknownerror(start_error)
        LOOP.run_until_complete(stop())
        if messagebox:
            messagebox.showerror("Error", error_msg)


def raise_startup_exception():
    raise Exception(
        "!! WRONG VERSION OF PYTHON !! "
        + "READ THE INSTRUCTIONS :: "
        + "https://github.com/surfranch/TelegramShillBot/blob/main/README.md"
    )


async def get_group_members(client, group: str) -> List[Dict]:
    """Scrape members from a group with enhanced user info"""
    try:
        # Get group entity
        entity = await client.get_entity(group)
        members = []
        offset = 0
        limit = 100  # Reduced batch size to avoid rate limits
        
        # Get admin list first
        admins = set()
        try:
            admin_participants = await client.get_participants(entity, filter=ChannelParticipantsAdmins)
            admins = {admin.id for admin in admin_participants}
            log_green(f"Found {len(admins)} admins")
        except Exception as e:
            log_red(f"Error getting admins: {str(e)}")
        
        while True:
            try:
                log_green(f"Fetching members batch (offset: {offset})")
                participants = await client(GetParticipantsRequest(
                    channel=entity,
                    filter=ChannelParticipantsSearch(''),
                    offset=offset,
                    limit=limit,
                    hash=0
                ))
                
                if not participants.users:
                    break
                    
                for user in participants.users:
                    # Skip bots
                    if user.bot or (user.username and user.username.endswith('_bot')):
                        continue
                        
                    # Get last seen status
                    last_seen = "Never"
                    if hasattr(user.status, 'was_online'):
                        last_seen = user.status.was_online.strftime("%Y-%m-%d %H:%M:%S")
                    elif hasattr(user, 'status'):
                        last_seen = str(user.status)
                    
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
                    log_green(f"Scraped member: {member['username']} from {group}")
                
                offset += len(participants.users)
                await asyncio.sleep(2)  # 2 second delay between batches
                
                if len(participants.users) < limit:
                    break
                    
            except FloodWaitError as e:
                log_red(f"Rate limit hit, waiting {e.seconds} seconds")
                await asyncio.sleep(e.seconds)
                continue
            except Exception as e:
                log_red(f"Error during scraping batch: {str(e)}")
                await asyncio.sleep(5)  # Wait before retrying
                continue
                
        log_green(f"Successfully scraped {len(members)} members from {group}")
        return members
        
    except Exception as e:
        log_red(f"Error scraping members from {group}: {str(e)}")
        return []


async def send_dm(client, user_id: int, message: str, file: str = None) -> bool:
    """Send DM to a user"""
    try:
        if file:
            await client.send_message(user_id, message, file=file)
        else:
            await client.send_message(user_id, message)
        return True
    except FloodWaitError as e:
        log_red(f"FloodWaitError: Waiting {e.seconds} seconds")
        await asyncio.sleep(e.seconds)
        return False
    except Exception as e:
        log_red(f"Error sending DM: {str(e)}")
        return False


def ensure_session_dir():
    """Ensure session directory exists"""
    session_dir = os.path.join(os.path.dirname(__file__), 'session')
    if not os.path.exists(session_dir):
        os.makedirs(session_dir)
    return session_dir


class TelegramBot:
    def __init__(self):
        self.client = None
        self.is_connected = False
        self.api_id = None
        self.api_hash = None
        self.phone = None
        self._lock = asyncio.Lock()
        
        # Create sessions directory if it doesn't exist
        os.makedirs('./sessions', exist_ok=True)

    async def connect(self, api_id: int, api_hash: str, phone: str):
        """Connect to Telegram with improved error handling"""
        try:
            async with self._lock:  # Use lock to prevent concurrent connections
                if self.client and self.is_connected:
                    await self.disconnect()  # Ensure clean disconnect before new connection
                
                self.api_id = api_id
                self.api_hash = api_hash
                self.phone = phone
                
                # Store session file in sessions directory with better naming
                session_file = os.path.join('./sessions', f"{phone.replace('+', '')}")
                
                # Initialize client with connection retry settings
                self.client = TelegramClient(
                    session_file, 
                    api_id, 
                    api_hash,
                    connection_retries=3,
                    retry_delay=1,
                    auto_reconnect=True,
                    system_version="4.16.30-vxCUSTOM"  # Add system version
                )

                # Connect with timeout
                try:
                    await asyncio.wait_for(self.client.connect(), timeout=30)
                except asyncio.TimeoutError:
                    return False, "Connection timeout"

                # Check authorization
                if not await self.client.is_user_authorized():
                    try:
                        await self.client.send_code_request(phone)
                        self.is_connected = False
                        return False, "Code requested"
                    except FloodWaitError as e:
                        return False, f"Too many attempts. Please wait {e.seconds} seconds"
                
                self.is_connected = True
                return True, "Connected successfully"

        except Exception as e:
            self.is_connected = False
            error_msg = str(e)
            if "wrong session" in error_msg.lower():
                # Delete invalid session file and retry
                try:
                    session_file = os.path.join('./sessions', f"{phone.replace('+', '')}.session")
                    if os.path.exists(session_file):
                        os.remove(session_file)
                    return False, "Session file was invalid and has been reset. Please try connecting again."
                except Exception as del_err:
                    return False, f"Error cleaning up session: {str(del_err)}"
            return False, f"Connection error: {error_msg}"

    async def sign_in(self, phone: str, code: str):
        """Sign in with improved error handling"""
        try:
            async with self._lock:
                if not self.client:
                    return False, "Client not initialized"
                
                try:
                    await self.client.sign_in(phone, code)
                    self.is_connected = True
                    return True, "Signed in successfully"
                except FloodWaitError as e:
                    return False, f"Too many attempts. Please wait {e.seconds} seconds"
                except Exception as e:
                    if "phone code invalid" in str(e).lower():
                        return False, "Invalid code. Please try again."
                    raise
                    
        except Exception as e:
            self.is_connected = False
            return False, f"Sign in error: {str(e)}"

    async def disconnect(self):
        """Disconnect with proper cleanup"""
        try:
            async with self._lock:
                if self.client:
                    try:
                        if not self.client.is_connected():
                            return
                        await self.client.disconnect()
                    except Exception:
                        pass  # Ignore errors during disconnect
                    finally:
                        self.client = None
                        self.is_connected = False
        except Exception:
            pass  # Ensure disconnect doesn't raise errors

    async def send_group_message(self, bot, group, message):
        """Enhanced group message sending with better error handling"""
        try:
            if not bot.client:
                return False

            operation = f"send_message_{bot.phone}"
            
            # Check rate limiter
            if not self.rate_limiter.can_proceed(operation):
                wait_time = self.rate_limiter.delays.get(operation, 1200)  # 20 minutes default
                self.log_message(f"Rate limit: waiting {wait_time} seconds for {bot.phone}")
                await asyncio.sleep(wait_time)
                return False
            
            try:
                entity = await bot.client.get_entity(group)
                permissions = await bot.client.get_permissions(entity)
                
                if not permissions.send_messages:
                    self.log_message(f"No permission to send messages in {group}")
                    return False

                await bot.client.send_message(
                    entity,
                    message,
                    file=self.media_path if hasattr(self, 'media_path') else None
                )
                
                # Success - save valid group
                self.save_valid_group(group)
                self.log_message(f"Successfully sent message to {group}")
                return True

            except Exception as e:
                self.log_message(f"Error sending to {group}: {str(e)}")
                return False

        except Exception as e:
            self.log_message(f"Error in send_group_message: {str(e)}")
            return False

    def save_valid_group(self, group_name):
        """Save successful group to CSV"""
        try:
            filepath = "./data/validgroupname.csv"
            
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

    async def check_2fa(self, password):
        """Handle two-factor authentication"""
        try:
            async with self._lock:
                if not self.client:
                    return False, "Client not initialized"
                
                try:
                    await self.client.sign_in(password=password)
                    self.is_connected = True
                    return True, "Signed in successfully"
                except Exception as e:
                    return False, str(e)
                    
        except Exception as e:
            self.is_connected = False
            return False, f"2FA error: {str(e)}"

    async def sign_in(self, phone, code):
        """Sign in with code and handle 2FA if needed"""
        try:
            async with self._lock:
                if not self.client:
                    return False, "Client not initialized"
                
                try:
                    await self.client.sign_in(phone, code)
                    self.is_connected = True
                    return True, "Signed in successfully"
                except Exception as e:
                    if "TWO_STEPS_VERIFICATION_REQUIRED" in str(e):
                        return False, "Two-steps verification is enabled and a password is required"
                    elif "PHONE_CODE_INVALID" in str(e):
                        return False, "Invalid code. Please try again."
                    else:
                        raise e
                    
        except Exception as e:
            self.is_connected = False
            return False, f"Sign in error: {str(e)}"


# Move these to module level (outside if __name__ == "__main__":)
STATE = {}

if __name__ == "__main__":
    header()
    if sys.version_info.major == 3:
        if sys.version_info.minor >= 10:
            LOOP = asyncio.new_event_loop()
            asyncio.set_event_loop(LOOP)
        elif sys.version_info.minor >= 5:
            LOOP = asyncio.get_event_loop()
        else:
            raise_startup_exception()
    else:
        raise_startup_exception()
    
    # Check if running in GUI mode
    if not hasattr(sys, '_gui_mode'):
        main()
