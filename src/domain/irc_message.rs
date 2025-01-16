use regex::Regex;

pub enum IrcMessage {
    Ping(String),
    Numeric {
        code: String,
        user: String,
        info: String,
    },
    PrivMsg {
        username: String,
        message: String,
    },
    Join(String),
    NamesList(String),
    EndOfNames,
    Unknown(String),
}

impl From<String> for IrcMessage {
    fn from(line: String) -> Self {
        if line.starts_with("PING ") {
            if let Some(server) = line.strip_prefix("PING ") {
                let server = server.trim_start_matches(':').to_string();
                return IrcMessage::Ping(server);
            }
        }

        static NUMERIC_RE: &str =
            r"^:(?P<server>[^ ]+)\s(?P<code>\d{3})\s(?P<user>\S+)\s(?P<info>.*)$";
        let numeric = Regex::new(NUMERIC_RE).unwrap();

        if let Some(caps) = numeric.captures(&line) {
            let code = caps["code"].to_string();
            let user = caps["user"].to_string();
            let info = caps["info"].to_string();

            match code.as_str() {
                "353" => {
                    let re_353 = Regex::new(r"^=(?P<channel>#[^ ]+)\s:(?P<names>.*)$").unwrap();
                    if let Some(names_caps) = re_353.captures(&info) {
                        let names = names_caps["names"].to_string();
                        return IrcMessage::NamesList(names);
                    }
                }
                "366" => {
                    let re_366 = Regex::new(r"^(?P<channel>#[^ ]+)\s:(?P<info>.*)$").unwrap();
                    if let Some(_) = re_366.captures(&info) {
                        return IrcMessage::EndOfNames;
                    }
                }
                _ => {
                    return IrcMessage::Numeric { code, user, info };
                }
            }
        }

        static PRIVMSG_RE: &str =
            r"^:(?P<username>[^!]+)![^@]+@[^ ]+\sPRIVMSG\s(?P<channel>#[^ ]+)\s:(?P<message>.*)$";
        let privmsg = Regex::new(PRIVMSG_RE).unwrap();

        if let Some(caps) = privmsg.captures(&line) {
            let username = caps["username"].to_string();
            let message = caps["message"].to_string();
            return IrcMessage::PrivMsg { username, message };
        }

        static JOIN_RE: &str = r"^:(?P<username>[^!]+)![^@]+@[^ ]+\sJOIN\s(?P<channel>#[^ ]+)$";
        let join_re = Regex::new(JOIN_RE).unwrap();

        if let Some(caps) = join_re.captures(&line) {
            let username = caps["username"].to_string();
            return IrcMessage::Join(username);
        }

        IrcMessage::Unknown(line.to_string())
    }
}
