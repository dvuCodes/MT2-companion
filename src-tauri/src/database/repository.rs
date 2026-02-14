use rusqlite::{Connection, Result};
use serde_json;

pub fn seed_data(conn: &Connection) -> Result<()> {
    // Disable foreign keys temporarily to allow seeding data
    // Some synergies and overrides reference cards that may not exist yet
    conn.execute( "PRAGMA foreign_keys = OFF", [])?;
    
    seed_expansions(conn)?;
    seed_cards(conn)?;
    seed_synergies(conn)?;
    seed_context_modifiers(conn)?;
    seed_champion_overrides(conn)?;
    
    // Re-enable foreign keys
    conn.execute( "PRAGMA foreign_keys = on ", [])?;
    
    Ok(())
}

fn seed_expansions(conn: &Connection) -> Result<()> {
    let expansions = vec![
        (
            "base",
            "Base game",
            "2025-05",
            "Original Monster Train 2 release",
        ),
        (
            "railforged",
            "Destiny of the Railforged",
            "2026-02-02",
            "Herzal and Heph join the fight with Forge Points and Smelt mechanics",
        ),
        (
            "wurmkin",
            "Wurmkin Clan",
            "2026-02-02",
            "Free update with Charged Echoes mechanic",
        ),
    ];

    for (id, name, date, desc) in expansions {
        conn.execute(
            "INSERT OR IGNORE INTO expansions (id, name, release_date, description) VALUES (?1, ?2, ?3, ?4)",
            [id, name, date, desc],
        )?;
    }
    Ok(())
}

fn seed_cards(conn: &Connection) -> Result<()> {
    let cards = get_all_cards_data();

    for card in cards {
        let keywords_json = serde_json::to_string(&card.keywords).unwrap_or_default();

        conn.execute(
            "INSERT OR IGNORE INTO cards 
             (id, name, clan, card_type, rarity, cost, base_value, tempo_score, value_score, keywords, description, expansion)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                &card.id,
                &card.name,
                &card.clan,
                &card.card_type,
                &card.rarity,
                card.cost,
                card.base_value,
                card.tempo_score,
                card.value_score,
                keywords_json,
                &card.description,
                &card.expansion,
            ],
        )?;
    }

    Ok(())
}

fn seed_synergies(conn: &Connection) -> Result<()> {
    let synergies = vec![
        // Banished synergies
        (
            "banished_deadly_plunge",
            "banished_titan_sentry",
            "sacrifice_value",
            1.25,
            "High HP target for sacrifice",
            true,
        ),
        (
            "banished_fel",
            "banished_just_cause",
            "champion_synergy",
            1.35,
            "Shift triggers Valor gain",
            false,
        ),
        (
            "banished_fel",
            "banished_selfless_sacrifice",
            "champion_synergy",
            1.30,
            "Shift enabler",
            false,
        ),
        (
            "banished_karmic_censer",
            "banished_just_cause",
            "double_trigger",
            1.60,
            "Shift triggers twice",
            false,
        ),
        // Underlegion synergies
        (
            "underlegion_morel_mistress",
            "underlegion_funguy_in_a_suit",
            "consume_loop",
            1.40,
            "Triggers on consume",
            true,
        ),
        (
            "underlegion_madame_lionsmane",
            "underlegion_morel_mistress",
            "champion_synergy",
            1.45,
            "Consume synergy with Sporesinger",
            false,
        ),
        (
            "underlegion_bolete",
            "underlegion_funguy_in_a_suit",
            "champion_synergy",
            1.40,
            "Funguy spawn synergy",
            false,
        ),
        // Pyreborne synergies
        (
            "pyreborne_fanning_the_flame",
            "pyreborne_pyregel_card",
            "snowball",
            1.20,
            "Lower HP from Pyregel = more kills",
            true,
        ),
        (
            "pyreborne_gildmonger",
            "upgrades_endless",
            "upgrade_synergy",
            1.50,
            "Infinite Dragon Hoard generation",
            false,
        ),
        (
            "pyreborne_lady_gilda",
            "pyreborne_bloated_whelp",
            "champion_synergy",
            1.50,
            "Whelp synergy with Bloat Matron",
            false,
        ),
        // Luna Coven synergies
        (
            "luna_coven_moonlit_glaive",
            "luna_coven_conduit",
            "scaling",
            1.50,
            "+3 attack per Magic Power",
            true,
        ),
        (
            "luna_coven_ekka",
            "luna_coven_witchweave",
            "champion_synergy",
            1.35,
            "0-cost Conduit trigger",
            false,
        ),
        // Melting Remnant synergies
        (
            "melting_remnant_rector_flicker",
            "melting_remnant_waxen_spike",
            "champion_synergy",
            1.35,
            "Burnout synergy with Reform",
            false,
        ),
        // Lazarus League synergies
        (
            "lazarus_league_plague_doctor",
            "lazarus_league_rage",
            "damage_amplification",
            1.30,
            "More damage = more Unstable stacks",
            false,
        ),
        (
            "lazarus_league_orechi",
            "lazarus_league_potion_kit",
            "champion_synergy",
            1.40,
            "Core equipment for Mix",
            false,
        ),
        // Railforged synergies (NEW EXPANSION)
        (
            "railforged_herzal_blacksmith",
            "railforged_forge_steward",
            "forge_synergy",
            1.30,
            "Forge point generation",
            true,
        ),
        (
            "railforged_heph_handy",
            "railforged_equipment",
            "equipment_synergy",
            1.40,
            "Multiple equipment slots",
            false,
        ),
        (
            "railforged_knuckler_steward",
            "railforged_full_throttle",
            "burst_synergy",
            1.35,
            "Burst stacks synergy",
            true,
        ),
    ];

    for (card_a, card_b, synergy_type, weight, desc, bidirectional) in synergies {
        conn.execute(
            "INSERT OR IGNORE INTO synergies 
             (card_a_id, card_b_id, synergy_type, weight, description, bidirectional)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                card_a,
                card_b,
                synergy_type,
                weight,
                desc,
                bidirectional,
            ],
        )?;
    }

    Ok(())
}

