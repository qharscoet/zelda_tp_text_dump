use std::{fmt, fs::File, io::{BufRead, BufReader, Write}, path::Path};
use regex::Regex;
use rust_xlsxwriter::{Color, Format, FormatAlign};
use std::sync::LazyLock;
use itertools::Itertools;

mod bmg_raw_parser;

const BANK_COUNT : usize = 10;
const FILENAMES : [&str;BANK_COUNT] = [
    "zel_00",
    "zel_01",
    "zel_02",
    "zel_03",
    "zel_04",
    "zel_05",
    "zel_06",
    "zel_07",
    "zel_08",
    "zel_99",
];

const LANGUAGES_COUNT : usize = 4;

const LANGUAGES : [&str;LANGUAGES_COUNT] = [
    "jp",
    "us",
    "fr",
    // "sp",
    "de",
    // "it"
];

const LANGUAGES_FULL : [&str;LANGUAGES_COUNT] = [
    "Japanese",
    "US English",
    "French",
    // "Spanish",
    "German",
    // "Italian"
];

static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?P<ID>[[:xdigit:]]+) (@(?P<slot>[[:xdigit:]]{4}) )?(?P<attribs>\[.+\]) = (?P<str>.+)?").unwrap());
static RE_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\\[x|z]\{(.*?)\}").unwrap());

const COLORS_RGB : [&str; 9] = [
    "#FFFFFF",
    "#f07878",
    "#aadc8c",
    "#a0b4dc",
    "#dcdc82",
    "#b4c8e6",
    "#c8a0dc",
    "#ffffff",
    "#dcaa78",
];

fn unpack_u16(v:u16) -> (u8,u8) {
    (((v & 0xFF00) >> 8) as u8, (v & 0x00FF) as u8)
}

fn get_u16_from_payload(payload : &[u8], idx : usize) -> u16 {

    let v1 = payload[idx];
    let v2 = payload[idx + 1];

    ((v1 as u16) << 8 | v2 as u16) as u16 
}

#[derive(Debug)]
struct MessageAttributes {
    payload : [u8; 16],
}


impl MessageAttributes {
    fn _get_message_id(&self) -> u16 {
        get_u16_from_payload(&self.payload, 0)
    }

    fn get_display_style(&self) -> u8 {
        self.payload[0x05]
    }

    fn _get_printing_style(&self) -> u8 {
        self.payload[0x06]
    }

    fn is_empty(&self) -> bool {
        self.payload.iter().all(|v| *v == 0)
    }
}

impl Default for MessageAttributes {
    fn default() -> Self {
        MessageAttributes { payload: [0;16]}
    }
}

impl fmt::Display for MessageAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{:X?}", self.payload)
    }
}

impl std::str::FromStr for MessageAttributes {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut values = s.split_inclusive(&[',', '/']).fold(Vec::new(), |mut acc, s| {
            if s.ends_with(',') {
                acc.push(u8::from_str_radix(&s[0..s.len() -1], 16).unwrap_or_default());
            } else if s.ends_with('/') {
                acc.push(u8::from_str_radix(&s[0..s.len() -1], 16).unwrap_or_default());
                let curr_len = acc.len();
                let next_len = ((curr_len + 4)/4) * 4; //align to next 32
                acc.resize(next_len, 0);

            } else {
                acc.push(u8::from_str_radix(s, 16).unwrap_or_default());
            };
            acc
        });
        values.resize(16, 0);
        Ok(MessageAttributes { payload: values.try_into().unwrap_or([0;16])})
    }
}

#[derive(Default)]
struct Tag {
    group : u8,
    number : u16,
    payload : Vec<u8>
}

impl Tag {
    
