#[derive(Debug, Clone)]
pub struct Synergy {
    pub card_a_id: String,
    pub card_b_id: String,
    pub synergy_type: String,
    pub weight: f64,
    pub description: String,
    pub bidirectional: bool,
}

impl Synergy {
    pub fn applies_to(&self, card_id: &str, other_card_id: &str) -> bool {
        let forward = self.card_a_id == card_id && self.card_b_id == other_card_id;
        let backward =
            self.bidirectional && self.card_b_id == card_id && self.card_a_id == other_card_id;
        forward || backward
    }

    pub fn applies_to_keyword(&self, card_keywords: &[String], other_keywords: &[String]) -> bool {
        // Check if keywords match the synergy type
        let type_matches = card_keywords.iter().any(|k| k == &self.synergy_type)
            || other_keywords.iter().any(|k| k == &self.synergy_type);
        type_matches
    }
}

/// Get synergies for a specific card
pub fn get_synergies_for_card<'a>(card_id: &'a str, all_synergies: &'a [Synergy]) -> Vec<&'a Synergy> {
    all_synergies
        .iter()
        .filter(|s| s.card_a_id == card_id || s.card_b_id == card_id || s.card_b_id == "*")
        .collect()
}

/// Get synergies between two specific cards
pub fn get_synergy_between<'a>(
    card_a_id: &'a str,
    card_b_id: &'a str,
    all_synergies: &'a [Synergy],
) -> Option<&'a Synergy> {
    all_synergies
        .iter()
        .find(|s| s.applies_to(card_a_id, card_b_id))
}

/// Get all synergies for a deck (checking every pair)
pub fn get_deck_synergies<'a>(
    deck_ids: &'a [String],
    all_synergies: &'a [Synergy],
) -> Vec<(&'a String, &'a String, &'a Synergy)> {
    let mut results = Vec::new();

    for i in 0..deck_ids.len() {
        for j in (i + 1)..deck_ids.len() {
            if let Some(synergy) = get_synergy_between(&deck_ids[i], &deck_ids[j], all_synergies) {
                results.push((&deck_ids[i], &deck_ids[j], synergy));
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_synergy(a: &str, b: &str, weight: f64, bidirectional: bool) -> Synergy {
        Synergy {
            card_a_id: a.to_string(),
            card_b_id: b.to_string(),
            synergy_type: "test".to_string(),
            weight,
            description: "Test synergy".to_string(),
            bidirectional,
        }
    }

    #[test]
    fn test_synergy_applies_forward() {
        let synergy = create_test_synergy("card_a", "card_b", 1.2, true);

        assert!(synergy.applies_to("card_a", "card_b"));
        assert!(!synergy.applies_to("card_b", "card_c"));
    }

    #[test]
    fn test_synergy_applies_backward_when_bidirectional() {
        let synergy = create_test_synergy("card_a", "card_b", 1.2, true);

        assert!(synergy.applies_to("card_b", "card_a"));
    }

    #[test]
    fn test_synergy_not_applies_backward_when_unidirectional() {
        let synergy = create_test_synergy("card_a", "card_b", 1.2, false);

        assert!(!synergy.applies_to("card_b", "card_a"));
    }

    #[test]
    fn test_get_synergies_for_card() {
        let synergies = vec![
            create_test_synergy("card_a", "card_b", 1.2, true),
            create_test_synergy("card_a", "card_c", 1.3, true),
            create_test_synergy("card_b", "card_c", 1.4, true),
        ];

        let card_a_synergies = get_synergies_for_card("card_a", &synergies);
        assert_eq!(card_a_synergies.len(), 2);

        let card_b_synergies = get_synergies_for_card("card_b", &synergies);
        assert_eq!(card_b_synergies.len(), 2); // bidirectional
    }

    #[test]
    fn test_get_synergy_between() {
        let synergies = vec![
            create_test_synergy("card_a", "card_b", 1.2, true),
            create_test_synergy("card_c", "card_d", 1.3, true),
        ];

        let result = get_synergy_between("card_a", "card_b", &synergies);
        assert!(result.is_some());
        assert_eq!(result.unwrap().weight, 1.2);

        let no_result = get_synergy_between("card_a", "card_c", &synergies);
        assert!(no_result.is_none());
    }

    #[test]
    fn test_get_deck_synergies() {
        let synergies = vec![
            create_test_synergy("card_a", "card_b", 1.2, true),
            create_test_synergy("card_b", "card_c", 1.3, true),
        ];

        let deck = vec![
            "card_a".to_string(),
            "card_b".to_string(),
            "card_c".to_string(),
        ];
        let deck_synergies = get_deck_synergies(&deck, &synergies);

        assert_eq!(deck_synergies.len(), 2);
    }
}