fn seed_context_modifiers(conn: &Connection) -> Result<()> {
    let modifiers = vec![
        (
            "missing_frontline",
            "frontline",
            15,
            "High",
            "No tank units in deck (HP<30)",
        ),
        (
            "missing_backline_clear",
            "sweep",
            20,
            "Critical",
            "No Sweep Explosive or Advance",
        ),
        (
            "missing_backline_clear",
            "explosive",
            20,
            "Critical",
            "No Sweep Explosive or Advance",
        ),
        (
            "has_reform_synergy",
            "burnout",
            25,
            "High",
            "Has Reform cards Burnout valued higher",
        ),
        (
            "deck_size_over_20",
            "draw",
            -10,
            "Medium",
            "Deck too large draw less valuable",
        ),
        (
            "covenant_high",
            "scaling",
            10,
            "Medium",
            "Covenant 15+ scaling matters more",
        ),
        (
            "has_consume_synergy",
            "consume",
            30,
            "High",
            "Morel Mistress or similar present",
        ),
        (
            "low_gold",
            "gold",
            15,
            "Medium",
            "<100 gold gold generation priority",
        ),
        (
            "no_pyregel",
            "pyregel",
            -10,
            "Low",
            "No pyregel applicators",
        ),
        (
            "ring_early",
            "tempo",
            15,
            "High",
            "Ring 1-3 tempo cards better",
        ),
        (
            "ring_late",
            "value",
            15,
            "High",
            "Ring 6+ value cards better",
        ),
        (
            "duplicate_common",
            "common ",
            -5,
            "Low",
            "3rd+ copy of common",
        ),
        (
            "has_forge_synergy",
            "forge",
            20,
            "High",
            "Forge points available",
        ),
        (
            "has_smelt_synergy",
            "smelt",
            25,
            "High",
            "Smelt mechanic present",
        ),
    ];

    for (condition, tag, modifier, priority, desc) in modifiers {
        conn.execute(
            "INSERT OR IGNORE INTO context_modifiers 
             (condition, card_tag, modifier, priority, description)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![condition, tag, modifier, priority, desc],
        )?;
    }

    Ok(())
}

