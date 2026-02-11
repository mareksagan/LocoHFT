//! Ultra-low latency execution engine (Rust side)
use crossbeam::channel::{bounded, Sender, Receiver};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Instant, SystemTime};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    pub symbol: String,
    pub price: f64,
    pub size: f64,
    pub timestamp: u64,
    pub exchange: String,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub symbol: String,
    pub side: Side,
    pub size: f64,
    pub price: f64,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug)]
pub struct Fill {
    pub order_id: String,
    pub price: f64,
    pub size: f64,
    pub pnl: f64,
}

pub struct ExecutionEngine {
    orders: HashMap<String, Order>,
    positions: HashMap<String, f64>,
    avg_prices: HashMap<String, f64>,
    tx: Sender<Fill>,
    rx: Receiver<Fill>,
}

impl ExecutionEngine {
    pub fn new() -> Self {
        let (tx, rx) = bounded(10000);
        Self {
            orders: HashMap::new(),
            positions: HashMap::new(),
            avg_prices: HashMap::new(),
            tx,
            rx,
        }
    }
    
    /// Execute order in <1 microsecond (simulated)
    pub fn execute(&mut self, symbol: &str, side: Side, size: f64, price: f64) -> Fill {
        let id = Uuid::new_v4().to_string();
        let order = Order {
            id: id.clone(),
            symbol: symbol.to_string(),
            side,
            size,
            price,
            timestamp: Instant::now(),
        };
        
        // Update positions (PnL calc)
        let current_pos = *self.positions.get(symbol).unwrap_or(&0.0);
        let avg_price = *self.avg_prices.get(symbol).unwrap_or(&0.0);
        
        let new_pos = match side {
            Side::Buy => current_pos + size,
            Side::Sell => current_pos - size,
        };
        
        // Realized PnL calculation
        let pnl = if (current_pos > 0.0 && new_pos < 0.0) || (current_pos < 0.0 && new_pos > 0.0) {
            // Crossed zero line
            let closed = current_pos.min(size);
            (price - avg_price) * closed * if current_pos > 0.0 { 1.0 } else { -1.0 }
        } else {
            0.0
        };
        
        // Update average price
        if new_pos != 0.0 {
            let total_cost = current_pos * avg_price + size * price;
            self.avg_prices.insert(symbol.to_string(), total_cost / new_pos.abs());
        }
        
        self.positions.insert(symbol.to_string(), new_pos);
        
        Fill {
            order_id: id,
            price,
            size,
            pnl,
        }
    }
    
    pub fn get_position(&self, symbol: &str) -> f64 {
        *self.positions.get(symbol).unwrap_or(&0.0)
    }
    
    pub fn get_all_positions(&self) -> HashMap<String, f64> {
        self.positions.clone()
    }
}

pub struct RiskEngine {
    max_position: f64,
    max_drawdown: f64,
    daily_pnl: Vec<f64>,
}

impl RiskEngine {
    pub fn new() -> Self {
        Self {
            max_position: 1000.0,  // Max 1000 shares
            max_drawdown: 0.05,    // 5%
            daily_pnl: Vec::new(),
        }
    }
    
    pub fn check_pre_trade(&self, symbol: &str, proposed_qty: f64, current_pos: f64) -> bool {
        // Position limit check (microsecond speed)
        if (current_pos + proposed_qty).abs() > self.max_position {
            log::warn!("Position limit breached for {}", symbol);
            return false;
        }
        true
    }
    
    pub fn calculate_var(&self, returns: &[f64]) -> f64 {
        if returns.len() < 30 {
            return 0.0;
        }
        let mut sorted = returns.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = (returns.len() as f64 * 0.05) as usize;  // 95% VaR
        sorted.get(idx).cloned().unwrap_or(0.0)
    }
}
