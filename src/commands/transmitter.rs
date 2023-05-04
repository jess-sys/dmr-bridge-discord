use dmr_bridge_discord::packet::{PacketType, USRP};
use rodio::dynamic_mixer::{mixer, DynamicMixerController};
use serenity::async_trait;
use std::net::UdpSocket;
use std::sync::Arc;
use std::{env, thread, time};
use std::time::Duration;

use rodio::buffer::SamplesBuffer;
use rodio::source::UniformSourceIterator;

use songbird::{Event, EventContext, EventHandler as VoiceEventHandler};

pub struct Transmitter {
    mixer_controller: Arc<DynamicMixerController<i16>>,
}

pub struct TransmitterWrapper {
    transmitter: Arc<Transmitter>,
}

impl TransmitterWrapper {
    pub fn new(transmitter: Arc<Transmitter>) -> Self {
        Self { transmitter }
    }
}

#[async_trait]
impl VoiceEventHandler for TransmitterWrapper {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        self.transmitter.act(ctx).await
    }
}

impl Transmitter {
    pub fn new() -> Self {
        // You can manage state here, such as a buffer of audio packet bytes so
        // you can later store them in intervals.
        let dmr_target_rx_addr =
            env::var("TARGET_RX_ADDR").expect("Expected a target rx address in the environment");

        let socket =
            UdpSocket::bind("0.0.0.0:0").expect("Couldn't bind udp socket for transmission");

        socket
            .connect(dmr_target_rx_addr)
            .expect("Couldn't connect to DMR audio receiver");

        let (mixer_controller, mixer) = mixer(2, 48000);

        thread::spawn(move || {
            let mut sequence = 0;
            let mut mixer = mixer.peekable();
            let mut last_time_streaming_audio = time::Instant::now();
            let mut talked_since_last_time_streaming_audio = false;
            let mut discord_voice_buffer = [0i16; 1920];
            let mut discord_voice_buffer_index = 0;
            let mut usrp_voice_buffer = [0i16; 160];

            loop {
                if talked_since_last_time_streaming_audio
                    && last_time_streaming_audio
                        .duration_since(time::Instant::now())
                        .as_millis()
                        > 20
                {
                    talked_since_last_time_streaming_audio = false;
                    const EMPTY_VOICE_BUFFER: [i16; 160] = [0i16; 160];
                    let usrp_packet = USRP {
                        sequence_counter: sequence,
                        stream_id: 0,
                        push_to_talk: false,
                        talk_group: 0,
                        packet_type: PacketType::Voice,
                        multiplex_id: 0,
                        reserved: 0,
                        audio: EMPTY_VOICE_BUFFER,
                    }
                    .to_buffer();
                    socket
                        .send(&usrp_packet)
                        .expect("Failed to send USRP voice end packet");
                    println!("Sent USRP voice end packet");
                    sequence += 1;
                }
                if mixer.peek().is_none() {
                    const TWO_MILLIS: Duration = Duration::from_millis(2);
                    thread::sleep(TWO_MILLIS);
                    continue;
                }
                while discord_voice_buffer_index < discord_voice_buffer.len() {
                    let sample = mixer.next();
                    if let Some(sample) = sample {
                        discord_voice_buffer[discord_voice_buffer_index] = sample;
                    } else {
                        break;
                    }
                    discord_voice_buffer_index += 1;
                }
                if discord_voice_buffer_index == discord_voice_buffer.len() {
                    println!("Received Discord voice audio packet");
                    discord_voice_buffer_index = 0;
                    let source = SamplesBuffer::new(2, 48000, discord_voice_buffer.to_vec());
                    let mut source = UniformSourceIterator::new(source, 1, 8000);
                    for sample in usrp_voice_buffer.iter_mut() {
                        *sample = source
                            .next()
                            .expect("Unreachable: buffer does not have enough samples");
                    }
                    let usrp_packet = USRP {
                        sequence_counter: sequence,
                        stream_id: 0,
                        push_to_talk: true,
                        talk_group: 0,
                        packet_type: PacketType::Voice,
                        multiplex_id: 0,
                        reserved: 0,
                        audio: usrp_voice_buffer,
                    };
                    sequence += 1;
                    socket
                        .send(&usrp_packet.to_buffer())
                        .expect("Failed to send USRP voice audio packet");
                    println!("Sent USRP voice audio packet");
                    last_time_streaming_audio = time::Instant::now();
                    talked_since_last_time_streaming_audio = true;
                } else {
                    println!("Received incomplete Discord voice audio packet");
                }
            }
        });

        println!("Transmitter started");

        Self { mixer_controller }
    }
}

#[async_trait]
impl VoiceEventHandler for Transmitter {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        use EventContext as Ctx;
        match ctx {
            Ctx::VoicePacket(data) => {
                // An event which fires for every received audio packet,
                // containing the decoded data.
                if let Some(audio) = data.audio {
                    if !audio.is_empty() {
                        self.mixer_controller
                            .add(SamplesBuffer::new(2, 48000, audio.clone()));
                    }
                } else {
                    println!("RTP packet, but no audio. Driver may not be configured to decode.");
                }
            }
            _ => {
                // We won't be registering this struct for any more event classes.
                unimplemented!()
            }
        }

        None
    }
}
