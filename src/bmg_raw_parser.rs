use std::{
    cmp::min, fmt::{self, Display}, fs::File, io::{self, Read}, ops::Range, path::Path, str::Utf8Error,
};

use itertools::Itertools;
use thiserror::Error;

use crate::bmg_message::{MessageAttributes, MessageSingleLang, MessageText, Tag, TextPart, get_raw_msg};
use crate::utils::{get_u16, get_u32};


#[derive(Error, Debug)]
pub enum BMGParseError {
    #[error("Error Reading String values")]
    InvalidSectionID(#[from] Utf8Error),

    #[error("unknown data store error")]
    Unknown,
}

#[derive(Clone)]
struct BMGHeader {
    magic: String,
    filesize: u32,
    sections_cnt: u32,
    encoding: u8,
}


struct INF1Entry {
    offset: u32,
    attributes: Vec<u8>,
}

impl fmt::Display for INF1Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Offset: {}, Attributes: {:X?}", self.offset, self.attributes)
    }
}

struct INF1Data {
    count : u16,
    entry_size : u16,
    pad : u32,
    entries : Vec<INF1Entry>,
}

struct DAT1Data {
    data : Vec<u8>,
}

impl DAT1Data {
    
    fn get_msg_at(&self,offset:usize, encoding : u8) -> MessageText {

        if self.data[offset] == 0x00 {
            return Vec::new();
        }
        
        let encoding = match encoding {
            1 => encoding_rs::WINDOWS_1252,
            3 => encoding_rs::SHIFT_JIS,
            _ => encoding_rs::WINDOWS_1252, // Default to WINDOWS_1252 if unknown
        };

        let mut it = self.data[offset..].iter().peekable();
        let mut end = false;
        let mut full_string = String::new();
        let mut text_parts : Vec<TextPart> = Vec::new();
        while !end {
            let str_bytes = it.peeking_take_while(|&&b| b!=0x00 && b!=0x1A).map(|b| *b).collect::<Vec<_>>();
            let str = encoding.decode(&str_bytes).0;

            full_string += &str;
            text_parts.push(TextPart::Text(str.to_string()));
            
            match it.next().unwrap_or(&0x00) {
                0x00 => end = true,
                0x1A => {
                    let size_bytes = it.next().unwrap_or(&0x00);
                    let text_tag = it.by_ref().take(*size_bytes as usize - 2).map(|b| *b).collect::<Vec<_>>(); //Accounting for the 0x1A and size byte
                    text_parts.push(TextPart::Tag(Tag::from(&text_tag)));
                },
                _ => {}
            }
        }

        text_parts
        
    }
}

struct MID1Data {
    count : u16,
    _format : u16,
    _unknown : u32,
    ids : Vec<u32>
}


enum BMGSectionData {
    INF1(INF1Data),
    DAT1(DAT1Data),
    MID1(MID1Data),
    FLW1,
    FLI1
}
struct BMGSection {
    section_type: String,
    size: u32,
    range: Range<usize>,
    data: BMGSectionData
}

struct BMGData {
    header: BMGHeader,
    sections: [Option<BMGSection>; BMGData::SECTION_COUNT],
}

impl BMGData {
    const SECTION_COUNT : usize = 6;
    const INF1 : usize = 0;
    const DAT1 : usize = 1;
    const MID1 : usize = 2;
    const STR1 : usize = 3;
    const FLW1 : usize = 4;
    const FLI1 : usize = 5;

    fn get_idx(type_str : &str) -> Option<usize> {
        match type_str {
            "INF1" => Some(BMGData::INF1),
            "DAT1" => Some(BMGData::DAT1),
            "MID1" => Some(BMGData::MID1),
            "STR1" => Some(BMGData::STR1),
            "FLW1" => Some(BMGData::FLW1),
            "FLI1" => Some(BMGData::FLI1),
            _ => None
        }
    }
}

pub struct BMGRawParser {
    data: Vec<u8>,
    data_parsed: BMGData,
}


impl BMGRawParser {
    fn new(data: Vec<u8>) -> Self {

        let parsed = BMGRawParser::parse_data(&data).unwrap_or_else(|_| BMGData {
            header: BMGHeader {
                magic: String::new(),
                filesize: 0,
                sections_cnt: 0,
                encoding: 0,
            },
            sections: [const { None } ;6],
        });

       BMGRawParser { data, data_parsed: parsed}
    }


    fn get_header(&self) -> &BMGHeader {
        &self.data_parsed.header
    }

    fn get_sections(&self) -> &[Option<BMGSection>; BMGData::SECTION_COUNT] {
        &self.data_parsed.sections
    }

    fn get_section(&self, idx : usize) -> Option<&BMGSectionData> {
        if let Some(section) = &self.data_parsed.sections[idx] {
            Some(&section.data)
        } else {
            None
        }
    }

