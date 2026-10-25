//! Card system — floating UI panels decided by the AI agent.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CardType {
    #[default]
    Info,
    Task,
    Alert,
    Weather,
    Metric,
    Plan,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CardAction {
    pub label: String,
    pub action_id: String,
}

impl CardAction {
    pub fn new(label: &str, action_id: &str) -> Self {
        Self {
            label: label.to_string(),
            action_id: action_id.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub card_type: CardType,
    pub age_ticks: u32,
    pub max_age: u32,
    #[serde(default)]
    pub app_id: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub focused: bool,
    #[serde(default)]
    pub minimized: bool,
    #[serde(default)]
    pub actions: Vec<CardAction>,
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
            app_id: None,
            kind: None,
            focused: false,
            minimized: false,
            actions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardHitZone {
    CardBody,
    Dismiss,
    MinimizeToggle,
    Action(usize),
}

pub struct CardManager {
    cards: Vec<Card>,
    next_id: u64,
    focused_id: Option<u64>,
}

impl Default for CardManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CardManager {
    pub fn new() -> Self {
        Self {
            cards: Vec::new(),
            next_id: 1,
            focused_id: None,
        }
    }

    pub fn add_card(&mut self, mut card: Card) -> u64 {
        card.id = self.next_id;
        self.next_id += 1;

        // If explicitly requested or first card, take focus
        if card.focused || self.cards.is_empty() {
            self.focused_id = Some(card.id);
            for c in &mut self.cards {
                c.focused = false;
            }
            card.focused = true;
        }

        // Limit to 6 visible cards max
        if self.cards.len() >= 6 {
            let removed = self.cards.remove(0);
            if self.focused_id == Some(removed.id) {
                self.focused_id = self.cards.first().map(|c| c.id);
            }
        }
        let id = card.id;
        self.cards.push(card);
        id
    }

    pub fn dismiss_card(&mut self, id: u64) -> bool {
        let initial_len = self.cards.len();
        self.cards.retain(|c| c.id != id);
        let removed = self.cards.len() < initial_len;
        if removed && self.focused_id == Some(id) {
            self.focused_id = self.cards.last().map(|c| c.id);
            if let Some(fid) = self.focused_id {
                self.focus_card(fid);
            }
        }
        removed
    }

    pub fn toggle_minimized(&mut self, id: u64) {
        if let Some(card) = self.cards.iter_mut().find(|c| c.id == id) {
            card.minimized = !card.minimized;
        }
    }

    pub fn focus_card(&mut self, id: u64) {
        self.focused_id = Some(id);
        for c in &mut self.cards {
            c.focused = c.id == id;
        }
    }

    pub fn focus_next(&mut self) {
        if self.cards.is_empty() {
            return;
        }
        let curr_idx = self
            .focused_id
            .and_then(|fid| self.cards.iter().position(|c| c.id == fid))
            .unwrap_or(0);
        let next_idx = (curr_idx + 1) % self.cards.len();
        let target_id = self.cards[next_idx].id;
        self.focus_card(target_id);
    }

    pub fn focus_prev(&mut self) {
        if self.cards.is_empty() {
            return;
        }
        let curr_idx = self
            .focused_id
            .and_then(|fid| self.cards.iter().position(|c| c.id == fid))
            .unwrap_or(0);
        let prev_idx = if curr_idx == 0 {
            self.cards.len() - 1
        } else {
            curr_idx - 1
        };
        let target_id = self.cards[prev_idx].id;
        self.focus_card(target_id);
    }

    pub fn focused_card(&self) -> Option<&Card> {
        self.focused_id
            .and_then(|fid| self.cards.iter().find(|c| c.id == fid))
    }

    pub fn focused_card_mut(&mut self) -> Option<&mut Card> {
        let fid = self.focused_id?;
        self.cards.iter_mut().find(|c| c.id == fid)
    }

    pub fn tick(&mut self) {
        for card in &mut self.cards {
            // Keep focused card alive longer while the user is reading/interacting
            if card.focused && card.age_ticks >= card.max_age.saturating_sub(60) {
                // Decay at half rate when focused
                if card.age_ticks % 2 == 0 {
                    card.age_ticks += 1;
                }
            } else {
                card.age_ticks += 1;
            }
        }
        // Remove expired cards
        self.cards.retain(|c| c.age_ticks < c.max_age);

        // Keep focused_id valid
        if let Some(fid) = self.focused_id {
            if !self.cards.iter().any(|c| c.id == fid) {
                self.focused_id = self.cards.last().map(|c| c.id);
                if let Some(new_fid) = self.focused_id {
                    for c in &mut self.cards {
                        c.focused = c.id == new_fid;
                    }
                }
            }
        }
    }

    pub fn active_cards(&self) -> Vec<&Card> {
        self.cards.iter().collect()
    }

    /// Tests whether a screen coordinate falls within a card and identifies the zone.
    pub fn hit_test(&self, mx: u32, my: u32, screen_w: u32) -> Option<(u64, CardHitZone)> {
        let card_w: u32 = 340;
        let margin: u32 = 16;
        let statusbar_h: u32 = 32;
        let start_x = screen_w.saturating_sub(card_w + margin);
        let mut curr_y = statusbar_h + margin;

        for card in &self.cards {
            let card_h = if card.minimized {
                36
            } else if !card.actions.is_empty() {
                142
            } else {
                115
            };

            if mx >= start_x && mx <= start_x + card_w && my >= curr_y && my <= curr_y + card_h {
                // Check dismiss button (top right: [x + card_w - 26, x + card_w - 6], [y + 6, y + 26])
                if mx >= start_x + card_w - 26
                    && mx <= start_x + card_w - 6
                    && my >= curr_y + 6
                    && my <= curr_y + 26
                {
                    return Some((card.id, CardHitZone::Dismiss));
                }

                // Check minimize toggle button (next to dismiss: [x + card_w - 50, x + card_w - 30], [y + 6, y + 26])
                if mx >= start_x + card_w - 50
                    && mx <= start_x + card_w - 30
                    && my >= curr_y + 6
                    && my <= curr_y + 26
                {
                    return Some((card.id, CardHitZone::MinimizeToggle));
                }

                // If not minimized and has actions, check action buttons along bottom
                if !card.minimized && !card.actions.is_empty() {
                    let action_y = curr_y + card_h - 32;
                    if my >= action_y && my <= curr_y + card_h - 6 {
                        let btn_w = 90;
                        let btn_gap = 8;
                        let start_btn_x = start_x + 14;
                        for (idx, _) in card.actions.iter().enumerate() {
                            let bx = start_btn_x + idx as u32 * (btn_w + btn_gap);
                            if mx >= bx && mx <= bx + btn_w {
                                return Some((card.id, CardHitZone::Action(idx)));
                            }
                        }
                    }
                }

                return Some((card.id, CardHitZone::CardBody));
            }

            curr_y += card_h + 10;
        }

        None
    }
}
