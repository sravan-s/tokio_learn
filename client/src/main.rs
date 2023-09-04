use shared::{Config, Message};
use std::error::Error;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    task::JoinSet,
};

async fn recieve_messages(server: String) {
    let mut reader = TcpStream::connect(server).await.unwrap();
    let (mut reader, _writer) = reader.split();
    loop {
        let mut buf = [0; 1024];
        let n = reader.read(&mut buf).await.unwrap();
        if n == 0 {
            break;
        }
        let msg = String::from_utf8(buf[..n].to_vec()).unwrap();
        let msg_str = Message::from_string(msg).pretty_print();
        println!("{}", msg_str);
    }
}

async fn send_messages(server: String, username: String, id: String) {
    let mut reader = TcpStream::connect(server).await.unwrap();
    let (_reader, mut writer) = reader.split();
    loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();
        let msg = Message::new(username.to_string(), id.to_string(), buf);
        let msg = msg.to_string();
        writer.write_all(msg.as_bytes()).await.unwrap();
        println!("sending: {}", msg);
    }
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

    let mut set = JoinSet::new();

    // recieve messages
    {
        let server = server.clone();
        set.spawn(async move {
            recieve_messages(server).await;
        });
    }

    // send messages
    {
        let server = server.clone();
        let username = username.to_string().clone();
        let id = id.to_string().clone();
        set.spawn(async move {
            send_messages(server, username, id).await;
        });
    }

    while let Some(res) = set.join_next().await {
        let out = res?;
        println!("Disconnected {:?}", out);
        break;
        // ...
    }

    Ok(())
}
