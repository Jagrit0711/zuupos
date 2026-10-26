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
    Booking,
    Gallery,
    Stopwatch,
    Notes,
    Calculator,
    Music,
    Terminal,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CardDetailRow {
    pub icon: String,   // "calendar", "bed", "location", "card"
    pub label: String,  // e.g. "Nov 12 - Nov 16"
    pub value: String,  // e.g. "4 Nights"
}

impl CardDetailRow {
    pub fn new(icon: &str, label: &str, value: &str) -> Self {
        Self {
            icon: icon.to_string(),
            label: label.to_string(),
            value: value.to_string(),
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
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub status_badge: Option<String>,
    #[serde(default)]
    pub detail_rows: Vec<CardDetailRow>,
    #[serde(default)]
    pub gallery_images: Vec<String>,
    #[serde(default)]
    pub pos_x: Option<i32>,
    #[serde(default)]
    pub pos_y: Option<i32>,
    #[serde(default)]
    pub custom_width: Option<u32>,
    #[serde(default)]
    pub custom_height: Option<u32>,
}

impl Card {
    pub fn new_booking(
        title: &str,
        subtitle: &str,
        status_badge: &str,
        details: Vec<CardDetailRow>,
    ) -> Self {
        Self {
            id: 0,
            title: title.to_string(),
            body: String::new(),
            card_type: CardType::Booking,
            age_ticks: 0,
            max_age: 10000,
            app_id: None,
            kind: Some("booking".to_string()),
            focused: false,
            minimized: false,
            actions: Vec::new(),
            subtitle: Some(subtitle.to_string()),
            status_badge: Some(status_badge.to_string()),
            detail_rows: details,
            gallery_images: Vec::new(),
            pos_x: None,
            pos_y: None,
            custom_width: Some(330),
            custom_height: Some(255),
        }
    }

    pub fn new_gallery(title: &str, images: Vec<String>) -> Self {
        Self {
            id: 0,
            title: title.to_string(),
            body: String::new(),
            card_type: CardType::Gallery,
            age_ticks: 0,
            max_age: 10000,
            app_id: None,
            kind: Some("gallery".to_string()),
            focused: false,
            minimized: false,
            actions: Vec::new(),
            subtitle: None,
            status_badge: None,
            detail_rows: Vec::new(),
            gallery_images: images,
            pos_x: None,
            pos_y: None,
            custom_width: Some(440),
            custom_height: Some(435),
        }
    }

    pub fn new_stopwatch() -> Self {
        Self {
            id: 0,
            title: "Stopwatch".to_string(),
            body: String::new(),
            card_type: CardType::Stopwatch,
            age_ticks: 0,
            max_age: 100000,
            app_id: Some("clock".to_string()),
            kind: Some("stopwatch".to_string()),
            focused: true,
            minimized: false,
            actions: vec![
                CardAction::new("Pause", "toggle_timer"),
                CardAction::new("Lap", "lap_timer"),
                CardAction::new("Reset", "reset_timer"),
            ],
            subtitle: Some("Active Focus Session".to_string()),
            status_badge: Some("LIVE STOPWATCH".to_string()),
            detail_rows: vec![
                CardDetailRow::new("clock", "Lap 1", "00:08.45"),
                CardDetailRow::new("pin", "Focus Goal", "Deep Work (Rust OS)"),
            ],
            gallery_images: Vec::new(),
            pos_x: Some(440),
            pos_y: Some(170),
            custom_width: Some(380),
            custom_height: Some(250),
        }
    }

    pub fn new_notes(title: &str, items: Vec<&str>) -> Self {
        let details = items
            .into_iter()
            .map(|item| CardDetailRow::new("check", item, "Active"))
            .collect();

        Self {
            id: 0,
            title: title.to_string(),
            body: "Autonomous scratchpad synced with agentd memory.".to_string(),
            card_type: CardType::Notes,
            age_ticks: 0,
            max_age: 100000,
            app_id: Some("notes".to_string()),
            kind: Some("notes".to_string()),
            focused: true,
            minimized: false,
            actions: vec![
                CardAction::new("+ New Task", "add_task"),
                CardAction::new("Copy All", "copy_notes"),
            ],
            subtitle: Some("System Scratchpad".to_string()),
            status_badge: Some("SYNCED".to_string()),
            detail_rows: details,
            gallery_images: Vec::new(),
            pos_x: Some(400),
            pos_y: Some(160),
            custom_width: Some(420),
            custom_height: Some(270),
        }
    }

    pub fn new_calculator(expr: &str, result: &str, hex: &str) -> Self {
        Self {
            id: 0,
            title: "Calculator".to_string(),
            body: expr.to_string(),
            card_type: CardType::Calculator,
            age_ticks: 0,
            max_age: 100000,
            app_id: Some("calc".to_string()),
            kind: Some("calculator".to_string()),
            focused: true,
            minimized: false,
            actions: vec![
                CardAction::new("Copy", "copy_calc"),
                CardAction::new("Clear", "clear_calc"),
            ],
            subtitle: Some(format!("Result: {}", result)),
            status_badge: Some("COMPUTED".to_string()),
            detail_rows: vec![
                CardDetailRow::new("pin", "Decimal", result),
                CardDetailRow::new("cpu", "Hexadecimal", hex),
            ],
            gallery_images: Vec::new(),
            pos_x: Some(450),
            pos_y: Some(180),
            custom_width: Some(360),
            custom_height: Some(230),
        }
    }

    pub fn new_music(track: &str, artist: &str) -> Self {
        Self {
            id: 0,
            title: track.to_string(),
            body: "Synthesized lo-fi stream for deep engineering focus.".to_string(),
            card_type: CardType::Music,
            age_ticks: 0,
            max_age: 100000,
            app_id: Some("media".to_string()),
            kind: Some("music".to_string()),
            focused: true,
            minimized: false,
            actions: vec![
                CardAction::new("⏮ Prev", "prev_track"),
                CardAction::new("⏸ Pause", "toggle_play"),
                CardAction::new("⏭ Next", "next_track"),
            ],
            subtitle: Some(artist.to_string()),
            status_badge: Some("NOW PLAYING".to_string()),
            detail_rows: vec![
                CardDetailRow::new("wave", "Bitrate", "320 kbps FLAC"),
                CardDetailRow::new("clock", "Elapsed", "01:42 / 03:30"),
            ],
            gallery_images: Vec::new(),
            pos_x: Some(420),
            pos_y: Some(170),
            custom_width: Some(400),
            custom_height: Some(255),
        }
    }

    pub fn new_weather(city: &str, temp: &str, condition: &str) -> Self {
        Self {
            id: 0,
            title: city.to_string(),
            body: format!("Current conditions: {}. Perfect weather for deep coding.", condition),
            card_type: CardType::Weather,
            age_ticks: 0,
            max_age: 100000,
            app_id: Some("weather".to_string()),
            kind: Some("weather".to_string()),
            focused: true,
            minimized: false,
            actions: vec![CardAction::new("Hourly", "hourly"), CardAction::new("7-Day", "weekly")],
            subtitle: Some(format!("{} • {}", temp, condition)),
            status_badge: Some("LIVE WEATHER".to_string()),
            detail_rows: vec![
                CardDetailRow::new("pin", "Humidity", "48%"),
                CardDetailRow::new("cloud", "Wind", "8 km/h NW"),
            ],
            gallery_images: Vec::new(),
            pos_x: Some(430),
            pos_y: Some(170),
            custom_width: Some(360),
            custom_height: Some(230),
        }
    }

    pub fn new_terminal(cmd: &str, output: &str) -> Self {
        Self {
            id: 0,
            title: format!("$ {}", cmd),
            body: output.to_string(),
            card_type: CardType::Terminal,
            age_ticks: 0,
            max_age: 100000,
            app_id: Some("terminal".to_string()),
            kind: Some("terminal".to_string()),
            focused: true,
            minimized: false,
            actions: vec![CardAction::new("Run Diagnostics", "run_diag"), CardAction::new("Clear", "clear_term")],
            subtitle: Some("PID 1 • x86_64 User Stack".to_string()),
            status_badge: Some("EXECUTED (0ms)".to_string()),
            detail_rows: vec![
                CardDetailRow::new("cpu", "Context Switch", "<1.2µs"),
                CardDetailRow::new("mem", "Exit Code", "0 (SUCCESS)"),
            ],
            gallery_images: Vec::new(),
            pos_x: Some(340),
            pos_y: Some(150),
            custom_width: Some(520),
            custom_height: Some(280),
        }
    }
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
            subtitle: None,
            status_badge: None,
            detail_rows: Vec::new(),
            gallery_images: Vec::new(),
            pos_x: None,
            pos_y: None,
            custom_width: None,
            custom_height: None,
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

    pub fn clear(&mut self) {
        self.cards.clear();
        self.focused_id = None;
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

    pub fn get_card_mut(&mut self, id: u64) -> Option<&mut Card> {
        self.cards.iter_mut().find(|c| c.id == id)
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
        let default_w: u32 = 340;
        let margin: u32 = 16;
        let statusbar_h: u32 = 32;
        let default_x = screen_w.saturating_sub(default_w + margin);
        let mut curr_y = statusbar_h + margin;

        for card in self.cards.iter().rev() {
            let card_w = card.custom_width.unwrap_or(default_w);
            let card_h = if card.minimized {
                36
            } else {
                card.custom_height.unwrap_or(if !card.actions.is_empty() {
                    142
                } else {
                    115
                })
            };
            let x = card.pos_x.map(|px| px as u32).unwrap_or(default_x);
            let y = card.pos_y.map(|py| py as u32).unwrap_or(curr_y);

            if mx >= x && mx <= x + card_w && my >= y && my <= y + card_h {
                // Check dismiss button (top right: [x + card_w - 28, x + card_w - 6], [y + 6, y + 28])
                if mx >= x + card_w - 28 && mx <= x + card_w - 6 && my >= y + 6 && my <= y + 28 {
                    return Some((card.id, CardHitZone::Dismiss));
                }

                // Check action buttons along bottom
                if !card.minimized && !card.actions.is_empty() {
                    let action_y = y + card_h - 48;
                    if my >= action_y && my <= y + card_h {
                        let btn_w = 100;
                        let btn_gap = 12;
                        let start_btn_x = x + 24;
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

            if card.pos_y.is_none() {
                curr_y += card_h + 10;
            }
        }

        None
    }
}
