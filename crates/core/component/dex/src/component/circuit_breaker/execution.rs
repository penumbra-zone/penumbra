/// The execution circuit breaker meters the number of operations (search and execution)
/// performed to fulfill a batch swap.
///
/// The Dex component MUST call `CircuitBreaker::exceed_limits` before an execution round.
///
/// Note that in practice, this means that a batch swap can result in a partial fill
/// even if there were enough liquidity to fulfill all of it.
#[derive(Debug, Clone)]
pub(crate) struct ExecutionCircuitBreaker {
    /// The current number of operations performed.
    pub counter: u32,
    /// The maximum number of operations allowed.
    pub max: u32,
}

impl ExecutionCircuitBreaker {
    pub fn new(max: u32) -> Self {
        Self { max, counter: 0 }
    }

    pub fn increment(&mut self) {
        self.counter += 1;
    }

    pub fn exceeded_limits(&self) -> bool {
        self.counter >= self.max
    }
}