    fn get_simple_replacement(&self) -> &str {
        match self.group {
            0x00 => {
                match self.number {
                    0x08 => "• ",
                    0x09 => "• ",
                    0x0A => "[A] ",
                    0x0B => "[B] ",
                    0x0C => "[C] ",
                    0x0D => "[L] ",
                    0x0E => "[R] ",
                    0x0F => "[X] ",
                    0x10 => "[Y] ",
                    0x11 => "[Z] ",
                    0x12 => "[DPad] ",
                    0x13 => "[Analog] ",
                    0x14 => "🡄 ",
                    0x15 => "🡆 ",
                    0x16 => "🡅 ",
                    0x17 => "🡇 ",
                    0x18 => "[AnalogUp] ",
                    0x19 => "[AnalogDown] ",
                    0x1A => "[AnalogLeft] ",
                    0x1B => "[AnalogRight] ",
                    0x1C => "[AnalogVertical] ",
                    0x1D => "[AnalogHorizontal] ",
                    0x23 => "[RedTarget] ",
                    0x24 => "[YellowTarget] ",
                    0x2E => "[XorY] ",
                    0x39 => "♥ ",
                    0x00 =>	"[Link]",
                    0x22 =>	"[Epona]",
                    0x29 =>	"[CurrentScent]",
                    0x2B =>	"[WarpingTo]",
                    0x2D =>	"[Bomb-Name]",
                    0x31 =>	"[Bomb-Count]",
                    0x32 =>	"[Bomb-Price]",
                    0x35 =>	"[nop000035]",
                    0x37 =>	"[Bombcap]",
                    0x3B =>	"[ReturnedBug]",
                    0x3C =>	"[LetterSender]",
                    0x3E =>	"[CurrentLetterPage]",
                    0x3F =>	"[MaxLetterPage]",
                    _ => ""
                }
            },
            0x03 => {
                match self.number {
                    0x01 =>	"[WiiA]",
                    0x02 =>	"[WiiB]",
                    0x03 =>	"[WiiHome]",
                    0x04 =>	"[WiiMinus]",
                    0x05 =>	"[WiiPlus]",
                    0x06 =>	"[Wii1]",
                    0x07 =>	"[Wii2]",
                    0x08 =>	"[WiiD-WE]",
                    0x09 =>	"[WiiD-N]",
                    0x0A =>	"[WiiD-S]",
                    0x0B =>	"[WiiD-WE]",
                    0x0C =>	"[WiiD-E]",
                    0x0D =>	"[WiiD-W]",
                    0x0E =>	"[Wiimote]",
                    0x0F =>	"[WReticule]",
                    0x10 =>	"[WNunchunk]",
                    0x11 =>	"[Wiimote]",
                    0x12 =>	"[Fairy]",
                    0x13 =>	"[WiiC]",
                    0x14 =>	"[WiiZ]",
                    _ => ""
                }
            },
            0x04 => {
                match self.number {
                    0x00 =>	"巫",
                    0x01 =>	"嗅",
                    0x02 =>	"眷",
                    0x03 =>	"蜀",
                    0x04 =>	"蟲",
                    0x05 =>	"裔",
                    0x06 =>	"惧",
                    0x07 =>	"綺",
                    0x08 =>	"罠",
                    0x09 =>	"祓",
                    0x0A =>	"墟",
                    0x0B =>	"絆",
                    0x0C =>	"僭",
                    0x0D =>	"憑",
                    _ => ""
                }
            },
            0x05 => {
                match self.number {
                    0x00 =>	"[Time]",
                    0x03 =>	if self.payload[0] == 0  {"[ReturnedBugs]" } else {"[RemainingBugs]"},
                    0x04 =>	"noop",
                    0x07 =>	"[RiverPoints]",
                    0x08 =>	"[FishLength]",
                    0x09 =>	"[MartGoalLef]",
                    0x0A =>	"[LetterCount]",
                    0x0B =>	"[PoesNeeded]",
                    0x0C =>	if self.payload[0] == 0 {"[LatestScore]" } else {"[HighScore]"},
                    0x0D =>	"[FishCount]",
                    0x0E =>	"[RollGoal]",
                    _ => ""
                }
            },
            0x06 => {
                match self.number {
                    0x02 => "♂",	
                    0x03 => "♀",	
                    0x04 => "★",	
                    0x05 => "※",	
                    0x06 => "←",	
                    0x07 => "→",	
                    0x08 => "↑",	
                    0x09 => "↓",	
                    0x0A => "⧫",
                    0x0B => " ",    
                    _ => "",
                }
            },
            _=> "",
        }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}", match self.group {
            0xFF => {
                match self.number {
                    0x00 => "[Color]",
                    0x01 => "[Size]",
                    0x02 => "[Ruby]",
                    _ => "",
                }
            },
            _ => self.get_simple_replacement()
        })
    }
}

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{:0>2X}/{:0>4X} payload : {:X?}", self.group, self.number, self.payload)
    }
}

