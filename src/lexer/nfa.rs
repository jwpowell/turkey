use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;

pub struct NfaBuilder<T> {
    nfa: Nfa<T>,
}

impl<T> NfaBuilder<T>
where
    T: Clone,
{
    pub fn new(cmp: fn(&T, &T) -> std::cmp::Ordering) -> Self {
        NfaBuilder { nfa: Nfa::new(cmp) }
    }

    pub fn node(&mut self) -> usize {
        self.nfa.node()
    }

    pub fn edge(&mut self, from: usize, to: usize, lo: char, hi: char, output: T) {
        self.nfa.edge(from, to, lo, hi, output);
    }

    pub fn epsilon(&mut self, from: usize, to: usize) {
        self.nfa.epsilon(from, to);
    }

    pub fn start(&mut self, state: usize) {
        self.nfa.set_start(state);
    }

    pub fn accept(&mut self, state: usize) {
        self.nfa.set_accept(state);
    }

    pub fn build(mut self) -> Nfa<T> {
        self.nfa.optimize();

        self.nfa
    }

    pub fn merge(&mut self, other: &Self) -> (usize, usize, usize, usize) {
        let self_start = self.node();
        let self_accept = self.node();
        let other_start = self.node();
        let other_accept = self.node();

        let mut map = HashMap::new();

        for i in 0..other.nfa.node_count() {
            map.insert(i, self.node());
        }

        for from in 0..other.nfa.node_count() {
            for &(lo, hi, to, ref output) in &other.nfa.edges[from] {
                self.edge(map[&from], map[&to], lo, hi, output.clone());
            }

            for &to in &other.nfa.epsilons[from] {
                self.epsilon(map[&from], map[&to]);
            }
        }

        for i in 0..self.nfa.start.len() {
            self.epsilon(self_start, self.nfa.start[i]);
        }

        for i in 0..self.nfa.accept.len() {
            self.epsilon(self.nfa.accept[i], self_accept);
        }

        for &s in &other.nfa.start {
            self.epsilon(other_start, map[&s]);
        }

        for &s in &other.nfa.accept {
            self.epsilon(map[&s], other_accept);
        }

        (self_start, self_accept, other_start, other_accept)
    }
}

pub struct Nfa<T> {
    start: Vec<usize>,
    accept: Vec<usize>,
    edges: Vec<Vec<(char, char, usize, T)>>,
    epsilons: Vec<Vec<usize>>,
    cmp: fn(&T, &T) -> std::cmp::Ordering,
}

impl<T> Nfa<T>
where
    T: Clone + Ord,
{
    fn with_natural_ordering() -> Self {
        Nfa {
            start: Vec::new(),
            accept: Vec::new(),
            edges: Vec::new(),
            epsilons: Vec::new(),
            cmp: <T as Ord>::cmp,
        }
    }
}

