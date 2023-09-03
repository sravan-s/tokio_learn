pub struct Config {
    pub server: String,
    pub port: String,
}

impl Config {
    pub fn default() -> Self {
        Self {
            server: "127.0.0.1".to_string(),
            port: "8080".to_string(),
        }
    }
}

pub struct Message {
    pub username: String,
    pub id: String,
    pub msg: String,
}

impl Message {
    // I know these delimiters are not the best, but I'm lazy
    // Donot want to bring in serde or any other dependency
    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", self.username, self.id, self.msg)
    }
    pub fn new(username: String, id: String, msg: String) -> Self {
        Self { username, id, msg }
    }
    pub fn from_string(msg: String) -> Self {
        let mut split = msg.split(':');
        let user = split.next().unwrap();
        let username = match user {
            "0" => "Admin".to_string(),
            _ => user.to_string(),
        };
        let id = split.next().unwrap().to_string();
        let msg = split.next().unwrap().to_string();
        Self { username, id, msg }
    }
    pub fn pretty_print(&self) -> String {
        let username = match self.id.as_str() {
            "0" => format!("{} (Admin)", self.username),
            _ => self.username.clone(),
        };
        format!("{}: {}", username, self.msg)
    }
}
