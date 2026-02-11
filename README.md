# 🚀 LocoHFT

> **High-frequency trading engine powered by Rust + Python AI**

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://rust-lang.org)
[![Python](https://img.shields.io/badge/Python-3.8+-blue.svg)](https://python.org)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

LocoHFT is a hybrid high-frequency trading system combining Rust's blazing-fast execution speed with Python's AI/ML capabilities. Features an interactive terminal UI, risk management, and both practice and live trading modes.

![Terminal UI Demo](https://via.placeholder.com/800x400/1a1a2e/00ff88?text=LocoHFT+Terminal+Interface)

---

## ✨ Features

| Feature | Description |
|---------|-------------|
| ⚡ **Ultra-Low Latency** | Sub-microsecond order execution via Rust engine |
| 🤖 **AI Trading Strategies** | Python-embedded ML algorithms via PyO3 |
| 📊 **Smart Analysis** | Bollinger Bands mean reversion, momentum detection |
| 🛡️ **Risk Management** | Position limits, VaR calculations, automatic stop-loss |
| 🎮 **Practice Mode** | Paper trading with $100K virtual balance |
| 💡 **AI Recommendations** | Personalized portfolio allocation based on goals |
| 📚 **Learning Center** | Built-in educational content for beginners |
| 💾 **SQLite Persistence** | Portfolio and trade history storage |

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Terminal UI                           │
│                  (dialoguer + indicatif)                     │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                      Rust Core                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │   Engine     │  │  Risk Mgmt   │  │   Portfolio  │       │
│  │  Execution   │  │     VaR      │  │   Manager    │       │
│  └──────┬───────┘  └──────────────┘  └──────────────┘       │
└─────────┼────────────────────────────────────────────────────┘
          │ PyO3 Bridge
┌─────────▼────────────────────────────────────────────────────┐
│                    Python AI Layer                           │
│  ┌──────────────────┐      ┌──────────────────────────┐     │
│  │ LocoHFTStrategy  │      │ Recommendation Engine    │     │
│  │ (Bollinger Bands)│      │ (Portfolio Optimizer)    │     │
│  └──────────────────┘      └──────────────────────────┘     │
└──────────────────────────────────────────────────────────────┘
```

---

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) 1.70+
- [Python](https://python.org) 3.8+

### Build & Run

```bash
# Clone the repository
git clone https://github.com/yourusername/LocoHFT.git
cd LocoHFT

# Build release binary
cargo build --release

# Run the application
./target/release/LocoHFT.exe
```

### Windows Standalone Build

```bash
# Run the standalone build script (downloads required tools automatically)
build-standalone.bat

# Output will be in dist/LocoHFT.exe
```

---

## 📖 Usage Guide

### Main Menu

| Option | Description |
|--------|-------------|
| 📈 **Stock Analysis** | Analyze individual stocks, view market overview |
| 💰 **My Portfolio** | View holdings, trade history, buy/sell stocks |
| 🤖 **AI Trading** | Enable automated AI trading with risk controls |
| 📚 **Learning Center** | Educational content for beginners |
| ⚙️ **Settings** | Toggle practice/live mode, configure API keys |

### AI Trading Strategy

The built-in `LocoHFTStrategy` uses **Bollinger Bands Mean Reversion**:

```python
# Buys when price < lower band (oversold)
# Sells when price > upper band (overbought)
upper = SMA + 2σ
lower = SMA - 2σ
```

### Practice Mode vs Live Trading

| Mode | Risk | Balance | Purpose |
|------|------|---------|---------|
| 🎮 Practice | None | $100,000 virtual | Learn & test strategies |
| 🔴 Live | Real | Your actual funds | Real trading |

---

## ⚙️ Configuration

### API Keys (Optional)

For real market data, get a free API key from [Alpha Vantage](https://www.alphavantage.co/):

```
Settings → API Keys → Enter your key
```

*Without API keys, the app uses demo data for learning.*

### Risk Settings

Edit `python/strategy.py` to customize:

```python
self.lookback = 20          # Bollinger Bands period
self.position_limit = 1000  # Max shares per symbol
self.var_limit = 0.02       # 2% Value-at-Risk limit
```

---

## 🛠️ Development

### Project Structure

```
LocoHFT/
├── src/
│   ├── main.rs           # TUI application entry
│   ├── engine.rs         # Execution & risk engine
│   └── python_bridge.rs  # PyO3 Python interop
├── python/
│   ├── strategy.py       # AI trading strategies
│   ├── recommendations.py # Portfolio optimizer
│   └── analytics.py      # Data analysis tools
├── Cargo.toml
└── README.md
```

### Adding Custom Strategies

1. Edit `python/strategy.py`
2. Implement your logic in `on_market_data()`
3. Return a `Signal` dict with action, size, price, confidence

```python
def on_market_data(self, symbol, price, volume, timestamp):
    # Your custom logic here
    if should_buy:
        return Signal("BUY", 100, price, 0.85, {}).dict()
    return None
```

---

## 🧪 Testing

```bash
# Run unit tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

---

## 📦 Dependencies

### Rust Crates
- `tokio` - Async runtime
- `pyo3` - Python interoperability
- `dialoguer` / `indicatif` - Terminal UI
- `rusqlite` - Embedded database
- `crossbeam` - High-performance channels
- `reqwest` - HTTP client for APIs

### Python Libraries
- Standard library only (embedded)

---

## ⚠️ Risk Disclaimer

> **Trading involves significant risk of loss.** 
> 
> - This software is for **educational purposes**
> - Past performance does not guarantee future results
> - Only trade with money you can afford to lose
> - The AI is a tool, not financial advice
> - Always do your own research

---

## 📄 License

MIT License - See [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

- Built with ❤️ in Rust
- Python AI integration via [PyO3](https://pyo3.rs/)
- Terminal UI powered by [console-rs](https://github.com/console-rs)

---

<p align="center">
  <b>Made for traders who love speed and intelligence</b><br>
  🦀 ⚡ 🐍
</p>
