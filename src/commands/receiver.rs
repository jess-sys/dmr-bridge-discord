use serenity::async_trait;

use byteorder::{ByteOrder, BigEndian, LittleEndian};

use std::sync::atomic::{AtomicU32, Ordering};
use std::{env, thread, time};
use std::net::UdpSocket;
use std::sync::Arc;

use std::sync::mpsc::{sync_channel, SyncSender};

use songbird::{
    Event,
    EventContext,
    EventHandler as VoiceEventHandler,
};

#[derive(PartialEq, Debug)]
pub enum USRPVoicePacketType {
    START,
    AUDIO,
    END
}

pub struct Receiver {
    sequence: AtomicU32,
    tx: SyncSender<Option<(USRPVoicePacketType, Vec<u8>)>>
}

impl Drop for Receiver {
    fn drop(&mut self) {
        self.tx.send(None).unwrap();
    }
}

pub struct ReceiverWrapper {
    receiver: Arc<Receiver>
}

impl ReceiverWrapper {
    pub fn new(receiver: Arc<Receiver>) -> Self {
        Self {
           receiver
        }
    }
}

#[async_trait]
impl VoiceEventHandler for ReceiverWrapper {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        self.receiver.act(ctx).await
    }
}


impl Receiver {
    pub fn new() -> Self {
        // You can manage state here, such as a buffer of audio packet bytes so
        // you can later store them in intervals.
        let dmr_target_rx_addr = env::var("DMR_TARGET_RX_ADDR")
            .expect("Expected a target rx address in the environment");

        let socket = UdpSocket::bind("127.0.0.1:0")
            .expect("Couldn't bind udp socket for discord's audio receiver");

        socket.connect(dmr_target_rx_addr)
            .expect("Couldn't connect to DMR's audio transmitter");

        let (tx, rx) = sync_channel::<Option<(USRPVoicePacketType, Vec<u8>)>>(128);

        thread::spawn(move || {
            let mut can_transmit = false;
            loop {
                match rx.recv() {
                    Ok(packet) => match packet {
                        Some((packet_type, packet_data)) => {
                            if packet_type == USRPVoicePacketType::START {
                                can_transmit = true;
                            }
                            if can_transmit == true {
                                let two_millis = time::Duration::from_millis(2);
                                thread::sleep(two_millis);
                                println!("[INFO] SEND PACKET: {:?} (length: {}, ptt: {})",
                                         packet_type, packet_data.len(),
                                         BigEndian::read_u32(&packet_data[12..16]));
                                match socket.send(&*packet_data) {
                                    Ok(_) => {},
                                    Err(_) => return,
                                }
                            }
                            if packet_type == USRPVoicePacketType::END {
                                can_transmit = false;
                            }
                        },
                        None => return,
                    },
                    Err(_) => return,
                }
            }
        });

        Self { 
            sequence: AtomicU32::new(0),
            tx
        }
    }

    pub fn write_header(&self, buffer: &mut [u8], transmit: bool, packet_type: u32) {
        buffer[..4].copy_from_slice(b"USRP");
        let sequence = self.sequence.load(Ordering::Relaxed);
        BigEndian::write_u32(&mut buffer[4..8], sequence);
        self.sequence.fetch_add(1, Ordering::SeqCst);
        LittleEndian::write_u32(&mut buffer[20..24], packet_type);
        if packet_type != 2 {
            BigEndian::write_u32(&mut buffer[8..12], 2);
            BigEndian::write_u32(&mut buffer[12..16], transmit as u32);
            BigEndian::write_u32(&mut buffer[16..20], 7);
            BigEndian::write_u32(&mut buffer[24..28], 0);
            BigEndian::write_u32(&mut buffer[28..32], 0);
        }
    }
}

#[async_trait]
impl VoiceEventHandler for Receiver {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        use EventContext as Ctx;
        match ctx {
            Ctx::SpeakingUpdate(data) => {
                if data.speaking {
                    let mut start_buffer = [0u8; 352];
                    let header: [u8; 21] = [0x08, 0x14, 0x1F, 0xC2, 0x39, 0x0C, 0x67, 0xDE, 0x45,
                    0x00, 0x00, 0x07, 0x02, 0x00, 0x32, 0x30, 0x38, 0x31, 0x33, 0x33, 0x37];
                    self.write_header(&mut start_buffer, false, 2);
                    start_buffer[32..53].copy_from_slice(&header);
                    self.tx.send(Some((USRPVoicePacketType::START, Vec::from(start_buffer))))
                        .expect("Couldn't send discord's audio packet through DMR transmitter");
                } else {
                    let mut end_buffer = [0u8; 32];
                    self.write_header(&mut end_buffer, false, 0);
                    self.tx.send(Some((USRPVoicePacketType::END, Vec::from(end_buffer))))
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
                        LittleEndian::write_i16_into(&audio_chunk, &mut packet_buffer[32..]);
                        self.tx.send(Some((USRPVoicePacketType::AUDIO, Vec::from(packet_buffer))))
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