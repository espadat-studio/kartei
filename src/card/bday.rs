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

    pub fn from_iso(text: &str) -> Option<Self> {
        read_date(text, None)
    }

    pub fn to_iso(&self) -> String {
        match self {
            Self::Date {
                year: Some(year),
                month,
                day,
            } => format!("{year:04}-{month:02}-{day:02}"),
            Self::Date {
                year: None,
                month,
                day,
            } => format!("--{month:02}-{day:02}"),
            Self::Invalid(raw) => raw.clone(),
        }
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

pub(crate) fn write(
    year: Option<u16>,
    month: u8,
    day: u8,
    existing: Option<(&[String], &str)>,
    is_v4: bool,
) -> (Vec<String>, String) {
    let (params, value) = existing.unwrap_or((&[], ""));
    let (date, time) = value.split_at(value.find('T').unwrap_or(value.len()));
    let omit = params.iter().position(|p| is_omit_year(p));
    let month_day = date
        .strip_prefix("--")
        .or(date.get(4..))
        .unwrap_or_default();
    let is_apple = omit.is_some() || (!is_v4 && !date.starts_with("--"));
    let is_basic = match date {
        "" => is_v4,
        _ if year.is_none() && !is_apple && !date.starts_with("--") => true,
        _ => !month_day.contains('-'),
    };
    let sep = if is_basic { "" } else { "-" };
    let month_day = format!("{month:02}{sep}{day:02}");
    let mut kept: Vec<String> = params
        .iter()
        .filter(|p| !is_omit_year(p))
        .cloned()
        .collect();
    let date = match year {
        Some(year) => format!("{year:04}{sep}{month_day}"),
        None if is_apple => {
            let param = omit.map_or_else(
                || "X-APPLE-OMIT-YEAR=1604".to_owned(),
                |i| params[i].clone(),
            );
            let omitted = omitted_year(std::slice::from_ref(&param))
                .unwrap_or("1604")
                .to_owned();
            kept.insert(omit.unwrap_or(0), param);
            format!("{omitted}{sep}{month_day}")
        }
        None => format!("--{month_day}"),
    };
    (kept, date + time)
}

fn is_omit_year(param: &str) -> bool {
    param
        .split_once('=')
        .is_some_and(|(name, _)| name.eq_ignore_ascii_case("X-APPLE-OMIT-YEAR"))
}

fn omitted_year(params: &[String]) -> Option<&str> {
    params
        .iter()
        .find(|param| is_omit_year(param))
        .and_then(|param| param.split_once('='))
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
