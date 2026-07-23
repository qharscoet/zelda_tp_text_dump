use std::{cmp::min, fs::File, io::{self, Read}, path::Path, ops::Range, str::Utf8Error};

use thiserror::Error;

use crate::{message::MessageParser, utils};

#[derive(Error, Debug)]
pub enum MSBTParseError {
    #[error("Error Reading String values")]
    InvalidSectionID(#[from] Utf8Error),

    #[error("unknown data store error")]
    UnknownSectionID,

    #[error("block offset is outside file")]
    #[allow(dead_code)]
    OffsetOutOfBounds
}

#[derive(Default)]
struct LMSHeader {
    magic: String,
    big_endian : bool,
    _unknown : u16,
    encoding: u8,
    version : u8,
    blocks_cnt: u16,
    _unknown2 : u16,
    filesize: u32,
}

enum MSBTBlockData {
    LBL1,
    TXT2,
    ATR1,
}

struct MSBTBlock {
    block_type: String,
    size: u32,
    range: Range<usize>,
    data : MSBTBlockData
}

struct MSBTData {
    header : LMSHeader,
    blocks : [Option<MSBTBlock>; MSBTData::SECTION_COUNT],
}


impl MSBTData {
    const SECTION_COUNT : usize = 4;
    const LBL1 : usize = 0;
    const TXT2 : usize = 1;
    const ATR1 : usize = 2;
    const TSY1 : usize = 3;

    fn get_idx(type_str : &str) -> Option<usize> {
        match type_str {
            "LBL1" => Some(MSBTData::LBL1),
            "TXT2" => Some(MSBTData::TXT2),
            "ATR1" => Some(MSBTData::ATR1),
            "TSY1" => Some(MSBTData::TSY1),
            _ => None
        }
    }
}

pub struct MSBTParser {
    _data: Vec<u8>,
    data_parsed: MSBTData,
}

impl MSBTParser {
    fn new(data: Vec<u8>) -> Self {

        let parsed = MSBTParser::parse_data(&data).unwrap_or(MSBTData { header: Default::default(), blocks: [const {None}; MSBTData::SECTION_COUNT] });

        
       MSBTParser { _data : data, data_parsed: parsed}
    }

    fn parse_data(data: &[u8]) -> Result<MSBTData, MSBTParseError>{
        
        let header = MSBTParser::parse_header(data)?;

        let big_endian = header.big_endian;
        let get_u32 = if big_endian { utils::get_u32_be } else {utils::get_u32_le};

        let mut blocks = [const {None}; MSBTData::SECTION_COUNT];
        let mut offset = 0x20;

        for i in 0..header.blocks_cnt {
            if offset + 4 < data.len()
            {
                let section_type = str::from_utf8(&data[offset..offset + 4])?.to_string();
                let section_size = get_u32(&data, offset + 4);
    
                let range_start = offset + 8;
                let range_end = min(range_start + section_size as usize, data.len()); //size includes the header
                let range = range_start..range_end as usize;
                
                if let Some(idx) = MSBTData::get_idx(&section_type) {
                    blocks[idx] = Some(MSBTBlock {
                        block_type : section_type,
                        size : section_size,
                        range : range.clone(),
                        data: MSBTParser::parse_section(&data[offset..range_end], big_endian)?
                    });
                }
            
                let block_end = offset + 0x10 + section_size as usize; //0x10 is block header size
                offset = ((block_end + 16) / 16) * 16; //next 16-bytes aligned address
            } else {
                println!("Invalid offset for section {i}");
            }
        }

        Ok(MSBTData { header, blocks})
    }

    fn parse_header(data : &[u8]) -> Result<LMSHeader, MSBTParseError> {
        let big_endian = utils::get_u16_be(&data, 0x8) == 0xFEFF;

        let get_u32 = if big_endian { utils::get_u32_be } else {utils::get_u32_le};
        let get_u16 = if big_endian { utils::get_u16_be } else {utils::get_u16_le};

        Ok(LMSHeader {
            magic: str::from_utf8(&data[0..8])?.to_string(),
            big_endian : big_endian,
            _unknown : get_u16(data, 0xA),
            encoding : data[0xC],
            version : data[0xD],
            blocks_cnt : get_u16(data, 0xE),
            _unknown2 : get_u16(data, 0x10),
            filesize: get_u32(&data, 0x12),
        })
    }

    fn parse_section(data : &[u8], big_endian : bool) -> Result<MSBTBlockData, MSBTParseError> {

        let get_u32 = if big_endian { utils::get_u32_be } else {utils::get_u32_le};
        let get_u16 = if big_endian { utils::get_u16_be } else {utils::get_u16_le};

        let section_type = str::from_utf8(&data[0..4])?;
        let section_size = get_u32(&data, 4);

        let range_start = 0x10;
        let range_end = min(range_start + section_size as usize, data.len()); //size includes the header
        let range = range_start..range_end as usize;


        let section_data = &data[range];

        match section_type {
            "LBL1" => Ok(MSBTBlockData::LBL1),
            "TXT2" => Ok(MSBTBlockData::TXT2),
            "ATR1" => Ok(MSBTBlockData::ATR1),
            _ => Err(MSBTParseError::UnknownSectionID)
        }

    }

    #[allow(dead_code)]
    fn get_header(&self) -> &LMSHeader {
        &self.data_parsed.header
    }

    #[allow(dead_code)]
    fn get_blocks(&self) -> &[Option<MSBTBlock>; MSBTData::SECTION_COUNT] {
        &self.data_parsed.blocks
    }

    fn get_block(&self, idx : usize) -> Option<&MSBTBlockData> {
        if let Some(section) = &self.data_parsed.blocks[idx] {
            Some(&section.data)
        } else {
            None
        }
    }

    fn print(&self) {
        let header = self.get_header();
        println!("Magic: {}", header.magic);
        println!("Endian: {}", header.big_endian);
        println!("Version: {}", header.version);
        println!("Filesize: {}", header.filesize);
        println!("Blocks count: {}", header.blocks_cnt);
        println!("Encoding: {}", header.encoding);


        for section in self.get_blocks().iter().flatten() {
            println!("Section type: {}", section.block_type);
            println!("Section size: {}", section.size);
            println!("Section data range: {:X?}", section.range);
        }
        
    }
}

impl MessageParser for MSBTParser {
    fn get_all_messages(&self) -> Vec<crate::message::MessageSingleLang> {
        todo!()
    }

    fn get_encoding(&self) -> &'static encoding_rs::Encoding {
        todo!()
    }
}


pub fn open_msbt(filename: &Path) -> io::Result<MSBTParser> {
    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(MSBTParser::new(buffer))
}


#[allow(dead_code)]
pub fn print_msbt(path : &Path) {
    match open_msbt(path) {
        Ok(parser) => {
            parser.print();
            // parser.print_flow();
            // println!("Message 0x66 : {}", get_raw_msg(parser.get_msg(0x66).text));
        }
        Err(e) => {
            eprintln!("Error opening BMG file: {}", e);
        }
    }
}

