use futures_util::SinkExt;
use futures_util::{future, StreamExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:9000").await.unwrap();

    while let Ok((stream, _)) = listener.accept().await {
        let handle = tokio::spawn(accept_connection(stream));
        handle.await?;
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    println!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    println!("New WebSocket connection: {}", addr);

    let (mut write, mut read) = ws_stream.split();

    write.send(Message::Ping("".into())).await.unwrap();
    write
        .send(Message::Text("TEST MESSAGE".into()))
        .await
        .unwrap();

    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(msg) => {
                        match msg {
                            Ok(msg) => match msg {
                                Message::Ping(data) => write.send(Message::Pong(data)).await.unwrap(),
                                Message::Pong(data) => println!("Received pong"),
                                Message::Text(data) => {
                                    let test = data.to_string();

                                    if test == "ping" {
                                        write.send(Message::Text("pong".into())).await.unwrap();
                                    }
                                },
                                Message::Binary(data) => {
                                    match automerge::sync::Message::decode(&data) {
                                        Ok(message) => println!("{:?}", message.version),
                                        Err(error) => println!("{}", error.to_string())
                                    }
                                }
                                _ => break,
                            }
                            Err(error) => {
                                println!("{}", error.to_string());
                                break;
                            }
                        };
                    }
                    None => break,
                }
            }
        }
    }

    println!("Disconnected");
}
