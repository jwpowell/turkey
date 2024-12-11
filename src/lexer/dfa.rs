use std::collections::HashMap;
use std::hash::Hash;

/// Builder for constructing a DFA.
///
/// This builder allows the caller to provide arbitrary state identifiers. The built DFA is not
/// guaranteed to use the same identifiers and so does not expose them.
pub struct DfaBuilder<T> {
    dfa: Dfa<T>,
    state_map: HashMap<u64, usize>,
}

impl<T> DfaBuilder<T>
where
    T: Clone,
{
    /// Creates a new DFA builder.
    pub fn new() -> Self {
        DfaBuilder {
            dfa: Dfa {
                start: 0,
                states: Vec::new(),
            },
            state_map: HashMap::new(),
        }
    }

    /// Adds a state to the DFA.
    pub fn add_state(&mut self, state: u64) {
        let id = self.dfa.states.len();

        self.state_map.insert(state, id);

        self.dfa.states.push(DfaState {
            accept: false,
            edges: Vec::new(),
        });
    }

    pub fn with_state(mut self, state: u64) -> Self {
        self.add_state(state);
        self
    }

    pub fn with_states(mut self, states: &[u64]) -> Self {
        for &state in states {
            self.add_state(state);
        }

        self
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
    pub fn add_edge(&mut self, from: u64, to: u64, guard: (char, char), output: T) {
        let from = self.get_id(from);
        let to = self.get_id(to);

        let edges = &mut self.dfa.states[from].edges;

        for &(lo, hi, _, _) in edges.iter() {
            (lo..=hi).contains(&guard.0).then(|| {
                panic!("overlapping edge guards");
            });

            (lo..=hi).contains(&guard.1).then(|| {
                panic!("overlapping edge guards");
            });
        }

        edges.push((guard.0, guard.1, to, output));
        edges.sort_by_key(|&(lo, _, _, _)| lo);
    }

    pub fn with_edge(mut self, from: u64, to: u64, guard: (char, char), output: T) -> Self {
        self.add_edge(from, to, guard, output);
        self
    }

    fn get_id(&self, state: u64) -> usize {
        *self
            .state_map
            .get(&state)
            .expect("invalid state identifier")
    }

    /// Sets the start state of the DFA.
    ///
    /// # Panics
    ///
    /// Panics if `state` is not a valid state identifier.
    pub fn set_start(&mut self, state: u64) {
        self.dfa.start = self.get_id(state);
    }

    pub fn with_start(mut self, state: u64) -> Self {
        self.set_start(state);
        self
    }

    /// Sets the accept state of the DFA.
    ///    
    /// # Panics
    ///
    /// Panics if `state` is not a valid state identifier.
    pub fn set_accept(&mut self, state: u64, accept: bool) {
        let id = self.get_id(state);

        self.dfa.states[id].accept = accept;
    }

    pub fn with_accept(mut self, state: u64, accept: bool) -> Self {
        self.set_accept(state, accept);
        self
    }

    /// Builds the DFA and consumes the builder.
    pub fn build(self) -> Dfa<T> {
        if self.dfa.states.is_empty() {
            panic!("invalid DFA build: no states in DFA");
        }

        self.dfa
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
    /// `edges` must always be sorted according to the guard using the lower bound of the range.
    ///
    /// The `DfaGuard::Any` variant never appears in the list with other edges, therefore it need
    /// not be ordered. However, since we need a key, we can order it by the dummy value `'\0'`.
    edges: Vec<(char, char, usize, T)>,
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
        let state = self.state.ok_or(())?;
        let edges = &self.dfa.states[state].edges;

        for &(lo, hi, to, ref output) in edges.iter() {
            if lo <= c && c <= hi {
                self.state = Some(to);
                return Ok(output.clone());
            }
        }

        self.state = None;
        Err(())
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dfa_builder() {
        let mut builder = DfaBuilder::new();
        builder.add_state(1);
        builder.add_state(2);
        builder.add_edge(1, 2, ('a', 'a'), "a to b");
        builder.set_start(1);
        builder.set_accept(2, true);

        let dfa = builder.build();
        let mut runner = DfaRunner::new(&dfa);

        assert_eq!(runner.next('a'), Ok("a to b"));
        assert!(runner.is_accept());
    }

    #[test]
    #[should_panic(expected = "invalid state identifier")]
    fn test_invalid_state() {
        let mut builder: DfaBuilder<()> = DfaBuilder::new();
        builder.add_state(1);
        builder.set_start(2);
    }

    #[test]
    fn test_complex_dfa() {
        let mut builder = DfaBuilder::new();
        builder.add_state(1);
        builder.add_state(2);
        builder.add_state(3);
        builder.add_state(4);
        builder.add_edge(1, 2, ('a', 'a'), "a to b");
        builder.add_edge(2, 3, ('b', 'b'), "b to c");
        builder.add_edge(3, 4, ('c', 'c'), "c to d");
        builder.add_edge(4, 1, ('d', 'd'), "d to a");
        builder.set_start(1);
        builder.set_accept(4, true);

        let dfa = builder.build();
        let mut runner = DfaRunner::new(&dfa);

        assert_eq!(runner.next('a'), Ok("a to b"));
        assert!(!runner.is_accept());
        assert_eq!(runner.next('b'), Ok("b to c"));
        assert!(!runner.is_accept());
        assert_eq!(runner.next('c'), Ok("c to d"));
        assert!(runner.is_accept());
        assert_eq!(runner.next('d'), Ok("d to a"));
        assert!(!runner.is_accept());
        assert_eq!(runner.next('a'), Ok("a to b"));
        assert!(!runner.is_accept());
    }

    #[test]
    fn test_dfa_reset() {
        let mut builder = DfaBuilder::new();
        builder.add_state(1);
        builder.add_state(2);
        builder.add_edge(1, 2, ('a', 'a'), "a to b");
        builder.set_start(1);
        builder.set_accept(2, true);

        let dfa = builder.build();
        let mut runner = DfaRunner::new(&dfa);

        assert_eq!(runner.next('a'), Ok("a to b"));
        assert!(runner.is_accept());

        runner.reset();
        assert_eq!(runner.next('a'), Ok("a to b"));
        assert!(runner.is_accept());
    }

    #[test]
    fn test_dfa_error_state() {
        let mut builder = DfaBuilder::new();
        builder.add_state(1);
        builder.add_state(2);
        builder.add_edge(1, 2, ('a', 'a'), "a to b");
        builder.set_start(1);
        builder.set_accept(2, true);

        let dfa = builder.build();
        let mut runner = DfaRunner::new(&dfa);

        assert_eq!(runner.next('b'), Err(()));
        assert!(runner.is_error());
    }
}
