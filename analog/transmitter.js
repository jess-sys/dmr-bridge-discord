const stream = require('stream');
const dgram = require('dgram');

const converter = require('./converter');
const logger = require('../helpers/logger');

function create_tx_socket(connection) {
    const socket = dgram.createSocket({ type: 'udp4' });
    let audioPackets = {};
    
    setInterval(() => {
        console.log(audioPackets);
        return;
    }, 250);

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
        if (user.id in audioPackets === false)
            audioPackets[user.id] = []
        connection.receiver.createStream(user, { mode: 'pcm' })
            .on("data", (chunk) => {
                const newChunk = converter.collapse_pcm_data(chunk, 12);
                audioPackets[user.id].push(newChunk);
            })
    })
}

module.exports = {
    create_tx_socket
}