impl std::str::FromStr for Tag {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        
        if let Some(caps) = RE_TAG.captures(s) {
            let args = caps.get(1).map_or("", |m| m.as_str());
            let values = args.split(",").collect::<Vec<_>>();

            let x_escape = s.starts_with(r"\x");
            let start_idx = x_escape as usize;

            if x_escape {
                let (_total_size, group) = unpack_u16(u16::from_str_radix(values[start_idx], 16).unwrap_or_default());
    
                let number = u16::from_str_radix(values[start_idx + 1], 16).unwrap();
                let payload : Vec<_> = values[start_idx+2..].iter().flat_map(|s| u16::from_str_radix(s, 16)).map(|v| unpack_u16(v)).flat_map(|(v1,v2)| [v1,v2]).collect();
                // println!("{:#x}, {:#x}, {:?}", total_size, group, payload);

                Ok(Tag { group:group, number:number, payload:payload})
            } else {
                Err("Z escapes not implemented yet")
            }
        } else  {
            Err("Couldn't capture data")
        }
    }
}

type MessageText = Vec<TextPart>;
struct Message {
    text : [MessageText; LANGUAGES_COUNT],
    attribs : MessageAttributes,
    id : usize
}

enum TextPart {
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

impl Message {
    fn is_empty(&self) -> bool {
        self.text.iter().all(|str| str.is_empty())
    }

    fn get_html_formatted(&self, lang_id : usize, ignore_tags : bool) -> String {
        
        if ignore_tags {
            self.get_raw(lang_id).replace("\n", "<br>")
            //RE_TAG.replace_all(&s, |c : &Captures| c[0].parse::<Tag>().unwrap_or_default().get_simple_replacement().to_owned()).to_string()
        } else {

            let mut res_str = String::new();

            let mut current_color = 0;
            let mut current_size = 100;

            let mut needs_ruby : Option<(u8, String)> = None;

            for part in &self.text[lang_id] {
                match part {
                    TextPart::Text(text) => {
                        if text != "" {
                            let text = text.replace("\n", "<br>");
                            if let Some((over_count, ruby_text)) = needs_ruby {
                                let mut chars = text.chars();
                                let base_text : String = chars.by_ref().take(over_count as usize).collect();
                                let remaining_text : String = chars.collect();
                                res_str += &format!("<ruby>{}<rp>(</rp><rt>{}</rt><rp>)</rp></ruby>{}", base_text, ruby_text, remaining_text);
                                needs_ruby = None;
                            } else {
                                res_str += &text;
                            }
                        }
                    },
                    TextPart::Tag(tag) => {
                        match tag.group {
                            0xFF => {
                                match tag.number {
                                    0x00 => { // change color
                                        let new_color = tag.payload[0] as usize;
                                        if current_color != 0 {
                                            res_str += "</span>";
                                        }
                                        if new_color != 0 {
                                            res_str += &format!("<span style='color:{};'>", COLORS_RGB[new_color]);
                                        }
                                        current_color = new_color;
                                    },
                                    0x01 => {

                                        let new_size = get_u16_from_payload(&tag.payload, 0);
                                        if current_size != 100 {
                                            res_str += "</span>"
                                        }
                                        if new_size != 100 {
                                            res_str += &format!("<span style='font-size:{}%;'>", new_size);
                                        }
                
                                        current_size = new_size;
                                    },
                                    0x02 => {
                                        // todo!()
                                        let over_count = tag.payload[0];
                                        let raw_shiftjs : Vec<_>= tag.payload[1..].iter().map(|v| *v).collect();
                                        let decoded_ruby = encoding_rs::SHIFT_JIS.decode(&raw_shiftjs).0;
                                        needs_ruby = Some((over_count, decoded_ruby.to_string()));
                                        //println!("{}", decoded_ruby);
                                    },
                                    _ => {}
                                }
                            }
                            _ => { res_str += tag.get_simple_replacement(); }
                        }
                    }
                }                
            }

            res_str
        }
        
    }

