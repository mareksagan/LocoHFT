"""
AI Investment Recommendation Engine
Provides personalized investment recommendations based on user goals and timeframe
"""

import json
import math
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass, asdict
from datetime import datetime, timedelta
from enum import Enum

class InvestmentGoal(Enum):
    FIXED_INCOME = "fixed_income"
    BALANCED = "balanced"
    GROWTH = "growth"
    AGGRESSIVE_GROWTH = "aggressive_growth"
    DIVIDEND = "dividend"
    CAPITAL_PRESERVATION = "capital_preservation"

class TimeHorizon(Enum):
    SHORT_TERM = "short_term"      # < 1 year
    MEDIUM_TERM = "medium_term"    # 1-3 years
    LONG_TERM = "long_term"        # 3-10 years
    VERY_LONG_TERM = "very_long"   # 10+ years

@dataclass
class AssetAllocation:
    stocks: float
    bonds: float
    cash: float
    alternatives: float  # REITs, commodities, etc.
    
@dataclass
class Recommendation:
    symbol: str
    name: str
    type: str  # stock, bond, etf, etc.
    allocation_percent: float
    rationale: str
    risk_level: str
    expected_return: str
    timeframe: str
    confidence: float  # 0.0 - 1.0

@dataclass
class InvestmentPlan:
    goal: str
    time_horizon: str
    risk_profile: str
    total_expected_return: str
    allocation: AssetAllocation
    recommendations: List[Recommendation]
    strategy_summary: str
    warnings: List[str]

