const dgram = require('dgram');
const stream = require('stream');
const OpusScript = require("opusscript");

const logger = require('../helpers/logger');

function create_rx_socket(connection) {
    //const encoder = new OpusScript(8000, 1, OpusScript.Application.VOIP);
    const socket = dgram.createSocket({ type: 'udp4', recvBufferSize: 320 });
    let queueBuffer = [];
    let garbageListener = setTimeout(() => {
        if (queueBuffer.length === 0)
            return;
        const buffer = Buffer.concat(queueBuffer);
        const bufferStream = stream.Readable.from(buffer);
        logger.success('RX', 'AUDIO', 'Got new audio frame of size ' + buffer.length);
        //const opusBuffer = encoder.encode(buffer, buffer.length / 2);
        //const opusStream = stream.Readable.from(opusBuffer);
        connection.play(bufferStream, { type: "converted", bitrate: 0.001 });
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