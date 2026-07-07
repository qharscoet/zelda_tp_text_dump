use std::{
    cmp::min, fmt::{self, Display}, fs::File, io::{self, Read}, ops::Range, str::Utf8Error,
};

use encoding_rs::mem::decode_latin1;
use thiserror::Error;

fn get_u32(data: &[u8], idx: usize) -> u32 {
    u32::from_be_bytes(data[idx..idx + 4].try_into().unwrap())
}

fn get_u16(data: &[u8], idx: usize) -> u16 {
    u16::from_be_bytes(data[idx..idx + 2].try_into().unwrap())
}

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
    
    fn get_text_at(&self,offset:usize, encoding : u8) -> String {
        let bytes : Vec<u8> = self.data[offset..].iter().take_while(|&&b| b != 0x00).map(|b| *b).collect();
        println!("Bytes at offset {}: {:X?}", offset, bytes);
        let encoding = match encoding {
            1 => encoding_rs::WINDOWS_1252,
            3 => encoding_rs::SHIFT_JIS,
            _ => encoding_rs::WINDOWS_1252, // Default to WINDOWS_1252 if unknown
        };
        let s = encoding.decode(&bytes);
        // Placeholder implementation
        format!("Text at offset {} : {}", offset, s.0)
    }
}

struct MID1Data {

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
    sections: Vec<BMGSection>,
}

struct BMGRawParser {
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
            sections: Vec::new(),
        });

       BMGRawParser { data, data_parsed: parsed}
    }


    fn get_header(&self) -> &BMGHeader {
        &self.data_parsed.header
    }

    fn get_sections(&self) -> &Vec<BMGSection> {
        &self.data_parsed.sections
    }

    fn print(&self) {
        let header = self.get_header();
        println!("Magic: {}", header.magic);
        println!("Filesize: {}", header.filesize);
        println!("Sections count: {}", header.sections_cnt);
        println!("Encoding: {}", header.encoding);

        for section in self.get_sections() {
            println!("Section type: {}", section.section_type);
            println!("Section size: {}", section.size);
            println!("Section data range: {:?}", section.range);

            match &section.data {
                BMGSectionData::INF1(INF1Data{count,entry_size, pad, entries})=> {
                    println!("\t INF1 Section: count={}, entry_size={}, pad={}", count, entry_size, pad);
                    println!("\t Entry 0: {}", entries[0]);
                    println!("\t Entry 1: {}", entries[1]);
                    println!("\t Entry 2: {}", entries[2]);
                },
                BMGSectionData::DAT1(dat1data) => {
                    println!("\t DAT1 Section: data length={}", dat1data.data.len());
                    println!("\t First 10 bytes: {:X?}", &dat1data.data[0..10]);
                    println!("\t First message: {}", dat1data.get_text_at(1, header.encoding));
                },
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

                println!("INF1 Section: count={}, entry_size={}, pad={}", count, entry_size, pad);
                let mut entries = Vec::new();
                let mut entry_offset = 0x08;

                for _ in 0..count {
                    let entry_offset_value = get_u32(&section_data, entry_offset);
                    let attributes = section_data[entry_offset + 4..entry_offset + entry_size as usize].to_vec();

                    entries.push(INF1Entry {
                        offset: entry_offset_value,
                        attributes,
                    });

                    entry_offset += entry_size as usize;
                }

                Ok(BMGSectionData::INF1(INF1Data {
                    count,
                    entry_size,
                    pad,
                    entries,
                }))
            }
            "DAT1" => {
                println!("DAT1 Section: size={}", section_data.len());
                println!("First 10 bytes: {:X?}", &data[offset..offset+10]);
                Ok(BMGSectionData::DAT1(DAT1Data { data: section_data.to_vec()}))
            }
            _ => Ok(BMGSectionData::FLW1), // Placeholder for other section types

        }


    }

    fn parse_data(data: &[u8]) -> Result<BMGData, BMGParseError> {
        let header = BMGRawParser::parse_header(&data)?;
        let mut sections = Vec::new();
        let mut offset = 0x20;

        for _ in 0..header.sections_cnt {
            let section_type = str::from_utf8(&data[offset..offset + 4])?.to_string();
            let section_size = get_u32(&data, offset + 4);

            let range_start = offset + 8;
            let range_end = min(offset + section_size as usize, data.len()); //size includes the header
            let range = range_start..range_end as usize;

            sections.push(BMGSection {
                section_type : section_type,
                size : section_size,
                range : range,
                data: BMGRawParser::parse_section(data, offset)?
            });
            offset += section_size as usize;
        }

        Ok(BMGData { header, sections })
    }
}


fn open_bmg(filename: &str, bank_index: usize) -> Result<BMGRawParser, io::Error> {
    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(BMGRawParser::new(buffer))
}


pub fn print_bmg(path : &str) {
    match open_bmg(path, 0) {
        Ok(parser) => {
            parser.print();
        }
        Err(e) => {
            eprintln!("Error opening BMG file: {}", e);
        }
    }
}
