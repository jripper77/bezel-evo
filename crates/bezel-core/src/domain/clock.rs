//! Date and time text for clock elements, from a local wall-clock time the
//! driving adapter provides (the core never reads the clock).

use std::time::Duration;

use super::frame::Rgba;
use super::theme::TextPart;

/// A local date and time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalTime {
    /// Year, e.g. 2026.
    pub year: i32,
    /// 1..=12.
    pub month: u8,
    /// 1..=31.
    pub day: u8,
    /// 0..=23.
    pub hour: u8,
    /// 0..=59.
    pub minute: u8,
    /// 0..=59.
    pub second: u8,
    /// 0 = Monday … 6 = Sunday.
    pub weekday: u8,
    /// 0..=999 within the second, for hands that sweep; 0 when the
    /// adapter does not know it.
    pub millisecond: u16,
}

impl LocalTime {
    /// The same moment without its milliseconds: what a digital clock shows.
    pub fn whole_second(self) -> Self {
        Self {
            millisecond: 0,
            ..self
        }
    }
}

/// Marks around an analog dial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DialTicks {
    /// No marks.
    None,
    /// One mark per hour.
    #[default]
    Hours,
    /// One mark per hour and a small one per minute.
    Minutes,
}

/// Numbers on an analog dial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DialNumbers {
    /// No numbers.
    #[default]
    None,
    /// 3, 6, 9 and 12.
    Quarters,
    /// 1 to 12.
    All,
    /// I to XII.
    Roman,
}

/// Shape of an analog clock's hands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HandShape {
    /// Thin lines with flat ends.
    Thin,
    /// Lines with round ends.
    #[default]
    Rounded,
    /// Lines ending in an arrowhead.
    Arrow,
}

/// The date written on an analog dial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DialDate {
    /// No date.
    #[default]
    None,
    /// The day of the month.
    Day,
    /// The day and the short month name.
    DayMonth,
}

/// A clock drawn with hands (D-2026-10-20-studio-ux-v2-07).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnalogClock {
    /// Marks around the dial.
    pub ticks: DialTicks,
    /// Numbers on the dial.
    pub numbers: DialNumbers,
    /// The hour hand shows.
    pub hour_hand: bool,
    /// The minute hand shows.
    pub minute_hand: bool,
    /// The second hand shows.
    pub second_hand: bool,
    /// Hour hand length as a multiple of its original length (1: unchanged).
    pub hour_length: f32,
    /// Minute hand length as a multiple of its original length.
    pub minute_length: f32,
    /// Second hand length as a multiple of its original length; tail unchanged.
    pub second_length: f32,
    /// Shape of the hands.
    pub hand_shape: HandShape,
    /// Width of the dial's outline, pixels (0: none).
    pub border: f32,
    /// Dial colour.
    pub face: Rgba,
    /// Colour of the marks, the numbers, the date and the outline.
    pub marks: Rgba,
    /// Hour hand colour.
    pub hour_color: Rgba,
    /// Minute hand colour.
    pub minute_color: Rgba,
    /// Second hand (and hub) colour.
    pub second_color: Rgba,
    /// The second hand sweeps smoothly instead of stepping each second.
    pub sweep: bool,
    /// The date on the dial.
    pub date: DialDate,
}

impl Default for AnalogClock {
    fn default() -> Self {
        Self {
            ticks: DialTicks::Hours,
            numbers: DialNumbers::None,
            hour_hand: true,
            minute_hand: true,
            second_hand: true,
            hour_length: 1.0,
            minute_length: 1.0,
            second_length: 1.0,
            hand_shape: HandShape::Rounded,
            border: 2.0,
            face: Rgba::opaque(0x0b, 0x0d, 0x0f),
            marks: Rgba::opaque(0xae, 0xb5, 0xbd),
            hour_color: Rgba::opaque(0xe8, 0xea, 0xed),
            minute_color: Rgba::opaque(0xe8, 0xea, 0xed),
            second_color: Rgba::opaque(0xff, 0x92, 0x48),
            sweep: false,
            date: DialDate::None,
        }
    }
}

/// Directions of an analog clock's hands, degrees clockwise from 12.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandAngles {
    /// Hour hand: moves with the minutes and seconds.
    pub hour: f32,
    /// Minute hand: moves with the seconds.
    pub minute: f32,
    /// Second hand: on the second, or between seconds when it sweeps.
    pub second: f32,
}

