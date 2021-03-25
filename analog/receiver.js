const dgram = require('dgram');
const stream = require('stream');
const binary = require('binary');
const fs = require('fs');
const OpusScript = require("opusscript");
const { Writable } = require('stream');
var Queue = require('better-queue');

const logger = require('../helpers/logger');

function parse_receiver_data(msg) {
    let vars = binary.parse(msg)
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
    vars.usrp = vars?.usrp?.toString();
    return vars;
}

function create_rx_socket(connection) {
    const encoder = new OpusScript(8000, 1, OpusScript.Application.AUDIO);
    const socket = dgram.createSocket({ type: 'udp4', reuseAddr: true, recvBufferSize: 352 });
    let last_key = 0;
    let q = new Queue((buffer, cb) => {
        console.log(buffer.length);
        const opusBuffer = encoder.encode(audio, buffer.length / 2);
        const opusStream = stream.Readable.from(opusBuffer);
        const dispatcher = connection.play(opusStream, { type: 'opus' });
        dispatcher.on("finish", cb);
    }, {
        merge: (buffer0, buffer1, cb) {
            buffer0.data += buffer1.data;
            cb(null, buffer0);
        }
    });

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
        if (rinfo.address !== process.env.DMR_TARGET || rinfo.size !== 352)
            return;
        const { usrp, seq, memory, keyup, talkgroup, type, mpxid, reserved, audio } = parse_receiver_data(msg);
        console.log({
            "usrp": usrp, 
            "seq": seq, 
            "memory": memory, 
            "keyup": keyup, 
            "talkgroup": talkgroup, 
            "type": type, 
            "mpxid": mpxid, 
            "reserved": reserved
        })
        if (usrp === 'USRP') {
            if (keyup == 0) {
                logger.info('RX', 'PTT', 'A Radio pressed the PTT button');
            }
            if (type == 0) {
                const buffer = audio;
                q.push({ id: "buffer", data: buffer });
            }
        } else {
            logger.warn('RX', 'WARNING', 'Badly formatted message, ignoring');
        }
    });

    socket.on("listening", () => {
        logger.success('RX', 'UDP', 'Listening on port ' + process.env.DMR_TARGET_TX_PORT)
    })
    return socket;
}

module.exports = {
    create_rx_socket
}