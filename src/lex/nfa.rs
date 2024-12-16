use std::collections::HashMap;

pub struct Nfa {
    start: Vec<usize>,
    accept: Vec<usize>,
    nodes: Vec<NfaNode>,

    optimized: bool,

    current: Vec<usize>,
    next: Vec<usize>,
}

#[derive(Debug, Default, Clone)]
pub struct NfaNode {
    edges: Vec<(char, char, usize)>,
    epsilons: Vec<usize>,
}

impl Nfa {
    pub fn new() -> Nfa {
        Nfa {
            start: vec![],
            accept: vec![],
            nodes: vec![],
            optimized: true,
            current: vec![],
            next: vec![],
        }
    }

    pub fn add_start(&mut self, start: usize) {
        self.optimized = false;
        self.start.push(start);
    }

    pub fn add_accept(&mut self, accept: usize) {
        self.optimized = false;
        self.accept.push(accept);
    }

    pub fn create_node(&mut self) -> usize {
        self.optimized = false;
        let index = self.nodes.len();

        self.nodes.push(NfaNode::default());

        index
    }

    pub fn add_edge(&mut self, from: usize, lo: char, hi: char, to: usize) {
        self.optimized = false;

        let overlaps = self.nodes[from]
            .edges
            .iter()
            .any(|&(l, h, _)| (l..h).contains(&lo) || (l..h).contains(&hi));

        if overlaps {
            let intermediate = self.create_node();
            self.add_epsilon(from, intermediate);
            self.add_edge(intermediate, lo, hi, to);
        } else {
            self.nodes[from].edges.push((lo, hi, to));
        }
    }

    pub fn add_epsilon(&mut self, from: usize, to: usize) {
        self.optimized = false;
        self.nodes[from].epsilons.push(to);
    }

    pub fn merge(&mut self, other: &Nfa) -> (usize, usize) {
        self.optimized = false;

        let mut map = HashMap::new();

        for &index in other.start.iter() {
            map.entry(index).or_insert_with(|| self.create_node());
        }

        for &index in other.accept.iter() {
            map.entry(index).or_insert_with(|| self.create_node());
        }

        for index in 0..other.nodes.len() {
            map.entry(index).or_insert_with(|| self.create_node());
        }

        for from in 0..other.nodes.len() {
            for &(lo, hi, to) in other.nodes[from].edges.iter() {
                let from = *map.get(&from).unwrap();
                let to = *map.get(&to).unwrap();
                self.add_edge(from, lo, hi, to);
            }
        }

        for from in 0..other.nodes.len() {
            for &to in other.nodes[from].epsilons.iter() {
                let from = *map.get(&from).unwrap();
                let to = *map.get(&to).unwrap();
                self.add_epsilon(from, to);
            }
        }

        let start = self.create_node();
        let accept = self.create_node();

        for &from in other.start.iter() {
            self.add_epsilon(start, *map.get(&from).unwrap());
        }

        for &to in other.accept.iter() {
            self.add_epsilon(*map.get(&to).unwrap(), accept);
        }

        (start, accept)
    }

    pub fn reset(&mut self) {
        if !self.optimized {
            self.optimize();
        }

        self.current.clear();
        self.current.extend(self.start.iter());
    }

    fn optimize(&mut self) {
        if self.optimized {
            return;
        }

        self.epsilon_closure();
        self.remove_unreachable_nodes();
        self.remove_dead_nodes();

        self.optimized = true;
    }

