// Heuristic per-block tournament selector between candidate predictors.

use crate::predictor::PredictorMode;
use crate::predictor::adaptive_linear::AdaptiveLinearPredictor;

pub struct PredictorSelector;

impl PredictorSelector {
    /// Select the best predictor mode for an integer slice.
    ///
    /// Checks for Constant block first, then evaluates sum of absolute residuals
    /// for Delta, Delta-of-Delta, and Adaptive FIR.
    pub fn select_integer_mode(
        samples: &[i64],
        prev_sample: i64,
        history: [i64; 3],
    ) -> PredictorMode {
        if samples.is_empty() {
            return PredictorMode::Delta;
        }

        // 1. Evaluate Constant
        let first = samples[0];
        if samples.iter().all(|&x| x == first) {
            return PredictorMode::Constant;
        }

        // 2. Evaluate Delta
        let mut delta_score = 0u64;
        let mut prev = prev_sample;
        for &val in samples {
            let diff = val.wrapping_sub(prev);
            delta_score = delta_score.saturating_add(diff.unsigned_abs());
            prev = val;
        }

        // 3. Evaluate Delta-of-Delta (second difference)
        let mut delta2_score = 0u64;
        let mut p1 = history[0];
        let mut p2 = history[1];
        for &val in samples {
            let pred = p1.wrapping_add(p1.wrapping_sub(p2));
            let diff = val.wrapping_sub(pred);
            delta2_score = delta2_score.saturating_add(diff.unsigned_abs());
            p2 = p1;
            p1 = val;
        }

        // 4. Evaluate Adaptive Linear FIR
        let mut fir_score = 0u64;
        let mut fir = AdaptiveLinearPredictor::new(history);
        for &val in samples {
            let pred = fir.predict();
            let err = val.wrapping_sub(pred);
            fir_score = fir_score.saturating_add(err.unsigned_abs());
            fir.update(val, pred);
        }

        // 5. Selection rule
        // Prefer DeltaOfDelta if it beats Delta by >= 5% and is at least as good as FIR
        if (delta2_score as u128 * 100) < (delta_score as u128 * 95) && delta2_score <= fir_score {
            PredictorMode::DeltaOfDelta
        } else if (fir_score as u128 * 100) < (delta_score as u128 * 95) && fir_score < delta2_score {
            PredictorMode::AdaptiveLinear
        } else {
            PredictorMode::Delta
        }
    }
}
