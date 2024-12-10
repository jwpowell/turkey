/// Builder for constructing a DFA.
///
/// This builder allows the caller to provide arbitrary state identifiers. The built DFA is not
/// guaranteed to use the same identifiers and so does not expose them.
pub struct DfaBuilder<T>(std::marker::PhantomData<T>);

/// A guard for a DFA edge.
#[derive(Debug, Clone, Copy)]
pub enum DfaGuard {
    /// Matches any character.
    Any,

    /// Matches a specific character.
    Char(char),

    /// Matches a character in a range.
    Range(char, char),
}

impl<T> DfaBuilder<T>
where
    T: Clone,
{
    /// Creates a new DFA builder.
    pub fn new() -> Self {
        todo!()
    }

    /// Adds a state to the DFA.
    pub fn add_state(&mut self, state: u64) {
        todo!()
    }

    /// Adds an edge to the DFA.
    ///
    /// The edge goes from state `from` to state `to` and is triggered by the provided guard. The
    /// edge outputs the provided value.
    ///
    /// # Panics
    ///
    /// Panics if `from` or `to` are not valid state identifiers. Also panics if the provided
    /// guard overlaps with an existing edge.
    pub fn add_edge(&mut self, from: u64, to: u64, guard: DfaGuard, output: T) {
        todo!()
    }

    /// Sets the start state of the DFA.
    ///
    /// # Panics
    ///
    /// Panics if `state` is not a valid state identifier.
    pub fn set_start(&mut self, state: u64, start: bool) {
        todo!()
    }

    /// Sets the accept state of the DFA.
    ///    
    /// # Panics
    ///
    /// Panics if `state` is not a valid state identifier.
    pub fn set_accept(&mut self, state: u64, accept: bool) {
        todo!()
    }

    /// Builds the DFA and consumes the builder.
    pub fn build(self) -> Dfa<T> {
        todo!()
    }
}

/// A deterministic finite automaton (DFA).
///
/// Users produce a `Dfa` using the `DfaBuilder`. The `Dfa` deliberately hides its implementation
/// details, including identifiers for states, its transition mechanism, and how the edges are
/// represented internally. This allows the implementation to be optimized later without affecting
/// users.
pub struct Dfa<T> {
    /// The start state of the DFA.
    start: usize,

    /// The states of the DFA.
    states: Vec<DfaState<T>>,
}

/// A state of the DFA.
struct DfaState<T> {
    /// Indicates that this state is an accept state.
    accept: bool,

    /// The edges of the DFA.
    ///
    /// `edges` must always be sorted according to the guard using the following keys:
    ///
    /// - `DfaGuard::Char(c)`: `c`
    /// - `DfaGuard::Range(a, b)`: `a`
    /// - `DfaGuard::Any`: `'\0'`
    ///
    /// The `DfaGuard::Any` variant never appears in the list with other edges, therefore it need
    /// not be ordered. However, since we need a key, we can order it by the dummy value `'\0'`.
    edges: Vec<(DfaGuard, usize, T)>,
}

/// A runner for a DFA.
///
/// The runner is used to execute the DFA on an input string. The runner is deliberately separate
/// from the `Dfa` to allow for multiple runners to execute the same DFA concurrently without
/// requiring multiple copies of the DFA.
pub struct DfaRunner<'a, T> {
    dfa: &'a Dfa<T>,
    state: Option<usize>,
}

impl<'a, T> DfaRunner<'a, T>
where
    T: Clone,
{
    /// Creates a new runner for the provided DFA.
    pub fn new(dfa: &'a Dfa<T>) -> Self {
        DfaRunner {
            dfa,
            state: Some(dfa.start),
        }
    }

    /// Resets the runner to the start state.
    pub fn reset(&mut self) {
        self.state = Some(self.dfa.start);
    }

    /// Advances the runner by one character.
    ///
    /// Returns `Ok` if the runner successfully transitions to a new state. The value in `Ok` is the   
    /// output of the edge that was traversed. Returns `Err` if the runner has no edge for the given
    /// character.
    ///
    /// Once the runner returns an error, it will continue to return an error for all subsequent
    /// characters until `reset` is called.
    pub fn next(&mut self, c: char) -> Result<T, ()> {
        todo!()
    }

    /// Returns `true` if the runner is in an accept state and `false` otherwise, including if the
    /// runner is in an error state.
    pub fn is_accept(&self) -> bool {
        if let Some(state) = self.state {
            self.dfa.states[state].accept
        } else {
            false
        }
    }

    /// Returns `true` if the runner is in an error state and `false` otherwise.
    pub fn is_error(&self) -> bool {
        self.state.is_none()
    }
}
