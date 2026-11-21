use konekode_space::Set;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Empty,
    Lengths,
    Weights,
    Noise,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Noise {
    careless: f64,
    lucky: f64,
}

impl Noise {
    pub fn new(careless: f64, lucky: f64) -> Result<Self, Error> {
        if careless > 0.0 && lucky > 0.0 && careless + lucky < 1.0 {
            Ok(Self { careless, lucky })
        } else {
            Err(Error::Noise)
        }
    }

    #[must_use]
    pub const fn careless(self) -> f64 {
        self.careless
    }

    #[must_use]
    pub const fn lucky(self) -> f64 {
        self.lucky
    }

    #[must_use]
    pub fn likelihood(self, knows: bool, correct: bool) -> f64 {
        match (knows, correct) {
            (true, true) => 1.0 - self.careless,
            (true, false) => self.careless,
            (false, true) => self.lucky,
            (false, false) => 1.0 - self.lucky,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Response {
    pub item: usize,
    pub correct: bool,
}

#[derive(Clone, Debug)]
pub struct Belief {
    states: Vec<Set>,
    weights: Vec<f64>,
}

impl Belief {
    pub fn uniform(states: Vec<Set>) -> Result<Self, Error> {
        let weights = vec![1.0; states.len()];
        Self::weighted(states, weights)
    }

    pub fn weighted(states: Vec<Set>, weights: Vec<f64>) -> Result<Self, Error> {
        if states.is_empty() {
            return Err(Error::Empty);
        }
        if states.len() != weights.len() {
            return Err(Error::Lengths);
        }
        if weights
            .iter()
            .any(|weight| !weight.is_finite() || *weight < 0.0)
            || weights.iter().sum::<f64>() <= 0.0
        {
            return Err(Error::Weights);
        }
        let mut belief = Self { states, weights };
        belief.normalize();
        Ok(belief)
    }

    #[must_use]
    pub fn states(&self) -> &[Set] {
        &self.states
    }

    #[must_use]
    pub fn weights(&self) -> &[f64] {
        &self.weights
    }

    pub fn observe(&mut self, response: Response, noise: Noise) {
        for (state, weight) in self.states.iter().zip(self.weights.iter_mut()) {
            *weight *= noise.likelihood(state.contains(response.item), response.correct);
        }
        self.normalize();
    }

    pub fn learn(&mut self, item: usize, gain: f64) {
        let gain = gain.clamp(0.0, 1.0);
        let mut moved = vec![0.0; self.weights.len()];
        for (from, state) in self.states.iter().enumerate() {
            if state.contains(item) {
                continue;
            }
            let mut next = state.clone();
            next.insert(item);
            if let Some(to) = self.states.iter().position(|other| *other == next) {
                let share = self.weights[from] * gain;
                moved[from] -= share;
                moved[to] += share;
            }
        }
        for (weight, change) in self.weights.iter_mut().zip(moved) {
            *weight = (*weight + change).max(0.0);
        }
        self.normalize();
    }

    #[must_use]
    pub fn marginal(&self, item: usize) -> f64 {
        self.states
            .iter()
            .zip(&self.weights)
            .filter(|(state, _)| state.contains(item))
            .map(|(_, weight)| weight)
            .sum()
    }

    #[must_use]
    pub fn best(&self) -> (&Set, f64) {
        let mut top = 0;
        for (at, weight) in self.weights.iter().enumerate() {
            if *weight > self.weights[top] {
                top = at;
            }
        }
        (&self.states[top], self.weights[top])
    }

    #[must_use]
    pub fn entropy(&self) -> f64 {
        -self
            .weights
            .iter()
            .filter(|weight| **weight > 0.0)
            .map(|weight| weight * weight.ln())
            .sum::<f64>()
    }

    #[must_use]
    pub fn question(&self, candidates: &Set) -> Option<usize> {
        candidates.iter().min_by(|left, right| {
            (self.marginal(*left) - 0.5)
                .abs()
                .total_cmp(&(self.marginal(*right) - 0.5).abs())
        })
    }

    fn normalize(&mut self) {
        let total: f64 = self.weights.iter().sum();
        if total > 0.0 {
            for weight in &mut self.weights {
                *weight /= total;
            }
        }
    }
}
