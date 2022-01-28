use byteorder::{BigEndian, ByteOrder, LittleEndian};
use serenity::prelude::Mutex as SerenityMutex;
use songbird::{input::Input, tracks::create_player, Call};
use std::env;
use std::net::UdpSocket;
use std::sync::{
    mpsc::{sync_channel, SyncSender},
    Arc, Mutex,
};
use std::thread;

use songbird::input::{Codec, Container, Reader};
use tokio::runtime::Runtime;

#[derive(PartialEq, Debug)]
pub enum USRPVoicePacketType {
    START,
    AUDIO,
    END,
}

pub struct Transmitter {
    discord_channel: Mutex<Option<Arc<SerenityMutex<Call>>>>,
    tx: SyncSender<Option<Vec<u8>>>,
}

impl Drop for Transmitter {
    fn drop(&mut self) {
        self.tx.send(None).unwrap();
    }
}

impl Transmitter {
    pub fn new() -> Self {
        // You can manage state here, such as a buffer of audio packet bytes so
        // you can later store them in intervals.
        let dmr_target_tx_addr = env::var("DMR_TARGET_TX_ADDR")
            .expect("Expected a target tx address in the environment");

        let socket = UdpSocket::bind(dmr_target_tx_addr)
            .expect("Couldn't bind udp socket for discord's audio transmitter");

        let discord_channel = Mutex::<Option<Arc<SerenityMutex<Call>>>>::new(None);

        let (tx, rx) = sync_channel::<Option<Vec<u8>>>(128);

        thread::spawn(move || loop {
            match rx.recv() {
                Ok(packet) => match packet {
                    Some(packet_data) => {
                        let (audio, _audio_handle) = create_player(Input::new(
                            false,
                            Reader::from_memory(packet_data),
                            Codec::Pcm,
                            Container::Raw,
                            None,
                        ));
                        {
                            let channel = discord_channel.lock().unwrap();
                            match &*channel {
                                Some(device) => {
                                    let rt = Runtime::new().unwrap();
                                    let mut call = rt.block_on(async { device.lock().await });
                                    call.play_only(audio);
                                }
                                None => {
                                    println!("Missing discord channel");
                                }
                            }
                        }
                    }
                    None => return,
                },
                Err(_) => return,
            }
        });

        let sub_tx = tx.clone();
        thread::spawn(move || loop {
            let mut buffer = [0u8; 352];

            match socket.recv(&mut buffer) {
                Ok(packet_size) => {
                    let packet_type_as_num = LittleEndian::read_u32(&mut buffer[20..24]);
                    let packet_type = match packet_type_as_num {
                        0 => {
                            if packet_size == 32 {
                                USRPVoicePacketType::END
                            } else {
                                USRPVoicePacketType::AUDIO
                            }
                        }
                        2 => USRPVoicePacketType::START,
                        _ => USRPVoicePacketType::AUDIO,
                    };
                    println!(
                        "[INFO] RECEIVED PACKET: {:?} (length: {}, ptt: {})",
                        packet_type,
                        packet_size,
                        BigEndian::read_u32(&buffer[12..16])
                    );
                    if packet_type == USRPVoicePacketType::AUDIO {
                        match sub_tx.send(Some(Vec::from(&buffer[32..]))) {
                            Err(_) => return,
                            _ => {}
                        }
                    }
                }
                Err(_) => return,
            }
        });

        Self {
            discord_channel: Mutex::new(None),
            tx,
        }
    }

    pub fn set(&mut self, device: Arc<SerenityMutex<Call>>) {
        let device = Arc::clone(&device);
        let mut discord_channel = self.discord_channel.lock().unwrap();
        *discord_channel = Some(device);
    }

    pub fn unset(&mut self) {
        let mut discord_channel = self.discord_channel.lock().unwrap();
        *discord_channel = None;
    }
}
