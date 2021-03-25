const stream = require('stream');
const dgram = require('dgram');

function create_tx_socket() {
    const socket = dgram.createSocket({ type: 'udp4' });
    socket.connect(Number(process.env.DMR_TARGET_RX_PORT), process.env.DMR_TARGET);

}

module.exports = {
    create_tx_socket
}