require("dotenv").config();
const { CommandoClient } = require("discord.js-commando");
const path = require("path");
const client = new CommandoClient({
    commandPrefix: process.env.BOT_PREFIX,
});

client.registry
    .registerDefaultTypes()
    .registerGroups([
        ['basic', 'Basic Commands']
    ])
    .registerDefaultGroups()
    .registerDefaultCommands()
    .registerCommandsIn(path.join(__dirname, "commands"));

client.once("ready", () => {
	console.log(`Logged in as ${client.user.tag}! (${client.user.id})`);
});

client.on("voiceStateUpdate", (oldState, newState) => {
    if (newState.channel === null && oldState.channel.members.size === 1 && oldState.channel.id === oldState.guild.me.voice.channel?.id) {
        oldState.channel.leave();
    }
});

client.on("error", console.error);

client.login(process.env.BOT_TOKEN);