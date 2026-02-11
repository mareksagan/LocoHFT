"""
Python Strategy Module - Embedded in Rust Binary
This runs inside the Rust process via PyO3
"""

import json
import math
from typing import Dict, List, Optional, Any
from dataclasses import dataclass

@dataclass
class Signal:
    action: str  # BUY, SELL, HOLD
    size: float
    price: float
    confidence: float
    metadata: dict

class LocoHFTStrategy:
    def __init__(self, capital: float = 100000.0):
        self.capital = capital
        self.positions = {}
        self.price_history = {}
        self.lookback = 20
        
    def on_market_data(self, symbol: str, price: float, volume: float, timestamp: int) -> Optional[dict]:
        """Process tick data from Rust - returns signal dict or None"""
        
        # Store price history
        if symbol not in self.price_history:
            self.price_history[symbol] = []
        
        self.price_history[symbol].append(price)
        if len(self.price_history[symbol]) > self.lookback:
            self.price_history[symbol].pop(0)
        
        # Need minimum data
        if len(self.price_history[symbol]) < self.lookback:
            return None
        
        # Calculate Bollinger Bands in Python
        prices = self.price_history[symbol]
        sma = sum(prices) / len(prices)
        variance = sum((p - sma) ** 2 for p in prices) / len(prices)
        std = math.sqrt(variance)
        
        upper = sma + 2 * std
        lower = sma - 2 * std
        
        # Mean reversion logic
        current_pos = self.positions.get(symbol, 0)
        
        if price < lower and current_pos <= 0:
            signal = Signal("BUY", 100.0, price, 0.8, {"sma": sma, "z_score": (price - sma)/std})
            self.positions[symbol] = current_pos + 100
            return signal.__dict__
            
        elif price > upper and current_pos >= 0:
            signal = Signal("SELL", 100.0, price, 0.8, {"sma": sma, "z_score": (price - sma)/std})
            self.positions[symbol] = current_pos - 100
            return signal.__dict__
            
        return None
    
    def calculate_portfolio_weights(self, returns_data: List[List[float]]) -> List[float]:
        """Risk parity calculation in Python (fallback if Rust fails)"""
        # This would use numpy if available, pure python fallback here
        n = len(returns_data)
        return [1.0 / n] * n  # Equal weight fallback
    
    def on_risk_update(self, var_95: float, exposure: float) -> bool:
        """Risk callback from Rust - return False to halt trading"""
        if var_95 > self.capital * 0.02:  # 2% VaR limit
            print(f"[PYTHON] Risk limit breached: VaR ${var_95:.2f}")
            return False
        return True

# Global instance (Rust will instantiate this)
strategy = LocoHFTStrategy()
