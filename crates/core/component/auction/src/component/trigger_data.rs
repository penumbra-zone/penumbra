use anyhow::Result;

// A device that captures immutable trigger data description, and provides
// safe helpers to compute the next trigger height, or the current step index.
pub struct TriggerData {
    pub start_height: u64,
    pub end_height: u64,
    pub step_count: u64,
}

impl TriggerData {
    /// Compute the step index of the supplied height.
    ///
    /// # Errors
    /// This method errors if the block interval is not a multiple of the
    /// specified `step_count`, or if it operates over an invalid block
    /// interval (which should NEVER happen unless validation is broken).
    pub fn compute_step_index(&self, current_height: u64) -> Result<u64> {
        let TriggerData {
            start_height,
            end_height,
            step_count,
        } = self;

        let block_interval = end_height.checked_sub(*start_height).ok_or_else(|| {
            anyhow::anyhow!(
                "block interval calculation has underflowed (end={}, start={})",
                self.end_height,
                self.start_height
            )
        })?;

        // Compute the step size, based on the block interval and the number of
        // discrete steps the auction specifies.
        let step_size = block_interval
            .checked_div(*step_count)
            .ok_or_else(|| anyhow::anyhow!("step count is zero"))?;

        // Compute the step index for the current height, this should work even if
        // the supplied height does not fall perfectly on a step boundary. First, we
        // "clamp it" to a previous step index, then we increment by 1 to compute the
        // next one, and finally we determine a concrete trigger height based off that.
        let distance_from_start = current_height.saturating_sub(*start_height);

        distance_from_start
            .checked_div(step_size)
            .ok_or_else(|| anyhow::anyhow!("step size is zero"))
    }

    /// Compute the next trigger height.
    pub fn compute_next_trigger_height(&self, current_height: u64) -> u64 {
        let TriggerData {
            start_height,
            end_height,
            step_count,
        } = self;

        let block_interval = end_height - *start_height;

        // Compute the step size, based on the block interval and the number of
        // discrete steps the auction specifies.
        let step_size = block_interval / *step_count;

        // Compute the step index for the current height, this should work even if
        // the supplied height does not fall perfectly on a step boundary. First, we
        // "clamp it" to a previous step index, then we increment by 1 to compute the
        // next one, and finally we determine a concrete trigger height based off that.
        let distance_from_start = current_height.saturating_sub(*start_height);
        let prev_step_index = distance_from_start / step_size;
        let next_step_index = prev_step_index + 1;

        let next_step_size_from_start = step_size * next_step_index;
        start_height + next_step_size_from_start
    }

    /// Return the next trigger height if it is within the auction interval, and `None` otherwise.
    pub fn try_next_trigger_height(&self, current_height: u64) -> Option<u64> {
        let next_trigger_height = self.compute_next_trigger_height(current_height);
        if next_trigger_height > self.end_height {
            None
        } else {
            Some(next_trigger_height)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::component::trigger_data::TriggerData;

    #[test]
    fn test_current_height_equals_start_height() {
        let trigger = TriggerData {
            start_height: 100,
            end_height: 200,
            step_count: 5,
        };
        let current_height = 100;
        assert_eq!(trigger.try_next_trigger_height(current_height), Some(120));
    }

    #[test]
    fn test_current_height_equals_end_height() {
        let data = TriggerData {
            start_height: 100,
            end_height: 200,
            step_count: 5,
        };
        let current_height = 200;
        assert_eq!(data.try_next_trigger_height(current_height), None);
    }

    #[test]
    fn test_current_height_below_start_height() {
        let data = TriggerData {
            start_height: 100,
            end_height: 200,
            step_count: 5,
        };
        let current_height = 90;
        assert_eq!(data.try_next_trigger_height(current_height), Some(120));
    }

    #[test]
    fn test_current_height_above_end_height() {
        let data = TriggerData {
            start_height: 100,
            end_height: 200,
            step_count: 5,
        };
        let current_height = 210;
        assert_eq!(data.try_next_trigger_height(current_height), None);
    }

    #[test]
    fn test_current_height_below_boundary() {
        let data = TriggerData {
            start_height: 100,
            end_height: 200,
            step_count: 5,
        };
        let current_height = 119;
        assert_eq!(data.try_next_trigger_height(current_height), Some(120));
    }

    #[test]
    fn test_current_height_above_boundary() {
        let data = TriggerData {
            start_height: 100,
            end_height: 200,
            step_count: 5,
        };
        let current_height = 121;
        assert_eq!(data.try_next_trigger_height(current_height), Some(140));
    }
}