    fn get_xlsx_formatted(&self, lang_id : usize, ignore_tags : bool, default_color : Color ) -> Vec<(Format, String)> {
        let mut segments : Vec<(Format, String)> = Vec::new();

        if ignore_tags {
            segments.push((Format::new(), self.get_raw(lang_id)));
        } else {
            
            let mut current_color = 0;
            let mut current_size = 100;

            const DEFAULT_SIZE : f32 = 11.0;


            for part in &self.text[lang_id] {
                match part {
                    TextPart::Text(text) => {
                        if !text.is_empty() {
                            let color = if current_color == 0 { default_color } else { Color::from(COLORS_RGB[current_color])};
                            let size = DEFAULT_SIZE * (current_size as f32/100.0);
                            let format = Format::new().set_font_color(color).set_font_size(size);
                            segments.push((format, text.to_string()));
                        }
                    },
                    TextPart::Tag(tag) => {
                        match tag.group {
                            0xFF => {
                                match tag.number {
                                    0x00 => { // change color
                                        //color

                                        current_color = tag.payload[0] as usize;
                                    },
                                    0x01 => {

                                        current_size = get_u16_from_payload(&tag.payload, 0);
                                        //Size
                                    },
                                    0x02 => {
                                        //ruby
                                    },
                                    _ => {}
                                }
                            }
                            _ => { 
                                let s = tag.get_simple_replacement().to_string();
                                if !s.is_empty() {
                                    let color = if current_color == 0 { default_color } else { Color::from(COLORS_RGB[current_color])};
                                    let size = DEFAULT_SIZE * (current_size as f32/100.0);
                                    let format = Format::new().set_font_color(color).set_font_size(size);

                                    segments.push((format, s)); 
                                }
                            }
                        }
                    },
                }
            }
        }
        
        segments
    }

    fn get_raw(&self, lang_id : usize) -> String {
        self.text[lang_id].iter().map(|text_part| match text_part {
                TextPart::Text(s) => s.to_string(),
                TextPart::Tag(t) => t.get_simple_replacement().to_string()
            }).join("")
    }

}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{:?}", self.text.iter().map(|text_parts| {
            text_parts.iter().map(|part|
                part.to_string() ).join("")
        }).collect::<Vec<_>>())
    }
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}",self.text.iter().map(|lang_msg| 
            lang_msg.iter().map(|part| format!("{:?}", part)).collect::<Vec<_>>().join("\n")
            ).join("\n ------------------\n")   
        )
    }
}

impl Default for Message {
    fn default() -> Self {
        Message { text: Default::default(), attribs: MessageAttributes::default(), id : 0}
    }
}


#[derive(Default, Debug)]
struct BMGParser {
    msgs : [Vec<Message>; BANK_COUNT],
}

impl BMGParser {
    
    fn print(self) {
        for (bank_id,msg_bank) in self.msgs.iter().enumerate() {
            for (idx, msg) in msg_bank.iter().filter(|msg| !msg.is_empty()).enumerate() {
                println!("{} : {:#x} : {}", bank_id, idx, msg);
            }
        }
    }
}

trait Exporter {
    fn new(filepath: &Path) -> Self;
    fn begin(&mut self);
    fn set_headers(&mut self);
    fn add_row(&mut self , msg : &Message, ignore_tags : bool);
    fn end(&mut self);
}

struct HTMLExporter {
    file: Option<File>
}

impl Exporter for HTMLExporter  {
    fn new(filepath: &Path) -> Self {
        if let Ok(f) = File::create(filepath) {
            HTMLExporter { file: Some(f) }
        } else {
            println!("Can't open {}", filepath.display());
            HTMLExporter {file: None}
        }
    }
    