class RecommendationEngine:
    """AI-powered recommendation engine"""
    
    # Mock market data - in production, this would come from real APIs
    MARKET_DATA = {
        "AAPL": {"name": "Apple Inc.", "price": 175.50, "volatility": 0.25, "dividend_yield": 0.5, "sector": "Technology"},
        "MSFT": {"name": "Microsoft Corp.", "price": 380.25, "volatility": 0.22, "dividend_yield": 0.7, "sector": "Technology"},
        "JNJ": {"name": "Johnson & Johnson", "price": 155.80, "volatility": 0.15, "dividend_yield": 3.0, "sector": "Healthcare"},
        "PG": {"name": "Procter & Gamble", "price": 158.40, "volatility": 0.16, "dividend_yield": 2.4, "sector": "Consumer"},
        "VTI": {"name": "Vanguard Total Stock Market ETF", "price": 245.60, "volatility": 0.18, "dividend_yield": 1.4, "sector": "Diversified"},
        "BND": {"name": "Vanguard Total Bond Market ETF", "price": 72.50, "volatility": 0.06, "dividend_yield": 4.5, "sector": "Bonds"},
        "VNQ": {"name": "Vanguard Real Estate ETF", "price": 85.20, "volatility": 0.20, "dividend_yield": 4.2, "sector": "REIT"},
        "VEA": {"name": "Vanguard Developed Markets ETF", "price": 48.30, "volatility": 0.19, "dividend_yield": 3.1, "sector": "International"},
        "VOO": {"name": "Vanguard S&P 500 ETF", "price": 420.15, "volatility": 0.17, "dividend_yield": 1.3, "sector": "Diversified"},
        "SCHD": {"name": "Schwab US Dividend Equity ETF", "price": 78.90, "volatility": 0.16, "dividend_yield": 3.5, "sector": "Dividend"},
        "TLT": {"name": "iShares 20+ Year Treasury Bond ETF", "price": 92.40, "volatility": 0.14, "dividend_yield": 4.8, "sector": "Bonds"},
        "SHY": {"name": "iShares 1-3 Year Treasury Bond ETF", "price": 82.10, "volatility": 0.02, "dividend_yield": 4.2, "sector": "Bonds"},
        "GLD": {"name": "SPDR Gold Shares", "price": 185.30, "volatility": 0.15, "dividend_yield": 0.0, "sector": "Commodity"},
        "QQQ": {"name": "Invesco QQQ Trust", "price": 390.80, "volatility": 0.23, "dividend_yield": 0.6, "sector": "Technology"},
        "ARKK": {"name": "ARK Innovation ETF", "price": 48.50, "volatility": 0.45, "dividend_yield": 0.0, "sector": "Growth"},
        "VT": {"name": "Vanguard Total World Stock ETF", "price": 108.40, "volatility": 0.18, "dividend_yield": 2.0, "sector": "Diversified"},
    }
    
    def __init__(self):
        self.trend_data = self._analyze_market_trends()
    
    def _analyze_market_trends(self) -> Dict:
        """Analyze current market trends - AI analysis simulation"""
        return {
            "market_sentiment": "bullish",
            "interest_rate_trend": "stable",
            "inflation_outlook": "moderate",
            "tech_sector": "strong",
            "bond_yields": "attractive",
            "volatility": "normal"
        }
    
    def get_goal_description(self, goal: InvestmentGoal) -> str:
        descriptions = {
            InvestmentGoal.FIXED_INCOME: "Generate steady, predictable income through dividends and interest",
            InvestmentGoal.BALANCED: "Balance growth and income with moderate risk",
            InvestmentGoal.GROWTH: "Focus on capital appreciation with higher growth potential",
            InvestmentGoal.AGGRESSIVE_GROWTH: "Maximize returns with high-growth, higher-risk investments",
            InvestmentGoal.DIVIDEND: "Build a portfolio that generates regular dividend income",
            InvestmentGoal.CAPITAL_PRESERVATION: "Protect your principal with minimal risk",
        }
        return descriptions.get(goal, "")
    
    def calculate_allocation(self, goal: InvestmentGoal, horizon: TimeHorizon) -> AssetAllocation:
        """Calculate optimal asset allocation based on goal and timeframe"""
        
        # Base allocations by goal
        allocations = {
            InvestmentGoal.CAPITAL_PRESERVATION: {
                "stocks": 20, "bonds": 60, "cash": 15, "alternatives": 5
            },
            InvestmentGoal.FIXED_INCOME: {
                "stocks": 30, "bonds": 55, "cash": 5, "alternatives": 10
            },
            InvestmentGoal.DIVIDEND: {
                "stocks": 60, "bonds": 25, "cash": 5, "alternatives": 10
            },
            InvestmentGoal.BALANCED: {
                "stocks": 60, "bonds": 30, "cash": 5, "alternatives": 5
            },
            InvestmentGoal.GROWTH: {
                "stocks": 80, "bonds": 10, "cash": 0, "alternatives": 10
            },
            InvestmentGoal.AGGRESSIVE_GROWTH: {
                "stocks": 90, "bonds": 0, "cash": 0, "alternatives": 10
            },
        }
        
        base = allocations.get(goal, allocations[InvestmentGoal.BALANCED])
        
        # Adjust based on time horizon
        horizon_adjustments = {
            TimeHorizon.SHORT_TERM: {"stocks": -15, "bonds": 10, "cash": 5, "alternatives": 0},
            TimeHorizon.MEDIUM_TERM: {"stocks": 0, "bonds": 0, "cash": 0, "alternatives": 0},
            TimeHorizon.LONG_TERM: {"stocks": 5, "bonds": -5, "cash": 0, "alternatives": 0},
            TimeHorizon.VERY_LONG_TERM: {"stocks": 10, "bonds": -10, "cash": 0, "alternatives": 0},
        }
        
        adj = horizon_adjustments.get(horizon, {"stocks": 0, "bonds": 0, "cash": 0, "alternatives": 0})
        
        return AssetAllocation(
            stocks=max(0, min(100, base["stocks"] + adj["stocks"])),
            bonds=max(0, min(100, base["bonds"] + adj["bonds"])),
            cash=max(0, min(100, base["cash"] + adj["cash"])),
            alternatives=max(0, min(100, base["alternatives"] + adj["alternatives"]))
        )
    
    def generate_recommendations(self, goal: InvestmentGoal, horizon: TimeHorizon, 
                                  capital: float = 10000.0) -> InvestmentPlan:
        """Generate personalized investment recommendations"""
        
        allocation = self.calculate_allocation(goal, horizon)
        recommendations = []
        warnings = []
        
        # Generate recommendations based on goal type
        if goal == InvestmentGoal.FIXED_INCOME:
            recommendations = self._fixed_income_recommendations(allocation, horizon)
            risk_profile = "Conservative"
            expected_return = "4-6% annually"
            strategy = "Focus on high-quality bonds, dividend stocks, and income-generating assets. Prioritize stability and regular income over growth."
            
        elif goal == InvestmentGoal.DIVIDEND:
            recommendations = self._dividend_recommendations(allocation, horizon)
            risk_profile = "Conservative to Moderate"
            expected_return = "5-7% annually (including dividends)"
            strategy = "Build a diversified portfolio of dividend aristocrats and dividend-focused ETFs. Reinvest dividends for compounding growth."
            
        elif goal == InvestmentGoal.CAPITAL_PRESERVATION:
            recommendations = self._preservation_recommendations(allocation, horizon)
            risk_profile = "Very Conservative"
            expected_return = "2-4% annually"
            strategy = "Prioritize capital safety with high-quality short-term bonds and stable value investments. Focus on preserving purchasing power."
            warnings.append("Returns may not keep pace with inflation in high-inflation environments")
            
        elif goal == InvestmentGoal.GROWTH:
            recommendations = self._growth_recommendations(allocation, horizon)
            risk_profile = "Moderate to Aggressive"
            expected_return = "7-10% annually"
            strategy = "Invest in high-quality growth stocks and diversified equity ETFs. Accept short-term volatility for long-term appreciation."
            
        elif goal == InvestmentGoal.AGGRESSIVE_GROWTH:
            recommendations = self._aggressive_growth_recommendations(allocation, horizon)
            risk_profile = "Aggressive"
            expected_return = "10-15% annually (with higher volatility)"
            strategy = "Focus on high-growth sectors, emerging technologies, and disruptive innovation. High risk/high reward approach."
            warnings.append("Expect significant volatility - only suitable for investors with high risk tolerance")
            warnings.append("May experience losses of 30-50% in market downturns")
            
        else:  # BALANCED
            recommendations = self._balanced_recommendations(allocation, horizon)
            risk_profile = "Moderate"
            expected_return = "6-8% annually"
            strategy = "Maintain a balanced mix of growth stocks and stable income assets. Regular rebalancing to maintain target allocation."
        
        # Add timeframe-specific advice
        if horizon == TimeHorizon.SHORT_TERM:
            warnings.append("Short-term investing has limited growth potential - consider extending timeframe for better results")
        elif horizon == TimeHorizon.VERY_LONG_TERM:
            strategy += " With your long time horizon, you can ride out market cycles and benefit from compound growth."
        
        return InvestmentPlan(
            goal=goal.value.replace("_", " ").title(),
            time_horizon=horizon.value.replace("_", " ").title(),
            risk_profile=risk_profile,
            total_expected_return=expected_return,
            allocation=allocation,
            recommendations=recommendations,
            strategy_summary=strategy,
            warnings=warnings
        )
    
    def _fixed_income_recommendations(self, allocation: AssetAllocation, 
                                       horizon: TimeHorizon) -> List[Recommendation]:
        """Generate fixed income focused recommendations"""
        recs = []
        
        # Bonds allocation
        if horizon == TimeHorizon.SHORT_TERM:
            recs.append(Recommendation(
                symbol="SHY", name=self.MARKET_DATA["SHY"]["name"],
                type="ETF - Short Term Treasury",
                allocation_percent=30,
                rationale="Short-term treasuries provide stability and liquidity for near-term needs",
                risk_level="Very Low",
                expected_return="4-4.5%",
                timeframe="1-3 years",
                confidence=0.92
            ))
        else:
            recs.append(Recommendation(
                symbol="TLT", name=self.MARKET_DATA["TLT"]["name"],
                type="ETF - Long Term Treasury",
                allocation_percent=25,
                rationale="Long-term bonds offer higher yields and price appreciation when rates fall",
                risk_level="Low to Moderate",
                expected_return="4.5-5.5%",
                timeframe="5-10 years",
                confidence=0.88
            ))
            recs.append(Recommendation(
                symbol="BND", name=self.MARKET_DATA["BND"]["name"],
                type="ETF - Total Bond Market",
                allocation_percent=20,
                rationale="Diversified bond exposure across government and corporate bonds",
                risk_level="Low",
                expected_return="4-5%",
                timeframe="3-7 years",
                confidence=0.90
            ))
        
        # Dividend stocks for income
        recs.append(Recommendation(
            symbol="SCHD", name=self.MARKET_DATA["SCHD"]["name"],
            type="ETF - Dividend Focused",
            allocation_percent=20,
            rationale="Quality dividend-paying stocks with consistent payout history",
            risk_level="Low to Moderate",
            expected_return="5-7% (including dividends)",
            timeframe="3-5 years",
            confidence=0.85
        ))
        
        # REIT for income
        recs.append(Recommendation(
            symbol="VNQ", name=self.MARKET_DATA["VNQ"]["name"],
            type="ETF - Real Estate",
            allocation_percent=10,
            rationale="REITs provide income through property rents and potential appreciation",
            risk_level="Moderate",
            expected_return="5-7%",
            timeframe="5+ years",
            confidence=0.80
        ))
        
        return recs
    
    def _dividend_recommendations(self, allocation: AssetAllocation, 
                                   horizon: TimeHorizon) -> List[Recommendation]:
        """Generate dividend-focused recommendations"""
        recs = []
        
        recs.append(Recommendation(
            symbol="SCHD", name=self.MARKET_DATA["SCHD"]["name"],
            type="ETF - Dividend Equity",
            allocation_percent=30,
            rationale="Core holding for dividend growth with quality screening",
            risk_level="Low to Moderate",
            expected_return="5-7%",
            timeframe="Long-term",
            confidence=0.88
        ))
        
        recs.append(Recommendation(
            symbol="JNJ", name=self.MARKET_DATA["JNJ"]["name"],
            type="Stock - Healthcare",
            allocation_percent=20,
            rationale="Dividend aristocrat with 60+ years of increasing dividends",
            risk_level="Low",
            expected_return="6-8%",
            timeframe="Long-term",
            confidence=0.90
        ))
        
        recs.append(Recommendation(
            symbol="PG", name=self.MARKET_DATA["PG"]["name"],
            type="Stock - Consumer Staples",
            allocation_percent=20,
            rationale="Defensive consumer stock with reliable dividend growth",
            risk_level="Low",
            expected_return="6-7%",
            timeframe="Long-term",
            confidence=0.88
        ))
        
        recs.append(Recommendation(
            symbol="VNQ", name=self.MARKET_DATA["VNQ"]["name"],
            type="ETF - REITs",
            allocation_percent=15,
            rationale="Real estate exposure with high dividend yield",
            risk_level="Moderate",
            expected_return="5-7%",
            timeframe="5+ years",
            confidence=0.82
        ))
        
        recs.append(Recommendation(
            symbol="BND", name=self.MARKET_DATA["BND"]["name"],
            type="ETF - Bonds",
            allocation_percent=15,
            rationale="Stability and income through diversified bonds",
            risk_level="Low",
            expected_return="4-5%",
            timeframe="Medium-term",
            confidence=0.90
        ))
        
        return recs
    
    def _growth_recommendations(self, allocation: AssetAllocation, 
                                 horizon: TimeHorizon) -> List[Recommendation]:
        """Generate growth-focused recommendations"""
        recs = []
        
        recs.append(Recommendation(
            symbol="VTI", name=self.MARKET_DATA["VTI"]["name"],
            type="ETF - Total US Stock Market",
            allocation_percent=35,
            rationale="Broad US market exposure capturing overall economic growth",
            risk_level="Moderate",
            expected_return="7-10%",
            timeframe="Long-term",
            confidence=0.90
        ))
        
        recs.append(Recommendation(
            symbol="QQQ", name=self.MARKET_DATA["QQQ"]["name"],
            type="ETF - Nasdaq 100",
            allocation_percent=20,
            rationale="Tech-heavy growth exposure with leading innovative companies",
            risk_level="Moderate to High",
            expected_return="8-12%",
            timeframe="Long-term",
            confidence=0.85
        ))
        
        recs.append(Recommendation(
            symbol="MSFT", name=self.MARKET_DATA["MSFT"]["name"],
            type="Stock - Technology",
            allocation_percent=15,
            rationale="Leading tech company with cloud growth and AI positioning",
            risk_level="Moderate",
            expected_return="10-15%",
            timeframe="3-5 years",
            confidence=0.87
        ))
        
        recs.append(Recommendation(
            symbol="AAPL", name=self.MARKET_DATA["AAPL"]["name"],
            type="Stock - Technology",
            allocation_percent=10,
            rationale="Strong brand, cash generation, and services growth",
            risk_level="Moderate",
            expected_return="8-12%",
            timeframe="Long-term",
            confidence=0.85
        ))
        
        recs.append(Recommendation(
            symbol="VEA", name=self.MARKET_DATA["VEA"]["name"],
            type="ETF - International Developed",
            allocation_percent=10,
            rationale="International diversification with growth potential",
            risk_level="Moderate",
            expected_return="6-9%",
            timeframe="Long-term",
            confidence=0.82
        ))
        
        recs.append(Recommendation(
            symbol="BND", name=self.MARKET_DATA["BND"]["name"],
            type="ETF - Bonds",
            allocation_percent=10,
            rationale="Stability buffer for portfolio volatility",
            risk_level="Low",
            expected_return="4-5%",
            timeframe="Medium-term",
            confidence=0.88
        ))
        
        return recs
    
    def _aggressive_growth_recommendations(self, allocation: AssetAllocation, 
                                            horizon: TimeHorizon) -> List[Recommendation]:
        """Generate aggressive growth recommendations"""
        recs = []
        
        recs.append(Recommendation(
            symbol="ARKK", name=self.MARKET_DATA["ARKK"]["name"],
            type="ETF - Innovation",
            allocation_percent=20,
            rationale="High-growth disruptive innovation in genomics, AI, and fintech",
            risk_level="High",
            expected_return="15-25% (very volatile)",
            timeframe="5+ years",
            confidence=0.70
        ))
        
        recs.append(Recommendation(
            symbol="QQQ", name=self.MARKET_DATA["QQQ"]["name"],
            type="ETF - Nasdaq 100",
            allocation_percent=25,
            rationale="Concentrated tech exposure for maximum growth potential",
            risk_level="High",
            expected_return="10-15%",
            timeframe="Long-term",
            confidence=0.82
        ))
        
        recs.append(Recommendation(
            symbol="VTI", name=self.MARKET_DATA["VTI"]["name"],
            type="ETF - Total Stock Market",
            allocation_percent=25,
            rationale="Broad market foundation with growth tilt",
            risk_level="Moderate to High",
            expected_return="8-12%",
            timeframe="Long-term",
            confidence=0.85
        ))
        
        recs.append(Recommendation(
            symbol="MSFT", name=self.MARKET_DATA["MSFT"]["name"],
            type="Stock - Technology",
            allocation_percent=15,
            rationale="AI leader with strong competitive position",
            risk_level="Moderate",
            expected_return="12-18%",
            timeframe="3-5 years",
            confidence=0.85
        ))
        
        recs.append(Recommendation(
            symbol="GLD", name=self.MARKET_DATA["GLD"]["name"],
            type="ETF - Gold",
            allocation_percent=10,
            rationale="Inflation hedge and portfolio diversifier",
            risk_level="Moderate",
            expected_return="3-6%",
            timeframe="Long-term",
            confidence=0.75
        ))
        
        recs.append(Recommendation(
            symbol="VNQ", name=self.MARKET_DATA["VNQ"]["name"],
            type="ETF - Real Estate",
            allocation_percent=5,
            rationale="Real estate exposure with growth potential",
            risk_level="Moderate to High",
            expected_return="6-9%",
            timeframe="5+ years",
            confidence=0.78
        ))
        
        return recs
    
    def _balanced_recommendations(self, allocation: AssetAllocation, 
                                   horizon: TimeHorizon) -> List[Recommendation]:
        """Generate balanced portfolio recommendations"""
        recs = []
        
        recs.append(Recommendation(
            symbol="VOO", name=self.MARKET_DATA["VOO"]["name"],
            type="ETF - S&P 500",
            allocation_percent=35,
            rationale="Core US large-cap exposure for growth",
            risk_level="Moderate",
            expected_return="7-10%",
            timeframe="Long-term",
            confidence=0.92
        ))
        
        recs.append(Recommendation(
            symbol="VEA", name=self.MARKET_DATA["VEA"]["name"],
            type="ETF - International",
            allocation_percent=15,
            rationale="International diversification",
            risk_level="Moderate",
            expected_return="6-9%",
            timeframe="Long-term",
            confidence=0.85
        ))
        
        recs.append(Recommendation(
            symbol="BND", name=self.MARKET_DATA["BND"]["name"],
            type="ETF - Total Bond Market",
            allocation_percent=25,
            rationale="Stability and income generation",
            risk_level="Low",
            expected_return="4-5%",
            timeframe="Medium-term",
            confidence=0.90
        ))
        
        recs.append(Recommendation(
            symbol="SCHD", name=self.MARKET_DATA["SCHD"]["name"],
            type="ETF - Dividend Equity",
            allocation_percent=15,
            rationale="Quality dividend stocks for income and growth",
            risk_level="Low to Moderate",
            expected_return="6-8%",
            timeframe="Long-term",
            confidence=0.87
        ))
        
        recs.append(Recommendation(
            symbol="VNQ", name=self.MARKET_DATA["VNQ"]["name"],
            type="ETF - REITs",
            allocation_percent=10,
            rationale="Real estate for diversification and income",
            risk_level="Moderate",
            expected_return="5-7%",
            timeframe="5+ years",
            confidence=0.82
        ))
        
        return recs
    
    def _preservation_recommendations(self, allocation: AssetAllocation, 
                                       horizon: TimeHorizon) -> List[Recommendation]:
        """Generate capital preservation recommendations"""
        recs = []
        
        recs.append(Recommendation(
            symbol="SHY", name=self.MARKET_DATA["SHY"]["name"],
            type="ETF - Short-Term Treasury",
            allocation_percent=40,
            rationale="Maximum safety with government backing",
            risk_level="Very Low",
            expected_return="4-4.5%",
            timeframe="Short-term",
            confidence=0.95
        ))
        
        recs.append(Recommendation(
            symbol="BND", name=self.MARKET_DATA["BND"]["name"],
            type="ETF - Total Bond Market",
            allocation_percent=30,
            rationale="Diversified bond exposure for stability",
            risk_level="Low",
            expected_return="4-5%",
            timeframe="Medium-term",
            confidence=0.90
        ))
        
        recs.append(Recommendation(
            symbol="VT", name=self.MARKET_DATA["VT"]["name"],
            type="ETF - Total World Stock",
            allocation_percent=15,
            rationale="Minimal equity exposure for inflation protection",
            risk_level="Moderate",
            expected_return="5-7%",
            timeframe="Long-term",
            confidence=0.85
        ))
        
        recs.append(Recommendation(
            symbol="GLD", name=self.MARKET_DATA["GLD"]["name"],
            type="ETF - Gold",
            allocation_percent=10,
            rationale="Safe haven asset for portfolio protection",
            risk_level="Moderate",
            expected_return="3-5%",
            timeframe="Long-term",
            confidence=0.78
        ))
        
        recs.append(Recommendation(
            symbol="PG", name=self.MARKET_DATA["PG"]["name"],
            type="Stock - Consumer Staples",
            allocation_percent=5,
            rationale="Defensive stock with stable business",
            risk_level="Low",
            expected_return="5-6%",
            timeframe="Long-term",
            confidence=0.88
        ))
        
        return recs


