use crate::search;
use crate::vdir::AddressBook;

pub fn lines(book: &AddressBook, text: &str) -> Vec<String> {
    let mut cards: Vec<_> = book.cards.iter().map(|(_, card)| card).collect();
    cards.sort_by_cached_key(|card| search::sort_key(card));
    search::rank(cards.iter().copied(), text)
        .into_iter()
        .flat_map(|i| {
            let name = field(&cards[i].display_name());
            cards[i].emails().into_iter().map(move |email| {
                let label = field(&email.label.unwrap_or_default());
                format!("{}\t{name}\t{label}", field(&email.value))
            })
        })
        .collect()
}

fn field(value: &str) -> String {
    value.replace(['\t', '\r', '\n'], " ")
}
