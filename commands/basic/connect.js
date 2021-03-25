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
            const discord_receiver = connection.player;
            const discord_transmitter = connection.receiver;
            transceiver.rx.create_rx_socket(connection);
            //transceiver.tx.transmit();
        } else {
            message.reply("You should be in a voice channel to execute this command");
        }
    }
}