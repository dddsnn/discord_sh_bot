use model::{UserData, Tier, Timeframe, StatusReport};

pub fn unknown_request(msg_content: &str) -> String {
    "\"".to_owned() + msg_content + "\" is not a valid request. Type \"help\" to find out what is."
}

pub fn help() -> String {
    // TODO
    "This is an unhelpful help text. There'll be a better one, I promise.".to_owned()
}

pub fn want(ud: &UserData) -> String {
    // TODO sort based on tier (tier 6 should always be first etc.) and group to compactify the
    // information
    let mut kind = String::new();
    for (i, want) in ud.wants.iter().enumerate() {
        match want.tier {
            Tier::Tier6 => kind.push_str("tier 6 "),
            Tier::Tier8 => kind.push_str("tier 8 "),
            Tier::Tier10 => kind.push_str("tier 10 "),
        }
        if i == 0 {
            // First in the list, add a Stronghold.
            kind.push_str(" Stronghold ");
        }
        match want.time {
            Timeframe::Always => kind.push_str("whenever you're online"),
            Timeframe::UntilLogout => kind.push_str("until you log out"),
            Timeframe::Timespan { until } => {
                // TODO handle error better
                let time = {
                    if let Ok(tm_fmt) = until.strftime("%R UTC") {
                        format!("{}", tm_fmt)
                    } else {
                        "error formatting time".to_owned()
                    }
                };
                kind.push_str(&format!("until {}", time));
            }
        }
        if i + 2 < ud.wants.len() {
            // Before second-to-last one, add comma for enumeration.
            kind.push_str(", ");
        } else if i + 2 == ud.wants.len() {
            // Second-to-last one, add "and".
            kind.push_str(" and ");
        }
    }
    format!("Ok, I'll note you're up for {}.", kind)
}

pub fn dont_want() -> String {
    "Ok, I'll take you off the list.".to_owned()
}

pub fn status(status_report: &StatusReport) -> String {
    // TODO special case one player (is/are)
    // TODO better solution for multiline strings?
    format!("There is currently a total of {} players who want to play Stronghold.
{} want tier 6, {} tier 8 and {} tier 10.",
            status_report.num_wanting_total,
            status_report.num_wanting_t6,
            status_report.num_wanting_t8,
            status_report.num_wanting_t10)
}
