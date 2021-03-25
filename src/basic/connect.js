const { Command } = require("discord.js-commando");

module.exports = class ConnectCommand extends Command {
    constructor(client) {
        super(client, {
            name: "connect",
            aliases: ["join"],
            group: "basic",
            memberName: "connect",
            description: "The bot will join your channel and bridge DMR and Discord",
        })
    }

    run(message) {
        message.say("Connected...");
    }
}