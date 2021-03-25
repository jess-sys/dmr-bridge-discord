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
        if (message.guild.me.voice.channel) {
            message.guild.me.voice.channel.leave();
        } else {
            message.reply("I should be in a voice channel to execute this command");
        }
    }
}