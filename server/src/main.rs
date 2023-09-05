use shared::{Config, Message};
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::sync::broadcast::{self};
use tokio::task::JoinError;

#[derive(Debug)]
enum ThreadResult {
    Err,
    Logout,
    Shutdown,
}

async fn listen_task(
    mut my_sock_rx: OwnedReadHalf,
    my_tx: broadcast::Sender<Message>,
) -> Result<ThreadResult, JoinError> {
    let task = tokio::spawn(async move {
        let mut logged_in = false;
        println!("logged in, line 23, {:?}", logged_in);
        loop {
            println!("Waiting for message 25 logged in: {:?}", logged_in);
            let mut buf = vec![0; 1024];
            let n = match my_sock_rx.read(&mut buf).await {
                Ok(n) => {
                    if n == 0 {
                        return ThreadResult::Err;
                    }
                    n
                }
                Err(_) => {
                    eprintln!("Failed to read from socket");
                    return ThreadResult::Err;
                }
            };
            let message = match String::from_utf8(buf[..n].to_vec()) {
                Ok(m) => Message::from_string(m.to_string()),
                Err(_) => {
                    eprintln!("Failed to parse message");
                    return ThreadResult::Err;
                }
            };

            // maybe -> later filter by ID :)
            // Also keep list of IDs in mutex and check if ID is duplicate

            // Shutdown message
            if message.is_shutdown() {
                println!("Received shutdown message, {}", message);
                return ThreadResult::Shutdown;
            }

            // Login message
            if !logged_in && message.is_login() {
                let m = Message::new(
                    message.username.clone(),
                    message.id.clone(),
                    "has joined the chat".to_string(),
                );

                let _ = match my_tx.send(m) {
                    Ok(_) => {
                        logged_in = true;
                    }
                    Err(e) => {
                        eprintln!("Failed to send message @69 {}", e);
                        return ThreadResult::Err;
                    }
                };
                println!("logged in: {}, line 72, calling continue", logged_in);
                continue;
            }

            // Logout message
            if message.is_logout() && logged_in {
                let id = message.id.clone();
                let username = message.username.clone();
                let msg = format!("{} has left the chat", username);
                let message = Message::new(username, id, msg);

                let _ = match my_tx.send(message) {
                    Err(e) => {
                        eprintln!("Failed to send message {}", e);
                        return ThreadResult::Err;
                    }
                    _ => {}
                };
                return ThreadResult::Logout;
            }

            println!("logged in: {}, line 92", logged_in);
            // normal message && logged in
            let id = message.id.clone();
            let username = message.username.clone();
            let msg = message.msg.clone();
            let message = Message::new(username, id, msg);
            my_tx.send(message).unwrap();
            continue;
        }
    });
    println!("listen_task finished");
    task.await
}

async fn transmit_taask(
    mut my_sock_tx: OwnedWriteHalf,
    mut my_rx: broadcast::Receiver<Message>,
) -> Result<ThreadResult, JoinError> {
    let task = tokio::spawn(async move {
        loop {
            let internal_message = match my_rx.recv().await {
                Ok(m) => m.to_string(),
                Err(e) => {
                    eprintln!("Failed to receive message @111 {}", e);
                    return ThreadResult::Err;
                }
            };

            let _n = match my_sock_tx.write(internal_message.as_bytes()).await {
                Ok(n) => n,
                Err(_) => {
                    eprintln!("Failed to write to socket");
                    return ThreadResult::Err;
                }
            };
        }
    });
    task.await
}

async fn connect_client(
    my_sock_rx: tokio::net::tcp::OwnedReadHalf,
    my_sock_tx: tokio::net::tcp::OwnedWriteHalf,
    my_tx: broadcast::Sender<Message>,
    my_rx: broadcast::Receiver<Message>,
) {
    let listen = listen_task(my_sock_rx, my_tx);
    let transmit = transmit_taask(my_sock_tx, my_rx);
    tokio::select! {
        rtrn = listen => {
            match rtrn {
                Ok(ThreadResult::Shutdown) => {
                    // todo: send shutdown message to all clients
                    println!("Shutdown");
                    return;
                },
                Ok(ThreadResult::Logout) => {
                    println!("Logout");
                    return;
                },
                e => {
                    println!("Error {:?}", e);
                    return;
                },
            }
        }
        rtrn = transmit => {
            println!("transmit_task finished {:?}", rtrn);
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // setting up server
    let config = Config::default();
    let address = format!("{}:{}", config.server, config.port);
    let listener = match TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Failed to bind to address: {}", e);
            return Ok(());
        }
    };
    // end-settingup server
    //
    // setting up broadcast channels
    let (tx, _rx) = broadcast::channel::<Message>(64000);

    loop {
        let (my_socket, _) = listener.accept().await?;
        println!("loop @181 {:?}", my_socket);
        let (my_sock_rx, my_sock_tx) = my_socket.into_split();
        let my_tx = tx.clone();
        let my_rx = tx.subscribe();
        tokio::spawn(async move {
            connect_client(my_sock_rx, my_sock_tx, my_tx, my_rx).await;
        });
    }
}
