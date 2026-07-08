use std::fmt;

use itertools::Itertools;

use crate::utils::{get_u16, get_u32};

pub const LANGUAGES_COUNT : usize = 4;

#[derive(Debug, Default, Clone)]
pub struct MessageAttributes {
    pub payload : Vec<u8>,
}


impl MessageAttributes {
    fn _get_message_id(&self) -> u16 {
        get_u16(&self.payload, 0)
    }

    pub fn get_display_style(&self) -> u8 {
        self.payload[0x05]
    }

    fn _get_printing_style(&self) -> u8 {
        self.payload[0x06]
    }

    pub fn is_empty(&self) -> bool {
        self.payload.iter().all(|v| *v == 0)
    }
}

impl fmt::Display for MessageAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{:X?}", self.payload)
    }
}


pub type MessageText = Vec<TextPart>;
pub struct Message {
    pub text : [MessageText; LANGUAGES_COUNT],
    pub attribs : MessageAttributes,
    pub id : usize
}

#[derive(Default)]
pub struct MessageSingleLang {
    pub text : MessageText,
    pub attribs : MessageAttributes,
    pub id : usize
}


impl Message {
    pub fn is_empty(&self) -> bool {
        self.text.iter().all(|parts| parts.is_empty())
    }
}


pub fn get_raw_msg(msg : MessageText) -> String {
    msg.iter().map(|text_part| match text_part {
            TextPart::Text(s) => s.to_string(),
            TextPart::Tag(t) => t.get_simple_replacement().to_string()
        }).join("")
}



#[derive(Default, Clone)]
pub struct Tag {
    pub group : u8,
    pub number : u16,
    pub payload : Vec<u8>
}

#[derive(Clone)]
pub enum TextPart {
    Text(String),
    Tag(Tag)
}

impl fmt::Display for TextPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextPart::Text(s) => write!(f, "{}", s),
            TextPart::Tag(t) => write!(f, "{}", t),
        }
    }
}

impl fmt::Debug for TextPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextPart::Text(s) => write!(f, "Text : {}", s),
            TextPart::Tag(t) => write!(f, "Tag : {:?}", t),
        }
    }
}

impl From<&Vec<u8>> for Tag {
    fn from(value: &Vec<u8>) -> Self {
        if value.len() > 2 {
            Tag {
                group : value[0],
                number : get_u16(&value, 0x1),
                payload : Vec::from(&value[0x03..]),
            }
        } else {
            Tag::default()
        }
    }
}