# Functions that can be called from Rust via PyO3
def get_investment_recommendations(goal_str: str, horizon_str: str, capital: float = 10000.0) -> str:
    """
    Main entry point for Rust to get investment recommendations.
    Returns JSON string with recommendations.
    """
    try:
        goal = InvestmentGoal(goal_str)
    except ValueError:
        goal = InvestmentGoal.BALANCED
    
    try:
        horizon = TimeHorizon(horizon_str)
    except ValueError:
        horizon = TimeHorizon.MEDIUM_TERM
    
    engine = RecommendationEngine()
    plan = engine.generate_recommendations(goal, horizon, capital)
    
    # Convert to dictionary for JSON serialization
    result = {
        "goal": plan.goal,
        "time_horizon": plan.time_horizon,
        "risk_profile": plan.risk_profile,
        "total_expected_return": plan.total_expected_return,
        "allocation": {
            "stocks": plan.allocation.stocks,
            "bonds": plan.allocation.bonds,
            "cash": plan.allocation.cash,
            "alternatives": plan.allocation.alternatives,
        },
        "recommendations": [
            {
                "symbol": r.symbol,
                "name": r.name,
                "type": r.type,
                "allocation_percent": r.allocation_percent,
                "rationale": r.rationale,
                "risk_level": r.risk_level,
                "expected_return": r.expected_return,
                "timeframe": r.timeframe,
                "confidence": r.confidence,
            }
            for r in plan.recommendations
        ],
        "strategy_summary": plan.strategy_summary,
        "warnings": plan.warnings,
    }
    
    return json.dumps(result, indent=2)


