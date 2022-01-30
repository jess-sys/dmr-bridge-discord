# dmr-bridge-discord

[![License](https://img.shields.io/badge/License-GPLv3-blue?style=for-the-badge)](https://www.gnu.org/licenses/gpl-3.0)

Bridge a DMR network with a Discord voice channel.

## Getting started

This script is inspired by <https://github.com/jess-sys/DMRBridgeWAV/blob/master/DMRBridgeWAV>.

The target server is AnalogBridge (see <https://github.com/DVSwitch/Analog_Bridge>).

![Diagram](https://user-images.githubusercontent.com/20131496/151708716-22ff9881-efdc-49f8-9df0-e519cdddbc46.png)

### Build

Make sure you have [Rust installed](https://rustup.rs/)

```bash
cargo build --release
# or run it directly :
# cargo run
```

### Install

Install binaries to `/opt/dmr-bridge-discord/bin`, default config to `/opt/dmr-bridge-discord/.env` and install systemd service to `/lib/systemd/system/dmr-bridge-discord`.

```bash
# Coming soon
make install
make install-config
make install-systemd
```

### Configure

Edit the `.env` (the same directory or in /opt/dmr-bridge-discord) file to reflect your infrastructure :

* `BOT_TOKEN` : see [this link](https://github.com/reactiflux/discord-irc/wiki/Creating-a-discord-bot-&-getting-a-token) to know how to get a token
* `BOT_PREFIX` : prefix to add before the bot's commands 
* `DMR_TARGET_RX_ADDR` : your Analog Bridge IP and port
* `DMR_TARGET_TX_ADDR` : your dmr-bridge-discord IP and port (is localhost)

### Run

#### Systemctl service

```
systemctl start dmr-bridge-discord.service
# or enable it at boot:
# systemctl enable dmr-bridge-discord.service --now
```

#### Portable install

Do the following after you've built or [downloaded the pre-compiled version](https://github.com/jess-sys/dmr-bridge-discord/releases).

Then execute the binary in the same folder or export the environment variables present in the .env file.

```bash
./dmr-bridge-discord-linux
```

#### Inside a container

You can use the docker-compose configuration file:

```bash
# coming soon - not available atm
docker-compose up
```

### Usage

Here are the bot's commands:
* `!join` : Make the bot join the channel (you need to be in a voice channel first)
* `!leave` : Make the bot left the channel

The bot will join the voice channel you're in after your type `!join`.

Make sure you don't TX and RX at the same time, as AnalogBridge and the rest of the stack is half-duplex.

## Todo

* Discord multiple voice users at once (merge audio channels)
* Verbosity levels
* SMS and DTMF messages
* Full Docker support
* systemd services support

## Useless stuff (Copyright)

Copyright (C) 2022 Jessy SOBREIRO

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, version 2.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