/// Time between two frames of a sweeping second hand.
pub const SWEEP_STEP: Duration = Duration::from_millis(33);

impl AnalogClock {
    /// Where the hands point at `t`; `sweep` lets the second hand move
    /// between seconds (off with reduced motion).
    pub fn angles(&self, t: &LocalTime, sweep: bool) -> HandAngles {
        let millis = if sweep && self.sweep {
            f32::from(t.millisecond.min(999)) / 1000.0
        } else {
            0.0
        };
        let second = f32::from(t.second.min(59)) + millis;
        let minute = f32::from(t.minute.min(59)) + second / 60.0;
        let hour = f32::from(t.hour % 12) + minute / 60.0;
        HandAngles {
            hour: hour * 30.0,
            minute: minute * 6.0,
            second: second * 6.0,
        }
    }

    /// When the dial next changes after a frame drawn at `t`, from that
    /// frame: the next second while a second hand shows, a sweep step while
    /// it sweeps (`sweep` allowed); `None` without a second hand (the theme's
    /// own refresh moves the others).
    pub fn next_change(&self, t: &LocalTime, sweep: bool) -> Option<Duration> {
        if !self.second_hand {
            return None;
        }
        if sweep && self.sweep {
            return Some(SWEEP_STEP);
        }
        let left = 1000 - u64::from(t.millisecond.min(999));
        Some(Duration::from_millis(left))
    }

    /// The date written on the dial at `t`, in `language`; empty without.
    pub fn date_text(&self, t: &LocalTime, language: Language) -> String {
        match self.date {
            DialDate::None => String::new(),
            DialDate::Day => format_clock("%e", t, language),
            DialDate::DayMonth => format_clock("%e %b", t, language),
        }
    }

    /// The numbers on the dial, with the hour each stands at (1..=12).
    pub fn numerals(&self) -> Vec<(u8, String)> {
        const ROMAN: [&str; 12] = [
            "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XII",
        ];
        (1..=12u8)
            .filter_map(|hour| {
                let text = match self.numbers {
                    DialNumbers::None => return None,
                    DialNumbers::Quarters if hour % 3 != 0 => return None,
                    DialNumbers::Quarters | DialNumbers::All => hour.to_string(),
                    DialNumbers::Roman => ROMAN[usize::from(hour - 1)].to_string(),
                };
                Some((hour, text))
            })
            .collect()
    }
}

/// Language of day and month names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Language {
    /// English.
    #[default]
    English,
    /// Brazilian Portuguese.
    PortugueseBr,
    /// Italian.
    Italian,
}

/// The language of day and month names for a locale such as `pt-BR` (the
/// driving adapter reads the system's): Portuguese for any `pt` locale,
/// Italian for any `it` locale, English otherwise.
pub fn language_of(locale: Option<&str>) -> Language {
    match locale {
        Some(l) if l.to_ascii_lowercase().starts_with("pt") => Language::PortugueseBr,
        Some(l) if l.to_ascii_lowercase().starts_with("it") => Language::Italian,
        _ => Language::English,
    }
}

/// Letter case applied to the complete formatted date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ClockCase {
    /// Preserve the pattern and the natural day/month spelling.
    #[default]
    Normal,
    /// All letters uppercase.
    Upper,
    /// Capitalise each word, preserving the other letters.
    Title,
}

/// Apply a clock element's case without changing numeric fields.
pub fn clock_case(text: &str, casing: ClockCase) -> String {
    match casing {
        ClockCase::Normal => text.to_string(),
        ClockCase::Upper => text.to_uppercase(),
        ClockCase::Title => {
            let mut start = true;
            let mut out = String::new();
            for c in text.chars() {
                if c.is_alphabetic() {
                    if start {
                        out.extend(c.to_uppercase());
                    } else {
                        out.push(c);
                    }
                    start = false;
                } else {
                    out.push(c);
                    start = true;
                }
            }
            out
        }
    }
}

const IT_DAYS: [&str; 7] = [
    "lunedì",
    "martedì",
    "mercoledì",
    "giovedì",
    "venerdì",
    "sabato",
    "domenica",
];
const IT_MONTHS: [&str; 12] = [
    "gennaio",
    "febbraio",
    "marzo",
    "aprile",
    "maggio",
    "giugno",
    "luglio",
    "agosto",
    "settembre",
    "ottobre",
    "novembre",
    "dicembre",
];

