use rand::Rng;
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{self, Receiver, Sender};

struct Client {
    id: u32,
    name: String,
    ext_writer: OwnedWriteHalf,
    ext_reader: OwnedReadHalf,
    broadcast_sender: broadcast::Sender<String>,
    broadcast_reciever: broadcast::Receiver<String>,
}

impl Client {
    fn new(
        id: u32,
        name: String,
        stream: TcpStream,
        broadcast_sender: Sender<String>,
        broadcast_reciever: Receiver<String>,
    ) -> Client {
        let (ext_reader, ext_writer) = stream.into_split();
        Client {
            id,
            name,
            ext_reader,
            ext_writer,
            broadcast_sender,
            broadcast_reciever,
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // common broadcast channel
    let (sender, _) = broadcast::channel::<String>(10);

    // listens to port
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    loop {
        // someone connected
        let (mut reader, _address) = listener.accept().await?;
        let my_internal_sender = sender.clone();
        let my_internal_receiver = my_internal_sender.subscribe();
        // we spawn a new task to handle the connection
        tokio::spawn(async move {
            // create and set up user
            let id = rand::thread_rng().gen_range(0..1000);
            let name = id.to_string();
            let mut user = Client::new(id, name, reader, my_internal_sender, my_internal_receiver);
            println!("new user connected {} {}", user.name, user.id);
            // listen to broadcast channel and send to tcp stream
            tokio::spawn(async move {
                loop {
                    let msg = user.broadcast_reciever.recv().await.unwrap();
                    println!("broadcast message received for: {}", user.name);
                    user.ext_writer.write_all(msg.as_bytes()).await.unwrap();
                }
            });
            loop {
                let mut buf = [0; 1024];
                let n = user.ext_reader.read(&mut buf).await.unwrap();
                let msg = String::from_utf8(buf[0..n].to_vec()).unwrap();
                println!("TCP message received from: {}", user.id.clone());
                user.broadcast_sender.send(msg).unwrap();
            }
        });
    }
}
