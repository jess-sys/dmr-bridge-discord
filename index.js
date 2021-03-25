require("dotenv").config();
const { CommandoClient } = require("discord.js-commando");
const path = require("path");
const client = new CommandoClient({
    commandPrefix: process.env.BOT_PREFIX,
})

client.registry
    .registerDefaultTypes()
    .registerGroups([
        ['basic', 'Basic Commands']
    ])
    .registerDefaultGroups()
    .registerDefaultCommands()
    .registerCommandsIn(path.join(__dirname, "src"));

client.once("ready", () => {
	console.log(`Logged in as ${client.user.tag}! (${client.user.id})`);
})

client.on("error", console.error);

client.login(process.env.BOT_TOKEN);