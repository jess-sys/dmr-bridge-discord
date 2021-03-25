const stream = require('stream');
const dgram = require('dgram');

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
        const audioStream = connection.receiver.createStream(user, { mode: 'pcm' });
        audioStream.on('data', (chunk) => { 
            console.log(chunk, chunk.length);
            if (!(user.id in audioPacket))
                audioPacket[user.id] = [];
            audioPacket[user.id].push(chunk);
        });
        audioStream.on('end', () => { 
            if (user.id in audioPacket)
                delete audioPacket[user.id];
        });
    })
}

module.exports = {
    create_tx_socket
}