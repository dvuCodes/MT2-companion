use crate::database::repository::CardData;
use crate::scoring::{context, context::ContextModifier, synergies::Synergy};
use serde::{Deserialize, Serialize};

const SYNERGY_CAP: f64 = 1.5;
const MAX_SCORE: i32 = 120;
const S_TIER_THRESHOLD: i32 = 90;
const A_TIER_THRESHOLD: i32 = 80;
const B_TIER_THRESHOLD: i32 = 70;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringResult {
    pub score: i32,
    pub tier: String,
    pub base_value: i32,
    pub synergy_multiplier: f64,
    pub context_bonus: i32,
    pub champion_bonus: i32,
    pub reasons: Vec<String>,
}

pub struct ScoreCalculator;

impl ScoreCalculator {
    pub fn new() -> Self {
        Self
    }

    #[cfg(test)]
    pub fn new_test() -> Self {
        Self
    }

    pub fn calculate_base(&self, card: &CardData) -> i32 {
        card.base_value
    }

    pub fn calculate_synergy_multiplier(
        &self,
        card: &CardData,
        current_deck: &[CardData],
        synergies: Vec<Synergy>,
    ) -> f64 {
        let mut multiplier = 1.0;
        let mut reasons = Vec::new();

        for deck_card in current_deck {
            for synergy in &synergies {
                // Check if this pair matches
                let matches = (synergy.card_a_id == card.id && synergy.card_b_id == deck_card.id)
                    || (synergy.card_b_id == card.id && synergy.card_a_id == deck_card.id)
                    || (synergy.card_b_id == "*"
                        && card.keywords.iter().any(|k| k == &synergy.synergy_type));

                if matches {
                    multiplier += synergy.weight - 1.0;
                    reasons.push(synergy.description.clone());
                }
            }
        }

        // Cap at SYNERGY_CAP
        multiplier.min(SYNERGY_CAP)
    }

    pub fn calculate_full(
        &self,
        card: &CardData,
        current_deck: &[CardData],
        champion: &str,
        ring_number: i32,
        covenant: i32,
        synergies: &[Synergy],
        context_modifiers: &[ContextModifier],
        champion_override: Option<i32>,
    ) -> ScoringResult {
        let mut reasons = Vec::new();

        // 1. Base value
        let base_value = self.calculate_base(card);

        // 2. Synergy multiplier
        let synergy_multiplier =
            self.calculate_synergy_multiplier(card, current_deck, synergies.to_vec());
        let synergy_score = (base_value as f64 * synergy_multiplier) as i32;

        if synergy_multiplier > 1.0 {
            reasons.push(format!(
                "Synergy bonus: {:.0}%",
                (synergy_multiplier - 1.0) * 100.0
            ));
        }

        // 3. Context bonus
        let context_bonus = context::calculate_context_bonus(card, current_deck, context_modifiers);
        if context_bonus > 0 {
            reasons.push(format!("Context: +{}", context_bonus));
        }

        // 4. Champion override
        let champion_bonus = if let Some(override_val) = champion_override {
            reasons.push(format!("Champion favorite: {}", champion));
            override_val - base_value
        } else {
            0
        };

        // 5. Ring adjustment
        let ring_adjustment = if ring_number <= 3 && card.tempo_score > card.value_score {
            reasons.push("Early game tempo".to_string());
            10
        } else if ring_number >= 6 && card.value_score > card.tempo_score {
            reasons.push("Late game value".to_string());
            10
        } else {
            0
        };

        // Calculate final score
        let score =
            (synergy_score + context_bonus + champion_bonus + ring_adjustment).min(MAX_SCORE);

        // Determine tier
        let tier = if score >= S_TIER_THRESHOLD {
            "S".to_string()
        } else if score >= A_TIER_THRESHOLD {
            "A".to_string()
        } else if score >= B_TIER_THRESHOLD {
            "B".to_string()
        } else {
            "C".to_string()
        };

        ScoringResult {
            score,
            tier,
            base_value,
            synergy_multiplier,
            context_bonus,
            champion_bonus,
            reasons,
        }
    }

    pub fn calculate_with_database(
        &self,
        card_id: &str,
        current_deck_ids: &[String],
        champion: &str,
        ring_number: i32,
        covenant: i32,
    ) -> Result<ScoringResult, String> {
        // This would query the database in the actual implementation
        // For now, return an error indicating this needs the database
        Err("Database integration required".to_string())
    }
}
