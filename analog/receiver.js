const dgram = require('dgram');
const stream = require('stream');
const binary = require('binary');
const OpusScript = require("opusscript");

const logger = require('../helpers/logger');

function parse_receiver_data(msg) {
    const vars = binary.parse(msg)
        .buffer('usrp', 4)
        .word32bs('seq')
        .word32bs('memory')
        .word32bs('keyup')
        .word32bs('talkgroup')
        .word32bs('type')
        .word32bs('mpxid')
        .word32bs('reserved')
        .buffer('audio', 320)
        .vars;
    return vars;
}

function create_rx_socket(connection) {
    const encoder = new OpusScript(8000, 1, OpusScript.Application.VOIP);
    const socket = dgram.createSocket({ type: 'udp4', recvBufferSize: 320 });
    let queueBuffer = [];
    
    setInterval(() => {
        if (queueBuffer.length === 0)
            return;
        const queueMask = queueBuffer.splice(0, queueBuffer.length);
        const opusBuffer = queueMask.map((buffer) => encoder.encode(buffer, 160));
        const opusStream = stream.Readable.from(opusBuffer);
        if (Number(process.env.VERBOSE) >= 1) {
            logger.info('RX', 'PTT', 'PTT button pressed (audio size ' + queueMask.reduce((acc, buf) => acc + buf.length, 0) + ')');
        }
        connection.play(opusStream, { type: 'opus' });
    }, 250);

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
        const data = parse_receiver_data(msg);
        queueBuffer.push(data.audio);
    });

    socket.on("listening", () => {
        logger.success('RX', 'UDP', 'Listening on port ' + process.env.DMR_TARGET_TX_PORT)
    })
    return socket;
}

module.exports = {
    create_rx_socket
}