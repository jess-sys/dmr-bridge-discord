# dmr-bridge-discord

[![License](https://img.shields.io/badge/License-GPLv3-blue?style=for-the-badge)](https://www.gnu.org/licenses/gpl-3.0)

Bridge a DMR network with a Discord voice channel.

## Getting started

This script is inspired by <https://github.com/jess-sys/DMRBridgeWAV/blob/master/DMRBridgeWAV>.

The target server is AnalogBridge (see <https://github.com/DVSwitch/Analog_Bridge>).

![Diagram](https://i.ibb.co/2FGzLbY/DMRBridge-Discord.png)

### Install

```bash
# Coming soon
```

### Usage

First edit the `.env` file to reflect your infrastructure :

* `BOT_TOKEN` : see [this link](https://github.com/reactiflux/discord-irc/wiki/Creating-a-discord-bot-&-getting-a-token) to know how to get a token
* `DMR_TARGET` : your Analog Bridge IP

#### Manually

You can start DMRBridgeDiscord :

```bash
# Coming soon
```

#### Inside a container

You can use the docker-compose configuration file:

```bash
docker-compose up
```

## Todo

* Discord multiple voice users at once (merge audio channels)
* dtmf and text messages
* Full Docker support
* systemd service support

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
