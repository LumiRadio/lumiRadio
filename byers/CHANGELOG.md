# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog],
and this project adheres to [Semantic Versioning].

## [Unreleased]

### Added

- Added slash commands
  - `/song_request [song]` - Requests a song for the radio
    - To get an idea on what songs are available, type a song and see the completions.
    - This has a default cooldown of 1 1/2 hours.
    - Additionally, songs have individual cooldowns based on their length.
  - `/link_youtube` - Links your YouTube channel with your Discord account.
    - This will try and migrate the bot data for your account to the new bot. For this to work, you need to have the same name as in the old bot's data!
    - This system works upon trust. If there is abuse detected, your account may be frozen by luminantAegis or cozyGalvinism, which disables access to the bot's features.
  - `/unlink_youtube` - Unlinks your YouTube channel from your Discord account.
  - `/version` - (by default, only for people with the `MANAGE_GUILD` permission) Shows the currently running bot version and the changelog.
  - `/boondollars` - Shows your boondollars and watched/chatted hours and the position in the leaderboards.
  - `/pay` - Pays a user the specified amount of boondollars.
  - `/slots` - Uses the slot machine
- Added points and hours system
  - This should roughly follow the same rules as the old bot.
- Added context menus (right click)
  - For users
    - Give this user money - Does the same as `/pay`
    - PvP - Does the same as `/pvp`

## [0.1.0] - 2023-05-23

- initial release

<!-- Links -->
[keep a changelog]: https://keepachangelog.com/en/1.1.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html

<!-- Versions -->
[unreleased]: https://github.com/LumiRadio/lumiRadio/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/LumiRadio/lumiRadio/releases/tag/v0.1.0
