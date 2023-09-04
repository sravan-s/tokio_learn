use shared::{Config, Message};
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::broadcast::{self};
use tokio::task::JoinSet;
use uuid::Uuid;

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

    'outer: loop {
        let mut set = JoinSet::new();

        let (my_socket, _) = listener.accept().await?;
        let (mut my_sock_rx, mut my_sock_tx) = my_socket.into_split();
        let my_tx = tx.clone();
        let mut my_rx = tx.subscribe();

        set.spawn(async move {
            let mut logged_in = false;
            loop {
                println!("In listener loop");
                let mut buf = vec![0; 1024];
                let n = match my_sock_rx.read(&mut buf).await {
                    Ok(n) => {
                        if n == 0 {
                            return;
                        }
                        n
                    }
                    Err(_) => {
                        eprintln!("Failed to read from socket");
                        return;
                    }
                };
                let message = match String::from_utf8(buf[..n].to_vec()) {
                    Ok(m) => Message::from_string(m.to_string()),
                    Err(_) => {
                        eprintln!("Failed to parse message");
                        return;
                    }
                };
                if !logged_in && message.is_login() {
                    println!("Received login message, {}", message);
                    let m = Message::new(
                        message.username.clone(),
                        Uuid::new_v4().to_string(),
                        "has joined the chat".to_string(),
                    );

                    let _ = match my_tx.send(m) {
                        Ok(_) => {
                            logged_in = true;
                        }
                        Err(e) => {
                            eprintln!("Failed to send message @69 {}", e);
                            return;
                        }
                    };
                    continue;
                }
                if message.is_logout() && logged_in {
                    let id = message.id.clone();
                    let username = message.username.clone();
                    let msg = format!("{} has left the chat", username);
                    let message = Message::new(username, id, msg);

                    let _ = match my_tx.send(message) {
                        Ok(_) => {
                            logged_in = false;
                        }
                        Err(e) => {
                            eprintln!("Failed to send message {}", e);
                            return;
                        }
                    };
                    continue;
                }

                if logged_in {
                    let id = message.id.clone();
                    let username = message.username.clone();
                    let msg = message.msg.clone();
                    let message = Message::new(username, id, msg);
                    println!("Received message: {}", message);
                    my_tx.send(message).unwrap();
                    continue;
                }
            }
        });

        set.spawn(async move {
            loop {
                println!("Waiting for broadcast message");
                let internal_message = match my_rx.recv().await {
                    Ok(m) => m.to_string(),
                    Err(e) => {
                        eprintln!("Failed to receive message @111 {}", e);
                        return;
                    }
                };
                println!("broadcast received: internal_message{}", internal_message);

                let _n = match my_sock_tx.write(internal_message.as_bytes()).await {
                    Ok(n) => {
                        println!("wrote to socket {}", n);
                        n
                    }
                    Err(_) => {
                        eprintln!("Failed to write to socket");
                        return;
                    }
                };
            }
        });

        while let Some(res) = set.join_next().await {
            println!("Task finished with result {:?}", res);
            break 'outer;
        }
    }
    Ok(())
}
