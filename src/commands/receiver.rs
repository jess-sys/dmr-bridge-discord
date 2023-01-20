use byteorder::{BigEndian, ByteOrder, LittleEndian};
use dasp_interpolate::linear::Linear;
use dasp_signal::{self as signal, Signal};
use serenity::prelude::Mutex as SerenityMutex;
use songbird::input::{Codec, Container, Reader};
use songbird::{input::Input, Call};
use std::net::UdpSocket;
use std::sync::{
    mpsc::{sync_channel, SyncSender},
    Arc, Mutex, MutexGuard,
};
use std::thread;
use std::{env, time};
use tokio::runtime::Runtime;
use dmr_bridge_discord::USRPVoicePacketType;

pub struct Receiver {
    discord_channel: Arc<Mutex<Option<Arc<SerenityMutex<Call>>>>>,
    tx: SyncSender<Option<Vec<u8>>>,
}

impl Drop for Receiver {
    fn drop(&mut self) {
        self.tx.send(None).unwrap();
    }
}

impl Receiver {
    pub fn new() -> Self {
        // You can manage state here, such as a buffer of audio packet bytes so
        // you can later store them in intervals.
        let dmr_local_rx_addr = env::var("LOCAL_RX_ADDR")
            .expect("Expected a local rx address in the environment");

        let socket = UdpSocket::bind(dmr_local_rx_addr)
            .expect("Couldn't bind udp socket for reception");

        let discord_channel = Arc::new(Mutex::new(None));

        let (tx, rx) = sync_channel::<Option<Vec<u8>>>(512);

        let channel_ref = discord_channel.clone();
        thread::spawn(move || loop {
            match rx.recv() {
                Ok(packet) => match packet {
                    Some(packet_data) => {
                        let mut data: [i16; 160] = [0; 160];
                        LittleEndian::read_i16_into(&packet_data, &mut data);
                        let mut source = signal::from_iter(data.iter().cloned());
                        let first = source.next();
                        let second = source.next();
                        let interpolator = Linear::new(first, second);
                        let frames: Vec<_> = source
                            .from_hz_to_hz(interpolator, 8000.0, 48000.0)
                            .take(960)
                            .collect();
                        let mut new_data: [u8; 1920] = [0; 1920];
                        LittleEndian::write_i16_into(&frames, &mut new_data);
                        let audio = Input::new(
                            false,
                            Reader::from_memory(Vec::from(new_data)),
                            Codec::Pcm,
                            Container::Raw,
                            None,
                        );
                        {
                            let channel: MutexGuard<Option<Arc<SerenityMutex<Call>>>> =
                                channel_ref.lock().unwrap();
                            match &*channel {
                                Some(device) => {
                                    let rt = Runtime::new().unwrap();
                                    let mut call = rt.block_on(async { device.lock().await });
                                    call.play_source(audio);
                                    let two_millis = time::Duration::from_millis(18);
                                    thread::sleep(two_millis);
                                }
                                None => {}
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
                        let packet_type_as_num = LittleEndian::read_u32(&buffer[20..24]);
                        let packet_type = match packet_type_as_num {
                            0 => {
                                if packet_size == 32 {
                                    USRPVoicePacketType::End
                                } else {
                                    USRPVoicePacketType::Audio
                                }
                            }
                            2 => USRPVoicePacketType::Start,
                            _ => USRPVoicePacketType::Audio,
                        };
                        println!(
                            "[INFO] RECEIVED PACKET: {:?} (length: {}, ptt: {})",
                            packet_type,
                            packet_size,
                            BigEndian::read_u32(&buffer[12..16])
                        );
                        if packet_type == USRPVoicePacketType::Audio {
                            let audio = Vec::from(&buffer[32..]);
                            if audio.len() == 320 && sub_tx.send(Some(audio)).is_err() { return }
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
