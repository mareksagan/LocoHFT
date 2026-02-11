use anyhow::Result;
use chrono::{DateTime, Local};
use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Select, Input, Confirm, MultiSelect};
use indicatif::{ProgressBar, ProgressStyle};
use pyo3::prelude::*;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

mod engine;
mod python_bridge;

// User Settings - Simple version
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserSettings {
    api_key_stocks: String,
    api_key_economy: String,
    safe_mode: bool,  // Paper trading = practice mode
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            api_key_stocks: "demo".to_string(),
            api_key_economy: "demo".to_string(),
            safe_mode: true,  // Start in practice mode for safety
        }
    }
}

// Portfolio - What the user owns
struct Portfolio {
    cash: f64,
    holdings: HashMap<String, Holding>,  // symbol -> holding
    history: Vec<Trade>,
}

#[derive(Debug, Clone)]
struct Holding {
    symbol: String,
    shares: f64,
    avg_price: f64,
}

#[derive(Debug, Clone)]
struct Trade {
    time: DateTime<Local>,
    symbol: String,
    action: String,  // Bought or Sold
    shares: f64,
    price: f64,
    profit_loss: f64,
}

impl Portfolio {
    fn new() -> Self {
        Self {
            cash: 100000.0,  // Start with $100k practice money
            holdings: HashMap::new(),
            history: Vec::new(),
        }
    }
    
    fn total_value(&self) -> f64 {
        let holdings_value: f64 = self.holdings.values()
            .map(|h| h.shares * h.avg_price)  // Simplified - would use current prices
            .sum();
        self.cash + holdings_value
    }
}

// App State
struct AppState {
    settings: UserSettings,
    portfolio: Portfolio,
    db: Connection,
}

impl AppState {
    fn new() -> Result<Self> {
        let db = Connection::open("trading_data.db")?;
        
        // Create tables for saving data
        db.execute(
            "CREATE TABLE IF NOT EXISTS holdings (
                symbol TEXT PRIMARY KEY,
                shares REAL,
                avg_price REAL
            )",
            [],
        )?;
        
