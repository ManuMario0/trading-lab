use crate::model::portfolio::Portfolio;
use thiserror::Error;

/// Errors that can occur during policy validation.
#[derive(Error, Debug, Clone)]
pub enum PolicyError {
    #[error("Exposure violation: {0}")]
    ExposureViolation(String),
    #[error("Cash constraint violation: {0}")]
    CashViolation(String),
    #[error("General policy violation: {0}")]
    General(String),
}

/// Trait for implementing risk policies and validation rules.
/// Policies generally check a Target Portfolio against some constraints.
pub trait Policy {
    /// Validates the given portfolio against the policy.
    ///
    /// # Arguments
    ///
    /// * `portfolio` - The target portfolio to check.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the portfolio passes the policy.
    /// * `Err(PolicyError)` if a violation is detected.
    fn check(&self, portfolio: &Portfolio) -> Result<(), PolicyError>;

    /// Returns the name of the policy for logging purposes.
    fn name(&self) -> &str;
}

/// A simple policy that ensures no single position exceeds a maximum quantity (absolute).
/// Note: Real policies would likely check % weight or value, but quantity is simpler for a basic example.
pub struct MaxQuantityPolicy {
    pub max_quantity: f64,
}

impl Policy for MaxQuantityPolicy {
    fn check(&self, portfolio: &Portfolio) -> Result<(), PolicyError> {
        for (id, position) in &portfolio.positions {
            if position.get_quantity().abs() > self.max_quantity {
                return Err(PolicyError::ExposureViolation(format!(
                    "Position {} quantity {} exceeds limit {}",
                    id,
                    position.get_quantity(),
                    self.max_quantity
                )));
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "MaxQuantityPolicy"
    }
}
