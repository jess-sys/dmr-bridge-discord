function adjust_buffer(buffer, newSize) {
    if (newSize == buffer.length)
        return buffer;
    const tmpBuffer = Buffer.alloc(newSize - buffer.length);
    const newBuffer = Buffer.concat([tmpBuffer, buffer]);
    return newBuffer;
}

function split_buffer(buffer, frameSize) {
    const size = buffer.length / frameSize;
    const array = new Array(size);
    for (let index = 0; index < size; index++)
        array[index] = Buffer.from(buffer.slice(index * frameSize, (index + 1) * frameSize));
    return array;
}

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
        const maxLength = chunks.reduce((max, chunk) => Math.max(max, chunk.length), 0);
        const newAdjustedChunks = chunks.map((chunk) => adjust_buffer(chunk, maxLength));
        const newChunk = Buffer.allocUnsafe(maxLength);
        for (let newIndex = 0; newIndex < maxLength; newIndex += 2) {
            let collapsedData = 0;
            let collaspedChunk = 0;
            for (const chunk of newAdjustedChunks) {
                const data = chunk.readInt16LE(newIndex);
                if (data) {
                    collaspedChunk += 1;
                    collapsedData += data;
                }
            }
            collapsedData = collapsedData / collaspedChunk;
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
    collapse_pcm_data,
    adjust_buffer,
    split_buffer
}