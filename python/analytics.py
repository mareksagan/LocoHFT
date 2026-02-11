"""
Analytics helpers that can use Python's ML stack if available
"""

try:
    import numpy as np
    NUMPY_AVAILABLE = True
except ImportError:
    NUMPY_AVAILABLE = False

def calculate_volatility(prices: list) -> float:
    """Calculate realized volatility"""
    if len(prices) < 2:
        return 0.0
    
    if NUMPY_AVAILABLE:
        returns = np.diff(np.log(prices))
        return float(np.std(returns) * np.sqrt(252))
    else:
        # Pure Python implementation
        log_returns = [math.log(prices[i]/prices[i-1]) for i in range(1, len(prices))]
        mean = sum(log_returns) / len(log_returns)
        variance = sum((r - mean) ** 2 for r in log_returns) / len(log_returns)
        return (variance ** 0.5) * 15.8745  # Annualized approx

def detect_regime_change(volatility_short: float, volatility_long: float) -> str:
    """Detect volatility regime"""
    ratio = volatility_short / volatility_long if volatility_long > 0 else 1.0
    if ratio > 1.5:
        return "HIGH_VOL"
    elif ratio < 0.5:
        return "LOW_VOL"
    return "NORMAL"
