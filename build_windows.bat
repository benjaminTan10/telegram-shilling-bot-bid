@echo off
echo Building Telegram DM Bot...

REM Activate virtual environment
call .venv\Scripts\activate

REM Install required packages
pip install pyinstaller
pip install requests fqdn rfc3987 rfc3986-validator rfc3339-validator webcolors jsonpointer uri-template isoduration importlib_resources
pip install jsonschema[format]

REM Create Windows executable
pyinstaller telegram_dm_bot.spec --clean --noconfirm

echo Build complete!
pause