fn seed_champion_overrides(conn: &Connection) -> Result<()> {
    let overrides = vec![
        // Banished
        (
            "Fel",
            "Unchained ",
            "banished_just_cause",
            95,
            "Shift = permanent Valor",
        ),
        (
            "Fel",
            "Unchained ",
            "banished_selfless_sacrifice",
            90,
            "Shift enabler",
        ),
        (
            "Fel",
            "Savior ",
            "banished_armor_cards",
            85,
            "Scales with Valor",
        ),
        (
            "Talos",
            "Flight ",
            "banished_rising_rage",
            90,
            "Flight synergy",
        ),
        // Pyreborne
        (
            "Lord Fenix",
            "Any ",
            "pyreborne_pyreborne_brainspike",
            90,
            "Cooldown reduction",
        ),
        (
            "Lady Gilda",
            "Bloat Matron ",
            "pyreborne_bloated_whelp",
            95,
            "Core synergy",
        ),
        (
            "Lady Gilda",
            "Bloat Matron ",
            "pyreborne_gildmonger",
            92,
            "Dragon Hoard scaling",
        ),
        // Luna Coven
        (
            "Ekka",
            "Spellweaver ",
            "luna_coven_witchweave",
            92,
            "0-cost Conduit ",
        ),
        (
            "Ekka",
            "Spellweaver ",
            "luna_coven_moonlit_glaive",
            98,
            "S-tier equipment",
        ),
        // Underlegion
        (
            "Bolete the Guillotine",
            "Any ",
            "underlegion_funguy_in_a_suit",
            88,
            "Funguy spawn",
        ),
        (
            "Madame Lionsmane",
            "Sporesinger ",
            "underlegion_morel_mistress",
            95,
            "Consume synergy",
        ),
        // Lazarus League
        (
            "Orechi",
            "Brewmaster ",
            "lazarus_league_potion_kit",
            90,
            "Core equipment",
        ),
        // Melting Remnant
        (
            "Rector Flicker",
            "Any ",
            "melting_remnant_waxen_spike",
            88,
            "Burnout synergy ",
        ),
        // Railforged (NEW)
        (
            "Herzal",
            "Blacksmith",
            "railforged_forge_steward",
            92,
            "Forge generation",
        ),
        (
            "Herzal",
            "Pyresmith ",
            "railforged_pyreshot",
            88,
            "Pyreshot synergy",
        ),
        (
            "Heph",
            "Handy ",
            "railforged_equipment",
            90,
            "Equipment slots",
        ),
        (
            "Heph",
            "Metalworker",
            "railforged_scrap_metal",
            85,
            "Scrap Metal synergy",
        ),
    ];

    for (champion, path, card_id, value_override, reason) in overrides {
        conn.execute(
            "INSERT OR IGNORE INTO champion_overrides 
             (champion, path, card_id, value_override, reason)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![champion, path, card_id, value_override, reason],
        )?;
    }

    Ok(())
}

// Card data structure
#[derive(Debug, Clone)]
pub struct CardData {
    pub id: String,
    pub name: String,
    pub clan: String,
    pub card_type: String,
    pub rarity: String,
    pub cost: Option<i32>,
    pub base_value: i32,
    pub tempo_score: i32,
    pub value_score: i32,
    pub keywords: Vec<String>,
    pub description: String,
    pub expansion: String,
}

