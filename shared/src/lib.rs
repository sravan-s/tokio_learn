use std::fmt;

const ADMIN_ID: &str = "0";
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

#[derive(Clone, Debug)]
pub struct Message {
    pub username: String,
    pub id: String,
    pub msg: String,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.username, self.id, self.msg)
    }
}

impl Message {
    // username:id:msg
    // I know these delimiters are not the best, but I'm lazy
    // Donot want to bring in serde or any other dependency
    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", self.username, self.id, self.msg)
    }
    pub fn new(username: String, id: String, msg: String) -> Self {
        Self { username, id, msg }
    }
    // username:id:msg
    pub fn from_string(msg: String) -> Self {
        let mut split = msg.split(':');
        let user = split.next().unwrap();
        let username = match user {
            ADMIN_ID => "Admin".to_string(),
            _ => user.to_string(),
        };
        let id = split.next().unwrap().to_string();
        let msg = split.next().unwrap().trim().to_string();
        Self { username, id, msg }
    }
    pub fn pretty_print(&self) -> String {
        let username = match self.id.as_str() {
            ADMIN_ID => format!("{} (Admin)", self.username),
            _ => self.username.clone(),
        };
        format!("{}: {}", username, self.msg)
    }

    pub fn shutdown(username: String, id: String) -> Self {
        Self::new(username, id, "$$shutdown".to_string())
    }

    pub fn is_shutdown(&self) -> bool {
        self.msg == "$$shutdown"
    }

    pub fn logout(username: String, id: String) -> Self {
        Self::new(username, id, "$$logout".to_string())
    }

    pub fn is_logout(&self) -> bool {
        self.msg == "$$logout"
    }

    pub fn login(username: String, id: String) -> Self {
        Self::new(username, id, "$$login".to_string())
    }

    pub fn is_login(&self) -> bool {
        self.msg == "$$login"
    }

    pub fn pretty_logged_in(&self) -> String {
        format!("{}:{}:logged in:{}", ADMIN_ID, ADMIN_ID, self.username)
    }
}
