use scryfall::card::Card;

pub async fn get_card(card: String) -> Option<String> {
    match Card::named_fuzzy(&card).await {
        Ok(card) => {
            const MAX_TWITCH_CHAT_MESSAGE_LENGTH: usize = 500;

            let mut formatted = String::new();
            formatted.push_str(&card.name);

            if let Some(mana_cost) = card.mana_cost {
                formatted.push_str(&format!(" {mana_cost}"));
            }

            if let Some(type_line) = card.type_line {
                formatted.push_str(&format!(" | {type_line}"));
            }

            if let Some(oracle_text) = card.oracle_text {
                let oracle_text = oracle_text.replace("\n", ". ");
                formatted.push_str(&format!(" | {oracle_text}"));
            }

            match (card.power, card.toughness) {
                (Some(power), Some(toughness)) => {
                    formatted.push_str(&format!(" | {power}/{toughness}"));
                }
                (Some(power), None) => {
                    formatted.push_str(&format!(" | {power}/-"));
                }
                (None, Some(toughness)) => {
                    formatted.push_str(&format!(" | -/{toughness}"));
                }
                _ => {}
            }

            if let Some(loyalty) = card.loyalty {
                formatted.push_str(&format!(" | {loyalty}"));
            }

            match formatted.len() > MAX_TWITCH_CHAT_MESSAGE_LENGTH {
                true => {
                    let mut url = card.scryfall_uri;
                    let query: Vec<(String, String)> = url
                        .query_pairs()
                        .filter(|(k, _)| k != "utm_source")
                        .map(|(k, v)| (k.into_owned(), v.into_owned()))
                        .collect();
                    url.query_pairs_mut().clear().extend_pairs(&query);

                    Some(String::from(url.as_str()).replace("?", ""))
                }
                false => Some(formatted),
            }
        }
        Err(e) => {
            println!("Error: {}", e);
            None
        }
    }
}
