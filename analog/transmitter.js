const stream = require('stream');
const dgram = require('dgram');

const converter = require('./converter');
const logger = require('../helpers/logger');

function create_header(seq, transmit) {
    const header = Buffer.alloc(32);
    header.write("USRP", 0);
    header.writeUInt16BE(seq, 4);
    header.writeUInt16BE(Number(transmit), 12);
    return header;
}

function create_tx_socket(connection) {
    const socket = dgram.createSocket({ type: 'udp4' });
    let audioPackets = {};
    let seq = 0;
    
    setInterval(() => {
        let rawAudio = Object.values(audioPackets);
        if (rawAudio.length === 0)
            return;
        audioPackets = {};
        rawAudio = rawAudio.map((chunks) => Buffer.concat(chunks));
        rawAudio = converter.collapse_pcm_data(rawAudio);
        rawAudio = converter.split_buffer(rawAudio, 320);
        const startHeader = create_header(seq, true);
        seq += 1;
        socket.send(startHeader);
        stream.Readable.from(rawAudio)
            .on("data", (chunk) => {
                const header = create_header(seq, true);
                seq += 1
                const data = Buffer.concat([header, chunk]);
                socket.send(data);
            })
            .on("end", () => {
                const endHeader = create_header(seq, false);
                seq += 1
                socket.send(endHeader);
            })
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