fn get_all_cards_data() -> Vec<CardData> {
    let mut cards = Vec::new();

    // BANISHED (Base Game)
    cards.extend(vec![
        CardData {
            id: "banished_fel".to_string(),
            name: "Fel".to_string(),
            clan: "Banished".to_string(),
            card_type: "Champion".to_string(),
            rarity: "Champion".to_string(),
            cost: None,
            base_value: 85,
            tempo_score: 7,
            value_score: 8,
            keywords: vec![
                "multistrike".to_string(),
                "valor".to_string(),
                "revenge".to_string(),
            ],
            description: "Champion with Valor generation and multistrike capabilities".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "banished_talos".to_string(),
            name: "Talos".to_string(),
            clan: "Banished".to_string(),
            card_type: "Champion".to_string(),
            rarity: "Champion".to_string(),
            cost: None,
            base_value: 82,
            tempo_score: 8,
            value_score: 7,
            keywords: vec![
                "flight".to_string(),
                "shift".to_string(),
                "valor".to_string(),
            ],
            description: "Champion with Flight ability for consistent shifting".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "banished_just_cause".to_string(),
            name: "Just Cause".to_string(),
            clan: "Banished".to_string(),
            card_type: "Spell".to_string(),
            rarity: "Common".to_string(),
            cost: Some(0),
            base_value: 75,
            tempo_score: 9,
            value_score: 6,
            keywords: vec![
                "advance".to_string(),
                "shift".to_string(),
                "tempo".to_string(),
            ],
            description: "0-cost Advance spell - core to Banished strategy".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "banished_cleave".to_string(),
            name: "Cleave".to_string(),
            clan: "Banished".to_string(),
            card_type: "Spell".to_string(),
            rarity: "Common".to_string(),
            cost: Some(1),
            base_value: 70,
            tempo_score: 7,
            value_score: 6,
            keywords: vec![
                "sweep".to_string(),
                "backline_clear".to_string(),
                "aoe".to_string(),
            ],
            description: "Backline clear - essential for Covenant 10+".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "banished_steadfast_crusader".to_string(),
            name: "Steadfast Crusader".to_string(),
            clan: "Banished".to_string(),
            card_type: "Unit".to_string(),
            rarity: "Uncommon".to_string(),
            cost: Some(3),
            base_value: 78,
            tempo_score: 6,
            value_score: 8,
            keywords: vec![
                "advance".to_string(),
                "tank".to_string(),
                "frontline".to_string(),
                "valor".to_string(),
            ],
            description: "Tank with built-in Advance - excellent for Fel combos".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "banished_deadly_plunge".to_string(),
            name: "Deadly Plunge".to_string(),
            clan: "Banished".to_string(),
            card_type: "Spell".to_string(),
            rarity: "Rare".to_string(),
            cost: Some(1),
            base_value: 92,
            tempo_score: 8,
            value_score: 10,
            keywords: vec![
                "consume".to_string(),
                "lifesteal".to_string(),
                "sacrifice".to_string(),
                "removal".to_string(),
                "boss_killer".to_string(),
            ],
            description: "Kill a unit deal 3x HP damage Lifesteal. S-tier removal".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "banished_karmic_censer".to_string(),
            name: "Karmic Censer".to_string(),
            clan: "Banished".to_string(),
            card_type: "Artifact".to_string(),
            rarity: "Rare".to_string(),
            cost: Some(0),
            base_value: 88,
            tempo_score: 8,
            value_score: 9,
            keywords: vec![
                "artifact".to_string(),
                "shift".to_string(),
                "combo".to_string(),
            ],
            description: "Shift triggers twice - broken with combo decks".to_string(),
            expansion: "base".to_string(),
        },
    ]);

    // PYREBORNE (Base Game)
    cards.extend(vec![
        CardData {
            id: "pyreborne_lord_fenix".to_string(),
            name: "Lord Fenix".to_string(),
            clan: "Pyreborne".to_string(),
            card_type: "Champion".to_string(),
            rarity: "Champion".to_string(),
            cost: None,
            base_value: 84,
            tempo_score: 7,
            value_score: 8,
            keywords: vec![
                "dragon".to_string(),
                "pyregel".to_string(),
                "incant".to_string(),
                "spell_synergy".to_string(),
            ],
            description: "Dragon champion with Pyregel application".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "pyreborne_lady_gilda".to_string(),
            name: "Lady Gilda".to_string(),
            clan: "Pyreborne".to_string(),
            card_type: "Champion".to_string(),
            rarity: "Champion".to_string(),
            cost: None,
            base_value: 83,
            tempo_score: 6,
            value_score: 9,
            keywords: vec![
                "whelp".to_string(),
                "dragon_hoard".to_string(),
                "avarice".to_string(),
                "gold".to_string(),
            ],
            description: "Dragon champion with Dragon Hoard synergy".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "pyreborne_fanning_the_flame".to_string(),
            name: "Fanning the Flame".to_string(),
            clan: "Pyreborne".to_string(),
            card_type: "Spell".to_string(),
            rarity: "Uncommon".to_string(),
            cost: Some(1),
            base_value: 86,
            tempo_score: 8,
            value_score: 9,
            keywords: vec![
                "explosive".to_string(),
                "snowball".to_string(),
                "backline_clear".to_string(),
                "scaling_damage".to_string(),
            ],
            description: "S-tier snowballing spell - damage increases on kill".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "pyreborne_gildmonger".to_string(),
            name: "Gildmonger".to_string(),
            clan: "Pyreborne".to_string(),
            card_type: "Unit".to_string(),
            rarity: "Uncommon".to_string(),
            cost: Some(1),
            base_value: 79,
            tempo_score: 7,
            value_score: 8,
            keywords: vec![
                "dragon".to_string(),
                "dragon_hoard".to_string(),
                "value".to_string(),
                "gold".to_string(),
            ],
            description: "Gains Dragon Hoard on death - excellent with Endless".to_string(),
            expansion: "base".to_string(),
        },
    ]);

    // LUNA COVEN (Base Game)
    cards.extend(vec![
        CardData {
            id: "luna_coven_ekka".to_string(),
            name: "Ekka".to_string(),
            clan: "Luna Coven".to_string(),
            card_type: "Champion".to_string(),
            rarity: "Champion".to_string(),
            cost: None,
            base_value: 86,
            tempo_score: 7,
            value_score: 9,
            keywords: vec![
                "conduit".to_string(),
                "magic_power".to_string(),
                "spell_buff".to_string(),
            ],
            description: "Spell power champion with Conduit".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "luna_coven_witchweave".to_string(),
            name: "Witchweave".to_string(),
            clan: "Luna Coven".to_string(),
            card_type: "Spell".to_string(),
            rarity: "Common".to_string(),
            cost: Some(0),
            base_value: 76,
            tempo_score: 8,
            value_score: 6,
            keywords: vec![
                "free".to_string(),
                "flexible".to_string(),
                "conduit_trigger".to_string(),
            ],
            description: "0-cost damage or heal - excellent for Conduit".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "luna_coven_moonlit_glaive".to_string(),
            name: "Moonlit Glaive".to_string(),
            clan: "Luna Coven".to_string(),
            card_type: "Equipment".to_string(),
            rarity: "Rare".to_string(),
            cost: Some(3),
            base_value: 91,
            tempo_score: 7,
            value_score: 10,
            keywords: vec![
                "equipment".to_string(),
                "magic_power".to_string(),
                "scaling".to_string(),
                "s_tier".to_string(),
            ],
            description: "S-tier equipment: +3 attack per Magic Power".to_string(),
            expansion: "base".to_string(),
        },
    ]);

    // UNDERLEGION (Base Game)
    cards.extend(vec![
        CardData {
            id: "underlegion_bolete".to_string(),
            name: "Bolete the Guillotine".to_string(),
            clan: "Underlegion".to_string(),
            card_type: "Champion".to_string(),
            rarity: "Champion".to_string(),
            cost: None,
            base_value: 88,
            tempo_score: 8,
            value_score: 9,
            keywords: vec![
                "funguy".to_string(),
                "rally".to_string(),
                "spawn".to_string(),
                "spore".to_string(),
            ],
            description: "Funguy spawn champion with Rally".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "underlegion_madame_lionsmane".to_string(),
            name: "Madame Lionsmane".to_string(),
            clan: "Underlegion".to_string(),
            card_type: "Champion".to_string(),
            rarity: "Champion".to_string(),
            cost: None,
            base_value: 87,
            tempo_score: 7,
            value_score: 9,
            keywords: vec![
                "funguy".to_string(),
                "spawn".to_string(),
                "spore_scaling".to_string(),
            ],
            description: "Funguy champion with Sporesinger path".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "underlegion_morel_mistress".to_string(),
            name: "Morel Mistress".to_string(),
            clan: "Underlegion".to_string(),
            card_type: "Unit".to_string(),
            rarity: "Uncommon".to_string(),
            cost: Some(2),
            base_value: 84,
            tempo_score: 7,
            value_score: 9,
            keywords: vec![
                "consume".to_string(),
                "buff".to_string(),
                "funguy".to_string(),
                "value".to_string(),
            ],
            description: "S-tier with consume triggers - buffs on consume".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "underlegion_funguy_in_a_suit".to_string(),
            name: "Funguy in a Suit".to_string(),
            clan: "Underlegion".to_string(),
            card_type: "Unit".to_string(),
            rarity: "Common".to_string(),
            cost: Some(1),
            base_value: 72,
            tempo_score: 7,
            value_score: 6,
            keywords: vec![
                "funguy".to_string(),
                "consume".to_string(),
                "sacrifice_value ".to_string(),
            ],
            description: "Consume trigger Funguy".to_string(),
            expansion: "base".to_string(),
        },
    ]);

    // LAZARUS LEAGUE (Base Game)
    cards.extend(vec![
        CardData {
            id: "lazarus_league_orechi".to_string(),
            name: "Orechi".to_string(),
            clan: "Lazarus League".to_string(),
            card_type: "Champion".to_string(),
            rarity: "Champion".to_string(),
            cost: None,
            base_value: 85,
            tempo_score: 6,
            value_score: 9,
            keywords: vec![
                "mix".to_string(),
                "potion".to_string(),
                "reanimate ".to_string(),
                "brewmaster".to_string(),
            ],
            description: "Potion brewing Champion".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "lazarus_league_plague_doctor".to_string(),
            name: "Plague Doctor".to_string(),
            clan: "Lazarus League".to_string(),
            card_type: "Unit".to_string(),
            rarity: "Uncommon".to_string(),
            cost: Some(3),
            base_value: 89,
            tempo_score: 7,
            value_score: 9,
            keywords: vec![
                "unstable".to_string(),
                "damage".to_string(),
                "s_tier".to_string(),
                "scaling".to_string(),
            ],
            description: "S-tier unit - applies Unstable equal to damage".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "lazarus_league_potion_kit".to_string(),
            name: "Potion Kit".to_string(),
            clan: "Lazarus League".to_string(),
            card_type: "Equipment".to_string(),
            rarity: "Common".to_string(),
            cost: Some(2),
            base_value: 80,
            tempo_score: 6,
            value_score: 8,
            keywords: vec![
                "equipment".to_string(),
                "mix".to_string(),
                "potion".to_string(),
                "core".to_string(),
            ],
            description: "Core equipment for Mix builds".to_string(),
            expansion: "base".to_string(),
        },
    ]);

    // MELTING REMNANT (Base Game)
    cards.extend(vec![
        CardData {
            id: "melting_remnant_rector_flicker".to_string(),
            name: "Rector Flicker".to_string(),
            clan: "Melting Remnant".to_string(),
            card_type: "Champion".to_string(),
            rarity: "Champion".to_string(),
            cost: None,
            base_value: 87,
            tempo_score: 6,
            value_score: 9,
            keywords: vec![
                "reform".to_string(),
                "burnout".to_string(),
                "resurrection".to_string(),
            ],
            description: "Reform champion - resurrects units".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "melting_remnant_lady_of_the_house".to_string(),
            name: "Lady of the House".to_string(),
            clan: "Melting Remnant".to_string(),
            card_type: "Unit".to_string(),
            rarity: "Rare".to_string(),
            cost: Some(4),
            base_value: 86,
            tempo_score: 5,
            value_score: 9,
            keywords: vec![
                "burnout".to_string(),
                "big".to_string(),
                "frontline".to_string(),
                "tank".to_string(),
                "scaling".to_string(),
            ],
            description: "45/45 tank with Burnout".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "melting_remnant_waxen_spike".to_string(),
            name: "Waxen Spike".to_string(),
            clan: "Melting Remnant".to_string(),
            card_type: "Spell".to_string(),
            rarity: "Common".to_string(),
            cost: Some(1),
            base_value: 77,
            tempo_score: 6,
            value_score: 7,
            keywords: vec![
                "burnout".to_string(),
                "attack_buff".to_string(),
                "buff".to_string(),
                "aggressive".to_string(),
            ],
            description: "Buffs attack and applies Burnout".to_string(),
            expansion: "base".to_string(),
        },
    ]);

    // HELLHORNED (Base Game)
    cards.extend(vec![
        CardData {
            id: "hellhorned_hornbreaker_prince".to_string(),
            name: "Hornbreaker Prince".to_string(),
            clan: "Hellhorned".to_string(),
            card_type: "Champion".to_string(),
            rarity: "Champion".to_string(),
            cost: None,
            base_value: 83,
            tempo_score: 7,
            value_score: 8,
            keywords: vec![
                "rage".to_string(),
                "multistrike".to_string(),
                "damage".to_string(),
            ],
            description: "Rage-based Champion".to_string(),
            expansion: "base".to_string(),
        },
        CardData {
            id: "hellhorned_titan_sentry".to_string(),
            name: "Titan Sentry".to_string(),
            clan: "Hellhorned".to_string(),
            card_type: "Unit".to_string(),
            rarity: "Uncommon".to_string(),
            cost: Some(3),
            base_value: 79,
            tempo_score: 6,
            value_score: 8,
            keywords: vec![
                "armor".to_string(),
                "frontline".to_string(),
                "tank".to_string(),
                "revenge".to_string(),
            ],
            description: "Armor tank with Revenge".to_string(),
            expansion: "base".to_string(),
        },
    ]);

    // RAILFORGED (NEW EXPANSION)
    cards.extend(vec![
        CardData {
            id: "railforged_herzal".to_string(),
            name: "Herzal".to_string(),
            clan: "Railforged".to_string(),
            card_type: "Champion".to_string(),
            rarity: "Champion".to_string(),
            cost: Some(0),
            base_value: 85,
            tempo_score: 7,
            value_score: 8,
            keywords: vec![
                "forge".to_string(),
                "burst".to_string(),
                "blacksmith".to_string(),
            ],
            description: "Architect champion with Forge Points and Burst mechanics".to_string(),
            expansion: "railforged".to_string(),
        },
        CardData {
            id: "railforged_heph".to_string(),
            name: "Heph".to_string(),
            clan: "Railforged".to_string(),
            card_type: "Champion".to_string(),
            rarity: "Champion".to_string(),
            cost: Some(0),
            base_value: 84,
            tempo_score: 6,
            value_score: 9,
            keywords: vec![
                "equipment".to_string(),
                "artificer".to_string(),
                "smelt".to_string(),
            ],
            description: "Weaponsmith champion with equipment focus".to_string(),
            expansion: "railforged".to_string(),
        },
        CardData {
            id: "railforged_forge_steward".to_string(),
            name: "Forge Steward".to_string(),
            clan: "Railforged".to_string(),
            card_type: "Unit".to_string(),
            rarity: "Uncommon".to_string(),
            cost: Some(2),
            base_value: 78,
            tempo_score: 6,
            value_score: 8,
            keywords: vec![
                "deployment".to_string(),
                "revenge".to_string(),
                "forge".to_string(),
            ],
            description: "Deployment unit that generates Forge on Revenge".to_string(),
            expansion: "railforged".to_string(),
        },
        CardData {
            id: "railforged_knuckler_steward".to_string(),
            name: "Knuckler Steward".to_string(),
            clan: "Railforged".to_string(),
            card_type: "Unit".to_string(),
            rarity: "Rare".to_string(),
            cost: Some(3),
            base_value: 82,
            tempo_score: 7,
            value_score: 8,
            keywords: vec![
                "deployment".to_string(),
                "burst".to_string(),
                "steelguard".to_string(),
            ],
            description: "Burst unit with Steelguard protection".to_string(),
            expansion: "railforged".to_string(),
        },
        CardData {
            id: "railforged_full_throttle".to_string(),
            name: "Full Throttle".to_string(),
            clan: "Railforged".to_string(),
            card_type: "Spell".to_string(),
            rarity: "Uncommon".to_string(),
            cost: Some(1),
            base_value: 81,
            tempo_score: 8,
            value_score: 7,
            keywords: vec![ "burst".to_string(), "buff".to_string(), "tempo".to_string()],
            description: "Apply Burst 2 to a friendly unit".to_string(),
            expansion: "railforged".to_string(),
        },
        CardData {
            id: "railforged_smith".to_string(),
            name: "Smith".to_string(),
            clan: "Railforged".to_string(),
            card_type: "Spell".to_string(),
            rarity: "Common".to_string(),
            cost: Some(1),
            base_value: 74,
            tempo_score: 8,
            value_score: 6,
            keywords: vec![ "forge".to_string(), "resource".to_string()],
            description: "Forge: Add to Forge Point total".to_string(),
            expansion: "railforged".to_string(),
        },
    ]);

    // Add more cards as needed...
    // For now including core cards from each clan + new expansion

    cards
}





