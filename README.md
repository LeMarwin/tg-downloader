# Telegram video downloader bot

## Functionality
* Youtube audio
* Youtube video
* Tiktok video
* Instagram reels
* Webm
* Converts videos to round video notes

## Usage
Send an url. The bot automatically detects what needs to be downloaded

By default Youtube URLs are downloaded as audio files. To download as a video send `video https://youtu.be/...`

If you send a videofile, the bot will download it, crop to square, cut up to the first 60s and send back as a round video note

## Build
Uses nix to provide a dev shell or just run `nix build`

## Run
Set `TELOXIDE_TOKEN` to your bot's token and run

## Local Telegram API server

The bot is better utilized with a local Telegram API server.

See instructions on how to build and deploy [here](https://github.com/tdlib/telegram-bot-api)

Add `TELOXIDE_API_URL="http://localhost:8081` to env to make the bot work through a local server
