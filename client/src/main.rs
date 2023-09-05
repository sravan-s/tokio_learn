use chrono::Local;
use shared::{Config, Message};
use std::error::Error;

use tokio::{
    io::{stdin, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    task::JoinError,
};

async fn recieve_messages(server: String) -> Result<(), JoinError> {
    let task = tokio::spawn(async move {
        let mut reader = TcpStream::connect(server).await.unwrap();
        let (mut reader, _writer) = reader.split();
        loop {
            let mut buf = [0; 1024];
            let n = match reader.read(&mut buf).await {
                Ok(n) => {
                    if n == 0 {
                        return;
                    }
                    n
                }
                Err(e) => {
                    eprintln!("Failed to read from socket: {}", e);
                    return;
                }
            };
            let msg = String::from_utf8(buf[..n].to_vec()).unwrap();
            let msg_str = Message::from_string(msg).pretty_print();
            let date = Local::now().format("%Y-%m-%d %H:%M:%S");
            println!("\n{}: {}", date, msg_str);
        }
    });
    task.await
}

async fn send_messages(server: String, username: String, id: String) -> Result<(), JoinError> {
    let task = tokio::spawn(async move {
        let mut reader = TcpStream::connect(server).await.unwrap();
        let (_reader, mut writer) = reader.split();
        loop {
            let mut line = String::new();
            let stdin = stdin();
            let mut reader = BufReader::new(stdin);
            reader.read_line(&mut line).await.unwrap();

            let msg = Message::new(username.to_string(), id.to_string(), line);
            let msg = msg.to_string();
            let _ = match writer.write_all(msg.as_bytes()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to write to socket: {}", e);
                    return;
                }
            };
        }
    });
    task.await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = format!("{}:{}", Config::default().server, Config::default().port);
    println!("Connect to chat, Enter your username");
    let mut username = String::new();
    std::io::stdin().read_line(&mut username).unwrap();
    let username = username.trim();

    println!("Enter your ID");
    let mut id = String::new();
    std::io::stdin().read_line(&mut id).unwrap();
    let id = id.trim();

    let login = Message::login(username.to_string(), id.to_string()).to_string();
    let mut stream = TcpStream::connect(server.clone()).await?;
    stream.write_all(login.as_bytes()).await?;

    loop {
        tokio::select! {
            e = recieve_messages(server.clone()) => {
                println!("Server disconnected {:?}", e);
                break;
            }
            e = send_messages(server.clone(), username.to_string(), id.to_string()) => {
                println!("Client closed {:?}", e);
                break;
            }
        }
    }
    Ok(())
}
