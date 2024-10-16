use chrono::{DateTime, Utc};

pub struct Quote {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    pub adjclose: f64,
}

pub struct Portfolio {
    pub name: String,
    pub initial_balance: f64,
    pub positions: Vec<Position>,
}

pub struct Position {
    pub ticker: String,
    pub quantity: f64,
    pub purchase_price: f64,
    pub purchase_date: DateTime<Utc>,
    pub leverage: f64,
}

impl Portfolio {
    pub fn new(name: &str, initial_balance: f64) -> Self {
        Portfolio {
            name: name.to_string(),
            initial_balance,
            positions: Vec::new(),
        }
    }

    pub fn add_position(
        &mut self,
        ticker: &str,
        quantity: f64,
        purchase_price: f64,
        purchase_date: DateTime<Utc>,
    ) {
        self.positions.push(Position {
            ticker: ticker.to_string(),
            quantity,
            purchase_price,
            purchase_date,
            leverage: 1.0,
        });
    }

    pub fn calculate_value(&self, current_prices: &[(String, f64)]) -> f64 {
        let mut value = self.initial_balance;
        for position in &self.positions {
            if let Some(price) = current_prices
                .iter()
                .find(|(ticker, _)| *ticker == position.ticker)
                .map(|(_, price)| *price)
            {
                value += position.quantity * price * position.leverage;
            }
        }
        value
    }
}
