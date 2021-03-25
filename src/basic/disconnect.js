const { Command } = require("discord.js-commando");

module.exports = class DisconnectCommand extends Command {
    constructor(client) {
        super(client, {
            name: "disconnect",
            aliases: ["leave"],
            group: "basic",
            memberName: "disconnect",
            description: "The bot will leave your channel and unbridge DMR and Discord",
        })
    }

    run(message) {
        message.say("Disconnect...");
    }
}