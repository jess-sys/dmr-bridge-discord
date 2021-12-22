use std::env;
use std::sync::{Arc, Mutex, mpsc::{self, Sender, Receiver}};
use serenity::prelude::Mutex as SerenityMutex;
use songbird::Call;
use std::collections::HashMap;
use std::thread;
use tokio::task::JoinHandle;
use tokio::net::UdpSocket as UdpSocketAsync;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};


use byteorder::{ByteOrder, BigEndian, LittleEndian};

pub struct Transmitter {
    socket: UdpSocketAsync,
    channels: Mutex<HashMap<String, Arc<SerenityMutex<Call>>>>,
    update: AtomicBool,
    sender: Sender<>,
    receiver: Receiver<>
}

async fn runTransmitter(transmitter: Arc<&mut Transmitter>) {
    let mut buffer = [0u8; 352];

    loop {
        socket.recv(&mut buffer).await;
    }
}

impl Transmitter {
    pub fn new() -> Self {
        // You can manage state here, such as a buffer of audio packet bytes so
        // you can later store them in intervals.
        let socket = UdpSocket::bind("127.0.0.1:0")
            .expect("Couldn't bind udp socket for discord's audio transmitter");
        socket.connect(env::var("DMR_TARGET_TX_ADDR")
                .expect("Expected a target tx address in the environment"))
            .expect("Couldn't connect to DMR's audio receiver");
        let async_socket = UdpSocketAsync::from_std(socket)
            .expect("Failed to convert udp socket for discord's audio transmitter to async socket");
        let process = tokio::spawn(async {});
        process.abort();
        thread::spawn()

        Self { 
            socket: async_socket,
            channels: Mutex::new(HashMap::new()),
            update: AtomicBool::new(false),
            process: process
        }
    }

    fn kill(&mut self) {
        self.process.abort();
    }

    pub fn add(&mut self, id: String, device: Arc<SerenityMutex<Call>>) {
        let channels = self.channels.lock().await();
        channels.insert(id, device);
        self.update.store(true, Ordering::Relaxed);
        if channels.len() == 1 {
            let transmitter = Arc::new(self);
            let channels = Arc::new(self.channels);
            let update = Arc::new(self.update);
            self.process = tokio::spawn(async move {
                runTransmitter(transmitter.clone());
            });
        }
    }

    pub fn sub(&mut self, id: String) {
        let channels = self.channels.lock().unwrap();
        channels.remove(&id);
        self.update.store(true, Ordering::Relaxed);
        if channels.len() == 0 {
            self.kill();
        }
    }
}