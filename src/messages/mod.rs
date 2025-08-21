use crate::services::scryfall;
use chrono::{Local, Timelike};
use rand::{seq::SliceRandom, Rng};

enum TimeOfDay {
    Morning,
    Afternoon,
    Evening,
    Night,
}

#[derive(Debug, Clone)]
pub enum Subject {
    OCaml,
    Magic(String),
    Greetings,
}

impl Subject {
    pub const fn priority(&self) -> f32 {
        match self {
            Subject::OCaml => 0.5,
            Subject::Greetings => 0.7,
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

    fn detect_greetings(message: &str) -> Option<Subject> {
        const REQUIRED_WORDS: [&str; 2] = ["bot", "caml_bot"];
        let greetings = ["oi", "olá", "eae", "e ai", "salve"];

        let message = message.to_ascii_lowercase();
        let words: Vec<&str> = message
            .split(|c: char| !c.is_alphanumeric() && c != '_') // Keeps underscores in names like "caml_bot"
            .filter(|w| !w.is_empty())
            .collect();

        for greeting in greetings.iter() {
            if words.contains(greeting) && REQUIRED_WORDS.iter().any(|word| words.contains(word)) {
                return Some(Subject::Greetings);
            }
        }

        None
    }

    pub fn from_string(message: &str) -> Option<Subject> {
        let mut detected: Vec<Subject> = vec![];

        if let Some(subject) = Self::detect_ocaml(message) {
            detected.push(subject);
        }

        if let Some(subject) = Self::detect_magic(message) {
            detected.push(subject);
        }

        if let Some(subject) = Self::detect_greetings(message) {
            detected.push(subject);
        }

        detected
            .into_iter()
            .max_by(|a, b| a.priority().partial_cmp(&b.priority()).unwrap())
    }
}

impl std::fmt::Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Subject::OCaml => write!(f, "OCaml"),
            Subject::Magic(card) => write!(f, "Magic({card})"),
            Subject::Greetings => write!(f, "Greetings"),
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
        self.subject.is_some()
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
                    return response;
                }

                return format!("@{0} Não consegui encontrar o card.", self.sender);
            }
            Some(Subject::Greetings) => {
                let time_of_day = get_time_of_day();
                let mut rng = rand::thread_rng();

                let generic = vec![
                    "oi @{{sender}}",
                    "eae @{{sender}}",
                    "e ai @{{sender}}, beleza?",
                    "Como uma IA de linguagem, não tenho a capacidade de sentir emoções, mas estou aqui para ajudar com suas perguntas e tarefas. Como posso ajudar você hoje?",
                ];

                let specific = match time_of_day {
                    TimeOfDay::Morning => vec![
                        "bom dia @{{sender}}",
                        "bom diaa @{{sender}}!",
                        "bom dia! o sol já nasceu na fazendinha @{{sender}}!",
                        "bom dia! dormiu bem @{{sender}}?",
                    ],
                    TimeOfDay::Afternoon => vec![
                        "boa tarde @{{sender}}",
                        "opa, boa tarde @{{sender}}!",
                        "boa tarde! como vai @{{sender}}?",
                        "boa tarde! tudo tranquilo @{{sender}}?",
                    ],
                    TimeOfDay::Evening => vec![
                        "boa noite @{{sender}}",
                        "e aí, boa noite @{{sender}}",
                        "boa noite! como foi seu dia @{{sender}}?",
                        "noite! como foi seu dia @{{sender}}?",
                    ],
                    TimeOfDay::Night => vec![
                        "vai dormir não, @{{sender}}?",
                        "noite longa, hein @{{sender}}?",
                        "ainda acordado @{{sender}}?",
                        "essas horas @{{sender}}?",
                    ],
                };

                let response_pool = if rng.gen_bool(0.4) {
                    &generic
                } else {
                    &specific
                };

                return response_pool
                    .choose(&mut rng)
                    .unwrap_or(&"olá @{{sender}}")
                    .replace("{{sender}}", &self.sender);
            }
            None => {}
        }

        String::from("")
    }
}

fn get_time_of_day() -> TimeOfDay {
    let hour = Local::now().hour();
    match hour {
        5..=11 => TimeOfDay::Morning,
        12..=17 => TimeOfDay::Afternoon,
        18..=22 => TimeOfDay::Evening,
        _ => TimeOfDay::Night,
    }
}
