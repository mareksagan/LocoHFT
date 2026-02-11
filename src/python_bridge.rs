//! Python interoperability layer using PyO3
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyFloat, PyString};
use crate::engine::{Tick, Side, ExecutionEngine, RiskEngine};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct PythonStrategy {
    module: Py<PyModule>,
    strategy_instance: Py<PyAny>,
}

impl PythonStrategy {
    pub fn new(capital: f64) -> PyResult<Self> {
        Python::with_gil(|py| {
            // Embed Python code directly into binary using include_str!
            let strategy_code = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/python/strategy.py"));
            
            // Create module from embedded code
            let module = PyModule::from_code(py, strategy_code, "strategy.py", "strategy")?;
            
            // Instantiate the strategy class
            let strategy_class = module.getattr("LocoHFTStrategy")?;
            let instance = strategy_class.call1((capital,))?;
            
            Ok(PythonStrategy {
                module: module.into(),
                strategy_instance: instance.into(),
            })
        })
    }
    
    pub fn on_tick(&self, tick: &Tick) -> PyResult<Option<TradeSignal>> {
        Python::with_gil(|py| {
            // Call Python method: on_market_data(symbol, price, volume, timestamp)
            let result = self.strategy_instance.call_method1(
                py,
                "on_market_data",
                (
                    &tick.symbol,
                    tick.price,
                    tick.size,
                    tick.timestamp,
                ),
            )?;
            
            // If Python returns None, no signal
            if result.is_none(py) {
                return Ok(None);
            }
            
            // Extract signal from Python dict
            let dict: &PyDict = result.downcast(py)?;
            let action: String = dict.get_item("action")?.unwrap().extract()?;
            let size: f64 = dict.get_item("size")?.unwrap().extract()?;
            let price: f64 = dict.get_item("price")?.unwrap().extract()?;
            
            let side = match action.as_str() {
                "BUY" => Side::Buy,
                "SELL" => Side::Sell,
                _ => return Ok(None),
            };
            
            Ok(Some(TradeSignal { side, size, price }))
        })
    }
    
    pub fn check_risk(&self, var_95: f64, exposure: f64) -> PyResult<bool> {
        Python::with_gil(|py| {
            let result: bool = self.strategy_instance
                .call_method1(py, "on_risk_update", (var_95, exposure))?
                .extract(py)?;
            Ok(result)
        })
    }
}

#[derive(Debug)]
pub struct TradeSignal {
    pub side: Side,
    pub size: f64,
    pub price: f64,
}

pub struct HybridEngine {
    pub python: PythonStrategy,
    pub execution: Arc<Mutex<ExecutionEngine>>,
    pub risk: RiskEngine,
}

impl HybridEngine {
    pub fn new(capital: f64) -> PyResult<Self> {
        Ok(Self {
            python: PythonStrategy::new(capital)?,
            execution: Arc::new(Mutex::new(ExecutionEngine::new())),
            risk: RiskEngine::new(),
        })
    }
    
    pub fn process_tick(&mut self, tick: Tick) -> PyResult<()> {
        // 1. Python generates signal ( Strategy logic)
        let start = Instant::now();
        let signal = self.python.on_tick(&tick)?;
        let py_latency = start.elapsed().as_micros();
        
        if let Some(sig) = signal {
            log::info!(
                "[PYTHON] Signal generated in {}µs: {:?}",
                py_latency, sig
            );
            
            // 2. Rust executes (Microsecond latency)
            let mut exec = self.execution.lock().unwrap();
            
            if self.risk.check_pre_trade(&tick.symbol, sig.size, exec.get_position(&tick.symbol)) {
                let fill = exec.execute(&tick.symbol, sig.side, sig.size, tick.price);
                log::info!(
                    "[RUST] Executed fill: PnL=${:.2}",
                    fill.pnl
                );
            }
        }
        
        Ok(())
    }
}
