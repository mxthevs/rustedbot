#[derive(Debug, Clone)]
pub enum Subject {
    OCaml,
}

impl Subject {
    pub fn from_string(message: &str) -> Option<Subject> {
        if message.to_ascii_lowercase().contains("ocaml") {
            Some(Subject::OCaml)
        } else {
            None
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

    pub fn get_response(&self) -> String {
        match self.subject {
            Some(Subject::OCaml) => {
                let words = self.content.split(' ').collect::<Vec<&str>>();
                let ocaml_index = words
                    .iter()
                    .position(|&word| word.to_ascii_lowercase() == "ocaml");

                if let Some(index) = ocaml_index {
                    let ocaml = words[index..].join(" ");

                    if ocaml != "OCaml" {
                        return format!("@{0} Não é {ocaml}, é OCaml.", self.sender);
                    }
                }
            }
            None => {}
        }

        String::from("")
    }
}