    fn begin(&mut self) {
        if let Some(f) = &mut self.file {
            let _ = f.write("<!DOCTYPE html>
<html>
<head>
<style>
@font-face {
        font-family: 'fot-rodin_prondb';
        src: url(\"assets/FOT-RodinProN-DB.otf\");
        font-weight: normal;
        font-style: normal;
    }

    @font-face {
        font-family: 'reishotai';
        src: url(\"assets/Reishotai.otf\");
        font-weight: normal;
        font-style: normal;
        size-adjust: 120%;
    }

    body rt {
        color : white;
        font-family: 'reishotai', 'ＭＳ 明朝', serif;
    }

    body.nofuri rt {
        display:none;
    }


    header {
            text-align:center;
        }

  table {
    table-layout: fixed;
    width: 100%;
    overflow:auto;
    font-family: 'fot-rodin_prondb';
    
}
td {
    border: 1px solid white;
    background: rgb(0 0 0 / 90%);
    border-radius: 10px;
    padding:1em;
    }

tr {
    color: white;
    height: 48px;
}
</style>
</head>
<body>
<header>
  <img src=\"https://www.nintendo.com/jp/character/zelda/history/img/branch-c/02/pc/logo.png\"/>
</header>
<div id=\"options\">
    <input id=\"hide-furi\" type=\"checkbox\" name=\"HideFuri\" />
    <label for=\"HideFuri\">Hide Japanese Furigana</label>
</div>
<table>".as_bytes());
        }
        
    }

    fn set_headers(&mut self) {
        if let Some(f) = &mut self.file {
            let mut s = "<thead>
    <tr>".to_string();

          for lang in LANGUAGES_FULL {
              s +=  &format!("<th>{}</th>", lang);
          }

      s += "
    </tr>
  </thead>
  <tbody>";

          let _ = f.write(s.as_bytes());
        }
    }

    fn add_row(&mut self, msg : &Message, ignore_tags : bool) {
        if let Some(f) = &mut self.file {
            let display_style = match msg.attribs.get_display_style() {
                0x00 => "", //TODO : add dark background
                0x01 => "", // no background
                0x07 => "style='text-align: center;'",
                0x0C => "style='font-family: \"reishotai\", \"ＭＳ 明朝\", serif;'",
                0x0D => "style='color:#b4c8e6;'",
                0x0E => "style='color:#aadc8c;'",
                0x13 => "style='text-align: center; font-family: \"reishotai\", \"ＭＳ 明朝\", serif;'",
                _ => ""
            };
            let mut s  = format!("<tr {display_style}>");
    
            for i in 0..LANGUAGES_COUNT {
                s += &format!("<td>{}</td>\n", msg.get_html_formatted(i, ignore_tags));
            }
    
            s += "</tr>";
    
            let _ = f.write(s.as_bytes());
        }
    }

    fn end(&mut self) {
        if let Some(f) = &mut self.file {
            let _ = f.write(b"</tbody>
</table>
<script> 

const nofuriCheckbox = document.querySelector('#hide-furi');
    nofuriCheckbox.addEventListener('change', () => {
    document.querySelector('body').classList.toggle('nofuri', nofuriCheckbox.checked );
});

</script>
</body>
</html>");
        }
    }
    
}

struct CSVExporter {
    file: Option<File>
}


impl Exporter for CSVExporter {
    fn new(filepath: &Path) -> Self {
        if let Ok(f) = File::create(filepath) {
            CSVExporter { file: Some(f) }
        } else {
            println!("Can't open {}", filepath.display());
            CSVExporter {file: None}
        }
    }

    fn begin(&mut self) {
       
    }

    fn set_headers(&mut self) {
        if let Some(f) = &mut self.file {
            let mut s = "".to_string();

            for lang in LANGUAGES_FULL {
                s +=  &format!("{};", lang);
            }
            s += "\n";
            let _ = f.write(s.as_bytes());
        }

    }

    fn add_row(&mut self , msg : &Message, _ : bool) {
        if let Some(f) = &mut self.file {
            let mut s =  "".to_string();
    
            for i in 0..LANGUAGES_COUNT {
                s += &format!("\"{}\";", msg.get_raw(i));
            }

            s += "\n";
    
            let _ = f.write(s.as_bytes());
        }
    }

    fn end(&mut self) {
        
    }
}

struct XLSXExporter {
    filepath: String,
    workbook : rust_xlsxwriter::Workbook,
    current_row: usize
}

impl Exporter for XLSXExporter {
    fn new(filepath: &Path) -> Self {
        println!("Creating XLSX file : {}", filepath.display());
        XLSXExporter { filepath: filepath.display().to_string(), workbook: rust_xlsxwriter::Workbook::new(), current_row : 0 }
    }

    fn begin(&mut self) {
        // Add a worksheet to the workbook.
        let _worksheet = self.workbook.add_worksheet();
    }

    fn set_headers(&mut self) {
        if let Ok(worksheet) = self.workbook.worksheet_from_index(0) {
            let bold = Format::new().set_bold();
            let dark_bg = Format::new().set_font_color(Color::White).set_background_color(Color::Gray);

            let _ = worksheet.write_row_with_format(0, 0, LANGUAGES_FULL, &bold);
            let _ = worksheet.set_column_range_format(0, LANGUAGES_COUNT as u16, &dark_bg);
            if let Err(e) = worksheet.set_column_range_width(0, LANGUAGES_COUNT as u16, 50) {
                println!("Error setting col width : {e}");
            }
           

            self.current_row = 1;
        }
    }

    fn add_row(&mut self , msg : &Message, ignore_tags : bool) {
        if let Ok(worksheet) = self.workbook.worksheet_from_index(0) {
            for i in 0..LANGUAGES_COUNT {
                if ignore_tags {
                    let _ = worksheet.write(self.current_row as u32 , i as u16, msg.get_raw(i));
                } else {
                    let mut cell_color = Color::White;
                    let mut cell_align = FormatAlign::default();
                    match msg.attribs.get_display_style() {
                        0x00 => {}, //TODO : add dark background
                        0x01 => {}, // no background
                        0x07 => { cell_align = FormatAlign::Center;},
                        0x0C => {},//"style='font-family: \"reishotai\", \"ＭＳ 明朝\", serif;'",
                        0x0D => {cell_color = Color::from(COLORS_RGB[5])},//"style='color:#b4c8e6;'",
                        0x0E => {cell_color = Color::from(COLORS_RGB[2])},//"style='color:#aadc8c;'",
                        0x13 => { cell_align = FormatAlign::Center;},//"style='text-align: center; font-family: \"reishotai\", \"ＭＳ 明朝\", serif;'",
                        _ => {}
                    };

                    
                    let cell_format = Format::new().set_font_color(cell_color)
                                                    .set_background_color(Color::Gray)
                                                    .set_align(cell_align);

                    let segments = msg.get_xlsx_formatted(i, ignore_tags, cell_color);

                    if !segments.is_empty() {
                        let segments_ref : Vec<_>= segments.iter().map(|(a,b)| (a,b.as_str())).collect();
                        match worksheet.write_rich_string_with_format(self.current_row as u32 , i as u16, &segments_ref, &cell_format) {
                            Ok(_) => {},
                            Err(e) => {
                                println!("Error rich {e}");
                                // println!("row {}, col {} segments {:?}", self.current_row, i , segments.iter().map(|(_,s)| s).collect::<Vec<_>>());
                            },
                        }
                    }
                }
            }
            self.current_row += 1;
    
        }
    }

    fn end(&mut self) {
        // Save the file to disk.
        if let Ok(worksheet) = self.workbook.worksheet_from_index(0) {
            worksheet.autofit();
        }
        
        match self.workbook.save(&self.filepath) {
            Ok(_) => {},
            Err(e) => println!("Error saving : {e}")
        };
    }
}


impl BMGParser {
    fn feed_line(&mut self, line: &str, lang_idx : usize, bank_id : usize) { 
        //println!("{}", line);
        if let Some(groups) = RE.captures(line) {
            let id: usize = usize::from_str_radix(&groups["ID"], 16).unwrap_or_default();

            if id > 0 {
                //let slot = usize::from_str_radix(&groups["slot"], 16).unwrap();
                let idx = id -1;
                if idx + 1> self.msgs[bank_id].len() { self.msgs[bank_id].resize_with(idx + 1, || Message::default() );}
           
                self.msgs[bank_id][idx].id = id;

                if self.msgs[bank_id][idx].attribs.is_empty() {
                    let attribs = &groups["attribs"][1..groups["attribs"].len()-1];
                    self.msgs[bank_id][idx].attribs = attribs.parse().unwrap_or_default()
                }

                if let Some(str) = groups.name("str")
                {
                    if !self.msgs[bank_id][idx].text[lang_idx].is_empty()
                    {
                        println!("ALREADY USED : {}, {:#x}", bank_id, idx);
                    }

                    //self.msgs[bank_id][idx].text[lang_idx] = str.as_str().to_string();       


                    {
                        let s = str.as_str().replace(r"\n", "\n");
                        let tags_it = RE_TAG.find_iter(&s).flat_map(|m| m.as_str().parse::<Tag>()).map(|t| TextPart::Tag(t));
                        let str_it = RE_TAG.split(&s).map(|s| TextPart::Text(s.to_string()));

                        let text_parts = str_it.interleave(tags_it).collect::<Vec<_>>();

                        self.msgs[bank_id][idx].text[lang_idx] = text_parts;
                    }             
                }
            }
        } else  {
            println!("NO MATCH : {}", line);
        }
    }

    fn export_html(&self, filepath: &Path, ignore_tags : bool) {
        
        let mut exporter = HTMLExporter::new(filepath);
        exporter.begin();
        exporter.set_headers();

        for bank in &self.msgs {
            for msg in bank.iter().filter(|msg| !msg.is_empty()) {
                exporter.add_row(msg, ignore_tags);
            }
        }
        exporter.end();
    }

    fn export_csv(&self, filepath: &Path ) {
        
        let mut exporter = CSVExporter::new(filepath);
        exporter.begin();
        exporter.set_headers();

        for bank in &self.msgs {
            for msg in bank.iter().filter(|msg| !msg.is_empty()) {
                exporter.add_row(msg, true);
            }
        }
        exporter.end();
    }


    fn export_xlsx(&self, filepath: &Path, ignore_tags : bool) {
        let mut exporter = XLSXExporter::new(filepath);
        exporter.begin();
        exporter.set_headers();

        for bank in &self.msgs {
            for msg in bank.iter().filter(|msg| !msg.is_empty()) {
                exporter.add_row(msg, ignore_tags);
            }
        }
        exporter.end();
    }
}

fn process_file(lines : impl Iterator<Item=std::string::String>, lang_id : usize, bank_id : usize, parser : &mut BMGParser) {
    let iter = lines.skip_while(|l|  !RE.is_match(l) );
    
    for l in iter {
        parser.feed_line(&l, lang_id, bank_id);
    }

}

fn process_language(lang_idx : usize, lang_id : &str, parser : &mut BMGParser) {
    let str_path = &format!("./res/Msg{}", lang_id);
    let folder_path = Path::new(&str_path);

    for (bank_id,&basename) in FILENAMES.iter().enumerate() {
        let filename = basename.to_owned() + ".txt";
        println!("{} {}", lang_id, filename);
        if let Ok(file) = File::open(folder_path.join(&filename)) {
            let buf_reader = BufReader::new(file);
            process_file(buf_reader.lines().flatten(),lang_idx, bank_id,  parser);
        } else {
            println!("Couldn't open file {}" , filename);
        }
    }
}

fn main() {

    let mut parser : BMGParser = Default::default();
    
    for (lang_idx, lang) in LANGUAGES.iter().enumerate() {
        process_language(lang_idx,lang, &mut parser);
    }

    parser.export_html(Path::new("index.html"), false);
    parser.export_csv(Path::new("textdump.csv"));
    parser.export_xlsx(Path::new("textdump.xlsx"), false);
    bmg_raw_parser::print_bmg("./res/Msgjp/zel_00.bmg");

}
