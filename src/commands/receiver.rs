use serenity::async_trait;

use byteorder::{ByteOrder, BigEndian, LittleEndian};

use std::sync::atomic::{AtomicU32, Ordering};
use std::env;
use std::net::UdpSocket;

use songbird::{
    Event,
    EventContext,
    EventHandler as VoiceEventHandler,
};

pub struct Receiver {
    sequence: AtomicU32,
    socket: UdpSocket
}

impl Receiver {
    pub fn new() -> Self {
        // You can manage state here, such as a buffer of audio packet bytes so
        // you can later store them in intervals.
        let socket = UdpSocket::bind("127.0.0.1:0")
            .expect("Couldn't bind udp socket for discord's audio receiver");
        socket.connect(env::var("DMR_TARGET_RX_ADDR")
                .expect("Expected a target rx address in the environment"))
            .expect("Couldn't connect to DMR's audio transmitter");

        Self { 
            sequence: AtomicU32::new(0),
            socket
        }
    }

    pub fn write_header(&self, buffer: &mut [u8], transmit: bool) {
        buffer[..4].copy_from_slice(b"USRP");
        let sequence = self.sequence.load(Ordering::Relaxed);
        BigEndian::write_u32(&mut buffer[4..8], sequence);
        self.sequence.fetch_add(1, Ordering::SeqCst);
        BigEndian::write_u32(&mut buffer[8..12], transmit as u32);
    }
}

#[async_trait]
impl VoiceEventHandler for Receiver {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        use EventContext as Ctx;
        match ctx {
            Ctx::SpeakingUpdate(data) => {
                if !data.speaking {
                    let mut end_buffer = [0u8; 32];
                    self.write_header(&mut end_buffer, false);
                    self.socket.send(&end_buffer).expect("Couldn't send discord's audio packet through DMR transmitter");
                }
            }
            Ctx::VoicePacket(data) => {
                // An event which fires for every received audio packet,
                // containing the decoded data.
                if let Some(audio) = data.audio {
                    println!("{:?}", audio.len());
                    let mut values = audio.into_iter().peekable();
                    while values.peek().is_some() {
                        let mut buffer = [0u8; 352];
                        let audio_chunk: Vec<i16> = values.by_ref().take(160).cloned().collect();
                        self.write_header(&mut buffer, true);
                        LittleEndian::write_i16_into(audio_chunk.as_slice(), &mut buffer[32..]);
                        self.socket.send(&buffer).expect("Couldn't send discord's audio packet through DMR transmitter");
                    }
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