    fn print(&self) {
        let header = self.get_header();
        println!("Magic: {}", header.magic);
        println!("Filesize: {}", header.filesize);
        println!("Sections count: {}", header.sections_cnt);
        println!("Encoding: {}", header.encoding);

        for section in self.get_sections().iter().flatten() {
            println!("Section type: {}", section.section_type);
            println!("Section size: {}", section.size);
            println!("Section data range: {:?}", section.range);

            match &section.data {
                BMGSectionData::INF1(INF1Data{count,entry_size, pad, entries})=> {
                    println!("\t INF1 Section: count={}, entry_size={}, entry_len={}", count, entry_size, entries.len());
                    println!("\t Entry 0: {}", entries[0]);
                    println!("\t Entry 1: {}", entries[1]);
                    println!("\t Entry 2: {}", entries[2]);
                },
                BMGSectionData::DAT1(dat1data) => {
                    println!("\t DAT1 Section: data length={}", dat1data.data.len());
                    println!("\t First 10 bytes: {:X?}", &dat1data.data[0..10]);
                    println!("\t First message: {:#?}", dat1data.get_msg_at(1, header.encoding));
                },
                BMGSectionData::MID1(mid1data) => {
                    println!("\t MID1 Section: count {}", mid1data.count);
                    println!("\t Length {}", mid1data.count);
                    println!("\t ID[0] : {}", mid1data.ids[0]);
                    println!("\t ID[1] : {}", mid1data.ids[1]);
                    println!("\t ID[2] : {}", mid1data.ids[2]);
                }
                _ => {}
            }
        }
    }


    fn parse_header(data : &[u8]) -> Result<BMGHeader, BMGParseError> {
        Ok(BMGHeader {
            magic: str::from_utf8(&data[0..8])?.to_string(),
            filesize: get_u32(&data, 8),
            sections_cnt: get_u32(&data, 0x0C),
            encoding: data[0x10],
        })
    }

    fn parse_section(data: &[u8], offset: usize) -> Result<BMGSectionData, BMGParseError> {
        let section_type = str::from_utf8(&data[offset..offset + 4])?;
        let section_size = get_u32(&data, offset + 4);

        let range_start = offset + 8;
        let range_end = min(offset + section_size as usize, data.len()); //size includes the header
        let range = range_start..range_end as usize;


        let section_data = &data[range];

        match section_type {
            "INF1" => {
                let count = get_u16(&section_data, 0x00);
                let entry_size = get_u16(&section_data,  0x02);
                let pad = get_u32(&section_data, 0x04);

                let entries = section_data[0x08..].chunks_exact(entry_size as usize).map(|entry| {
                    let entry_offset_value = get_u32(&entry, 0x00);
                    let attributes = entry[0x04..].to_vec();

                    INF1Entry{offset:entry_offset_value, attributes}
                }).collect();

                Ok(BMGSectionData::INF1(INF1Data {
                    count,
                    entry_size,
                    pad,
                    entries,
                }))
            }
            "DAT1" => {
                Ok(BMGSectionData::DAT1(DAT1Data { data: section_data.to_vec()}))
            },
            "MID1" => {
                let count = get_u16(&section_data, 0x00);
                let format = get_u16(&section_data, 0x02);
                let unknown  = get_u32(&section_data, 0x04);

                let ids : Vec<_> = section_data[0x08..].chunks_exact(4).map(|v| get_u32(v, 0)).collect();


                Ok(BMGSectionData::MID1(MID1Data { count, _format : format, _unknown: unknown, ids: ids}))
            }
            _ => Ok(BMGSectionData::FLW1), // Placeholder for other section types

        }


    }

    fn parse_data(data: &[u8]) -> Result<BMGData, BMGParseError> {
        let header = BMGRawParser::parse_header(&data)?;
        let mut sections = [const {None}; BMGData::SECTION_COUNT];
        let mut offset = 0x20;

        for _ in 0..header.sections_cnt {
            let section_type = str::from_utf8(&data[offset..offset + 4])?.to_string();
            let section_size = get_u32(&data, offset + 4);

            let range_start = offset + 8;
            let range_end = min(offset + section_size as usize, data.len()); //size includes the header
            let range = range_start..range_end as usize;
            
            if let Some(idx) = BMGData::get_idx(&section_type) {
                sections[idx] = Some(BMGSection {
                    section_type : section_type,
                    size : section_size,
                    range : range,
                    data: BMGRawParser::parse_section(data, offset)?
                });
            }
        
            offset += section_size as usize;
        }

        Ok(BMGData { header, sections })
    }


    pub fn get_msg(&self, idx : usize) -> MessageSingleLang {
        if let Some(BMGSectionData::INF1(inf1)) = self.get_section(BMGData::INF1) {
            if let Some(BMGSectionData::DAT1(dat1)) = self.get_section(BMGData::DAT1) {
                let inf1_entry = &inf1.entries[idx];
                let data = dat1.get_msg_at(inf1_entry.offset as usize, self.data_parsed.header.encoding);

                let id = if let Some(BMGSectionData::MID1(mid1)) = self.get_section(BMGData::MID1) {
                    mid1.ids[idx]
                } else { 0 } as usize;

                let attribs = &inf1_entry.attributes;

                MessageSingleLang {
                    id : id,
                    attribs : MessageAttributes{payload : attribs.clone()},
                    text : data
                }
            } else {
                MessageSingleLang::default()
            }
        } else {
            MessageSingleLang::default()
        }
    }

    pub fn get_all_messages(&self) -> Vec<MessageSingleLang> {
       if let Some(BMGSectionData::INF1(inf1)) = self.get_section(BMGData::INF1) {
            (0..inf1.count).map(|i| {
                self.get_msg(i as usize)
            }).collect()
       } else {
        Vec::new()
       }
    }
}


pub fn open_bmg(filename: &Path) -> Result<BMGRawParser, io::Error> {
    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(BMGRawParser::new(buffer))
}


pub fn print_bmg(path : &Path) {
    match open_bmg(path) {
        Ok(parser) => {
            parser.print();
            // println!("Message 0x66 : {}", get_raw_msg(parser.get_msg(0x66).text));
        }
        Err(e) => {
            eprintln!("Error opening BMG file: {}", e);
        }
    }
}
