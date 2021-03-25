const dgram = require('dgram');
const stream = require('stream');
const OpusScript = require("opusscript");

const logger = require('../helpers/logger');

function create_rx_socket(connection) {
    const encoder = new OpusScript(8000, 1, OpusScript.Application.VOIP);
    const socket = dgram.createSocket({ type: 'udp4', recvBufferSize: 320 });
    let queueBuffer = [];
    let garbageListener = setTimeout(() => {
        if (queueBuffer.length === 0)
            return;
        const opusBuffer = queueBuffer.map((buffer) => encoder.encode(buffer, 160));
        const opusStream = stream.Readable.from(opusBuffer);
        logger.info('RX', 'PTT', 'PTT button released. Pushing audio frame of size ' + queueBuffer.reduce((acc, buf) => acc + buf.length), 0);
        connection.play(opusStream, { type: 'opus' });
        queueBuffer = [];
    }, 150);

    socket.bind(process.env.DMR_TARGET_TX_PORT);
    
    connection.on("disconnect", () => {
        logger.warn('RX', 'UDP', 'Closing socket')
        socket.close()
    });

    socket.on("error", (err) => {
        logger.error('RX', 'ERROR', err.name)
        socket.close();
    })

    socket.on("message", (msg, rinfo) => {
        if (queueBuffer.length === 0) {
            logger.info('RX', 'PTT', 'PTT button pressed');
        }
        queueBuffer.push(msg);
        garbageListener.refresh();
    });

    socket.on("listening", () => {
        logger.success('RX', 'UDP', 'Listening on port ' + process.env.DMR_TARGET_TX_PORT)
    })
    return socket;
}

module.exports = {
    create_rx_socket
}