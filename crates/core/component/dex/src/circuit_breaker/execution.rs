const MAX_PATH_SEARCHES: u32 = 64;
const MAX_EXECUTIONS: u32 = 64;

/// Holds the state of the execution circuit breaker.
/// Responsible for managing the conditions of halting execution of
/// a single batch swap. All execution circuit breaker triggers are
/// non-fatal and will allow the swap to be partially fulfilled up
/// to the search and execution limits managed by the circuit breaker.
///
/// The circuit breaker ensures the swap will not use unbounded time complexity.
struct ExecutionCircuitBreaker {
    /// The maximum number of times to perform path searches before stopping.
    pub max_path_searches: u32,
    /// The number of times path searches have been performed.
    pub current_path_searches: u32,
    /// The maximum number of times to execute against liquidity positions before stopping.
    pub max_executions: u32,
    /// The number of times liquidity positions have been executed against.
    pub current_executions: u32,
}

impl ExecutionCircuitBreaker {
    pub fn new() -> Self {
        Self {
            max_path_searches: MAX_PATH_SEARCHES,
            current_path_searches: 0,
            max_executions: MAX_EXECUTIONS,
            current_executions: 0,
        }
    }

    pub fn exceeded_limits(&self) -> bool {
        self.current_path_searches > self.max_path_searches
            || self.current_executions > self.max_executions
    }
}
