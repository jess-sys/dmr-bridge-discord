const dgram = require('dgram');
const stream = require('stream');
const binary = require('binary');
const fs = require('fs');
const { OpusEncoder } = require('@discordjs/opus');

const logger = require('../helpers/logger');

function parse_receiver_data(msg) {
    const vars = binary.parse(msg)
        .buffer('header', 4)
        .buffer('eye', 4)
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
    const encoder = new OpusEncoder(8000, 1);
    const socket = dgram.createSocket({ type: 'udp4', reuseAddr: true, recvBufferSize: 352 });
    socket.bind(process.env.DMR_TARGET_TX_PORT);
    let last_key = null;

    socket.on("error", (err) => {
        logger.error('RX', 'ERROR', err.name)
        socket.close();
    })

    socket.on("close", () => {

    });

    socket.on("message", (msg, rinfo) => {
        if (rinfo.address !== process.env.DMR_TARGET || rinfo.size !== 352)
            return;
        const { header, eye, seq, memory, keyup, talkgroup, type, mpxid, reserved, audio } = parse_receiver_data(msg);
        if (header?.toString('ascii') === 'USRP') {
            if (type == 0) {
                let player = connection.play(stream.Readable.from(encoder.encode(audio)));
                player.on("start", () => {
                    logger.warn('RX', "START");
                })
                player.on("speaking", (boolean) => {
                    console.log(boolean);
                    logger.warn('RX', "SPEAKING", boolean);
                })
                player.on("error", () => {
                    logger.error('RX', 'ERR_SPK');
                })
                if (keyup != last_key) {
                    if (keyup) {
                        logger.info('RX', 'STOP RECEIVING');
                    } else {
                        logger.info('RX', 'RECEIVING');
                    }
                    last_key = keyup;
                }
            }
        } else {
            logger.warn('RX', 'WARNING', 'Badly formatted message, ignoring');
        }
    });

    socket.on("listening", () => {
        const address = socket.address();
        logger.success('RX', 'LISTENING')
    })
    return socket;
}

module.exports = {
    create_rx_socket
}