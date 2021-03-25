const { Command } = require("discord.js-commando");
const transceiver = require('../../analog');

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
            transceiver.rx.create_rx_socket(connection);
            transceiver.tx.create_tx_socket(connection);
        } else {
            message.reply("You should be in a voice channel to execute this command");
        }
    }
}