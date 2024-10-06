pub(crate) trait Transition<T>
where
    T: Copy,
{
    fn transition(&self) -> Option<T>;
}

/// A simple Finite State Machine structure
pub(crate) struct HomieStateMachine<T>
where
    T: Transition<T> + Copy,
{
    state: Option<T>,
}

impl<T> HomieStateMachine<T>
where
    T: Transition<T> + Copy,
{
    /// Create a new FSM with the given initial state
    pub fn new(initial_state: T) -> Self {
        HomieStateMachine {
            state: Some(initial_state),
        }
    }

    /// Transition to the next state based on the current state
    pub fn transition(&mut self)
    where
        T: Transition<T>,
    {
        self.state = self.state.as_ref().and_then(|s| s.transition());
    }
}

impl<T> Iterator for HomieStateMachine<T>
where
    T: Transition<T> + Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let state = self.state; // get current state from FSM - this is so the initial value is
                                // also returned
        self.transition(); // transition to next state
        state // return the state
    }
}
