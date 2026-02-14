pub mod calculator;
pub mod context;
pub mod synergies;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::repository::CardData;
    
    fn create_test_card(id: &str, base_value: i32, tempo: i32, value: i32, keywords: Vec<&str>) -> CardData {
        CardData {
            id: id.to_string(),
            name: id.to_string(),
            clan: "Test".to_string(),
            card_type: "Unit".to_string(),
            rarity: "Common".to_string(),
            cost: Some(1),
            base_value,
            tempo_score: tempo,
            value_score: value,
            keywords: keywords.iter().map(|s| s.to_string()).collect(),
            description: "Test card".to_string(),
            expansion: "base".to_string(),
        }
    }
    
    #[test]
    fn test_base_value_calculation() {
        let card = create_test_card("test_card", 75, 6, 7, vec!["frontline"]);
        let calculator = calculator::ScoreCalculator::new_test();
        
        let score = calculator.calculate_base(&card);
        assert_eq!(score, 75);
    }
    
    #[test]
    fn test_synergy_multiplier_single() {
        let card = create_test_card("card_a", 75, 6, 7, vec!["shift"]);
        let deck_card = create_test_card("card_b", 70, 6, 7, vec!["shift"]);
        
        let calculator = calculator::ScoreCalculator::new_test();
        let multiplier = calculator.calculate_synergy_multiplier(&card, &[deck_card],
            vec![synergies::Synergy {
                card_a_id: "card_a".to_string(),
                card_b_id: "card_b".to_string(),
                synergy_type: "test".to_string(),
                weight: 1.20,
                description: "Test synergy".to_string(),
                bidirectional: true,
            }]
        );
        
        assert!((multiplier - 1.20).abs() < 0.01);
    }
    
    #[test]
    fn test_synergy_multiplier_capped() {
        let card = create_test_card("card_a", 75, 6, 7, vec!["shift"]);
        let deck_cards = vec![
            create_test_card("card_b", 70, 6, 7, vec!["shift"]),
            create_test_card("card_c", 70, 6, 7, vec!["shift"]),
            create_test_card("card_d", 70, 6, 7, vec!["shift"]),
        ];
        
        let calculator = calculator::ScoreCalculator::new_test();
        let synergy = synergies::Synergy {
            card_a_id: "card_a".to_string(),
            card_b_id: "*".to_string(),
            synergy_type: "test".to_string(),
            weight: 1.30,
            description: "Test".to_string(),
            bidirectional: true,
        };
        
        let multiplier = calculator.calculate_synergy_multiplier(&card, &deck_cards,
            deck_cards.iter().map(|_| synergy.clone()).collect()
        );
        
        // Should be capped at 1.5
        assert!(multiplier <= 1.5);
    }
    
    #[test]
    fn test_context_modifier_missing_frontline() {
        let card = create_test_card("tank_card", 70, 6, 8, vec!["frontline", "tank"]);
        let empty_deck: Vec<CardData> = vec![];
        
        let context_mods = vec![
            context::ContextModifier {
                condition: "missing_frontline".to_string(),
                card_tag: "frontline".to_string(),
                modifier: 15,
                priority: "High".to_string(),
                description: "No tank units".to_string(),
            }
        ];
        
        let context_bonus = context::calculate_context_bonus(&card, &empty_deck, &context_mods
        );
        
        assert_eq!(context_bonus, 15);
    }
    
    #[test]
    fn test_context_modifier_no_bonus_when_present() {
        let tank_card = create_test_card("tank_card", 70, 6, 8, vec!["frontline", "tank"]);
        let existing_tank = create_test_card("existing_tank", 70, 6, 8, vec!["frontline"]);
        
        let context_mods = vec![
            context::ContextModifier {
                condition: "missing_frontline".to_string(),
                card_tag: "frontline".to_string(),
                modifier: 15,
                priority: "High".to_string(),
                description: "No tank units".to_string(),
            }
        ];
        
        let context_bonus = context::calculate_context_bonus(
            &tank_card, &[existing_tank], &context_mods
        );
        
        assert_eq!(context_bonus, 0);
    }
    
    #[test]
    fn test_full_calculation_with_all_factors() {
        // Test a complete calculation scenario
        let card = create_test_card("deadly_plunge", 92, 8, 10, vec!["removal", "sacrifice"]);
        let deck = vec![
            create_test_card("titan_sentry", 79, 6, 8, vec!["frontline", "tank"]),
        ];
        
        let synergies = vec![
            synergies::Synergy {
                card_a_id: "deadly_plunge".to_string(),
                card_b_id: "titan_sentry".to_string(),
                synergy_type: "sacrifice_value".to_string(),
                weight: 1.25,
                description: "High HP target".to_string(),
                bidirectional: true,
            }
        ];
        
        let context_mods = vec![]; // No context bonuses needed
        
        let calculator = calculator::ScoreCalculator::new_test();
        let result = calculator.calculate_full(
            &card,
            &deck,
            "Fel",
            1,
            10,
            &synergies,
            &context_mods,
            None, // No champion override
        );
        
        // Base 92 * 1.25 synergy = 115
        assert!(result.score > 90);
        assert!(result.score <= 120);
    }
}
