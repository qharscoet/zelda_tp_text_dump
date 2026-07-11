use crate::bmg_message::Tag;

#[derive(Clone)]
pub struct GameConfig {
    pub name: &'static str,
    pub id : &'static str,

    pub get_color_hex: fn(usize) -> &'static str,
    pub get_tag_replacement: fn(&Tag) -> &str,

    pub get_languages : fn() -> &'static [(&'static str, &'static str)],
    pub get_filenames : fn() -> &'static [&'static str]
}


pub const TWW: GameConfig = GameConfig {
    name: "The Wind Waker",
    id: "tww",
    get_languages : || {
        const LANGUAGES : [(&str, &str);4] = [
            ("jp", "Japanese"),
            ("uk", "UK English"),
            ("fr", "French"),
            // ("sp", "Spanish"),
            ("de", "German"),
            // ("it" "Italian")
        ];

        &LANGUAGES
    },
    get_filenames : || {
        const FILENAMES : [&str;1] = [
            "zel_00",
        ];

        &FILENAMES
    },
    get_color_hex: |id| {
        const COLORS_RGB_TWW: [&str; 9] = [
            "#ffffff",
            "#ff6400",
            "#00ff00",
            "#7878ff",
            "#ffff3c",
            "#00ffff",
            "#ff00ff",
            "#828282",
            "#ff8000",
        ];

        COLORS_RGB_TWW[id]
    },
    get_tag_replacement : |tag| {
        match tag.group {
            0x00 => {
                match tag.number {
                    0x00 => "[Link]",
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
                    0x1E => " ",
                    0x1F => " ",
                    0x20 => "[CanonBalls]",
                    0x21 => "[BrokenVasePayment]",
                    0x22 => "[AuctionCharacter]",
                    0x23 => "[AuctionItem]",
                    0x24 => "[AuctionBid]",
                    0x25 => "[AuctionStartingBid]",
                    0x26 => "[PlayerActionBidSelector]",
                    0x27 => "[FlashingA]",
                    0x28 => "[OrcaBlowCount]",
                    0x29 => "[PiratePassword]",
                    0x2A => "[Starburst]",
                    0x2B => "[PostOfficeGameLetterCount]",
                    0x2C => "[PostOfficeGameRupeeReward]",
                    0x2D => "[PostBoxLetterCount]",
                    0x2E => "[RemainingKorokCount]",
                    0x2F => "[RemainingForestWaterTime]",
                    0x30 => "[FlightPlatformTime]",
                    0x31 => "[FlightPlatformRecord]",
                    0x32 => "[BeedlePointCount]",
                    0x33 => "[MsMariePendantCount]",
                    0x34 => "[MsMariePendantTotal]",
                    0x35 => "[PigGameTime]",
                    0x36 => "[SailingGameRupeeReward]",
                    0x37 => "[CurrentBombCapacity]",
                    0x38 => "[CurrentArrowCapacity]",
                    0x39 => "[Heart]",
                    0x3A => "[MusicNote]",
                    0x3B => "[TargetLetterCount]",
                    0x3C => "[FishmanHitCount]",
                    0x3D => "[FishmanRupeeReward]",
                    0x3E => "[BokoBabaSeedCount]",
                    0x3F => "[SkullNecklaceCount]",
                    0x40 => "[ChuJellyCount]",
                    0x41 => "[JoyPendantCount]",
                    0x42 => "[GoldenFeatherCount]",
                    0x43 => "[KnightsCrestCount]",
                    0x44 => "[BeedleRupeeOffer]",
                    0x45 => "[BokoBabaSellSelector]",
                    0x46 => "[SkullNecklaceSellSelector]",
                    0x47 => "[ChuJellySellSelector]",
                    0x48 => "[JoyPendantSellSelector]",
                    0x49 => "[GoldenFeatherSellSelector]",
                    0x4A => "[KnightsCrestSellSelector]",
                    _ => ""
                }
            }
            _=> ""
        }
    },
};

pub const TP: GameConfig = GameConfig {
    name: "Twilight Princess",
    id:"tp",
    get_languages : || {
        const LANGUAGES : [(&str, &str);4] = [
            ("jp", "Japanese"),
            ("us", "US English"),
            ("fr", "French"),
            // ("sp", "Spanish"),
            ("de", "German"),
            // ("it" "Italian")
        ];

        &LANGUAGES
    },
    get_filenames : || {
        const FILENAMES : [&str;10] = [
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

        &FILENAMES
    },
    get_color_hex: |id| {
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

        COLORS_RGB[id]
    },
    get_tag_replacement : |tag| {
        match tag.group {
            0x00 => {
                match tag.number {
                    0x00 =>	"[Link]",
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
                    0x1E => " ",
                    0x1F => " ",
                    0x23 => "[RedTarget] ",
                    0x24 => "[YellowTarget] ",
                    0x2E => "[XorY] ",
                    0x39 => "♥ ",
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
                match tag.number {
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
                match tag.number {
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
                match tag.number {
                    0x00 =>	"[Time]",
                    0x03 =>	if tag.payload[0] == 0  {"[ReturnedBugs]" } else {"[RemainingBugs]"},
                    0x04 =>	"noop",
                    0x07 =>	"[RiverPoints]",
                    0x08 =>	"[FishLength]",
                    0x09 =>	"[MartGoalLeft]",
                    0x0A =>	"[LetterCount]",
                    0x0B =>	"[PoesNeeded]",
                    0x0C =>	if tag.payload[0] == 0 {"[LatestScore]" } else {"[HighScore]"},
                    0x0D =>	"[FishCount]",
                    0x0E =>	"[RollGoal]",
                    _ => ""
                }
            },
            0x06 => {
                match tag.number {
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
};
