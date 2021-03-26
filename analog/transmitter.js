const stream = require('stream');
const dgram = require('dgram');
const ffmpeg = require("fluent-ffmpeg");

const logger = require('../helpers/logger');

function create_tx_socket(connection) {
    const socket = dgram.createSocket({ type: 'udp4' });
    let audioPacket = {};
    socket.connect(Number(process.env.DMR_TARGET_RX_PORT), process.env.DMR_TARGET);

    connection.on("disconnect", () => {
        logger.warn('TX', 'UDP', 'Closing socket')
        socket.close()
    });

    socket.on("error", (err) => {
        logger.error('TX', 'ERROR', err.name)
        socket.close();
    });

    connection.on("speaking", (user, speaking) => {
        const audioStream = connection.receiver.createStream(user, { mode: 'pcm' });
        if (!audioPacket[user.id]) {
            audioPacket[user.id] = new stream.Duplex();
            audioPacket[user.id].on("data", (chunk) => {
                console.log(chunk, chunk.length);
            })
        }
        const ffmpegCommand = ffmpeg(audioStream)
            .fromFormat('f16le')
            .addInputOptions([
                "-ar 44100",
                "-ac 2"
            ])
            .audioChannels(1)
            .audioFrequency(8000)
            .output(audioPacket[user.id])
            .run();
    })
}

module.exports = {
    create_tx_socket
}