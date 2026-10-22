//! Card system — floating UI panels decided by the AI agent.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CardType {
    #[default]
    Info,
    Task,
    Alert,
    Weather,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub card_type: CardType,
    pub age_ticks: u32,
    pub max_age: u32,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            id: 0,
            title: String::new(),
            body: String::new(),
            card_type: CardType::Info,
            age_ticks: 0,
            max_age: 300,
        }
    }
}

pub struct CardManager {
    cards: Vec<Card>,
    next_id: u64,
}

impl CardManager {
    pub fn new() -> Self {
        Self {
            cards: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add_card(&mut self, mut card: Card) {
        card.id = self.next_id;
        self.next_id += 1;
        // Limit to 4 visible cards max
        if self.cards.len() >= 4 {
            self.cards.remove(0);
        }
        self.cards.push(card);
    }

    pub fn tick(&mut self) {
        for card in &mut self.cards {
            card.age_ticks += 1;
        }
        // Remove expired cards
        self.cards.retain(|c| c.age_ticks < c.max_age);
    }

    pub fn active_cards(&self) -> Vec<&Card> {
        self.cards.iter().collect()
    }
}
