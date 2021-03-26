const stream = require('stream');
const dgram = require('dgram');

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

    setInterval(() => {
        console.log(audioPacket);
    }, 2000);

    connection.on("speaking", (user, speaking) => {
        return;
        const audioStream = connection.receiver.createStream(user, { mode: 'pcm' });
        audioStream.on('data', (chunk) => {
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