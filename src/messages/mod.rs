use crate::services::scryfall;

#[derive(Debug, Clone)]
pub enum Subject {
    OCaml,
    Magic(String),
}

impl Subject {
    pub const fn priority(&self) -> f32 {
        match self {
            Subject::OCaml => 0.5,
            Subject::Magic(_) => 1.,
        }
    }

    fn detect_ocaml(message: &str) -> Option<Subject> {
        if message.to_ascii_lowercase().contains("ocaml") {
            Some(Subject::OCaml)
        } else {
            None
        }
    }

    fn detect_magic(message: &str) -> Option<Subject> {
        if message.contains("[[") && message.contains("]]") {
            let start = message.find("[[").unwrap();
            let end = message.find("]]").unwrap();

            if start < end {
                let card = &message[start + 2..end];
                Some(Subject::Magic(String::from(card)))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn from_string(message: &str) -> Option<Subject> {
        let mut detected: Vec<Subject> = vec![];

        if let Some(subject) = Self::detect_ocaml(message) {
            detected.push(subject);
        }

        if let Some(subject) = Self::detect_magic(message) {
            detected.push(subject);
        }

        detected
            .into_iter()
            .max_by(|a, b| a.priority().partial_cmp(&b.priority()).unwrap())
            .map(|s| s)
    }
}

impl std::fmt::Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Subject::OCaml => write!(f, "OCaml"),
            Subject::Magic(card) => write!(f, "Magic({card})"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub sender: String,
    pub content: String,
    pub subject: Option<Subject>,
}

impl Message {
    pub fn make(message: &str, sender: &str) -> Message {
        Message {
            sender: String::from(sender),
            content: String::from(message),
            subject: Subject::from_string(message),
        }
    }

    pub fn has_subject(&self) -> bool {
        match self.subject {
            Some(_) => true,
            None => false,
        }
    }

    pub async fn get_response(&self) -> String {
        match &self.subject {
            Some(Subject::OCaml) => {
                let ocaml = self
                    .content
                    .split_whitespace()
                    .find(|word| word.eq_ignore_ascii_case("ocaml"))
                    .expect("OCaml word not found, but subject was detected");

                if ocaml != "OCaml" {
                    return format!("@{0} Não é {ocaml}, é OCaml.", self.sender);
                }
            }
            Some(Subject::Magic(card)) => {
                let response = scryfall::get_card(card.to_string()).await;

                if let Some(response) = response {
                    return format!("{response}");
                }

                return format!("@{0} Não consegui encontrar o card.", self.sender);
            }
            None => {}
        }

        String::from("")
    }
}
