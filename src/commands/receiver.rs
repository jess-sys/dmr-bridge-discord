use serenity::async_trait;

use byteorder::{ByteOrder, BigEndian, LittleEndian};

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::{env, thread};
use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::mpsc::{sync_channel, SyncSender};

use songbird::{
    Event,
    EventContext,
    EventHandler as VoiceEventHandler,
};

pub struct Receiver {
    sequence: AtomicU32,
    close_receiver: Arc<AtomicBool>,
    tx: SyncSender<Vec<u8>>
}

impl Drop for Receiver {
    fn drop(&mut self) {
        self.close_receiver.swap(true, Ordering::Relaxed);
    }
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

        let (tx, rx) = sync_channel::<Vec<u8>>(128);
        let close_receiver = Arc::new(AtomicBool::new(false));

        let close = close_receiver.clone();
        thread::spawn(move || {
            loop {
                if close.load(Ordering::Relaxed) {
                    close.swap(false, Ordering::Relaxed);
                    return;
                }
                match rx.recv() {
                    Ok(packet) => match socket.send(&*packet) {
                        Err(_) => {
                            close.swap(false, Ordering::Relaxed);
                            return;
                        }
                        _ => {}
                    },
                    Err(_) => {
                        close.swap(false, Ordering::Relaxed);
                        return;
                    }
                }
            }
        });

        Self { 
            sequence: AtomicU32::new(0),
            close_receiver,
            tx
        }
    }

    pub fn write_header(&self, buffer: &mut [u8], transmit: bool, packet_type: u32) {
        buffer[..4].copy_from_slice(b"USRP");
        let sequence = self.sequence.load(Ordering::Relaxed);
        BigEndian::write_u32(&mut buffer[4..8], sequence);
        self.sequence.fetch_add(1, Ordering::SeqCst);
        BigEndian::write_u32(&mut buffer[8..12], 2);
        BigEndian::write_u32(&mut buffer[12..16], transmit as u32);
        BigEndian::write_u32(&mut buffer[16..20], 7);
        BigEndian::write_u32(&mut buffer[20..24], packet_type);
        BigEndian::write_u32(&mut buffer[24..28], 0);
        BigEndian::write_u32(&mut buffer[28..32], 0);
    }
}

#[async_trait]
impl VoiceEventHandler for Receiver {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        use EventContext as Ctx;
        match ctx {
            Ctx::SpeakingUpdate(data) => {
                if data.speaking {
                    let mut start_buffer = [0u8; 64];
                    self.write_header(&mut start_buffer, false, 2);
                    start_buffer[32] = 8;
                    BigEndian::write_u32(&mut start_buffer[40..44], 7);
                    start_buffer[44] = 2;
                    start_buffer[46..53].copy_from_slice(b"2081337");
                    println!("Start packet");
                    self.tx.send(Vec::from(start_buffer))
                        .expect("Couldn't send discord's audio packet through DMR transmitter");
                } else {
                    let mut end_buffer = [0u8; 32];
                    self.write_header(&mut end_buffer, false, 0);
                    println!("End packet");
                    self.tx.send(Vec::from(end_buffer))
                        .expect("Couldn't send discord's audio packet through DMR transmitter");
                }
            }
            Ctx::VoicePacket(data) => {
                // An event which fires for every received audio packet,
                // containing the decoded data.
                if let Some(audio) = data.audio {
                    if audio.len() == 1920 {
                        let mut audio_chunk = Vec::with_capacity(160);
                        for i in 0..160 {
                            audio_chunk.push(audio[i * 12]);
                        }
                        let mut packet_buffer = [0u8; 352];
                        self.write_header(&mut packet_buffer, true, 0);
                        LittleEndian::write_i16_into(audio_chunk.as_slice(), &mut packet_buffer[32..]);
                        println!("Audio packet");
                        self.tx.send(Vec::from(packet_buffer))
                            .expect("Couldn't send discord's audio packet through DMR transmitter");
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