def get_available_goals() -> str:
    """Return available investment goals"""
    goals = [
        {"id": "capital_preservation", "name": "Capital Preservation", "description": "Protect my money with minimal risk"},
        {"id": "fixed_income", "name": "Fixed Income", "description": "Generate steady income from dividends and interest"},
        {"id": "dividend", "name": "Dividend Growth", "description": "Build portfolio for regular dividend income"},
        {"id": "balanced", "name": "Balanced Growth", "description": "Balance growth and income with moderate risk"},
        {"id": "growth", "name": "Growth", "description": "Focus on capital appreciation"},
        {"id": "aggressive_growth", "name": "Aggressive Growth", "description": "Maximize returns with higher risk tolerance"},
    ]
    return json.dumps(goals)


def get_available_timeframes() -> str:
    """Return available time horizons"""
    timeframes = [
        {"id": "short_term", "name": "Short Term", "description": "Less than 1 year", "suitable_for": "Capital preservation, emergency funds"},
        {"id": "medium_term", "name": "Medium Term", "description": "1-3 years", "suitable_for": "Fixed income, balanced portfolios"},
        {"id": "long_term", "name": "Long Term", "description": "3-10 years", "suitable_for": "Growth, dividend investing"},
        {"id": "very_long", "name": "Very Long Term", "description": "10+ years", "suitable_for": "Aggressive growth, retirement planning"},
    ]
    return json.dumps(timeframes)
