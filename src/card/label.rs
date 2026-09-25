pub(crate) fn decode(label: &str) -> String {
    label
        .strip_prefix("_$!<")
        .and_then(|inner| inner.strip_suffix(">!$_"))
        .unwrap_or(label)
        .to_owned()
}

pub(crate) fn from_types(params: &[String]) -> Option<String> {
    let types: Vec<String> = params
        .iter()
        .filter_map(|param| param.split_once('='))
        .filter(|(name, _)| name.eq_ignore_ascii_case("TYPE"))
        .flat_map(|(_, value)| value.trim_matches('"').split(','))
        .map(str::to_lowercase)
        .filter(|kind| !matches!(kind.as_str(), "pref" | "voice" | "internet"))
        .collect();
    (!types.is_empty()).then(|| types.join(", "))
}
