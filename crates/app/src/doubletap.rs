//! Recognising a double tap of Shift: two quick presses of Shift alone,
//! with no other key or modifier in between.

/// Longest a single tap may be held down.
const MAX_HOLD_MS: u64 = 300;
/// Longest gap between the first release and the second press.
const MAX_GAP_MS: u64 = 350;

#[derive(Clone, Copy, Debug)]
pub enum Input {
    ShiftDown(u64),
    ShiftUp(u64),
    /// Any other key, or Shift together with another modifier.
    Other,
}

#[derive(Default)]
pub struct DoubleTap {
    down_at: Option<u64>,
    last_up: Option<u64>,
}

impl DoubleTap {
    /// Feed one event (times in milliseconds); true when it completes a
    /// double tap.
    pub fn feed(&mut self, input: Input) -> bool {
        match input {
            Input::ShiftDown(t) => {
                if self.last_up.is_some_and(|up| t.saturating_sub(up) > MAX_GAP_MS) {
                    self.last_up = None;
                }
                self.down_at = Some(t);
                false
            }
            Input::ShiftUp(t) => {
                let Some(down) = self.down_at.take() else {
                    self.last_up = None;
                    return false;
                };
                if t.saturating_sub(down) > MAX_HOLD_MS {
                    self.last_up = None;
                    return false;
                }
                if self.last_up.take().is_some() {
                    return true;
                }
                self.last_up = Some(t);
                false
            }
            Input::Other => {
                *self = DoubleTap::default();
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Input::*;

    fn run(events: &[Input]) -> bool {
        let mut d = DoubleTap::default();
        events.iter().fold(false, |hit, e| d.feed(*e) || hit)
    }

    #[test]
    fn quick_double_tap() {
        assert!(run(&[ShiftDown(0), ShiftUp(80), ShiftDown(200), ShiftUp(270)]));
    }

    #[test]
    fn slow_gap_or_long_hold() {
        assert!(!run(&[ShiftDown(0), ShiftUp(80), ShiftDown(600), ShiftUp(650)]));
        assert!(!run(&[ShiftDown(0), ShiftUp(500), ShiftDown(600), ShiftUp(650)]));
    }

    #[test]
    fn typing_in_between_cancels() {
        assert!(!run(&[ShiftDown(0), ShiftUp(80), Other, ShiftDown(200), ShiftUp(270)]));
        // Shift held to type a capital letter.
        assert!(!run(&[ShiftDown(0), Other, ShiftUp(80), ShiftDown(200), ShiftUp(270)]));
    }

    #[test]
    fn triple_tap_fires_once() {
        let mut d = DoubleTap::default();
        let hits: Vec<bool> = [ShiftDown(0), ShiftUp(50), ShiftDown(150), ShiftUp(200), ShiftDown(300), ShiftUp(350)]
            .into_iter()
            .map(|e| d.feed(e))
            .collect();
        assert_eq!(hits.iter().filter(|h| **h).count(), 1);
    }
}
