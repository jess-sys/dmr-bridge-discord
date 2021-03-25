const stream = require('stream');
const dgram = require('dgram');

function send_audio() {
    const socket = dgram.createSocket({ type: 'udp4', reuseAddr: true });
    socket.connect(Number(process.env.DMR_TARGET_RX_PORT), process.env.DMR_TARGET);

}

module.exports = {
    send_audio
}