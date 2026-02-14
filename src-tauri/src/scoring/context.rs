use crate::database::repository::CardData;

#[derive(Debug, Clone)]
pub struct ContextModifier {
    pub condition: String,
    pub card_tag: String,
    pub modifier: i32,
    pub priority: String,
    pub description: String,
}

pub fn calculate_context_bonus(
    card: &CardData,
    current_deck: &[CardData],
    modifiers: &[ContextModifier],
) -> i32 {
    let mut total_bonus = 0;
    
    for modifier in modifiers {
        if should_apply_modifier(card, current_deck, modifier) {
            total_bonus += modifier.modifier;
        }
    }
    
    total_bonus
}

fn should_apply_modifier(
    card: &CardData,
    current_deck: &[CardData],
    modifier: &ContextModifier,
) -> bool {
    // Check if card has the required tag
    if !card.keywords.iter().any(|k| k == &modifier.card_tag) {
        return false;
    }
    
    // Check the condition
    match modifier.condition.as_str() {
        "missing_frontline" => {
            // Check if deck lacks frontline units
            !current_deck.iter().any(|c| {
                c.keywords.iter().any(|k| k == "frontline" || k == "tank")
            })
        }
        "missing_backline_clear" => {
            // Check if deck lacks backline clear
            !current_deck.iter().any(|c| {
                c.keywords.iter().any(|k| k == "sweep" || k == "explosive" || k == "advance")
            })
        }
        "has_reform_synergy" => {
            // Check if deck has Reform cards
            current_deck.iter().any(|c| {
                c.keywords.iter().any(|k| k == "reform")
            })
        }
        "has_consume_synergy" => {
            // Check if deck has Consume triggers
            current_deck.iter().any(|c| {
                c.keywords.iter().any(|k| k == "consume")
            })
        }
        "deck_size_over_20" => {
            current_deck.len() > 20
        }
        "covenant_high" => {
            // This would need covenant parameter
            false
        }
        "ring_early" => {
            // This would need ring parameter
            false
        }
        "ring_late" => {
            // This would need ring parameter
            false
        }
        "duplicate_common" => {
            // Check for duplicate commons
            let common_count = current_deck
                .iter()
                .filter(|c| c.rarity == "Common" && c.id == card.id)
                .count();
            common_count >= 2
        }
        "has_forge_synergy" => {
            current_deck.iter().any(|c| {
                c.keywords.iter().any(|k| k == "forge")
            })
        }
        "has_smelt_synergy" => {
            current_deck.iter().any(|c| {
                c.keywords.iter().any(|k| k == "smelt")
            })
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_card_with_tags(id: &str, tags: Vec<&str>) -> CardData {
        CardData {
            id: id.to_string(),
            name: id.to_string(),
            clan: "Test".to_string(),
            card_type: "Unit".to_string(),
            rarity: "Common".to_string(),
            cost: Some(1),
            base_value: 70,
            tempo_score: 6,
            value_score: 7,
            keywords: tags.iter().map(|s| s.to_string()).collect(),
            description: "Test".to_string(),
            expansion: "base".to_string(),
        }
    }
    
    #[test]
    fn test_missing_frontline_detection() {
        let tank_card = create_test_card_with_tags("tank", vec!["frontline", "tank"]);
        let empty_deck: Vec<CardData> = vec![];
        
        let modifier = ContextModifier {
            condition: "missing_frontline".to_string(),
            card_tag: "frontline".to_string(),
            modifier: 15,
            priority: "High".to_string(),
            description: "No tank".to_string(),
        };
        
        assert!(should_apply_modifier(&tank_card, &empty_deck, &modifier));
    }
    
    #[test]
    fn test_has_frontline_no_bonus() {
        let tank_card = create_test_card_with_tags("tank", vec!["frontline", "tank"]);
        let existing_tank = create_test_card_with_tags("existing", vec!["frontline"]);
        
        let modifier = ContextModifier {
            condition: "missing_frontline".to_string(),
            card_tag: "frontline".to_string(),
            modifier: 15,
            priority: "High".to_string(),
            description: "No tank".to_string(),
        };
        
        assert!(!should_apply_modifier(&tank_card, &[existing_tank], &modifier));
    }
    
    #[test]
    fn test_missing_backline_clear() {
        let sweep_card = create_test_card_with_tags("sweep", vec!["sweep"]);
        let empty_deck: Vec<CardData> = vec![];
        
        let modifier = ContextModifier {
            condition: "missing_backline_clear".to_string(),
            card_tag: "sweep".to_string(),
            modifier: 20,
            priority: "Critical".to_string(),
            description: "No clear".to_string(),
        };
        
        assert!(should_apply_modifier(&sweep_card, &empty_deck, &modifier));
    }
}
