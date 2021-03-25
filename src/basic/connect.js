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

    async run(message) {
        if (message.member.voice.channel) {
            if (message.member.voice.channel.id === message.guild.me.voice.channel?.id)
                return;
            const connection = await message.member.voice.channel.join();
            const player = connection.player;
            const receiver = connection.receiver;
        } else {
            message.reply("You should be in a voice channel to execute this command");
        }
    }
}