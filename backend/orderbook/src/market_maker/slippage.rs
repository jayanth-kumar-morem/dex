use rust_decimal::Decimal;
use super::types::MarketMakerError;

pub struct SlippageProtection {
    max_slippage: Decimal,
}

impl SlippageProtection {
    pub fn new(max_slippage: Decimal) -> Self {
        Self { max_slippage }
    }

    pub fn check_slippage(
        &self,
        actual_output: Decimal,
        min_output: Decimal,
    ) -> Result<(), MarketMakerError> {
        let slippage = (actual_output - min_output) / actual_output;
        
        if slippage > self.max_slippage {
            Err(MarketMakerError::SlippageExceeded)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_slippage_protection() {
        let protection = SlippageProtection::new(dec!(0.02)); // 2% max slippage
        
        // Test within allowed slippage
        let result = protection.check_slippage(dec!(100), dec!(98));
        assert!(result.is_ok());
        
        // Test exceeding allowed slippage
        let result = protection.check_slippage(dec!(100), dec!(97));
        assert!(result.is_err());
    }
} 