const EN_DAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];
const PT_DAYS: [&str; 7] = [
    "segunda-feira",
    "terça-feira",
    "quarta-feira",
    "quinta-feira",
    "sexta-feira",
    "sábado",
    "domingo",
];
const PT_DAYS_SHORT: [&str; 7] = ["seg", "ter", "qua", "qui", "sex", "sáb", "dom"];
const EN_MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const PT_MONTHS: [&str; 12] = [
    "janeiro",
    "fevereiro",
    "março",
    "abril",
    "maio",
    "junho",
    "julho",
    "agosto",
    "setembro",
    "outubro",
    "novembro",
    "dezembro",
];

fn day_name(t: &LocalTime, lang: Language, short: bool) -> String {
    let i = usize::from(t.weekday.min(6));
    match (lang, short) {
        (Language::Italian, false) => IT_DAYS[i].to_string(),
        (Language::Italian, true) => IT_DAYS[i].chars().take(3).collect(),
        (Language::English, false) => EN_DAYS[i].to_string(),
        (Language::English, true) => EN_DAYS[i][..3].to_string(),
        (Language::PortugueseBr, false) => PT_DAYS[i].to_string(),
        (Language::PortugueseBr, true) => PT_DAYS_SHORT[i].to_string(),
    }
}

fn month_name(t: &LocalTime, lang: Language, short: bool) -> String {
    let i = usize::from(t.month.clamp(1, 12) - 1);
    let full = match lang {
        Language::Italian => IT_MONTHS[i],
        Language::English => EN_MONTHS[i],
        Language::PortugueseBr => PT_MONTHS[i],
    };
    if short {
        full.chars().take(3).collect()
    } else {
        full.to_string()
    }
}

/// Formats `t` with a `strftime`-like pattern:
/// `%H` 24-hour, `%I` 12-hour, `%p` AM/PM, `%M` minute, `%S` second,
/// `%d` day, `%e` day without padding, `%m` month, `%Y` year, `%y` 2-digit
/// year, `%A`/`%a` weekday name full/short, `%B`/`%b` month name full/short,
/// `%%` a percent sign. Unknown directives are copied as they are.
pub fn format_clock(pattern: &str, t: &LocalTime, lang: Language) -> String {
    pieces(pattern, t, lang)
        .into_iter()
        .map(|(_, text)| text)
        .collect()
}

/// The text of a clock cut into its parts (D-2026-10-20-studio-ux-v2-06):
/// hours (`%H`, `%I`), minutes (`%M`), seconds (`%S`), the separators (the
/// fixed text between two of those) and the rest (`None`), in order, with
/// neighbours of the same part joined. Joined, they are [`format_clock`].
pub fn clock_parts(
    pattern: &str,
    t: &LocalTime,
    lang: Language,
) -> Vec<(Option<TextPart>, String)> {
    let pieces = pieces(pattern, t, lang);
    let timed = |p: &Piece| matches!(p, Piece::Time(_));
    let mut out: Vec<(Option<TextPart>, String)> = Vec::new();
    for (index, (piece, text)) in pieces.iter().enumerate() {
        let part = match piece {
            Piece::Time(part) => Some(*part),
            Piece::Literal
                if pieces[..index].iter().any(|(p, _)| timed(p))
                    && pieces[index + 1..]
                        .iter()
                        .find(|(p, _)| !matches!(p, Piece::Literal))
                        .is_some_and(|(p, _)| timed(p)) =>
            {
                Some(TextPart::Separators)
            }
            _ => None,
        };
        match out.last_mut() {
            Some((last, joined)) if *last == part => joined.push_str(text),
            _ => out.push((part, text.clone())),
        }
    }
    out
}

/// What a piece of a clock's text comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Piece {
    /// Fixed text of the pattern.
    Literal,
    /// Hours, minutes or seconds.
    Time(TextPart),
    /// Any other directive.
    Other,
}

