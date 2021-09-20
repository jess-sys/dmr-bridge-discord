use serenity::async_trait;

use byteorder::{ByteOrder, BigEndian, LittleEndian};

use state::Storage;

use songbird::{
    Event,
    EventContext,
    EventHandler as VoiceEventHandler,
};

static SEQUENCE: Storage<u32> = Storage::new();

pub struct Receiver;

impl Receiver {
    pub fn new() -> Self {
        // You can manage state here, such as a buffer of audio packet bytes so
        // you can later store them in intervals.
        SEQUENCE.set(0);
        Self { }
    }
}

#[async_trait]
impl VoiceEventHandler for Receiver {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        use EventContext as Ctx;
        match ctx {
            Ctx::VoicePacket(data) => {
                // An event which fires for every received audio packet,
                // containing the decoded data.
                if let Some(audio) = data.audio {
                    let mut values = audio.into_iter().peekable();
                    while values.peek().is_some() {
                        let mut buffer = [0u8; 352];
                        let audio_chunk: Vec<_> = values.by_ref().take(160).cloned().collect();
                        buffer.copy_from_slice(b"USRP");
                        BigEndian::write_u32(&mut buffer[4..8], SEQUENCE.get().clone());
                        SEQUENCE.set(SEQUENCE.get() + 1);
                        BigEndian::write_u32(&mut buffer[8..12], 1);
                        LittleEndian::write_i16_into(audio_chunk.as_slice(), &mut buffer[32..]);
                    }
                    let mut end_buffer = [0u8; 32];
                    end_buffer.copy_from_slice(b"USRP");
                    BigEndian::write_u32(&mut end_buffer[4..8], SEQUENCE.get().clone());
                    SEQUENCE.set(SEQUENCE.get() + 1);
                    BigEndian::write_u32(&mut end_buffer[8..12], 0);
                } else {
                    println!("RTP packet, but no audio. Driver may not be configured to decode.");
                }
            },
            _ => {
                // We won't be registering this struct for any more event classes.
                unimplemented!()
            }
        }

        None
    }
}