    fn epsilon_closure(&mut self) {
        let mut eps = vec![];
        let mut stack = vec![];
        let mut visited = vec![false; self.nodes.len()];

        for a in 0..self.nodes.len() {
            visited.fill(false);
            stack.clear();
            stack.extend(self.nodes[a].epsilons.iter().copied());
            eps.clear();

            while let Some(b) = stack.pop() {
                if visited[b] {
                    continue;
                }

                visited[b] = true;
                eps.push(b);

                // Assume that all nodes less than `a` are already epsilon closed. So, just add
                // their epsilons to `eps`, but don't push them onto the stack.
                if b < a {
                    eps.extend(self.nodes[b].epsilons.iter().copied());
                } else {
                    stack.extend(self.nodes[b].epsilons.iter().copied());
                }
            }

            eps.sort_unstable();
            eps.dedup();

            self.nodes[a].epsilons.clear();
            self.nodes[a].epsilons.extend(eps.iter().copied());
        }

        eps.clear();

        for &node in self.start.iter() {
            eps.extend(self.nodes[node].epsilons.iter().copied());
        }

        self.start.extend(eps.iter().copied());
        self.start.sort_unstable();
        self.start.dedup();
    }

    pub fn remove_unreachable_nodes(&mut self) {
        let mut visited = vec![false; self.nodes.len()];
        let mut stack = self.start.clone();

        while let Some(node) = stack.pop() {
            if visited[node] {
                continue;
            }

            visited[node] = true;

            for &(_, _, to) in self.nodes[node].edges.iter() {
                stack.push(to);
            }

            for &to in self.nodes[node].epsilons.iter() {
                stack.push(to);
            }
        }

        for node in 0..self.nodes.len() {
            if !visited[node] {
                self.remove_node(node);
            }
        }
    }

    fn remove_dead_nodes(&mut self) {
        let mut stack = self.accept.clone();
        let mut visited = vec![false; self.nodes.len()];

        while let Some(node) = stack.pop() {
            if visited[node] {
                continue;
            }

            visited[node] = true;

            for from in 0..self.nodes.len() {
                for &(_, _, to) in self.nodes[from].edges.iter() {
                    if to == node {
                        stack.push(from);
                    }
                }

                for &to in self.nodes[from].epsilons.iter() {
                    if to == node {
                        stack.push(from);
                    }
                }
            }
        }

        for node in 0..self.nodes.len() {
            if !visited[node] {
                self.remove_node(node);
            }
        }
    }

    pub fn remove_node(&mut self, deleted: usize) {
        self.optimized = false;

        let old_index = self.nodes.len() - 1;
        let new_index = deleted;

        self.nodes.swap_remove(deleted);

        for from in 0..self.nodes.len() {
            self.nodes[from].edges.retain(|&(_, _, to)| to != deleted);
            self.nodes[from].epsilons.retain(|&x| x != deleted);
        }

        self.start.retain(|&x| x != deleted);
        self.accept.retain(|&x| x != deleted);

        for node in self.start.iter_mut() {
            if *node == old_index {
                *node = new_index;
            }
        }

        for node in self.accept.iter_mut() {
            if *node == old_index {
                *node = new_index;
            }
        }

        for from in 0..self.nodes.len() {
            for (_, _, to) in self.nodes[from].edges.iter_mut() {
                if *to == old_index {
                    *to = new_index;
                }
            }

            for to in self.nodes[from].epsilons.iter_mut() {
                if *to == old_index {
                    *to = new_index;
                }
            }
        }
    }

    pub fn put(&mut self, c: char) {
        assert!(self.optimized, "must be optimized before simulating");

        self.next.clear();

        for &from in self.current.iter() {
            for &(c1, c2, to) in self.nodes[from].edges.iter() {
                if c1 <= c && c <= c2 {
                    self.next.push(to);

                    for &e in self.nodes[to].epsilons.iter() {
                        self.next.push(e);
                    }
                }
            }
        }

        std::mem::swap(&mut self.current, &mut self.next);
    }

    pub fn is_dead(&self) -> bool {
        assert!(self.optimized, "must be optimized before simulating");

        self.current.is_empty()
    }