fn pieces(pattern: &str, t: &LocalTime, lang: Language) -> Vec<(Piece, String)> {
    let mut out: Vec<(Piece, String)> = Vec::new();
    let mut chars = pattern.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            match out.last_mut() {
                Some((Piece::Literal, text)) => text.push(c),
                _ => out.push((Piece::Literal, c.to_string())),
            }
            continue;
        }
        let Some(d) = chars.next() else {
            out.push((Piece::Literal, "%".to_string()));
            break;
        };
        let hour12 = match t.hour % 12 {
            0 => 12,
            h => h,
        };
        let piece = match d {
            'H' => (Piece::Time(TextPart::Hours), format!("{:02}", t.hour)),
            'I' => (Piece::Time(TextPart::Hours), format!("{hour12:02}")),
            'M' => (Piece::Time(TextPart::Minutes), format!("{:02}", t.minute)),
            'S' => (Piece::Time(TextPart::Seconds), format!("{:02}", t.second)),
            other => (
                Piece::Other,
                match other {
                    'p' => if t.hour < 12 { "AM" } else { "PM" }.to_string(),
                    'd' => format!("{:02}", t.day),
                    'e' => t.day.to_string(),
                    'm' => format!("{:02}", t.month),
                    'Y' => t.year.to_string(),
                    'y' => format!("{:02}", t.year.rem_euclid(100)),
                    'A' => day_name(t, lang, false),
                    'a' => day_name(t, lang, true),
                    'B' => month_name(t, lang, false),
                    'b' => month_name(t, lang, true),
                    '%' => "%".to_string(),
                    other => format!("%{other}"),
                },
            ),
        };
        out.push(piece);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: LocalTime = LocalTime {
        year: 2026,
        month: 9,
        day: 30,
        hour: 21,
        minute: 5,
        second: 9,
        weekday: 2,
        millisecond: 0,
    };

    #[test]
    fn italian_names_and_unicode_case() {
        assert_eq!(language_of(Some("IT_it")), Language::Italian);
        assert_eq!(
            format_clock("%A %e %B", &T, Language::Italian),
            "mercoledì 30 settembre"
        );
        assert_eq!(format_clock("%a %b", &T, Language::Italian), "mer set");
        assert_eq!(
            clock_case("lunedì 5 ottobre", ClockCase::Upper),
            "LUNEDÌ 5 OTTOBRE"
        );
        assert_eq!(
            clock_case("lunedì 5 ottobre", ClockCase::Title),
            "Lunedì 5 Ottobre"
        );
        assert_eq!(clock_case("14:30 AM", ClockCase::Title), "14:30 AM");
    }

    #[test]
    fn numeric_directives() {
        assert_eq!(format_clock("%H:%M:%S", &T, Language::English), "21:05:09");
        assert_eq!(format_clock("%I:%M %p", &T, Language::English), "09:05 PM");
        assert_eq!(
            format_clock("%d/%m/%Y %y %e", &T, Language::English),
            "30/09/2026 26 30"
        );
        let midnight = LocalTime { hour: 0, ..T };
        assert_eq!(format_clock("%I %p", &midnight, Language::English), "12 AM");
    }

    #[test]
    fn names_in_both_languages() {
        assert_eq!(
            format_clock("%A, %B %e", &T, Language::English),
            "Wednesday, September 30"
        );
        assert_eq!(format_clock("%a %b", &T, Language::English), "Wed Sep");
        assert_eq!(
            format_clock("%A, %e de %B", &T, Language::PortugueseBr),
            "quarta-feira, 30 de setembro"
        );
        assert_eq!(format_clock("%a %b", &T, Language::PortugueseBr), "qua set");
    }

    #[test]
    fn portuguese_locales_pick_portuguese() {
        assert_eq!(language_of(Some("pt-BR")), Language::PortugueseBr);
        assert_eq!(language_of(Some("PT_pt")), Language::PortugueseBr);
        assert_eq!(language_of(Some("en-US")), Language::English);
        assert_eq!(language_of(None), Language::English);
    }

    #[test]
    fn a_clock_is_cut_into_hours_separators_minutes_and_seconds() {
        let parts = |p: &str| clock_parts(p, &T, Language::English);
        use TextPart::*;
        assert_eq!(
            parts("%H:%M:%S"),
            vec![
                (Some(Hours), "21".into()),
                (Some(Separators), ":".into()),
                (Some(Minutes), "05".into()),
                (Some(Separators), ":".into()),
                (Some(Seconds), "09".into()),
            ]
        );
        assert_eq!(
            parts("at %I.%M %p"),
            vec![
                (None, "at ".into()),
                (Some(Hours), "09".into()),
                (Some(Separators), ".".into()),
                (Some(Minutes), "05".into()),
                (None, " PM".into()),
            ],
            "text outside the time and other directives keep the text's style"
        );
        assert_eq!(
            parts("%H%a%M"),
            vec![
                (Some(Hours), "21".into()),
                (None, "Wed".into()),
                (Some(Minutes), "05".into()),
            ]
        );
        assert_eq!(parts("%d/%m"), vec![(None, "30/09".into())]);
        assert!(parts("").is_empty());
        for pattern in ["%H:%M:%S", "%A %e %B, %I:%M %p", "100%% %q %"] {
            let joined: String = parts(pattern).into_iter().map(|(_, t)| t).collect();
            assert_eq!(joined, format_clock(pattern, &T, Language::English));
        }
    }

    #[test]
    fn hands_point_at_the_time() {
        let clock = AnalogClock::default();
        let at = |hour, minute, second, millisecond| LocalTime {
            hour,
            minute,
            second,
            millisecond,
            ..T
        };
        let three = clock.angles(&at(15, 0, 0, 0), true);
        assert_eq!((three.hour, three.minute, three.second), (90.0, 0.0, 0.0));
        let half_six = clock.angles(&at(6, 30, 0, 0), true);
        assert_eq!((half_six.hour, half_six.minute), (195.0, 180.0));
        let late = clock.angles(&at(21, 45, 30, 500), true);
        assert!((late.hour - 292.75).abs() < 1e-3, "{}", late.hour);
        assert!((late.minute - 273.0).abs() < 1e-3);
        assert_eq!(late.second, 180.0, "steps on the second");
        let sweeping = AnalogClock {
            sweep: true,
            ..clock
        };
        assert!((sweeping.angles(&at(21, 45, 30, 500), true).second - 183.0).abs() < 1e-3);
        assert_eq!(
            sweeping.angles(&at(21, 45, 30, 500), false).second,
            180.0,
            "reduced motion steps"
        );
    }

    #[test]
    fn the_dial_changes_each_second_or_sweeps() {
        let clock = AnalogClock::default();
        let t = LocalTime {
            millisecond: 250,
            ..T
        };
        assert_eq!(
            clock.next_change(&t, true),
            Some(Duration::from_millis(750))
        );
        let sweeping = AnalogClock {
            sweep: true,
            ..clock
        };
        assert_eq!(sweeping.next_change(&t, true), Some(SWEEP_STEP));
        assert_eq!(
            sweeping.next_change(&t, false),
            Some(Duration::from_millis(750))
        );
        let still = AnalogClock {
            second_hand: false,
            ..sweeping
        };
        assert_eq!(still.next_change(&t, true), None);
        assert_eq!(t.whole_second().millisecond, 0);
    }

    #[test]
    fn numbers_and_date_on_the_dial() {
        let numbers = |n| {
            AnalogClock {
                numbers: n,
                ..AnalogClock::default()
            }
            .numerals()
        };
        assert!(numbers(DialNumbers::None).is_empty());
        let quarters: Vec<String> = numbers(DialNumbers::Quarters)
            .into_iter()
            .map(|n| n.1)
            .collect();
        assert_eq!(quarters, ["3", "6", "9", "12"]);
        assert_eq!(numbers(DialNumbers::All).len(), 12);
        assert_eq!(numbers(DialNumbers::Roman)[3], (4, "IV".to_string()));
        assert_eq!(numbers(DialNumbers::Roman)[11], (12, "XII".to_string()));
        let date = |d| {
            AnalogClock {
                date: d,
                ..AnalogClock::default()
            }
            .date_text(&T, Language::Italian)
        };
        assert_eq!(date(DialDate::None), "");
        assert_eq!(date(DialDate::Day), "30");
        assert_eq!(date(DialDate::DayMonth), "30 set");
    }

    #[test]
    fn escapes_and_unknown_directives() {
        assert_eq!(
            format_clock("100%% %q %", &T, Language::English),
            "100% %q %"
        );
        let odd = LocalTime {
            month: 0,
            weekday: 9,
            millisecond: 0,
            ..T
        };
        assert_eq!(format_clock("%b %a", &odd, Language::English), "Jan Sun");
    }
}
