use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

use crate::card::Card;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Hit {
    Phone,
    Fuzzy(u32),
}

pub fn rank<'a>(cards: impl IntoIterator<Item = &'a Card>, query: &str) -> Vec<usize> {
    let cards = cards.into_iter().enumerate();
    if query.is_empty() {
        return cards.map(|(i, _)| i).collect();
    }
    let pattern = Pattern::new(
        query,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );
    let digits = phone_digits(query);
    let mut matcher = Matcher::new(Config::DEFAULT);
    let mut buf = Vec::new();
    let mut hits: Vec<(usize, Hit)> = cards
        .filter_map(|(i, card)| {
            let fuzzy = texts(card)
                .iter()
                .filter_map(|text| pattern.score(Utf32Str::new(text, &mut buf), &mut matcher))
                .max()
                .map(Hit::Fuzzy);
            fuzzy
                .or_else(|| {
                    let digits = digits.as_ref()?;
                    has_phone(card, digits).then_some(Hit::Phone)
                })
                .map(|hit| (i, hit))
        })
        .collect();
    hits.sort_by(|(_, a), (_, b)| b.cmp(a));
    hits.into_iter().map(|(i, _)| i).collect()
}

fn texts(card: &Card) -> Vec<String> {
    let org = card.organization().into_iter();
    let emails = card.emails().into_iter().map(|email| email.value);
    std::iter::once(card.display_name())
        .chain(org.flat_map(|org| [org.company, org.department]))
        .chain(emails)
        .collect()
}

fn phone_digits(query: &str) -> Option<String> {
    let digits: String = query
        .chars()
        .filter(|c| !matches!(c, ' ' | '+' | '-' | '(' | ')'))
        .collect();
    (!digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())).then_some(digits)
}

fn has_phone(card: &Card, digits: &str) -> bool {
    card.phones().iter().any(|phone| {
        let phone: String = phone.value.chars().filter(char::is_ascii_digit).collect();
        phone.contains(digits)
    })
}

pub fn sort_key(card: &Card) -> (String, String, String) {
    let (family, given) = card.structured_name();
    let display_name = card.display_name().to_lowercase();
    if family.is_empty() && given.is_empty() {
        return (display_name, String::new(), String::new());
    }
    (family.to_lowercase(), given.to_lowercase(), display_name)
}