impl<T> Nfa<T>
where
    T: Clone,
{
    fn new(cmp: fn(&T, &T) -> std::cmp::Ordering) -> Self {
        Nfa {
            start: Vec::new(),
            accept: Vec::new(),
            edges: Vec::new(),
            epsilons: Vec::new(),
            cmp,
        }
    }

    fn node(&mut self) -> usize {
        let state = self.edges.len();
        self.ensure_node(state);

        state
    }

    fn node_count(&self) -> usize {
        self.edges.len()
    }

    fn ensure_node(&mut self, state: usize) {
        if state < self.edges.len() {
            return;
        }

        self.edges.resize_with(state + 1, Vec::new);
        self.epsilons.resize_with(state + 1, Vec::new);
    }

    fn ensure_nodes(&mut self, states: &[usize]) {
        self.ensure_node(states.iter().max().cloned().unwrap_or(0));
    }

    fn edge(&mut self, from: usize, to: usize, lo: char, hi: char, output: T) {
        if lo > hi {
            return;
        }

        let overlaps: fn(char, char, char, char) -> bool =
            |lo1, hi1, lo2, hi2| (lo1 <= lo2 && lo2 <= hi1) || (lo1 <= hi2 && hi2 <= hi1);

        self.ensure_nodes(&[from, to]);

        let mut need_epsilon = false;
        for &(l, h, _, _) in self.edges[from].iter() {
            if overlaps(lo, hi, l, h) {
                need_epsilon = true;
            }
        }

        if need_epsilon {
            let e = self.node();
            self.epsilon(from, e);
            self.edges[e].push((lo, hi, to, output));
        } else {
            self.edges[from].push((lo, hi, to, output));
        }
    }

    fn epsilon(&mut self, from: usize, to: usize) {
        self.ensure_nodes(&[from, to]);
        self.epsilons[from].push(to);
    }

    fn set_start(&mut self, state: usize) {
        self.start.push(state);
    }

    fn set_accept(&mut self, state: usize) {
        self.accept.push(state);
    }

    fn remove_node(&mut self, deleted: usize) {
        let old_id = self.edges.len() - 1;
        let new_id = deleted;

        self.edges.swap_remove(deleted);
        self.epsilons.swap_remove(deleted);

        for edges in &mut self.edges {
            edges.retain(|&(_, _, to, _)| to != deleted);
        }

        for epsilons in &mut self.epsilons {
            epsilons.retain(|&to| to != deleted);
        }

        self.start.retain(|&s| s != deleted);
        self.accept.retain(|&s| s != deleted);

        for edges in &mut self.edges {
            for (_, _, to, _) in edges.iter_mut() {
                if *to == old_id {
                    *to = new_id;
                }
            }
        }

        for epsilons in &mut self.epsilons {
            for to in epsilons.iter_mut() {
                if *to == old_id {
                    *to = new_id;
                }
            }
        }

        for s in &mut self.start {
            if *s == old_id {
                *s = new_id;
            }
        }

        for s in &mut self.accept {
            if *s == old_id {
                *s = new_id;
            }
        }
    }

    fn optimize_deduplicate(&mut self) {
        self.start.sort_unstable();
        self.start.dedup();

        self.accept.sort_unstable();
        self.accept.dedup();
    }

    fn optimize_epsilon_closure(&mut self) {
        let mut visited = vec![false; self.edges.len()];
        let mut stack = vec![];
        let mut closure = vec![];

        for i in 0..self.edges.len() {
            stack.clear();
            closure.clear();

            while let Some(s) = stack.pop() {
                if visited[s] {
                    continue;
                }

                visited[s] = true;

                for (j, &e) in self.epsilons[i].iter().enumerate() {
                    if i < j {
                        // all nodes below i are closed.
                        // only add to stack if it's not yet closed.
                        stack.push(e);
                    }

                    closure.push(e);
                }
            }

            self.epsilons[i].extend(closure.drain(..));
            self.epsilons[i].sort_unstable();
            self.epsilons[i].dedup();
        }
    }

    fn is_reachable(&self, from: usize, to: usize) -> bool {
        let mut visited = vec![false; self.edges.len()];
        let mut stack = vec![from];
        let mut more = true;

        while more {
            more = false;

            while let Some(s) = stack.pop() {
                if visited[s] {
                    continue;
                }

                visited[s] = true;

                for &(_, _, to, _) in &self.edges[s] {
                    more = true;
                    stack.push(to);
                }

                for &e in &self.epsilons[s] {
                    more = true;
                    stack.push(e);
                }
            }
        }

        visited[to]
    }

    fn optimize_remove_unreachable(&mut self) {
        let mut deleted = vec![];
        let mut more = true;

        for i in 0..self.node_count() {
            if !self.accept.iter().any(|&a| self.is_reachable(i, a)) {
                deleted.push(i);
            }
        }

        deleted.sort_unstable();
        deleted.dedup();

        for e in deleted.iter().rev() {
            self.remove_node(*e);
        }
    }

    fn optimize_remove_dead_nodes(&mut self) {
        let mut deleted = vec![];

        for i in 0..self.edges.len() {
            let dead = self.edges[i].is_empty()
                && self.epsilons[i].is_empty()
                && !self.accept.contains(&i);

            if dead {
                deleted.push(i);
            }
        }

        deleted.sort_unstable();
        deleted.dedup();

        for e in deleted.iter().rev() {
            self.remove_node(*e);
        }
    }

    fn optimize_start(&mut self) {
        let mut xs = vec![];

        for &s in &self.start {
            xs.extend(self.epsilons[s].iter().copied());
        }

        self.start.extend(xs);

        self.start.sort_unstable();
        self.start.dedup();
    }

    fn optimize(&mut self) {
        self.optimize_deduplicate();
        self.optimize_epsilon_closure();
        self.optimize_remove_unreachable();
        self.optimize_remove_dead_nodes();
        self.optimize_start();
    }

    fn write_dot<W: std::io::Write>(&self, io: &mut W) -> std::io::Result<()>
    where
        T: fmt::Debug,
    {
        writeln!(io, "digraph NFA {{")?;
        writeln!(io, "  start [shape=point];")?;

        for (i, edges) in self.edges.iter().enumerate() {
            writeln!(io, "  {} [shape=circle, label=\"{}\"];", i, i)?;

            for &(lo, hi, to, ref out) in edges {
                writeln!(
                    io,
                    "  {} -> {} [label=\"{}-{}: {:?}\"];",
                    i, to, lo, hi, out
                )?;
            }

            for &to in &self.epsilons[i] {
                writeln!(io, "  {} -> {} [label=\"ε\"];", i, to)?;
            }
        }

        for &s in &self.start {
            writeln!(io, "  start -> {};", s)?;
        }

        for &s in &self.accept {
            writeln!(io, "  {} [shape=doublecircle];", s)?;
        }

        writeln!(io, "}}")
    }

    fn print_dot(&self)
    where
        T: fmt::Debug,
    {
        self.write_dot(&mut std::io::stdout()).unwrap();
    }

    fn write_dot_to_file(&self, path: &str)
    where
        T: fmt::Debug,
    {
        let mut file = std::fs::File::create(path).unwrap();
        self.write_dot(&mut file).unwrap();
    }
}

