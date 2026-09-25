use std::fmt;

const MONTHS: [&str; 12] = [
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Birthday {
    Date {
        year: Option<u16>,
        month: u8,
        day: u8,
    },
    Invalid(String),
}

impl Birthday {
    pub(crate) fn parse(value: &str, params: &[String]) -> Self {
        read_date(value, omitted_year(params)).unwrap_or_else(|| Self::Invalid(value.to_owned()))
    }

    pub fn is_read_only(&self) -> bool {
        matches!(self, Self::Invalid(_))
    }
}

impl fmt::Display for Birthday {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Date { year, month, day } => {
                write!(f, "{day} {}", MONTHS[usize::from(*month) - 1])?;
                year.map_or(Ok(()), |year| write!(f, " {year}"))
            }
            Self::Invalid(raw) => f.write_str(raw),
        }
    }
}

fn omitted_year(params: &[String]) -> Option<&str> {
    params
        .iter()
        .filter_map(|param| param.split_once('='))
        .find(|(name, _)| name.eq_ignore_ascii_case("X-APPLE-OMIT-YEAR"))
        .map(|(_, year)| year)
}

fn read_date(value: &str, omitted_year: Option<&str>) -> Option<Birthday> {
    let date = value.split_once('T').map_or(value, |(date, _)| date);
    let (year, month_day) = match date.strip_prefix("--") {
        Some(month_day) => (None, month_day),
        None => {
            let year = date.get(..4)?;
            let rest = &date[4..];
            let year = (omitted_year != Some(year)).then_some(number(year)?);
            (year, rest.strip_prefix('-').unwrap_or(rest))
        }
    };
    let (month, day) = match month_day.len() {
        4 => (month_day.get(..2)?, month_day.get(2..)?),
        5 if month_day.as_bytes()[2] == b'-' => (&month_day[..2], &month_day[3..]),
        _ => return None,
    };
    let (month, day) = (
        u8::try_from(number(month)?).ok()?,
        u8::try_from(number(day)?).ok()?,
    );
    (1..=12).contains(&month).then_some(())?;
    (1..=days_in(month, year)).contains(&day).then_some(())?;
    Some(Birthday::Date { year, month, day })
}

fn number(digits: &str) -> Option<u16> {
    digits
        .bytes()
        .all(|b| b.is_ascii_digit())
        .then(|| digits.parse().ok())?
}

fn days_in(month: u8, year: Option<u16>) -> u8 {
    let is_leap = year.is_none_or(|y| y % 4 == 0 && (y % 100 != 0 || y % 400 == 0));
    match month {
        2 if is_leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}