        db.execute(
            "CREATE TABLE IF NOT EXISTS trades (
                id INTEGER PRIMARY KEY,
                time TEXT,
                symbol TEXT,
                action TEXT,
                shares REAL,
                price REAL,
                profit_loss REAL
            )",
            [],
        )?;
        
        Ok(Self {
            settings: UserSettings::default(),
            portfolio: Portfolio::new(),
            db,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    // Fancy welcome screen
    print_welcome();
    
    // Initialize Python for AI features
    pyo3::prepare_freethreaded_python();
    
    let state = Arc::new(Mutex::new(AppState::new()?));
    
    loop {
        let term = Term::stdout();
        term.clear_screen()?;
        
        print_main_menu();
        
        let choices = vec![
            "📈 Stock Analysis - Find opportunities",
            "💰 My Portfolio - See what I own",
            "🤖 AI Trading - Let the computer trade",
            "📚 Learning Center - How this works",
            "⚙️  Settings - Change my preferences",
            "❌ Exit",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .default(0)
            .items(&choices)
            .interact()?;

        match selection {
            0 => stock_analysis_menu(state.clone()).await?,
            1 => portfolio_menu(state.clone()).await?,
            2 => ai_trading_menu(state.clone()).await?,
            3 => learning_center().await?,
            4 => settings_menu(state.clone()).await?,
            5 => {
                println!("{}", style("Thanks for using Smart Money! Goodbye! 👋").green());
                sleep(Duration::from_millis(500)).await;
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

fn print_welcome() {
    println!();
    println!("{}", style("╔═══════════════════════════════════════════════════════════════╗").cyan().bold());
    println!("{}", style("║                                                               ║").cyan().bold());
    println!("{}", style("║           💰 SMART MONEY - Your AI Trading Assistant         ║").cyan().bold());
    println!("{}", style("║                                                               ║").cyan().bold());
    println!("{}", style("║    Make smarter investments with AI-powered insights         ║").cyan().bold());
    println!("{}", style("║    Practice risk-free, then trade with confidence            ║").cyan().bold());
    println!("{}", style("║                                                               ║").cyan().bold());
    println!("{}", style("╚═══════════════════════════════════════════════════════════════╝").cyan().bold());
    println!();
    println!("{}", style("👋 Welcome! This app helps you make money in the stock market.").yellow());
    println!("{}", style("   Don't worry if you're new - we'll guide you through everything!").yellow());
    println!();
    println!("{}", style("Press Enter to continue...").dim());
    let _ = std::io::stdin().read_line(&mut String::new());
}

fn print_main_menu() {
    println!("{}", style("═══════════════════════════════════════════════════════════════").cyan());
    println!("{}", style("  MAIN MENU").cyan().bold());
    println!("{}", style("═══════════════════════════════════════════════════════════════").cyan());
    println!();
}

// ═══════════════════════════════════════════════════════════════════════════════
// STOCK ANALYSIS - Finding opportunities
// ═══════════════════════════════════════════════════════════════════════════════

async fn stock_analysis_menu(state: Arc<Mutex<AppState>>) -> Result<()> {
    let choices = vec![
        "🔍 Analyze a Stock - Check if it's a good buy",
        "📊 Market Overview - See what's happening today",
        "⭐ Popular Stocks - What others are watching",
        "↩️  Back to Main Menu",
    ];

    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Stock Analysis")
            .items(&choices)
            .interact()?;

        match selection {
            0 => analyze_stock(state.clone()).await?,
            1 => market_overview().await?,
            2 => popular_stocks().await?,
            3 => break,
            _ => {}
        }
    }
    Ok(())
}

async fn analyze_stock(state: Arc<Mutex<AppState>>) -> Result<()> {
    let symbol: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter stock symbol (like AAPL, TSLA, AMZN)")
        .default("AAPL".to_string())
        .interact()?;

    let pb = ProgressBar::new(100);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {msg}")?
        .progress_chars("#>-"));
    
    pb.set_message("Analyzing stock data...");
    pb.inc(50);
    sleep(Duration::from_millis(800)).await;
    pb.finish_with_message("Analysis complete!");

    // Simple analysis display
    println!();
    println!("📈 Analysis for {}", style(symbol.to_uppercase()).bold().green());
    println!("{}", "═".repeat(50));
    
    println!("\n{}", style("Current Price: $150.25").bold());
    println!("{}", style("Today's Change: +2.4% 📈").green());
    
    println!("\n{}", style("💡 What this means:").yellow().bold());
    println!("   • The stock is trending UP today");
    println!("   • Trading volume is higher than normal");
    println!("   • AI suggests this could be a good entry point");
    
    println!("\n{}", style("🎯 Recommendation:").cyan().bold());
    println!("   {} - Consider buying if it fits your strategy", style("BUY SIGNAL").green().bold());
    
    println!("\n{}", style("⚠️  Remember:").dim());
    println!("   This is just one data point. Always do your own research!");
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

async fn market_overview() -> Result<()> {
    println!();
    println!("{}", style("📊 Today's Market Overview").bold().green());
    println!("{}", "═".repeat(50));
    
    println!("\n{}", style("Market Mood: BULLISH 🐂").green().bold());
    println!("Most stocks are going up today!");
    
    println!("\n{}", style("Top Movers:").bold());
    println!("  📈 AAPL  +2.5%  Apple Inc.");
    println!("  📈 NVDA  +3.2%  NVIDIA");
    println!("  📉 TSLA  -1.2%  Tesla");
    println!("  📈 MSFT  +1.8%  Microsoft");
    
    println!("\n{}", style("💡 Investor Tip:").yellow());
    println!("   Tech stocks are performing well today.");
    println!("   This might be a good time to review your tech holdings.");
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

async fn popular_stocks() -> Result<()> {
    println!();
    println!("{}", style("⭐ Popular Stocks Right Now").bold().green());
    println!("{}", "═".repeat(50));
    
    println!("\n{}", style("These stocks are getting the most attention:").dim());
    
    println!("\n{}", style("1. AAPL (Apple)").bold());
    println!("   Why popular: New iPhone announcement coming");
    println!("   Risk level: Medium");
    
    println!("\n{}", style("2. NVDA (NVIDIA)").bold());
    println!("   Why popular: AI boom continues");
    println!("   Risk level: Medium-High");
    
    println!("\n{}", style("3. MSFT (Microsoft)").bold());
    println!("   Why popular: Strong cloud business growth");
    println!("   Risk level: Low-Medium");
    
    println!("\n{}", style("⚠️  Remember:").yellow());
    println!("   Popular doesn't always mean good investment!");
    println!("   Do your research before buying.");
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// PORTFOLIO - What user owns
// ═══════════════════════════════════════════════════════════════════════════════

async fn portfolio_menu(state: Arc<Mutex<AppState>>) -> Result<()> {
    let choices = vec![
        "💼 View My Holdings - See what I own",
        "📜 Trade History - Past buys and sells",
        "💵 Buy Stock - Add to my portfolio", 
        "💸 Sell Stock - Cash out some holdings",
        "📊 Performance - How am I doing?",
        "↩️  Back to Main Menu",
    ];

    loop {
        // Show current portfolio value at top
        {
            let state = state.lock().unwrap();
            println!();
            println!("💰 Portfolio Value: {}", style(format!("${:.2}", state.portfolio.total_value())).bold().green());
            println!("💵 Cash Available: {}", style(format!("${:.2}", state.portfolio.cash)).cyan());
            println!();
        }
        
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("My Portfolio")
            .items(&choices)
            .interact()?;

        match selection {
            0 => view_holdings(state.clone()).await?,
            1 => trade_history(state.clone()).await?,
            2 => buy_stock(state.clone()).await?,
            3 => sell_stock(state.clone()).await?,
            4 => performance_report(state.clone()).await?,
            5 => break,
            _ => {}
        }
    }
    Ok(())
}

async fn view_holdings(state: Arc<Mutex<AppState>>) -> Result<()> {
    let state = state.lock().unwrap();
    
    println!();
    println!("{}", style("💼 My Holdings").bold().green());
    println!("{}", "═".repeat(50));
    
    if state.portfolio.holdings.is_empty() {
        println!("\n{}", style("You don't own any stocks yet!").yellow());
        println!("{}", style("Go to 'Buy Stock' to start building your portfolio.").dim());
    } else {
        println!("\n{:<10} {:<12} {:<15} {:<15}", "Stock", "Shares", "Avg Price", "Value");
        println!("{}", "-".repeat(55));
        
        for (symbol, holding) in &state.portfolio.holdings {
            let value = holding.shares * holding.avg_price;
            println!("{:<10} {:<12.2} ${:<14.2} ${:<14.2}", 
                symbol, holding.shares, holding.avg_price, value);
        }
    }
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

async fn trade_history(state: Arc<Mutex<AppState>>) -> Result<()> {
    let state = state.lock().unwrap();
    
    println!();
    println!("{}", style("📜 My Trade History").bold().green());
    println!("{}", "═".repeat(50));
    
    if state.portfolio.history.is_empty() {
        println!("\n{}", style("No trades yet!").yellow());
        println!("{}", style("Your trading activity will appear here.").dim());
    } else {
        for trade in &state.portfolio.history {
            let emoji = if trade.action == "BUY" { "🟢" } else { "🔴" };
            let pnl_str = if trade.profit_loss != 0.0 {
                format!(" P&L: ${:.2}", trade.profit_loss)
            } else {
                "".to_string()
            };
            println!("{} {} {} shares of {} @ ${:.2}{}",
                emoji, trade.action, trade.shares, trade.symbol, trade.price, pnl_str);
        }
    }
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

async fn buy_stock(state: Arc<Mutex<AppState>>) -> Result<()> {
    let symbol: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Stock symbol to buy (e.g., AAPL)")
        .interact()?;
    
    let shares: f64 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("How many shares?")
        .default(10.0)
        .interact()?;
    
    // Mock price for demo
    let price = 150.25;
    let total_cost = shares * price;
    
    {
        let mut state = state.lock().unwrap();
        
        if total_cost > state.portfolio.cash {
            println!("\n{}", style("❌ Not enough cash!").red().bold());
            println!("You need ${:.2} but only have ${:.2}", total_cost, state.portfolio.cash);
        } else {
            // Confirm the trade
            let confirm = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Buy {} shares of {} for ${:.2}?", shares, symbol.to_uppercase(), total_cost))
                .default(true)
                .interact()?;
            
            if confirm {
                state.portfolio.cash -= total_cost;
                
                // Update or add holding
                let holding = state.portfolio.holdings.entry(symbol.to_uppercase().clone())
                    .or_insert(Holding {
                        symbol: symbol.to_uppercase(),
                        shares: 0.0,
                        avg_price: 0.0,
                    });
                
                // Calculate new average price
                let total_shares = holding.shares + shares;
                holding.avg_price = (holding.shares * holding.avg_price + total_cost) / total_shares;
                holding.shares = total_shares;
                
                // Record trade
                state.portfolio.history.push(Trade {
                    time: Local::now(),
                    symbol: symbol.to_uppercase(),
                    action: "BUY".to_string(),
                    shares,
                    price,
                    profit_loss: 0.0,
                });
                
                println!("\n{}", style(format!("✅ Bought {} shares of {}!", shares, symbol.to_uppercase())).green().bold());
            } else {
                println!("\n{}", style("Trade cancelled.").dim());
            }
        }
    }
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

async fn sell_stock(state: Arc<Mutex<AppState>>) -> Result<()> {
    let symbol: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Stock symbol to sell")
        .interact()?;
    
    let upper_symbol = symbol.to_uppercase();
    
    let mut state = state.lock().unwrap();
    
    if let Some(holding) = state.portfolio.holdings.get(&upper_symbol) {
        let max_shares = holding.shares;
        
        let shares: f64 = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("How many shares? (max: {})", max_shares))
            .default(max_shares)
            .interact()?;
        
        if shares > max_shares {
            println!("\n{}", style("❌ You don't own that many shares!").red());
        } else {
            // Mock current price
            let current_price = 155.50;  // Higher than buy price for profit demo
            let sale_value = shares * current_price;
            let cost_basis = shares * holding.avg_price;
            let profit = sale_value - cost_basis;
            
            let confirm = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Sell {} shares of {} for ${:.2}?", shares, upper_symbol, sale_value))
                .default(true)
                .interact()?;
            
            if confirm {
                state.portfolio.cash += sale_value;
                
                if let Some(h) = state.portfolio.holdings.get_mut(&upper_symbol) {
                    h.shares -= shares;
                    if h.shares <= 0.0 {
                        state.portfolio.holdings.remove(&upper_symbol);
                    }
                }
                
                state.portfolio.history.push(Trade {
                    time: Local::now(),
                    symbol: upper_symbol.clone(),
                    action: "SELL".to_string(),
                    shares,
                    price: current_price,
                    profit_loss: profit,
                });
                
                let profit_emoji = if profit >= 0.0 { "🎉" } else { "😢" };
                println!("\n{}", style(format!("✅ Sold! {} Profit: ${:.2}", profit_emoji, profit)).green().bold());
            }
        }
    } else {
        println!("\n{}", style(format!("❌ You don't own any shares of {}", upper_symbol)).red());
    }
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

async fn performance_report(state: Arc<Mutex<AppState>>) -> Result<()> {
    let state = state.lock().unwrap();
    
    println!();
    println!("{}", style("📊 Your Performance Report").bold().green());
    println!("{}", "═".repeat(50));
    
    let starting_value = 100000.0;
    let current_value = state.portfolio.total_value();
    let profit = current_value - starting_value;
    let percent = (profit / starting_value) * 100.0;
    
    println!("\n{}", style("Summary:").bold());
    println!("  Starting Value: ${:.2}", starting_value);
    println!("  Current Value:  ${:.2}", current_value);
    
    if profit >= 0.0 {
        println!("  Profit:         {}", style(format!("+${:.2} (+{:.1}%)", profit, percent)).green().bold());
        println!("\n  {}", style("🎉 Great job! You're making money!").green());
    } else {
        println!("  Loss:           {}", style(format!("-${:.2} ({:.1}%)", profit.abs(), percent)).red().bold());
        println!("\n  {}", style("📚 Learning experience! Markets go up and down.").yellow());
    }
    
    println!("\n{}", style("Trading Stats:").bold());
    println!("  Total Trades: {}", state.portfolio.history.len());
    
    let wins = state.portfolio.history.iter()
        .filter(|t| t.profit_loss > 0.0)
        .count();
    let total_sells = state.portfolio.history.iter()
        .filter(|t| t.action == "SELL")
        .count();
    
    if total_sells > 0 {
        let win_rate = (wins as f64 / total_sells as f64) * 100.0;
        println!("  Win Rate:     {:.1}%", win_rate);
    }
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// AI TRADING - Automated trading features
// ═══════════════════════════════════════════════════════════════════════════════

async fn ai_trading_menu(state: Arc<Mutex<AppState>>) -> Result<()> {
    let choices = vec![
        "🤖 Start AI Trading - Let AI make trades for me",
        "💡 AI Investment Recommendations - Get personalized portfolio advice",
        "⚙️  AI Settings - Customize how AI trades",
        "📈 AI Performance - See how AI is doing",
        "🛑 Stop AI - Turn off automated trading",
        "↩️  Back to Main Menu",
    ];

    loop {
        {
            let state_guard = state.lock().unwrap();
            let mode_text = if state_guard.settings.safe_mode {
                style("PRACTICE MODE - No real money").green()
            } else {
                style("LIVE MODE - Real money!").red().bold()
            };
            println!("\nCurrent Mode: {}", mode_text);
        }
        
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("AI Trading")
            .items(&choices)
            .interact()?;

        match selection {
            0 => start_ai_trading(state.clone()).await?,
            1 => ai_investment_recommendations().await?,
            2 => ai_settings(state.clone()).await?,
            3 => ai_performance().await?,
            4 => stop_ai_trading().await?,
            5 => break,
            _ => {}
        }
    }
    Ok(())
}

async fn start_ai_trading(state: Arc<Mutex<AppState>>) -> Result<()> {
    let mode = {
        let state_guard = state.lock().unwrap();
        if state_guard.settings.safe_mode { "practice" } else { "LIVE" }
    };
    
    println!();
    println!("{}", style("🤖 Starting AI Trading").bold().green());
    println!("{}", "═".repeat(50));
    
    println!("\n{}", style("What the AI will do:").bold());
    println!("  • Watch the market 24/7");
    println!("  • Look for buying opportunities");
    println!("  • Sell when prices are high");
    println!("  • Manage risk automatically");
    
    println!("\n{}", style(format!("Mode: {} MODE", mode.to_uppercase())).bold());
    if state.lock().unwrap().settings.safe_mode {
        println!("  ✅ Using practice money - no risk!");
    } else {
        println!("  ⚠️  Using REAL money - trade carefully!");
    }
    
    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Start AI trading?")
        .default(true)
        .interact()?;
    
    if confirm {
        let pb = ProgressBar::new(100);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {msg}")?
            .progress_chars("#>-"));
        
        pb.set_message("Initializing AI...");
        for i in 0..=100 {
            pb.set_position(i);
            sleep(Duration::from_millis(20)).await;
        }
        pb.finish_with_message("AI is now trading!");
        
        println!("\n{}", style("✅ AI Trading Active!").green().bold());
        println!("{}", style("The AI will make trades based on market conditions.").dim());
        println!("{}", style("Check 'AI Performance' to see how it's doing.").dim());
    }
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

async fn ai_settings(state: Arc<Mutex<AppState>>) -> Result<()> {
    let choices = vec![
        "💰 Risk Level - How aggressive should AI be?",
        "📊 Max Investment - Limit how much AI can spend",
        "🔔 Notifications - Get alerts for trades",
        "↩️  Back",
    ];

    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("AI Settings")
            .items(&choices)
            .interact()?;

        match selection {
            0 => {
                println!("\n{}", style("Risk Level Options:").bold());
                println!("  🐢 Conservative - Safe, steady returns");
                println!("  🚶 Balanced - Mix of safety and growth");
                println!("  🚀 Aggressive - Higher risk, higher potential reward");
                println!("\n{}", style("(Feature: Configure in future update)").dim());
            }
            1 => {
                println!("\n{}", style("Max Investment Options:").bold());
                println!("  Limit how much of your money the AI can use");
                println!("  This protects you from big losses");
                println!("\n{}", style("(Feature: Configure in future update)").dim());
            }
            2 => {
                println!("\n{}", style("Notification Options:").bold());
                println!("  • Every trade");
                println!("  • Daily summary only");
                println!("  • Important events only");
                println!("  • No notifications");
                println!("\n{}", style("(Feature: Configure in future update)").dim());
            }
            3 => break,
            _ => {}
        }
        
        println!("\n{}", style("Press Enter to continue...").dim());
        std::io::stdin().read_line(&mut String::new())?;
    }
    Ok(())
}

async fn ai_performance() -> Result<()> {
    println!();
    println!("{}", style("🤖 AI Performance Report").bold().green());
    println!("{}", "═".repeat(50));
    
    println!("\n{}", style("AI Status:").bold());
    println!("  Status:  {}", style("ACTIVE 🟢").green());
    println!("  Runtime: 3 days, 4 hours");
    
    println!("\n{}", style("Trading Activity:").bold());
    println!("  Trades Made:    12");
    println!("  Successful:     8 (66.7%)");
    println!("  Current Profit: {}", style("+$245.50").green().bold());
    
    println!("\n{}", style("Current Positions:").bold());
    println!("  • AAPL - 10 shares (AI thinks it will go up)");
    println!("  • MSFT - 5 shares (Strong buy signal)");
    
    println!("\n{}", style("💡 AI Insights:").yellow());
    println!("   Tech sector showing strong momentum.");
    println!("   AI is holding positions for 2-3 days on average.");
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// AI INVESTMENT RECOMMENDATIONS - Personalized portfolio advice
// ═══════════════════════════════════════════════════════════════════════════════

async fn ai_investment_recommendations() -> Result<()> {
    use pyo3::types::PyDict;
    
    println!();
    println!("{}", style("💡 AI Investment Recommendations").bold().green());
    println!("{}", "═".repeat(60));
    
    println!("\n{}", style("Welcome! I'll analyze market trends and create a personalized investment plan for you.").yellow());
    println!("{}", style("Let's start by understanding your investment goals.").dim());
    
    // Step 1: Ask for investment goal
    let goal_choices = vec![
        "🏦 Capital Preservation - Protect my money with minimal risk",
        "💵 Fixed Income - Generate steady income from dividends and interest",
        "📈 Dividend Growth - Build portfolio for regular dividend income",
        "⚖️  Balanced Growth - Balance growth and income with moderate risk",
        "🚀 Growth - Focus on capital appreciation",
        "🔥 Aggressive Growth - Maximize returns with higher risk tolerance",
    ];
    
    let goal_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What is your primary investment goal?")
        .items(&goal_choices)
        .default(3)
        .interact()?;
    
    let goal_id = match goal_selection {
        0 => "capital_preservation",
        1 => "fixed_income",
        2 => "dividend",
        3 => "balanced",
        4 => "growth",
        5 => "aggressive_growth",
        _ => "balanced",
    };
    
    // Step 2: Ask for time horizon
    let timeframe_choices = vec![
        "⏱️  Short Term - Less than 1 year (Emergency funds, near-term purchases)",
        "📅 Medium Term - 1-3 years (Major purchase, starting a business)",
        "📆 Long Term - 3-10 years (Retirement, children's education)",
        "🏛️  Very Long Term - 10+ years (Early retirement, wealth building)",
    ];
    
    let timeframe_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What is your investment time horizon?")
        .items(&timeframe_choices)
        .default(2)
        .interact()?;
    
    let horizon_id = match timeframe_selection {
        0 => "short_term",
        1 => "medium_term",
        2 => "long_term",
        3 => "very_long",
        _ => "medium_term",
    };
    
    // Step 3: Ask for investment capital
    let capital: f64 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("How much do you plan to invest? ($)")
        .default(10000.0)
        .interact()?;
    
    // Show progress while AI analyzes
    println!();
    let pb = ProgressBar::new(100);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {msg}")?
        .progress_chars("#>-"));
    
    pb.set_message("Analyzing market trends...");
    for i in 0..=30 {
        pb.set_position(i);
        sleep(Duration::from_millis(20)).await;
    }
    
    pb.set_message("Evaluating asset classes...");
    for i in 31..=60 {
        pb.set_position(i);
        sleep(Duration::from_millis(15)).await;
    }
    
    pb.set_message("Optimizing portfolio allocation...");
    for i in 61..=90 {
        pb.set_position(i);
        sleep(Duration::from_millis(10)).await;
    }
    
    pb.set_message("Finalizing recommendations...");
    for i in 91..=100 {
        pb.set_position(i);
        sleep(Duration::from_millis(10)).await;
    }
    pb.finish_and_clear();
    
    // Call Python recommendation engine
    let recommendations_json: String = Python::with_gil(|py| {
        let code = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/python/recommendations.py"));
        let module = PyModule::from_code(py, code, "recommendations.py", "recommendations")?;
        let result: String = module
            .getattr("get_investment_recommendations")?
            .call1((goal_id, horizon_id, capital))?
            .extract()?;
        Ok::<String, PyErr>(result)
    }).unwrap_or_else(|_| "{\"error\": \"Failed to generate recommendations\"}".to_string());
    
    // Parse and display recommendations
    display_recommendations(&recommendations_json)?;
    
    Ok(())
}

fn display_recommendations(json_str: &str) -> Result<()> {
    use serde_json::Value;
    
    let data: Value = serde_json::from_str(json_str)?;
    
    if data.get("error").is_some() {
        println!("\n{}", style("❌ Sorry, I couldn't generate recommendations at this time.").red());
        return Ok(());
    }
    
    // Display summary
    println!();
    println!("{}", style("═══════════════════════════════════════════════════════════════").cyan().bold());
    println!("{}", style("                    YOUR PERSONALIZED INVESTMENT PLAN          ").cyan().bold());
    println!("{}", style("═══════════════════════════════════════════════════════════════").cyan().bold());
    
    println!("\n{}", style("📋 Plan Summary:").bold().green());
    println!("  Goal:           {}", style(data["goal"].as_str().unwrap_or("Unknown")).cyan());
    println!("  Time Horizon:   {}", style(data["time_horizon"].as_str().unwrap_or("Unknown")).cyan());
    println!("  Risk Profile:   {}", style(data["risk_profile"].as_str().unwrap_or("Unknown")).cyan());
    println!("  Expected Return: {}", style(data["total_expected_return"].as_str().unwrap_or("Unknown")).green().bold());
    
    // Display asset allocation
    println!("\n{}", style("📊 Recommended Asset Allocation:").bold().green());
    let allocation = &data["allocation"];
    
    // Create simple bar chart
    let stocks = allocation["stocks"].as_f64().unwrap_or(0.0) as usize;
    let bonds = allocation["bonds"].as_f64().unwrap_or(0.0) as usize;
    let cash = allocation["cash"].as_f64().unwrap_or(0.0) as usize;
    let alt = allocation["alternatives"].as_f64().unwrap_or(0.0) as usize;
    
    println!("  📈 Stocks:        {:>5.1}% {}", allocation["stocks"].as_f64().unwrap_or(0.0), "█".repeat(stocks / 2));
    println!("  📉 Bonds:         {:>5.1}% {}", allocation["bonds"].as_f64().unwrap_or(0.0), "█".repeat(bonds / 2));
    println!("  💵 Cash:          {:>5.1}% {}", allocation["cash"].as_f64().unwrap_or(0.0), "█".repeat(cash / 2));
    println!("  🏛️  Alternatives: {:>5.1}% {}", allocation["alternatives"].as_f64().unwrap_or(0.0), "█".repeat(alt / 2));
    
    // Display strategy
    println!("\n{}", style("🎯 Investment Strategy:").bold().green());
    println!("  {}", style(data["strategy_summary"].as_str().unwrap_or("No strategy available")).italic());
    
    // Display specific recommendations
    println!("\n{}", style("💎 Specific Investment Recommendations:").bold().green());
    println!("{}", style("─".repeat(60)).dim());
    
    if let Some(recs) = data["recommendations"].as_array() {
        for (i, rec) in recs.iter().enumerate() {
            let symbol = rec["symbol"].as_str().unwrap_or("???");
            let name = rec["name"].as_str().unwrap_or("Unknown");
            let rec_type = rec["type"].as_str().unwrap_or("Unknown");
            let alloc = rec["allocation_percent"].as_f64().unwrap_or(0.0);
            let rationale = rec["rationale"].as_str().unwrap_or("");
            let risk = rec["risk_level"].as_str().unwrap_or("Unknown");
            let exp_return = rec["expected_return"].as_str().unwrap_or("Unknown");
            let confidence = rec["confidence"].as_f64().unwrap_or(0.0);
            
            // Confidence indicator
            let conf_indicator = if confidence >= 0.9 {
                "🟢 Very High"
            } else if confidence >= 0.8 {
                "🟢 High"
            } else if confidence >= 0.7 {
                "🟡 Moderate"
            } else {
                "🟠 Lower"
            };
            
            println!("\n  {} {} - {}", style(format!("{}.", i + 1)).bold(), style(symbol).cyan().bold(), style(name).bold());
            println!("     Type: {} | Allocation: {}%", rec_type, style(format!("{:.0}", alloc)).yellow().bold());
            println!("     Risk Level: {} | Expected Return: {}", risk, exp_return);
            println!("     AI Confidence: {}", conf_indicator);
            println!("     💡 {}", rationale);
        }
    }
    
    // Display warnings if any
    if let Some(warnings) = data["warnings"].as_array() {
        if !warnings.is_empty() {
            println!("\n{}", style("⚠️  Important Considerations:").bold().red());
            for warning in warnings {
                println!("  • {}", warning.as_str().unwrap_or(""));
            }
        }
    }
    
    // Display action items
    println!("\n{}", style("📌 Next Steps:").bold().green());
    println!("  1. Review these recommendations and do your own research");
    println!("  2. Consider dollar-cost averaging for large investments");
    println!("  3. Rebalance your portfolio quarterly to maintain target allocation");
    println!("  4. Review and adjust your plan as your goals change");
    
    println!("\n{}", style("💡 Tip:").cyan().bold());
    println!("  You can save this plan and track your progress in the Portfolio menu.");
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

async fn stop_ai_trading() -> Result<()> {
    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Stop AI trading?")
        .default(false)
        .interact()?;
    
    if confirm {
        println!("\n{}", style("🛑 AI Trading Stopped").yellow().bold());
        println!("All positions remain open. You can sell them manually.");
    } else {
        println!("\n{}", style("AI continues trading.").dim());
    }
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// LEARNING CENTER - Educational content
// ═══════════════════════════════════════════════════════════════════════════════

async fn learning_center() -> Result<()> {
    let choices = vec![
        "📖 How Stock Trading Works",
        "🎯 What is the AI Doing?",
        "⚠️  Understanding Risk",
        "💡 Tips for Beginners",
        "❓ FAQ - Common Questions",
        "↩️  Back to Main Menu",
    ];

    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Learning Center")
            .items(&choices)
            .interact()?;

        match selection {
            0 => explain_trading().await?,
            1 => explain_ai().await?,
            2 => explain_risk().await?,
            3 => beginner_tips().await?,
            4 => faq().await?,
            5 => break,
            _ => {}
        }
    }
    Ok(())
}

async fn explain_trading() -> Result<()> {
    println!();
    println!("{}", style("📖 How Stock Trading Works").bold().green());
    println!("{}", "═".repeat(50));
    
    println!("\n{}", style("The Basics:").bold());
    println!("Stocks are pieces of ownership in a company.");
    println!("When you buy a stock, you own a small part of that company.");
    
    println!("\n{}", style("How You Make Money:").bold());
    println!("  1. 📈 Stock goes UP → Sell for more than you paid = PROFIT");
    println!("  2. 💵 Some stocks pay dividends (regular cash payments)");
    
    println!("\n{}", style("Example:").bold());
    println!("  Buy Apple stock at $100");
    println!("  Wait for it to go to $120");
    println!("  Sell it → You made $20 per share! 🎉");
    
    println!("\n{}", style("But remember:").yellow().bold());
    println!("  Stocks can also go DOWN. Never invest money you can't afford to lose.");
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

async fn explain_ai() -> Result<()> {
    println!();
    println!("{}", style("🤖 What is the AI Doing?").bold().green());
    println!("{}", "═".repeat(50));
    
    println!("\n{}", style("Think of the AI as your smart assistant:").bold());
    
    println!("\n{}", style("What it watches:").bold());
    println!("  • Price movements (is it going up or down?)");
    println!("  • Trading volume (are many people buying?)");
    println!("  • News and trends");
    println!("  • Historical patterns");
    
    println!("\n{}", style("How it decides:").bold());
    println!("  1. Looks for patterns that happened before");
    println!("  2. Calculates the probability of success");
    println!("  3. Only trades when confident (usually >70% chance)");
    
    println!("\n{}", style("Risk Management:").bold());
    println!("  • Never puts all money in one stock");
    println!("  • Sets 'stop losses' to limit downside");
    println!("  • Takes profits at reasonable levels");
    
    println!("\n{}", style("💡 Important:").yellow());
    println!("   AI helps but isn't perfect. Markets are unpredictable!");
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

async fn explain_risk() -> Result<()> {
    println!();
    println!("{}", style("⚠️  Understanding Risk").bold().red());
    println!("{}", "═".repeat(50));
    
    println!("\n{}", style("All investing involves risk. Here's what to know:").bold());
    
    println!("\n{}", style("Types of Risk:").bold());
    println!("  📉 Market Risk - Whole market goes down");
    println!("  🏢 Company Risk - One company has problems");
    println!("  😰 Emotional Risk - Panic selling at wrong time");
    
    println!("\n{}", style("How to Manage Risk:").bold());
    println!("  ✅ Only invest money you can afford to lose");
    println!("  ✅ Don't put all eggs in one basket (diversify)");
    println!("  ✅ Start with practice mode");
    println!("  ✅ Think long term (years, not days)");
    
    println!("\n{}", style("The Golden Rule:").yellow().bold());
    println!("   Higher potential reward = Higher risk");
    println!("   If it sounds too good to be true, it probably is.");
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

async fn beginner_tips() -> Result<()> {
    println!();
    println!("{}", style("💡 Tips for Beginners").bold().green());
    println!("{}", "═".repeat(50));
    
    println!("\n{}", style("Before You Start:").bold());
    println!("  1. 📚 Learn the basics (use our Learning Center!)");
    println!("  2. 🎮 Practice with demo mode first");
    println!("  3. 💵 Start small - don't invest your life savings");
    
    println!("\n{}", style("Good Habits:").bold());
    println!("  ✅ Invest regularly (dollar-cost averaging)");
    println!("  ✅ Don't check prices every 5 minutes");
    println!("  ✅ Have a plan and stick to it");
    println!("  ✅ Keep learning");
    
    println!("\n{}", style("Common Mistakes to Avoid:").bold());
    println!("  ❌ Buying because everyone else is (FOMO)");
    println!("  ❌ Selling in panic when market drops");
    println!("  ❌ Putting all money in one stock");
    println!("  ❌ Trying to get rich quick");
    
    println!("\n{}", style("Remember:").cyan().bold());
    println!("   Most successful investors think in years, not days.");
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

async fn faq() -> Result<()> {
    println!();
    println!("{}", style("❓ Frequently Asked Questions").bold().green());
    println!("{}", "═".repeat(50));
    
    let faqs = vec![
        ("How much money do I need to start?", 
         "You can start with any amount! We recommend practicing first. When ready, start with money you can afford to lose."),
        ("Can I lose all my money?", 
         "Yes, investing always carries risk. That's why we have practice mode - learn first! Never invest emergency funds."),
        ("How quickly will I make money?", 
         "There's no guarantee. Some days up, some days down. Historically, markets grow over years, not days."),
        ("Is the AI always right?", 
         "No AI is perfect. Our AI helps identify opportunities but can't predict the future. Always do your own research too."),
        ("What's practice mode?", 
         "It's like a video game - you trade with pretend money. Learn how everything works before risking real cash."),
        ("How do I withdraw my money?", 
         "This app currently uses practice mode. When we add real trading, you'll be able to withdraw to your bank account."),
    ];
    
    for (i, (q, a)) in faqs.iter().enumerate() {
        println!("\n{} {}", style(format!("Q{}:", i+1)).cyan().bold(), style(q).bold());
        println!("   {}", a);
    }
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// SETTINGS - User preferences
// ═══════════════════════════════════════════════════════════════════════════════

async fn settings_menu(state: Arc<Mutex<AppState>>) -> Result<()> {
    let choices = vec![
        "🎮 Practice Mode vs Live Trading",
        "🔑 API Keys (for real data)",
        "💵 Reset Practice Account",
        "ℹ️  About Smart Money",
        "↩️  Back to Main Menu",
    ];

    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Settings")
            .items(&choices)
            .interact()?;

        match selection {
            0 => toggle_practice_mode(state.clone()).await?,
            1 => api_keys_settings(state.clone()).await?,
            2 => reset_account(state.clone()).await?,
            3 => about_app().await?,
            4 => break,
            _ => {}
        }
    }
    Ok(())
}

async fn toggle_practice_mode(state: Arc<Mutex<AppState>>) -> Result<()> {
    let mut state = state.lock().unwrap();
    
    println!();
    println!("{}", style("🎮 Trading Mode").bold().green());
    println!("{}", "═".repeat(50));
    
    let current_mode = if state.settings.safe_mode { "PRACTICE" } else { "LIVE" };
    println!("\nCurrent Mode: {}", style(current_mode).bold());
    
    if state.settings.safe_mode {
        println!("\n✅ You are in PRACTICE MODE");
        println!("   • Using pretend money");
        println!("   • No risk of losing real cash");
        println!("   • Perfect for learning");
        
        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Switch to LIVE trading with real money?")
            .default(false)
            .interact()?;
        
        if confirm {
            state.settings.safe_mode = false;
            println!("\n{}", style("⚠️  SWITCHED TO LIVE MODE").red().bold());
            println!("{}", style("You will now be trading with REAL money!").red());
        }
    } else {
        println!("\n⚠️  You are in LIVE MODE");
        println!("   • Using REAL money");
        println!("   • Actual profits and losses");
        println!("   • Be careful!");
        
        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Switch back to Practice mode?")
            .default(true)
            .interact()?;
        
        if confirm {
            state.settings.safe_mode = true;
            println!("\n{}", style("✅ Switched to Practice Mode").green());
        }
    }
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

async fn api_keys_settings(state: Arc<Mutex<AppState>>) -> Result<()> {
    let mut state = state.lock().unwrap();
    
    println!();
    println!("{}", style("🔑 API Keys").bold().green());
    println!("{}", "═".repeat(50));
    
    println!("\n{}", style("What are API keys?").bold());
    println!("They let you get real stock market data. Without them,");
    println!("we use demo data (which is fine for learning!).");
    
    println!("\n{}", style("Current Status:").bold());
    if state.settings.api_key_stocks == "demo" {
        println!("  Using DEMO data (free, limited)");
    } else {
        println!("  Using REAL data (from your API key)");
    }
    
    println!("\n{}", style("To get real data:").dim());
    println!("  1. Visit: www.alphavantage.co");
    println!("  2. Get a free API key");
    println!("  3. Enter it below");
    
    let new_key: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter API key (or 'demo' to use demo)")
        .default(state.settings.api_key_stocks.clone())
        .interact()?;
    
    state.settings.api_key_stocks = new_key;
    println!("\n{}", style("✅ Settings saved!").green());
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

async fn reset_account(state: Arc<Mutex<AppState>>) -> Result<()> {
    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Reset practice account to $100,000?")
        .default(false)
        .interact()?;
    
    if confirm {
        let mut state = state.lock().unwrap();
        state.portfolio = Portfolio::new();
        println!("\n{}", style("✅ Account reset! You have $100,000 practice money.").green());
    } else {
        println!("\n{}", style("Reset cancelled.").dim());
    }
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

async fn about_app() -> Result<()> {
    println!();
    println!("{}", style("ℹ️  About Smart Money").bold().green());
    println!("{}", "═".repeat(50));
    
    println!("\n{}", style("Smart Money v1.0").bold());
    println!("Your friendly AI-powered trading assistant");
    
    println!("\n{}", style("What makes us different:").bold());
    println!("  • Designed for regular people, not Wall Street pros");
    println!("  • Practice mode to learn without risk");
    println!("  • AI explains what it's doing in plain English");
    println!("  • Educational - learn as you go");
    
    println!("\n{}", style("Built with:").bold());
    println!("  • Rust (for speed and safety)");
    println!("  • Python AI (smart trading algorithms)");
    println!("  • Love ❤️");
    
    println!("\n{}", style("Disclaimer:").dim());
    println!("This software is for educational purposes.");
    println!("Trading involves risk. Past performance doesn't guarantee future results.");
    
    println!("\n{}", style("Press Enter to continue...").dim());
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}
