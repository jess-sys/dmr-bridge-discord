function collapse_pcm_data(chunks, channels) {
    if (Buffer.isBuffer(chunks) && Number.isInteger(channels)) {
        const packetSize = channels * 2;
        const newLength = chunks.length / channels;
        const newChunk = Buffer.allocUnsafe(newLength);
        for (let newIndex = 0; newIndex < newLength; newIndex += 2) {
            let collapsedData = 0;
            for (let offset = 0; offset < packetSize; offset += 2)
                collapsedData += chunks.readInt16LE(newIndex * channels + offset);
            collapsedData = ~~(collapsedData / channels);
            newChunk.writeInt16LE(collapsedData, newIndex);
        }
        return newChunk;
    } else if (Array.isArray(chunks)) {
        const newChunk = Buffer.allocUnsafe(chunk[0].length);
        const channels = chunks.length;
        for (let newIndex = 0; newIndex < chunk[0].length; newIndex += 2) {
            let collapsedData = 0;
            for (const chunk of chunks)
                collapsedData += chunk.readInt16LE(newIndex);
            collapsedData = ~~(collapsedData / channels);
            newChunk.writeInt16LE(collapsedData, newIndex);
        }
        return newChunk;
    } else {
        throw new Error("Invalid parameters.");
    }
}

function expand_pcm_data(chunk, channel) {
    
}

function interpolate_pcm_data(chunk) {

}

module.exports = {
    collapse_pcm_data
}