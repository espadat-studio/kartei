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
        .filter(|kind| !is_hidden(kind))
        .collect();
    (!types.is_empty()).then(|| types.join(", "))
}

pub(crate) fn retype(params: &[String], label: Option<&str>) -> Vec<String> {
    let mut params: Vec<String> = params
        .iter()
        .filter_map(|param| match param.split_once('=') {
            Some((name, value)) if name.eq_ignore_ascii_case("TYPE") => {
                let hidden: Vec<&str> = value
                    .trim_matches('"')
                    .split(',')
                    .filter(|kind| is_hidden(kind))
                    .collect();
                (!hidden.is_empty()).then(|| format!("{name}={}", hidden.join(",")))
            }
            _ => Some(param.clone()),
        })
        .collect();
    params.extend(label.map(|label| format!("TYPE={}", label.to_uppercase())));
    params
}

pub(crate) fn next(labels: &[&'static str], label: Option<&str>) -> &'static str {
    let current = labels.iter().position(|l| Some(*l) == label);
    labels[current.map_or(0, |i| (i + 1) % labels.len())]
}

fn is_hidden(kind: &str) -> bool {
    ["pref", "voice", "internet"]
        .iter()
        .any(|hidden| kind.eq_ignore_ascii_case(hidden))
}
