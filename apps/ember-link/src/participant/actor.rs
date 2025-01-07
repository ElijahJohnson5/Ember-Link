use tokio::sync::mpsc::{self};

#[derive(Clone)]
pub struct ParticipantHandle {
    pub sender: mpsc::UnboundedSender<ParticipantMessage>,
}

#[derive(Debug)]
pub enum ParticipantMessage {
    TestMessage { message: String },
}

pub struct Participant {
    receiver: mpsc::UnboundedReceiver<ParticipantMessage>,
}

impl ParticipantHandle {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let participant = Participant::new(receiver);

        tokio::spawn(run_participant(participant));

        Self { sender }
    }
}

impl Participant {
    pub fn new(receiver: mpsc::UnboundedReceiver<ParticipantMessage>) -> Self {
        Self { receiver }
    }

    async fn handle_message(&mut self, msg: ParticipantMessage) {
        match msg {
            ParticipantMessage::TestMessage { message } => {
                println!("Test message {}", message);
            }
        }
    }
}

async fn run_participant(mut participant: Participant) {
    loop {
        tokio::select! {
            opt_msg = participant.receiver.recv() => {
                let msg = match opt_msg {
                    Some(msg) => msg,
                    None => break,
                };

                participant.handle_message(msg).await
            }
        }
    }

    println!("Participant finished");
}
