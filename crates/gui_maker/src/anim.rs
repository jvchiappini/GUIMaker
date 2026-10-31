//! Small per-widget value/bool animations with easing.

#[derive(Clone, Copy, Debug, Default)]
pub struct AnimState {
    pub value: f32,
    pub target: f32,
}

impl AnimState {
    pub fn new(target: f32) -> Self {
        Self { value: target, target }
    }

    pub fn advance(&mut self, target: f32, duration: f32, dt: f32) -> f32 {
        self.target = target;
        if duration <= 0.0 || dt <= 0.0 {
            self.value = target;
            return self.value;
        }
        let factor = (dt / duration).clamp(0.0, 1.0);
        self.value += (target - self.value) * factor;
        if (target - self.value).abs() < 1e-3 {
            self.value = target;
        }
        self.value
    }

    pub fn settled(&self) -> bool {
        (self.target - self.value).abs() < 1e-3
    }
}

pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

pub fn ease_in_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_reaches_target_and_settles() {
        let mut state = AnimState::new(0.0);
        let mut value = 0.0;
        for _ in 0..200 {
            value = state.advance(1.0, 0.15, 1.0 / 60.0);
        }
        assert_eq!(value, 1.0);
        assert!(state.settled());
    }

    #[test]
    fn zero_duration_snaps() {
        let mut state = AnimState::new(0.0);
        assert_eq!(state.advance(1.0, 0.0, 0.016), 1.0);
    }
}
