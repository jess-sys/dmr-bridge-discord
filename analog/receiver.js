const dgram = require('dgram');
const stream = require('stream');
const SampleRate = require('node-libsamplerate');

const logger = require('../helpers/logger');

function create_rx_socket(connection) {
    const resampler = new SampleRate({
        type: SampleRate.SRC_SINC_BEST_QUALITY,
        channels: 1,
        fromRate: 8000,
        fromDepth: 16,
        toRate: 48000,
        toDepth: 16
    });
    const socket = dgram.createSocket({ type: 'udp4', recvBufferSize: 320 });
    let queueBuffer = [];
    let garbageListener = setInterval(() => {
        if (queueBuffer.length === 0)
            return;
        const queueMask = queueBuffer.splice(0, queueBuffer.length);
        const bufferStream = stream.Readable.from(queueMask);
        const resamplerStream = bufferStream.pipe(resampler);
        if (Number(process.env.VERBOSE) >= 1) {
            logger.info('RX', 'PTT', 'PTT button pressed (audio size ' + queueMask.reduce((acc, buf) => acc + buf.length, 0) + ')');
        }
        connection.play(resamplerStream, { type: 'converted' });
    }, 250);

    socket.bind(process.env.DMR_TARGET_TX_PORT);
    
    connection.on("disconnect", () => {
        logger.warn('RX', 'UDP', 'Closing socket')
        socket.close()
    });

    socket.on("error", (err) => {
        logger.error('RX', 'ERROR', err.name)
        socket.close();c
    })

    socket.on("message", (msg, rinfo) => {
        queueBuffer.push(msg);
    });

    socket.on("listening", () => {
        logger.success('RX', 'UDP', 'Listening on port ' + process.env.DMR_TARGET_TX_PORT)
    })
    return socket;
}

module.exports = {
    create_rx_socket
}