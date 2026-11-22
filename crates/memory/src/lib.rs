use libm::{exp, log, pow, sqrt};

pub const DAY_MS: f64 = 86_400_000.0;

const PASS: f64 = 4.0;
const FAIL: f64 = 1.0;
const STABILITY_MIN: f64 = 0.001;
const STABILITY_MAX: f64 = 36_500.0;
const DIFFICULTY_SPAN: f64 = 100.0;
const RECALL_EDGE: f64 = 1e-6;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weights {
    pub compress: f64,
    pub decay: f64,
    pub curve: f64,
    pub brittle: f64,
    pub ladder: f64,
    pub start: f64,
    pub ease: f64,
}

impl Weights {
    pub const BINARY: Self = Self {
        compress: 2.1210,
        decay: 0.1721,
        curve: 0.4579,
        brittle: 0.3157,
        ladder: 2.4905,
        start: 0.1583,
        ease: 0.6506,
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Memory {
    pub stability: f64,
    pub brittleness: f64,
    pub difficulty: f64,
}

const fn grade(pass: bool) -> f64 {
    if pass { PASS } else { FAIL }
}

fn brittleness(weights: &Weights, rating: f64) -> f64 {
    weights.brittle * pow(weights.ladder, 1.0 - rating)
}

fn harder(weights: &Weights, difficulty: f64, rating: f64) -> f64 {
    (difficulty + (3.0 - rating).max(0.0)).clamp(weights.ease, weights.ease + DIFFICULTY_SPAN)
}

impl Memory {
    #[must_use]
    pub fn first(weights: &Weights, pass: bool) -> Self {
        let rating = grade(pass);
        Self {
            stability: (weights.start * pow(weights.ladder, rating - 1.0))
                .clamp(STABILITY_MIN, STABILITY_MAX),
            brittleness: brittleness(weights, rating),
            difficulty: harder(weights, weights.ease, rating),
        }
    }

    fn shape(&self, weights: &Weights) -> f64 {
        weights.decay * pow(self.difficulty, weights.curve.max(0.0))
    }

    fn age(&self, weights: &Weights, days: f64) -> f64 {
        pow(days.max(0.0), 1.0 / weights.compress) + sqrt(self.brittleness * self.stability)
    }

    #[must_use]
    pub fn recall(&self, weights: &Weights, days: f64) -> f64 {
        let base = 1.0 + self.age(weights, days) / self.stability;
        exp(-self.shape(weights) * log(base)).max(RECALL_EDGE)
    }

    #[must_use]
    pub fn review(&self, weights: &Weights, days: f64, pass: bool) -> Self {
        let rating = grade(pass);
        let recall = self.recall(weights, days);
        let stability = if pass {
            self.stability * exp(-log(recall) / self.shape(weights))
        } else {
            self.stability * pow(1.0 - recall, weights.curve.max(0.0))
        };
        Self {
            stability: stability.clamp(STABILITY_MIN, STABILITY_MAX),
            brittleness: brittleness(weights, rating),
            difficulty: harder(weights, self.difficulty, rating),
        }
    }
}

#[must_use]
pub fn days_between(earlier_ms: i64, later_ms: i64) -> f64 {
    #[expect(
        clippy::cast_precision_loss,
        clippy::as_conversions,
        reason = "milliseconds stay exact in f64 below 2^53, about 285 thousand years"
    )]
    let gap = later_ms.saturating_sub(earlier_ms) as f64;
    (gap / DAY_MS).max(0.0)
}
