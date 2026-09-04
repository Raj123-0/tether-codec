// Heuristic per-block tournament selector between candidate predictors.

use crate::predictor::PredictorMode;
use crate::predictor::adaptive_linear::AdaptiveLinearPredictor;

pub struct PredictorSelector;

impl PredictorSelector {
    /// Select the best predictor mode for an integer slice.
    ///
    /// Evaluates sum of absolute residuals for Delta vs Adaptive FIR.
    /// Prefers the simpler Delta predictor unless Adaptive FIR outperforms it by >= 5%.
    pub fn select_integer_mode(
        samples: &[i64],
        prev_sample: i64,
        history: [i64; 3],
    ) -> PredictorMode {
        if samples.is_empty() {
            return PredictorMode::Delta;
        }

        // 1. Evaluate Delta
        let mut delta_score = 0u64;
        let mut prev = prev_sample;
        for &val in samples {
            let diff = val.wrapping_sub(prev);
            delta_score = delta_score.saturating_add(diff.unsigned_abs());
            prev = val;
        }

        // 2. Evaluate Adaptive Linear FIR
        let mut fir_score = 0u64;
        let mut fir = AdaptiveLinearPredictor::new(history);
        for &val in samples {
            let pred = fir.predict();
            let err = val.wrapping_sub(pred);
            fir_score = fir_score.saturating_add(err.unsigned_abs());
            fir.update(val, pred);
        }

        // 3. Selection rule: FIR must beat Delta by at least 5%
        if (fir_score as u128 * 100) < (delta_score as u128 * 95) {
            PredictorMode::AdaptiveLinear
        } else {
            PredictorMode::Delta
        }
    }
}
