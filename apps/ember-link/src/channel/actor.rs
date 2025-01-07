use tokio::sync::mpsc::{self};

#[derive(Clone)]
pub struct ChannelHandle {
    pub sender: mpsc::UnboundedSender<ChannelMessage>,
}

#[derive(Debug)]
pub enum ChannelMessage {
    TestMessage { message: String },
}

pub struct Channel {
    receiver: mpsc::UnboundedReceiver<ChannelMessage>,
}

impl ChannelHandle {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let channel = Channel::new(receiver);

        tokio::spawn(run_channel(channel));

        Self { sender }
    }
}

impl Channel {
    pub fn new(receiver: mpsc::UnboundedReceiver<ChannelMessage>) -> Self {
        Self { receiver }
    }

    async fn handle_message(&mut self, msg: ChannelMessage) {
        match msg {
            ChannelMessage::TestMessage { message } => {
                println!("Test message {}", message);
            }
        }
    }
}

async fn run_channel(mut channel: Channel) {
    loop {
        tokio::select! {
            opt_msg = channel.receiver.recv() => {
                let msg = match opt_msg {
                    Some(msg) => msg,
                    None => break,
                };

                channel.handle_message(msg).await
            }
        }
    }

    println!("Channel finished");
}
