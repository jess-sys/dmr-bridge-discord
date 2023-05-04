use byteorder::{ByteOrder, LittleEndian};
use dmr_bridge_discord::packet::USRP;
use rodio::buffer::SamplesBuffer;
use rodio::source::UniformSourceIterator;
use serenity::prelude::Mutex as SerenityMutex;
use songbird::input::{Codec, Container, Reader};
use songbird::{input::Input, Call};
use std::net::UdpSocket;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::{env};
use std::ops::Deref;
use std::time::Duration;
use tokio::runtime::Runtime;

pub struct Receiver {
    discord_channel: Arc<Mutex<Option<Arc<SerenityMutex<Call>>>>>,
}

impl Receiver {
    pub fn new() -> Self {
        // You can manage state here, such as a buffer of audio packet bytes so
        // you can later store them in intervals.
        let dmr_local_rx_addr =
            env::var("LOCAL_RX_ADDR").expect("Expected a local rx address in the environment");

        let socket =
            UdpSocket::bind(dmr_local_rx_addr).expect("Couldn't bind udp socket for reception");

        let discord_channel = Arc::new(Mutex::new(None));

        let channel = Arc::clone(&discord_channel);

        thread::spawn(move || {
            let mut buffer = [0u8; 352];
            let mut discord_voice_buffer = [0i16; 1920];
            let mut discord_voice_buffer_as_bytes = [0u8; 3840];

            loop {
                match socket.recv(&mut buffer) {
                    Ok(packet_size) => {
                        if packet_size >= 32 {
                            if let Some(usrp_packet) = USRP::from_buffer(buffer) {
                                println!("Received USRP voice audio packet");
                                let source = SamplesBuffer::new(1, 8000, usrp_packet.audio);
                                let mut source = UniformSourceIterator::new(source, 2, 48000);
                                for sample in discord_voice_buffer.iter_mut() {
                                    *sample = source
                                        .next()
                                        .expect("Unreachable: buffer does not have enough samples");
                                }
                                LittleEndian::write_i16_into(
                                    &discord_voice_buffer,
                                    &mut discord_voice_buffer_as_bytes,
                                );
                                let audio = Input::new(
                                    false,
                                    Reader::from_memory(Vec::from(discord_voice_buffer_as_bytes)),
                                    Codec::Pcm,
                                    Container::Raw,
                                    None,
                                );
                                {
                                    let channel: MutexGuard<Option<Arc<SerenityMutex<Call>>>> =
                                        channel.lock().unwrap();
                                    if let Some(device) = channel.deref() {
                                        let rt = Runtime::new().unwrap();
                                        let mut call =
                                            rt.block_on(async { device.lock().await });
                                        println!("Sent Discord voice audio packet");
                                        call.play_source(audio);
                                        const TWO_MILLIS: Duration = Duration::from_millis(2);
                                        thread::sleep(TWO_MILLIS);
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => return,
                }
            }
        });

        println!("Receiver started");

        Self { discord_channel }
    }

    pub fn set(&mut self, device: Arc<SerenityMutex<Call>>) {
        let device = Arc::clone(&device);
        let mut discord_channel = self.discord_channel.lock().unwrap();
        *discord_channel = Some(device);

        println!("Receiver has been associated with a discord channel");
    }

    pub fn unset(&mut self) {
        let mut discord_channel = self.discord_channel.lock().unwrap();
        *discord_channel = None;

        println!("Receiver has been disassociated from a discord channel");
    }
}
