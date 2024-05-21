# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog],
and this project adheres to [Semantic Versioning].

## [1.3.0] - 2024-05-03

### Added

- Implemented Sentry into Byers to catch errors
- Added `/song favourite` to favourite the currently playing song
- Added `/song unfavourite` to unfavourite the specified song
- Added `/song request_favourite` to search and request one of your favourited songs

### Changed

- Migrated from sqlx to SeaORM

### Fixed

- Fixed the slot machine showing "you lost" even though there was a payout
- Fixed Byers not actually giving people its lunch money when it loses
- Fixed the slot machine incorrectly ignoring proper payout of Bar symbols

## [1.2.1] - 2024-05-03

### Fixed

- Fixed the slot machine not working
- Fixed the slot machine description having the wrong indents

## [1.2.0] - 2024-05-03

### Added

- Added a 15 minute hydration reminder
- Added `/minigames slots_info` which tells you information about the slot machine like the payout table.
- Added `/config manage_role` for automatically assigning roles based on interaction time
- Added `/config delete_role_config` for removing rules for automatically assigning roles.
  - This does NOT delete the role!
- Added `/admin song_info` for getting information about a song.
- Added `/admin generate_playlist` to regenerate the playlist for Liquidsoap.
  - Liquidsoap will automatically reload the playlist once it's changed
- Added changelog comparison links to the changelog
- Added functionality for frohike to drop indices on moved and deleted directories
- Added bitrate to song metadata
- Added tags to song metadata
- Added pre and post messages to `/pvp`
- Added back the hydration reminder

### Changed

