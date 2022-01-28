use byteorder::{BigEndian, ByteOrder, LittleEndian};
use dasp_interpolate::linear::Linear;
use dasp_signal::{self as signal, Signal};
use serenity::prelude::Mutex as SerenityMutex;
use songbird::input::{Codec, Container, Reader};
use songbird::{input::Input, tracks::create_player, Call};
use std::env;
use std::net::UdpSocket;
use std::sync::{
    mpsc::{sync_channel, SyncSender},
    Arc, Mutex, MutexGuard,
};
use std::thread;
use tokio::runtime::Runtime;

#[derive(PartialEq, Debug)]
pub enum USRPVoicePacketType {
    START,
    AUDIO,
    END,
}

pub struct Transmitter {
    discord_channel: Arc<Mutex<Option<Arc<SerenityMutex<Call>>>>>,
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

        let discord_channel = Arc::new(Mutex::new(None));

        let (tx, rx) = sync_channel::<Option<Vec<u8>>>(128);

        let channel_ref = discord_channel.clone();
        thread::spawn(move || loop {
            match rx.recv() {
                Ok(packet) => match packet {
                    Some(packet_data) => {
                        let mut data = Vec::with_capacity(160);
                        LittleEndian::read_i16_into(&packet_data, &mut data);
                        let mut source = signal::from_iter(data.iter().cloned());
                        let first = source.next();
                        let second = source.next();
                        let interpolator = Linear::new(first, second);
                        let frames: Vec<_> = source
                            .from_hz_to_hz(interpolator, 8000.0, 96000.0)
                            .take(1920)
                            .collect();
                        let mut new_data = Vec::with_capacity(3840);
                        LittleEndian::write_i16_into(&frames, &mut new_data);
                        let (audio, _audio_handle) = create_player(Input::new(
                            false,
                            Reader::from_memory(new_data),
                            Codec::Pcm,
                            Container::Raw,
                            None,
                        ));
                        {
                            let channel: MutexGuard<Option<Arc<SerenityMutex<Call>>>> =
                                channel_ref.lock().unwrap();
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
                    if packet_size >= 32 {
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
                            let audio = Vec::from(&buffer[32..]);
                            if audio.len() == 320 {
                                match sub_tx.send(Some(audio)) {
                                    Err(_) => return,
                                    _ => {}
                                }
                            }
                        }
                    }

                }
                Err(_) => return,
            }
        });

        Self {
            discord_channel,
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
