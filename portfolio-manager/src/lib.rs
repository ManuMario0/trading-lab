use log::{info, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use trading::model::{
    allocation_batch::AllocationBatch,
    instrument::InstrumentId,
    market_data::PriceUpdate,
    portfolio::{Actual, Portfolio, Target},
};
use trading::Manager;

use crate::model::{
    config::AllocationConfig, consolidated::ConsolidatedPortfolio, ids::MultiplexerId,
};
use crate::risk_guard::{RiskContext, RiskDecision, RiskGuard};

pub mod model;
pub mod risk_guard;

pub struct PortfolioManager {
    // State
    target_portfolio: Portfolio, // The specific target for this manager
    actual_portfolio: Portfolio, // The last known actual state
    prices: HashMap<InstrumentId, f64>,

    // Config & Logic
    config: AllocationConfig,
    risk_guard: RiskGuard,

    // Aggregated State (In a real system, this might be shared or updated differently)
    // For this dummy, we assume this PM manages the "Global" scope.
    consolidated: ConsolidatedPortfolio,

    // ID of this entity (for passing to RiskContext if needed, though mostly ID is for the Strategy)
    // Since we receive anonymous allocations (batched), we assume they apply to the "Global" ID or similar.
    id: MultiplexerId,
}

impl PortfolioManager {
    pub fn new(config: AllocationConfig, risk_guard: RiskGuard) -> Self {
        Self {
            target_portfolio: Portfolio::new(),
            actual_portfolio: Portfolio::new(),
            prices: HashMap::new(),
            config,
            risk_guard,
            consolidated: ConsolidatedPortfolio::default(),
            id: MultiplexerId::new("Global"),
        }
    }

    fn update_prices(&mut self, update: &PriceUpdate) {
        self.prices
            .insert(update.get_instrument_id(), update.get_last());
    }

    fn build_context(&self) -> RiskContext {
        RiskContext {
            portfolio: &self.actual_portfolio, // Risk checks often compare new Target vs Actual (Nav limits)
            prices: &self.prices,
            total_equity: self.actual_portfolio.total_equity, // Use actual equity
            allocation_config: &self.config,
            consolidated: &self.consolidated,
            multiplexer_id: &self.id,
        }
    }
}

impl Manager for PortfolioManager {
    fn on_allocation(&mut self, batch: AllocationBatch) -> Option<Target> {
        info!("Received batch with {} allocations", batch.len());

        // 1. Construct Proposed Target
        // Start with current, apply updates.
        // Assuming Allocation in batch OVERWRITES positions?
        // Or is it a delta?
        // Usually Allocation = "Desired State".
        // If multiple allocations in batch, do they stack?
        // Let's assume they are merged.

        let mut proposed_target = self.target_portfolio.clone();

        for allocation in batch.iter() {
            for (inst_id, pos) in allocation.get_positions() {
                // Determine logic: Overwrite?
                // "Allocation" struct usually implies "This is what I want to hold".
                // If Multiplexer aggregates, it sends the "Net Desired Position".
                proposed_target.update_position(*inst_id, pos.get_quantity());
            }
        }

        // 2. Risk Check
        // We check the 'proposed_target' as a single Allocation (conceptually).
        // Adapt: RiskGuard::check expects `Allocation`.
        // We can convert proposed_target (Portfolio) -> Allocation.
        let mut check_alloc = trading::model::Allocation::new();
        for (inst, pos) in &proposed_target.positions {
            check_alloc.update_position(*inst, pos.get_quantity());
        }

        let ctx = self.build_context();
        match self.risk_guard.check(&check_alloc, &ctx) {
            RiskDecision::Approved => {
                info!("Risk Check Passed. Updating Target.");
                self.target_portfolio = proposed_target;
                // Update Consolidated (Approximation)
                // In this dummy, Consolidated = Local Target (Simplification)
                // Actually Consolidated should track *actuals*.
                // But for Risk Context next time, we use actuals.
                Some(Target(self.target_portfolio.clone()))
            }
            RiskDecision::Rejected(reason) => {
                warn!("Risk Check REJECTED: {}", reason);
                // Policy: Reject update, return nothing (Or return old target?)
                // Returning None implies "No Change" / "No New Target".
                None
            }
        }
    }

    fn on_portfolio(&mut self, portfolio: Actual) -> Option<Target> {
        // Update Actual State
        self.actual_portfolio = portfolio.0.clone();
        self.consolidated.total_equity = self.actual_portfolio.total_equity; // Sync simple

        // Check if we need to re-balance or cut risk?
        // For Dummy PM, passive. Just update state.
        // Optionally re-emit target if we want to enforce persistence?
        None
    }

    fn on_market_data(&mut self, market_data: PriceUpdate) -> Option<Target> {
        self.update_prices(&market_data);
        // Passive.
        None
    }
}