pub struct NfaRunner<'a, T> {
    nfa: &'a Nfa<T>,
    states: Vec<usize>,
}

impl<'a, T> NfaRunner<'a, T>
where
    T: Clone,
{
    pub fn new(nfa: &'a Nfa<T>) -> Self {
        NfaRunner {
            nfa,
            states: nfa.start.clone(),
        }
    }

    pub fn reset(&mut self) {
        self.states.clear();
        self.states.extend_from_slice(&self.nfa.start);
    }

    pub fn step(&mut self, c: char) -> Option<T> {
        let mut next = vec![];
        let mut output = None;

        for &s in &self.states {
            for &(lo, hi, to, ref out) in &self.nfa.edges[s] {
                if lo <= c && c <= hi {
                    next.push(to);
                    next.extend_from_slice(&self.nfa.epsilons[to]);

                    if let Some(ref mut old_out) = output {
                        if (self.nfa.cmp)(out, old_out) == std::cmp::Ordering::Greater {
                            *old_out = out.clone();
                        }
                    } else {
                        output = Some(out.clone());
                    }
                }
            }
        }

        self.states.clear();
        self.states.extend(next);
        self.states.sort_unstable();
        self.states.dedup();

        output
    }

    pub fn is_accept(&self) -> bool {
        self.states.iter().any(|&s| self.nfa.accept.contains(&s))
    }

    pub fn is_dead(&self) -> bool {
        self.states.is_empty() || self.nfa.edges.iter().all(|edges| edges.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Eq)]
    struct Output(char);

    impl fmt::Debug for Output {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    fn cmp(a: &Output, b: &Output) -> std::cmp::Ordering {
        a.0.cmp(&b.0)
    }

    #[test]
    fn test_nfa_builder() {
        let mut builder = NfaBuilder::new(cmp);
        let start = builder.node();
        let accept = builder.node();
        builder.edge(start, accept, 'a', 'z', Output('a'));
        builder.start(start);
        builder.accept(accept);
        let nfa = builder.build();

        assert_eq!(nfa.start, vec![start]);
        assert_eq!(nfa.accept, vec![accept]);
        assert_eq!(nfa.edges.len(), 2);
        assert_eq!(nfa.edges[start], vec![('a', 'z', accept, Output('a'))]);
    }

    #[test]
    fn test_nfa_runner() {
        let mut builder = NfaBuilder::new(cmp);
        let start = builder.node();
        let accept = builder.node();
        builder.edge(start, accept, 'a', 'z', Output('a'));
        builder.start(start);
        builder.accept(accept);
        let nfa = builder.build();

        let mut runner = NfaRunner::new(&nfa);
        assert!(!runner.is_accept());
        assert_eq!(runner.step('a'), Some(Output('a')));
        assert!(runner.is_accept());
        assert_eq!(runner.step('b'), None);
        assert!(!runner.is_accept());
    }

    #[test]
    fn test_nfa_epsilon() {
        let mut builder = NfaBuilder::new(cmp);
        let start = builder.node();
        let middle = builder.node();
        let accept = builder.node();
        builder.epsilon(start, middle);
        builder.edge(middle, accept, 'a', 'z', Output('a'));
        builder.start(start);
        builder.accept(accept);
        let nfa = builder.build();

        let mut runner = NfaRunner::new(&nfa);
        assert!(!runner.is_accept());
        assert_eq!(runner.step('a'), Some(Output('a')));
        assert!(runner.is_accept());
    }

    #[test]
    fn test_nfa_multiple_edges() {
        let mut builder = NfaBuilder::new(cmp);
        let start = builder.node();
        let accept1 = builder.node();
        let accept2 = builder.node();
        builder.edge(start, accept1, 'a', 'm', Output('a'));
        builder.edge(start, accept2, 'n', 'z', Output('n'));
        builder.start(start);
        builder.accept(accept1);
        builder.accept(accept2);

        let nfa = builder.build();

        let mut runner = NfaRunner::new(&nfa);
        assert!(!runner.is_accept());
        assert_eq!(runner.step('a'), Some(Output('a')));
        assert!(runner.is_accept());
        runner.reset();
        assert_eq!(runner.step('n'), Some(Output('n')));
        assert!(runner.is_accept());
    }

    #[test]
    fn test_nfa_dead_state() {
        let mut builder = NfaBuilder::new(cmp);
        let start = builder.node();
        let dead = builder.node();
        builder.edge(start, dead, 'a', 'a', Output('a'));
        builder.edge(dead, start, 'z', 'z', Output('z'));
        builder.start(start);

        let nfa = builder.build();

        let mut runner = NfaRunner::new(&nfa);
        assert!(!runner.is_accept());
        assert_eq!(runner.step('a'), None);
        assert!(runner.is_dead());
        assert_eq!(runner.step('b'), None);
        assert!(runner.is_dead());
    }

    #[test]
    fn test_nfa_complex() {
        let mut builder = NfaBuilder::new(cmp);
        let start = builder.node();
        let middle1 = builder.node();
        let middle2 = builder.node();
        let accept1 = builder.node();
        let accept2 = builder.node();
        let accept3 = builder.node();

        builder.edge(start, middle1, 'a', 'c', Output('a'));
        builder.edge(middle1, middle2, 'd', 'f', Output('d'));
        builder.edge(middle2, accept1, 'g', 'i', Output('g'));
        builder.edge(middle2, accept2, 'j', 'l', Output('j'));
        builder.edge(start, accept3, 'm', 'o', Output('m'));

        let dead1 = builder.node();
        let dead2 = builder.node();
        let dead3 = builder.node();

        builder.edge(dead1, dead2, 'a', 'z', Output('z'));
        builder.edge(accept3, dead3, 'p', 'z', Output('z'));

        builder.epsilon(middle1, accept3);
        builder.start(start);
        builder.accept(accept1);
        builder.accept(accept2);
        builder.accept(accept3);

        let nfa = builder.build();

        let mut runner = NfaRunner::new(&nfa);
        assert!(!runner.is_accept());
        assert_eq!(runner.step('a'), Some(Output('a')));
        assert!(runner.is_accept());
        assert_eq!(runner.step('d'), Some(Output('d')));
        assert!(!runner.is_accept());
        assert_eq!(runner.step('g'), Some(Output('g')));
        assert!(runner.is_accept());

        runner.reset();
        assert_eq!(runner.step('a'), Some(Output('a')));
        assert!(runner.is_accept());
        assert_eq!(runner.step('d'), Some(Output('d')));
        assert!(!runner.is_accept());
        assert_eq!(runner.step('j'), Some(Output('j')));
        assert!(runner.is_accept());

        runner.reset();
        assert_eq!(runner.step('m'), Some(Output('m')));
        assert!(runner.is_accept());
    }

    #[test]
    fn test_complex_nfa_optimization() {
        let mut builder = NfaBuilder::new(cmp);

        let start = builder.node();

        let mut fully_connected = vec![];

        for _ in 0..5 {
            fully_connected.push(builder.node());
        }

        let mut iter = ('a'..='z').chain('A'..='Z').chain('0'..='9');

        for &i in &fully_connected {
            for &j in &fully_connected {
                let c = iter.next().unwrap();
                builder.edge(i, j, c, c, Output('a'));
            }
        }

        //assert_eq!(builder.nfa.node_count(), 33);

        let middle = builder.node();

        builder.edge(
            start,
            *fully_connected.first().unwrap(),
            'a',
            'z',
            Output('a'),
        );

        builder.edge(
            *fully_connected.last().unwrap(),
            middle,
            'a',
            'z',
            Output('a'),
        );

        fully_connected.clear();

        for _ in 0..5 {
            fully_connected.push(builder.node());
        }

        for &i in &fully_connected {
            for &j in &fully_connected {
                let c = iter.next().unwrap();
                builder.edge(i, j, c, c, Output('a'));
            }
        }

        let last = builder.node();

        builder.edge(
            middle,
            *fully_connected.first().unwrap(),
            'a',
            'z',
            Output('a'),
        );

        builder.edge(
            *fully_connected.last().unwrap(),
            last,
            'a',
            'z',
            Output('a'),
        );

        builder.start(start);
        builder.accept(last);

        //builder.nfa.write_dot_to_file("a.dot");
        let nfa = builder.build();
        //nfa.write_dot_to_file("b.dot");
    }
}