    pub fn is_accept(&self) -> bool {
        assert!(self.optimized, "must be optimized before simulating");

        self.current.iter().any(|&from| self.accept.contains(&from))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn build_nfa(
        start: usize,
        accept: usize,
        edges: &[(usize, char, char, usize)],
        epsilons: &[(usize, usize)],
    ) -> Nfa {
        let mut nfa = Nfa::new();
        let mut map = HashMap::new();

        for (from, lo, hi, to) in edges {
            let from = *map.entry(*from).or_insert_with(|| nfa.create_node());
            let to = *map.entry(*to).or_insert_with(|| nfa.create_node());
            nfa.add_edge(from, *lo, *hi, to);
        }

        for (from, to) in epsilons {
            let from = *map.entry(*from).or_insert_with(|| nfa.create_node());
            let to = *map.entry(*to).or_insert_with(|| nfa.create_node());
            nfa.add_epsilon(from, to);
        }

        let start = *map.entry(start).or_insert_with(|| nfa.create_node());
        let accept = *map.entry(accept).or_insert_with(|| nfa.create_node());

        nfa.add_start(start);
        nfa.add_accept(accept);

        let io = std::fs::File::create("before.dot").unwrap();
        write_nfa_dot(&mut nfa, io).unwrap();

        nfa.reset();

        let io = std::fs::File::create("after.dot").unwrap();
        write_nfa_dot(&mut nfa, io).unwrap();
        nfa
    }

    fn write_nfa_dot<W>(nfa: &Nfa, mut io: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(io, "digraph NFA {{")?;
        writeln!(io, "  rankdir=LR;")?;

        for node in 0..nfa.nodes.len() {
            writeln!(io, "  {} [label=\"{}\", shape=circle];", node, node)?;
        }

        writeln!(io, "  _start_point [shape=point];")?;
        for node in nfa.start.iter() {
            writeln!(io, "  _start_point -> {};", node)?;
        }

        if !nfa.accept.is_empty() {
            writeln!(io, "  _accept_point [shape=point, style=invis];")?;
        }
        for node in nfa.accept.iter() {
            writeln!(io, "  {} [shape=doublecircle];", node)?;
            writeln!(io, "  {} -> _accept_point [style=invis];", node)?;
        }

        for node in 0..nfa.nodes.len() {
            for &(c1, c2, to) in nfa.nodes[node].edges.iter() {
                writeln!(io, "  {} -> {} [label=\"[{}-{}]\"];", node, to, c1, c2)?;
            }

            for &to in nfa.nodes[node].epsilons.iter() {
                writeln!(io, "  {} -> {} [style=dotted];", node, to)?;
            }
        }

        writeln!(io, "}}")?;

        Ok(())
    }

    fn test_nfa(nfa: &mut Nfa, input: &str, is_dead: bool, is_accept: bool, _testname: &str) {
        for c in input.chars() {
            nfa.put(c);
        }

        assert_eq!(
            nfa.is_dead(),
            is_dead,
            "is_dead failure. input: {:?}, current: {:?}, accept: {:?}",
            input,
            nfa.current,
            nfa.accept
        );
        assert_eq!(
            nfa.is_accept(),
            is_accept,
            "is_accept failure. input: {:?}, current: {:?}, accept: {:?}",
            input,
            nfa.current,
            nfa.accept
        );

        nfa.reset();
    }

    #[test]
    fn test_nfa_empty() {
        let mut nfa = build_nfa(0, 1, &[], &[]);

        assert!(nfa.is_dead());
        assert!(!nfa.is_accept());

        nfa.put('a');

        assert!(nfa.is_dead());
        assert!(!nfa.is_accept());
    }

    #[test]
    fn test_nfa_edge() {
        let mut nfa = build_nfa(0, 1, &[(0, 'a', 'a', 1)], &[]);

        assert!(!nfa.is_dead());
        assert!(!nfa.is_accept());

        nfa.put('a');

        assert!(!nfa.is_dead());
        assert!(
            nfa.is_accept(),
            "current: {:?}, accept: {:?}",
            nfa.current,
            nfa.accept
        );

        nfa.put('a');

        assert!(nfa.is_dead());
        assert!(!nfa.is_accept());
    }

    #[test]
    fn test_nfa_reset() {
        let mut nfa = build_nfa(0, 1, &[(0, 'a', 'a', 1)], &[]);

        assert!(!nfa.is_dead());
        assert!(!nfa.is_accept());

        nfa.put('a');

        assert!(!nfa.is_dead());
        assert!(nfa.is_accept());

        nfa.put('a');

        assert!(nfa.is_dead());
        assert!(!nfa.is_accept());

        nfa.reset();

        assert!(!nfa.is_dead());
        assert!(!nfa.is_accept());

        nfa.put('a');

        assert!(!nfa.is_dead());
        assert!(nfa.is_accept());

        nfa.put('a');

        assert!(nfa.is_dead());
        assert!(!nfa.is_accept());
    }

    #[test]
    fn test_nfa_epsilon_transition() {
        let mut nfa = build_nfa(0, 2, &[(1, 'a', 'a', 2)], &[(0, 1)]);

        test_nfa(&mut nfa, "", false, false, "test_nfa_epsilon_transition");
        test_nfa(&mut nfa, "a", false, true, "test_nfa_epsilon_transition");
        test_nfa(&mut nfa, "aa", true, false, "test_nfa_epsilon_transition");
    }

    #[test]
    fn test_nfa_multiple_edges() {
        let mut nfa = build_nfa(0, 2, &[(0, 'a', 'z', 1), (1, '0', '9', 2)], &[]);

        test_nfa(&mut nfa, "a0", false, true, "test_nfa_multiple_edges");
        test_nfa(&mut nfa, "z9", false, true, "test_nfa_multiple_edges");
        test_nfa(&mut nfa, "m5", false, true, "test_nfa_multiple_edges");
        test_nfa(&mut nfa, "aa", true, false, "test_nfa_multiple_edges");
        test_nfa(&mut nfa, "5a", true, false, "test_nfa_multiple_edges");
    }

    #[test]
    fn test_nfa_range_transition() {
        let mut nfa = build_nfa(0, 1, &[(0, '0', '9', 1)], &[]);

        test_nfa(&mut nfa, "0", false, true, "test_nfa_range_transition");
        test_nfa(&mut nfa, "5", false, true, "test_nfa_range_transition");
        test_nfa(&mut nfa, "9", false, true, "test_nfa_range_transition");
        test_nfa(&mut nfa, "a", true, false, "test_nfa_range_transition");
        test_nfa(&mut nfa, ":", true, false, "test_nfa_range_transition");
    }

    #[test]
    fn test_nfa_multiple_epsilon() {
        let mut nfa = build_nfa(0, 3, &[(1, 'a', 'a', 2)], &[(0, 1), (2, 3)]);

        test_nfa(&mut nfa, "", false, false, "test_nfa_multiple_epsilon");
        test_nfa(&mut nfa, "a", false, true, "test_nfa_multiple_epsilon");
        test_nfa(&mut nfa, "aa", true, false, "test_nfa_multiple_epsilon");
    }

    #[test]
    fn test_nfa_branching_paths() {
        let mut nfa = build_nfa(0, 2, &[(0, 'a', 'a', 1), (0, 'b', 'b', 2)], &[]);

        test_nfa(&mut nfa, "a", true, false, "test_nfa_branching_paths");
        test_nfa(&mut nfa, "b", false, true, "test_nfa_branching_paths");
        test_nfa(&mut nfa, "ab", true, false, "test_nfa_branching_paths");
    }

    #[test]
    fn test_nfa_abstar() {
        let mut nfa = build_nfa(
            0,
            2,
            &[(0, 'a', 'a', 1), (1, 'b', 'b', 2)],
            &[(0, 2), (2, 0)],
        );

        test_nfa(&mut nfa, "", false, true, "test_nfa_abstar");
        test_nfa(&mut nfa, "a", false, false, "test_nfa_abstar");
        test_nfa(&mut nfa, "ab", false, true, "test_nfa_abstar");
        test_nfa(&mut nfa, "aba", false, false, "test_nfa_abstar");
        test_nfa(&mut nfa, "abab", false, true, "test_nfa_abstar");
    }
}
