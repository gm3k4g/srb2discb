# SRB2 Discord Bot written in Rust

## Prerequisites

- This guide assumes you are either on a Linux machine and using the console/terminal. It's not been tested on Windows/Mac, but it should work there as long as you're using a terminal.

## Guide

#### <a id="bot_setting"></a>Setting up the Discord Bot

* If you've already done this, you can skip down to [Linking SRB2 and the Discord Bot](#bot_linking).

1. Create a Discord bot. Sign in to the discord portal, create new Application. You can give a name to your application (e.g. SRB2D Bot. You cannot put 'Discord' in the name of bots. If you want your Bot to have a specific name, you can also go to the `Bot` tab, and change the `USERNAME` field to something of your preference. Just remember that even Bot names need to be unique, i.e. not taken by other users/bots.)
2. Go to the `Bot` tab on the left. At the `TOKEN` section, click on Copy, and paste the contents (token) somewhere safe, and remember where you stored it. Scroll down, and flip `MESSAGE CONTENT INTENT` on, and flip `PUBLIC BOT` off.

3. Click on the `OAuth2` tab, click on `URL Generator`. Tick the `bot` option on, and copy the URL. Paste the invite in your browser and invite the bot into your server.

4. You can set up specific channels for the SRB2 server such as chat, log or information. For now we'll keep it this way. Create a new category, and you can preferably name it after your srb2 server (e.g. '[24/7] CTF server' as the category's name). Use the + ssymbols next to the category's name to create the channels log, chat and infomration. We can set the permissions of all these to be private for now. 

5. Starting with the log channel, go to channel settings (hover the mouse on the channel and click the gear) and then go to Channel permissions. Scroll down to `Advanced permissions` and on `ROLES/MEMBERS`, click the + symbol and click on the Bot's name. This will hide the log channel for everyone except for the bot (and us, the server owners).

6. From the list, it's recommended to enable the following permissions for the Bot: `View Channel`, `Send Message`, `Embed Links`, `Attach Files`, `Add Reactions`, `Use External Emoji`, `Use External Stickers`, `Manage Messages`, `Read Message History`. Save changes and return.

#### <a id="bot_linking"></a>Linking SRB2 and the Discord Bot

(TODO)
