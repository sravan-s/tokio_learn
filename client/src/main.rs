use shared::{Config, Message};
use std::error::Error;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    task::JoinSet,
};

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

    let login = format!("{}:{}", username, id);
    let mut stream = TcpStream::connect(server.clone()).await?;
    stream.write_all(login.as_bytes()).await?;

    let mut set = JoinSet::new();

    {
        let server = server.clone();
        set.spawn(async move {
            let mut reader = TcpStream::connect(server.clone()).await.unwrap();
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
        });
    }

    {
        let server = server.clone();
        set.spawn(async move {
            let mut reader = TcpStream::connect(server).await.unwrap();
            let (_reader, mut writer) = reader.split();
            loop {
                let mut buf = String::new();
                std::io::stdin().read_line(&mut buf).unwrap();
                writer.write_all(buf.as_bytes()).await.unwrap();
            }
        });
    }

    while let Some(res) = set.join_next().await {
        let out = res?;
        println!("Task completed with {:?}", out);
        // ...
    }

    Ok(())
}
