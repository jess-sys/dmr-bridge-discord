const stream = require('stream');
const dgram = require('dgram');

const converter = require('./converter');
const logger = require('../helpers/logger');

let seq = 0;

function create_header(seq, transmit) {
    const header = Buffer.alloc(32);
    header.write("USRP", 0);
    header.writeUInt32BE(seq, 4);
    seq += 1;
    header.writeUInt32BE(Number(transmit), 12);
    return header;
}

function send_data(socket, chunk) {
    return new Promise((resolve, reject) => {
        socket.send(chunk, (err) => {
            if (err)
                reject(err);
            resolve();
        })
    })
}

function create_tx_socket(connection) {
    const socket = dgram.createSocket({ type: 'udp4' });
    let audioPackets = {};
    
    setInterval(async () => {
        let rawAudio = Object.values(audioPackets);
        if (rawAudio.length === 0)
            return;
        audioPackets = {};
        rawAudio = rawAudio.map((chunks) => Buffer.concat(chunks));
        rawAudio = converter.collapse_pcm_data(rawAudio);
        rawAudio = converter.split_buffer(rawAudio, 320);
        try {
            const startHeader = create_header(seq, true);
            await send_data(socket, startHeader);
            for (const chunk of rawAudio) {
                const header = create_header(seq, true);
                const data = Buffer.concat([header, chunk]);
                await send_data(socket, data);
            }
            const endHeader = create_header(seq, false);
            await send_data(socket, endHeader);
        } catch {
            socket.close();
        }
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
        connection.receiver.createStream(user, { mode: 'pcm' })
        .on("data", (chunk) => {
            const newChunk = converter.collapse_pcm_data(chunk, 12);
                if (user.id in audioPackets === false)
                    audioPackets[user.id] = []
                audioPackets[user.id].push(newChunk);
            })
    })
}

module.exports = {
    create_tx_socket
}