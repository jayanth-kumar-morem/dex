use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use super::types::MarketMakerError;

pub struct PriceImpactCalculator {
    max_impact: Decimal,
    impact_multiplier: Decimal,
}

impl PriceImpactCalculator {
    pub fn new() -> Self {
        Self {
            max_impact: dec!(0.25),  // Increased to 25% max impact
            impact_multiplier: dec!(1.1), // Reduced multiplier for smoother scaling
        }
    }

    pub fn with_config(max_impact: Decimal, impact_multiplier: Decimal) -> Self {
        Self {
            max_impact,
            impact_multiplier,
        }
    }

    // Helper method to calculate square root of a Decimal
    fn decimal_sqrt(&self, value: Decimal) -> Decimal {
        if value == Decimal::ZERO {
            return Decimal::ZERO;
        }

        let mut x = value;
        let mut x0;
        
        // Newton's method for square root with better initial guess
        x = value / dec!(2); // Start with value/2 as initial guess
        
        for _ in 0..20 {  // Usually converges in < 10 iterations
            x0 = x;
            x = (x + value / x) / dec!(2);
            
            // Check for convergence with higher precision
            if (x - x0).abs() < dec!(0.0000000001) {
                break;
            }
        }
        
        // Round to 8 decimal places for consistency
        x.round_dp(8)
    }

    pub fn calculate_price_impact(
        &self,
        input_amount: Decimal,
        reserve: Decimal,
        depth_factor: Decimal,
    ) -> Result<Decimal, MarketMakerError> {
        if reserve == Decimal::ZERO {
            return Err(MarketMakerError::InsufficientLiquidity);
        }

        // Calculate base impact with gentler formula
        let base_impact = input_amount / (reserve + input_amount * dec!(2));
        
        // Apply depth factor with smoother transition
        let depth_adjusted_impact = if input_amount > reserve * dec!(0.1) {
            base_impact * (Decimal::ONE + 
                (input_amount / reserve - dec!(0.1)) * self.impact_multiplier * depth_factor)
        } else {
            base_impact
        };

        // Check against maximum allowed impact
        if depth_adjusted_impact > self.max_impact {
            return Err(MarketMakerError::PriceImpactTooHigh);
        }

        Ok(depth_adjusted_impact)
    }

    pub fn estimate_output_with_impact(
        &self,
        input_amount: Decimal,
        input_reserve: Decimal,
        output_reserve: Decimal,
    ) -> Result<(Decimal, Decimal), MarketMakerError> {
        // Calculate depth factor with gentler scaling
        let depth_factor = if input_reserve > output_reserve {
            self.decimal_sqrt(input_reserve / output_reserve)
        } else {
            self.decimal_sqrt(output_reserve / input_reserve)
        };

        let impact = self.calculate_price_impact(input_amount, input_reserve, depth_factor)?;
        
        let k = input_reserve * output_reserve;
        let new_input_reserve = input_reserve + input_amount;
        let ideal_output = output_reserve - (k / new_input_reserve);
        
        // Reduce impact penalty
        let actual_output = ideal_output * (Decimal::ONE - (impact * dec!(0.5)));
        
        Ok((actual_output, impact))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_decimal_sqrt() {
        let calculator = PriceImpactCalculator::new();
        
        // Test with exact squares
        let sqrt4 = calculator.decimal_sqrt(dec!(4));
        println!("sqrt4: {}", sqrt4);
        assert!((sqrt4 - dec!(2)).abs() < dec!(0.01));
        
        let sqrt9 = calculator.decimal_sqrt(dec!(9));
        println!("sqrt9: {}", sqrt9);
        assert!((sqrt9 - dec!(3)).abs() < dec!(0.01));
        
        // Test irrational number
        let sqrt2 = calculator.decimal_sqrt(dec!(2));
        println!("sqrt2: {}", sqrt2);
        assert!((sqrt2 - dec!(1.4142135624)).abs() < dec!(0.01));
    }

    #[test]
    fn test_normal_price_impact() {
        let calculator = PriceImpactCalculator::new();
        
        // Small order (1% of reserve)
        let impact = calculator
            .calculate_price_impact(dec!(100), dec!(10000), dec!(1))
            .unwrap();
        assert!(impact < dec!(0.02)); // Less than 2% impact
        
        // Medium order (5% of reserve)
        let impact = calculator
            .calculate_price_impact(dec!(500), dec!(10000), dec!(1))
            .unwrap();
        assert!(impact < dec!(0.10)); // Less than 10% impact
    }

    #[test]
    fn test_large_order_impact() {
        let calculator = PriceImpactCalculator::new();
        
        // Large order (15% of reserve)
        let impact = calculator
            .calculate_price_impact(dec!(1500), dec!(10000), dec!(1.1))
            .unwrap();
        assert!(impact > dec!(0.05)); // Should have significant impact
        assert!(impact < dec!(0.25)); // But less than max impact
    }

    #[test]
    fn test_max_impact_threshold() {
        let calculator = PriceImpactCalculator::new();
        
        // Very large order (50% of reserve)
        let result = calculator.calculate_price_impact(
            dec!(5000),
            dec!(10000),
            dec!(2.0)
        );
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MarketMakerError::PriceImpactTooHigh));
    }

    #[test]
    fn test_output_estimation() {
        let calculator = PriceImpactCalculator::new();
        
        // Test balanced pool
        let (output, impact) = calculator
            .estimate_output_with_impact(dec!(100), dec!(10000), dec!(10000))
            .unwrap();
            
        assert!(output > dec!(90)); // Should get close to ideal output
        assert!(impact < dec!(0.1)); // Impact should be reasonable
        
        // Test imbalanced pool
        let (output, impact) = calculator
            .estimate_output_with_impact(dec!(100), dec!(20000), dec!(10000))
            .unwrap();
            
        assert!(output > dec!(45)); // Should get reasonable output
        assert!(impact > dec!(0.001)); // Should have higher impact due to imbalance
    }

    #[test]
    fn test_custom_config() {
        let calculator = PriceImpactCalculator::with_config(dec!(0.05), dec!(2.0));
        
        // Test with stricter impact limit
        let result = calculator.calculate_price_impact(
            dec!(1000),
            dec!(10000),
            dec!(1.5)
        );
        assert!(result.is_err()); // Should fail with 5% limit
    }
} 