# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog],
and this project adheres to [Semantic Versioning].

## [1.0.2] - 2023-09-19

### Fixed

- Fixed a bug where Langley wouldn't receive any requests because it only listened to GET requests instead of POST requests.
- Fixed missing commands (e.g. `/config`, `/user`)

## [1.0.1] - 2023-09-19

### Fixed

- Fixed a bug where the bot would try to get the last played song from the database and fail because it was empty.

## [1.0.0] - 2023-09-19

### Added

- Added slash commands
  - `/song request [song]` - Requests a song for the radio
    - To get an idea on what songs are available, type a song and see the completions.
    - This has a default cooldown of 1 1/2 hours.
    - Additionally, songs have individual cooldowns based on their length.
  - `/youtube link` - Links your YouTube channel with your Discord account.
    - This will try and migrate the bot data for your account to the new bot.
    - If it says that it couldn't find any data, please contact me.
  - `/version` - (by default, only for people with the `MANAGE_GUILD` permission) Shows the currently running bot version and the changelog.
  - `/boondollars` - Shows your boondollars and watched/chatted hours and the position in the leaderboards.
  - `/pay` - Pays a user the specified amount of boondollars.
  - `/minigames slots` - Uses the slot machine.
  - `/minigames rolldice` - Plays the dice game.
  - `/minigames strife` - Plays the strife minigame (basically PvE where you can gather money).
  - `/minigames pvp` - Fight another user.
  - `/add can` - Adds a can to Can Town
  - `/add bear` - Adds a bear...? to Can Town...?
  - `/add john` - no.
  - `/admin control_cmd [cmd]` - (admin only) Sends a command to the music server.
  - `/admin volume [0-100]` - (admin only) Sets the volume of the radio.
  - `/admin skip [type]` - (admin only) Skips the next song on either the radio, the song request queue or the admin queue.
  - `/admin queue [song]` - (admin only) Requests a song to be played next.
    - This has priority over regular song requests.
  - `/config manage_channel [channel] [allow point accumulation] [allow time accumulation]` - Configures a channel for point and watch time accumulation.
  - `/config set_can_count [amount]` - (admin only) A highly dangerous command! It creates cans in Can Town out of thin air, as if they have been transported from an alternative universe!
  - `/user get [property]` - (admin only) Gets a property of a user.
  - `/user set [property] [value]` - (admin only) Sets a property of a user.
- Added points and hours system
  - This should roughly follow the same rules as the old bot.
- Also added a grist system
  - Currently, you can get grist by doing `/strife`, although there is no way to use grist yet.
  - Perhaps there will be a way to craft weapons in the future that sway the fight in your favor.
- Added context menus (right click)
  - For users
    - Give this user money - Does the same as `/pay`.
    - PvP - Does the same as `/pvp`.
- Implemented communication with the music server (Liquidsoap) via Unix socket and telnet.
- Implemented an OAuth2 flow for Discord linking.
  - You will have to link your YouTube channel to your Discord account first for this to work!

## [0.1.0] - 2023-05-23

- initial release

<!-- Links -->
[keep a changelog]: https://keepachangelog.com/en/1.1.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html

<!-- Versions -->
[unreleased]: https://github.com/LumiRadio/lumiRadio/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/LumiRadio/lumiRadio/releases/tag/v0.1.0
