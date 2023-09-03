use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

pub struct Message {
    id: i64,
    msg: String,
    from: String,
}

impl Message {
    pub fn new(id: i64, msg: String, from: String) -> Message {
        Message { id, msg, from }
    }

    pub fn serialize(&self) -> String {
        format!("{}:{}:{}", self.id, self.msg, self.from)
    }

    pub fn deserialize(msg: String) -> Message {
        let mut parts = msg.split(":");
        let id = parts.next().unwrap().parse::<i64>().unwrap();
        let msg = parts.next().unwrap().to_string();
        let from = parts.next().unwrap().to_string();
        Message { id, msg, from }
    }
}

pub struct SerializedClient {
    id: String,
    name: String,
}

pub struct Client {
    id: String,
    name: String,
    ext_writer: OwnedWriteHalf,
    ext_reader: OwnedReadHalf,
    broadcast_sender: broadcast::Sender<String>,
    broadcast_reciever: broadcast::Receiver<String>,
}
impl Client {
    fn new(
        id: String,
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

    fn to_serialized(&self) -> SerializedClient {
        SerializedClient {
            id: self.id.clone(),
            name: self.name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
