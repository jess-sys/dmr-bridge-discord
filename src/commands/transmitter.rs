use std::env;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}, mpsc::{sync_channel, SyncSender}};
use serenity::prelude::Mutex as SerenityMutex;
use songbird::{Call, tracks::create_player, input::Input};
use std::thread;
use std::net::UdpSocket;


use songbird::input::{Codec, Container, Reader};
use tokio::runtime::Runtime;

pub struct Transmitter {
    discord_channel: Mutex<Option<Arc<SerenityMutex<Call>>>>,
    close_sender: Arc<AtomicBool>,
    close_receiver: Arc<AtomicBool>,
    tx: Option<SyncSender<Vec<u8>>>,
}

impl Drop for Transmitter {
    fn drop(&mut self) {
        self.unset();
    }
}

impl Transmitter {
    pub fn new() -> Self {
        // You can manage state here, such as a buffer of audio packet bytes so
        // you can later store them in intervals.

        Self {
            discord_channel: Mutex::new(None),
            close_sender: Arc::new(AtomicBool::new(false)),
            close_receiver: Arc::new(AtomicBool::new(false)),
            tx: None
        }
    }

    pub fn start(&mut self) {
        self.start_receiver();
        self.start_sender();
    }

    pub fn stop(&mut self) {
        self.close_receiver.swap(true, Ordering::Relaxed);
        self.close_sender.swap(true, Ordering::Relaxed);
    }

    pub fn start_sender(&mut self) {
        let dmr_target_tx_addr = env::var("DMR_TARGET_TX_ADDR")
            .expect("Expected a target tx address in the environment");

        let socket = UdpSocket::bind("127.0.0.1:0")
            .expect("Couldn't bind udp socket for discord's audio transmitter");

        socket.connect(dmr_target_tx_addr)
            .expect("Couldn't connect to DMR's audio receiver");

        if self.tx.is_some() {
            let tx = self.tx.clone().unwrap();
            let close = self.close_sender.clone();
            self.close_sender.swap(false, Ordering::Relaxed);
            thread::spawn(move || {

                loop {
                    let mut buffer = [0u8; 352];

                    if close.load(Ordering::Relaxed) {
                        close.swap(false, Ordering::Relaxed);
                        return;
                    }
                    match socket.recv(&mut buffer) {
                        Ok(_n) => match tx.send(Vec::from(&buffer[32..])) {
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
        }
    }

    pub fn start_receiver(&mut self) {
        let discord_channel = self.discord_channel.lock().unwrap().clone();
        let close = self.close_receiver.clone();
        let (tx, rx) = sync_channel(128);
        self.tx = Some(tx);

        self.close_receiver.swap(false, Ordering::Relaxed);
        thread::spawn(move || {
            loop {
                if close.load(Ordering::Relaxed) {
                    close.swap(false, Ordering::Relaxed);
                    return;
                }
                match rx.recv() {
                    Ok(packet) => {
                        let (audio, _audio_handle) = create_player(Input::new(
                            false,
                            Reader::from_memory(packet),
                            Codec::Pcm,
                            Container::Raw,
                            None));
                        match discord_channel.clone() {
                            Some(device) => {
                                let rt = Runtime::new().unwrap();
                                let mut call = rt.block_on(async {
                                    device.lock().await
                                });
                                call.play_only(audio);
                            },
                            None => {}
                        }
                    }
                    Err(_) => {
                        close.swap(false, Ordering::Relaxed);
                        return;
                    }
                }
            }
        });
    }

    pub fn set(&mut self, device: Arc<SerenityMutex<Call>>) {
        self.stop();
        let device = Arc::clone(&device);
        {
            let mut discord_channel = self.discord_channel.lock().unwrap();
            *discord_channel = Some(device);
        }
        self.start();
    }

    pub fn unset(&mut self) {
        self.stop();
        {
            let mut discord_channel = self.discord_channel.lock().unwrap();
            *discord_channel = None;
        }
    }
}