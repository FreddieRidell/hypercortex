use rand::seq::IteratorRandom;
use std::cmp::{Eq, PartialEq};

const CHARS: &'static str = "23456789abcdefghkmnpqrstwxyz";
const NUMBER_OF_CHARS_IN_FULL_ID: u8 = 16;

#[derive(Debug, Clone)]
pub struct Id {
    content: String,
}

impl Id {
    pub fn generate() -> Self {
        Self {
            content: Id::get_easy_type_id(),
        }
    }

    pub fn new(content: &str) -> Self {
        Self {
            content: String::from(content),
        }
    }

    fn get_easy_type_id() -> String {
        let mut result = String::new();

        for _ in 0..NUMBER_OF_CHARS_IN_FULL_ID {
            let random = CHARS
                .chars()
                .choose(&mut rand::thread_rng())
                .expect("Couldn't get random char");

            result.push(random);
        }

        result
    }
}

impl PartialEq for Id {
    fn eq(&self, other: &Id) -> bool {
        self.content.contains(other.content.as_str())
            || other.content.contains(self.content.as_str())
    }
}

impl Eq for Id {}
