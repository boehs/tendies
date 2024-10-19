use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct Position {
    pub ticker: String,
    pub quantity: f64,
    pub share_price: f64,
    pub purchase_date: DateTime<Utc>,
    pub leverage: f64,
}

impl Position {
    pub fn value(&self) -> f64 {
        self.quantity * self.share_price
    }

    pub fn profit(&self, current_price: f64) -> f64 {
        self.quantity * (current_price - self.share_price)
    }

    pub fn profit_percent(&self, current_price: f64) -> f64 {
        (self.profit(current_price) / self.value()) * 100.0
    }
}

pub trait PositionVec {
    fn purchased_value(&self) -> f64;
    /// Calculate the total profit of all positions in the vector
    /// ```rust
    /// use tendies::position::{Position, PositionVec};
    /// portfolio.get_positions_for_ticker("AAPL").profit(&current_prices);
    /// ```
    fn profit(&self, current_prices: &[(&str, f64)]) -> f64;
    fn profit_percent(&self, current_prices: &[(&str, f64)]) -> f64;
}

impl PositionVec for Vec<&Position> {
    fn purchased_value(&self) -> f64 {
        self.iter().map(|position| position.value()).sum()
    }

    fn profit(&self, current_prices: &[(&str, f64)]) -> f64 {
        // match ticker to current price
        let prices: std::collections::HashMap<&str, f64> = current_prices.iter().cloned().collect();
        self.iter()
            .map(|position| {
                let current_price = prices.get(&position.ticker.as_str()).unwrap();
                position.profit(*current_price)
            })
            .sum()
    }

    fn profit_percent(&self, current_prices: &[(&str, f64)]) -> f64 {
        self.profit(current_prices) / self.purchased_value() * 100.0
    }
}