- Refactored the codebase for Byers
- Reworked slots minigame to use "Easy Vegas" slot machine as base
- Made roll dice minigame use emojis
- Changed `/boondollars` back to a message
- Changed "boonbucks" to "Boondollars"
- Changed "Can Town" to "Can City" and added more names based on the can count
- Changed `/boondollars` to show hours in increments of 5 minutes
- Changed `/pvp` to only draw as much money as the challenger has when fighting Byers (so you can't lose more than you have)
- `/pvp` now pings the challenged person
- Renamed `/minigames roll_dice` to `/minigames rolldice`
- Renamed `/pay` to `/give`
- Changed Liquidsoap's playlist to use a dedicated playlist file
  - Previously this could cause PNG files to be "played" as videos
- Updated sqlx to 0.7.1
- Updated poise to 0.6

### Fixed

- Fixed frohike not being able to drop indices on moved directories
- Fixed `/addcan` not returning any message when it's on cooldown
- Fixed multiple issues with frohike's automatic indexing
  - Only audio files will be indexed now
  - Switched from inotify to polling to improve index accuracy

## [1.1.9] - 2023-10-06

### Fixed

- Fixed bug where you could generate money by paying yourself.

## [1.1.8] - 2023-09-23

### Changed

- Allow Byers to be challenged to PvP

### Fixed

- Fixed slots not being winnable with 3 oranges or lemons
- Fixed critical bug where rolling dices would not award money because it was never written to the database

## [1.1.7] - 2023-09-22

### Removed

- Disabled the strife minigame for now due to serenity not being able to properly process interactions.

## [1.1.6] - 2023-09-22

### Changed

- Allow !addcan with a comment

### Fixed

- Fixed being able to pay negative boondollars

## [1.1.5] - 2023-09-22

### Fixed

- Fixed the quest roll not increasing properly
- Fixed strife cooldown a little bit more

## [1.1.4] - 2023-09-21

### Changed

- Changed encoding from ABR to CBR

### Fixed

- Fixed slots jackpot being displayed as 0
- Fixed formatting in `/boondollars` command

## [1.1.3] - 2023-09-21

### Changed

- Changed server's roll to quest roll

### Fixed

- Fixed the strife cooldown triggering too late
- Fixed the payout of the dice minigame

## [1.1.2] - 2023-09-21

### Changed

- Made commands count towards activity

## [1.1.1] - 2023-09-21

### Added

- Added `/song search` - Allows you to search songs and additionally request one of them

### Changed

- Byers now sends a message when the song request queue is empty
- Switched to cargo-chef for caching dependencies and speed up future builds

### Fixed

- Fix inlining problems with `/boondollars`

## [1.1.0] - 2023-09-20

### Added

- Add `/song queue` as a command - This will show the current song request queue

### Changed

- Changed message for boondollars to be an embed instead

### Fixed

- Fixed song requests still being ephemeral
- Fixed roll dice minigame being able to produce unobtainable server rolls
- Fixed cooldowns triggering even if the songs weren't successful
- Fixed PvP cooldown only triggering for the challenger
- Fixed strife always producing a lich queen to fight

## [1.0.8] - 2023-09-19

### Added

- Added a cooldown message for `/add can`
- Added 2 new commands: `/song history` and `/song playing`.

### Changed

- Changed song IDs over to use the file hash instead of the file path
- Display the currently playing song as "album - title" instead of "artist - title"

### Fixed

- Fixed a missing comment in the `/listen` command.
- Fixed the skip command by setting the ID of the icecast source to "lumiradio".

## [1.0.7] - 2023-09-19

### Added

- Added a prefix command variant for !addcan and !addbear
  - These are currently disabled while the bot is still in beta.
- Added a `/listen` command which displays the link to the radio.
- Added an automatic rollover of the dice roll from 666 to 111.

### Changed

- Changed the cooldown message to also use a relative string.
- Made `/strife` send a new message and only edit the old message to remove the buttons.

### Fixed

- Fixed next rank hours not being displayed correctly.
- Fixed not being able to add a can without a user account.
- Fixed the starting dice roll being 1 instead of 111.

## [1.0.6]

### Changed

- Add manual workflow trigger for GitHub Actions.

## [1.0.5]

### Changed

- Made most embeds non-ephemeral.

### Fixed

- Fixed decimal formatting in `/boondollars`.
- Fixed point accumulation not working because of unparseable dates.
- Check if user exists before trying to request a song.
- Fix missing add commands.

## [1.0.4] - 2023-09-19

### Fixed

- Hot fix: empty buffer is not emptied after checking if the message has ended.

## [1.0.3] - 2023-09-19

### Fixed

- Fixed a bug where the bot would try to fill the buffer infinitely when waiting for a response from liquidsoap.

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
[unreleased]: https://github.com/LumiRadio/lumiRadio/compare/v1.1.8...HEAD
[0.1.0]: https://github.com/LumiRadio/lumiRadio/releases/tag/v0.1.0
[1.0.0]: https://github.com/LumiRadio/lumiRadio/compare/v0.1.0..v1.0.0
[1.0.1]: https://github.com/LumiRadio/lumiRadio/compare/v1.0.0..v1.0.1
[1.0.2]: https://github.com/LumiRadio/lumiRadio/compare/v1.0.1..v1.0.2
[1.0.3]: https://github.com/LumiRadio/lumiRadio/compare/v1.0.2..v1.0.3
[1.0.4]: https://github.com/LumiRadio/lumiRadio/compare/v1.0.3..v1.0.4
[1.0.5]: https://github.com/LumiRadio/lumiRadio/compare/v1.0.4..v1.0.5
[1.0.6]: https://github.com/LumiRadio/lumiRadio/compare/v1.0.5..v1.0.6
[1.0.7]: https://github.com/LumiRadio/lumiRadio/compare/v1.0.6..v1.0.7
[1.0.8]: https://github.com/LumiRadio/lumiRadio/compare/v1.0.7..v1.0.8
[1.1.0]: https://github.com/LumiRadio/lumiRadio/compare/v1.0.8..v1.1.0
[1.1.1]: https://github.com/LumiRadio/lumiRadio/compare/v1.1.0..v1.1.1
[1.1.2]: https://github.com/LumiRadio/lumiRadio/compare/v1.1.1..v1.1.2
[1.1.3]: https://github.com/LumiRadio/lumiRadio/compare/v1.1.2..v1.1.3
[1.1.4]: https://github.com/LumiRadio/lumiRadio/compare/v1.1.3..v1.1.4
[1.1.5]: https://github.com/LumiRadio/lumiRadio/compare/v1.1.4..v1.1.5
[1.1.6]: https://github.com/LumiRadio/lumiRadio/compare/v1.1.5..v1.1.6
[1.1.7]: https://github.com/LumiRadio/lumiRadio/compare/v1.1.6..v1.1.7
[1.1.8]: https://github.com/LumiRadio/lumiRadio/compare/v1.1.7..v1.1.8
[1.1.9]: https://github.com/LumiRadio/lumiRadio/compare/v1.1.8..v1.1.9
[1.2.0]: https://github.com/LumiRadio/lumiRadio/compare/v1.1.9..v1.2.0
[1.2.1]: https://github.com/LumiRadio/lumiRadio/compare/v1.2.0..v1.2.1
[1.3.0]: https://github.com/LumiRadio/lumiRadio/compare/v1.2.0..v1.3.0
