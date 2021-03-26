const stream = require('stream');
const dgram = require('dgram');
const { Converter } = require("ffmpeg-stream");

const logger = require('../helpers/logger');

function create_tx_socket(connection) {
    const converter = new Converter();
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
        const inputStream = converter.createInputStream({
            f: "f32le",
            ac: 2
        })
        const outputStream = converter.createOutputStream({
            f: "f16le",
            ac: 1
        })
        const processedStream = audioStream.pipe(inputStream).pipe(outputStream);
        processedStream.on('data', (chunk) => {
            console.log(chunk, chunk.length);
            if (!(user.id in audioPacket))
                audioPacket[user.id] = [];
            audioPacket[user.id].push({date: Date.now(), chunk});
        });
    })
}

module.exports = {
    create_tx_socket
}