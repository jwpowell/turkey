use std::collections::HashMap;
use std::fmt::Debug;

use super::regex::{Regex, RegexNode};
use crate::lexer::regex::{char, epsilon, one_of, range};

#[derive(Debug)]
pub struct Nfa<T> {
    start: usize,
    accept: usize,
    nodes: Vec<NfaNode<T>>,
}

#[derive(Debug, Clone)]
struct NfaNode<T> {
    edges: Vec<(char, char, usize, T)>,
    epsilon: Vec<usize>,
}

impl<T> Nfa<T>
where
    T: Clone + Debug,
{
    pub fn new() -> Self {
        let mut nfa = Nfa {
            start: 0,
            accept: 1,
            nodes: Vec::new(),
        };

        nfa.start = nfa.create_node();
        nfa.accept = nfa.create_node();

        nfa
    }

    fn get_start(&self) -> usize {
        self.start
    }

    fn get_accept(&self) -> usize {
        self.accept
    }

    fn create_node(&mut self) -> usize {
        let index = self.nodes.len();

        self.nodes.push(NfaNode {
            edges: Vec::new(),
            epsilon: Vec::new(),
        });

        index
    }

    fn add_edge(&mut self, from: usize, to: usize, lo: char, hi: char, data: T) {
        // I'm going to do something terrible here. Forgive me.

        fn overlaps(lo1: char, hi1: char, lo2: char, hi2: char) -> bool {
            (lo1 <= lo2 && lo2 <= hi1) || (lo1 <= hi2 && hi2 <= hi1)
        }

        let overlapping = self.nodes[from]
            .edges
            .iter()
            .any(|&(lo1, hi1, _, _)| overlaps(lo1, hi1, lo, hi));

        if overlapping {
            let node = self.create_node();

            self.nodes[node].edges.push((lo, hi, to, data));
            self.add_epsilon(from, node);
        } else {
            self.nodes[from].edges.push((lo, hi, to, data));
        }
    }

    fn add_epsilon(&mut self, from: usize, to: usize) {
        self.nodes[from].epsilon.push(to);

        self.epsilon_closure();

        self.nodes[from].epsilon.sort_unstable();
        self.nodes[from].epsilon.dedup();
    }

    fn set_accept(&mut self, state: usize) {
        self.add_epsilon(state, self.accept);
    }

    fn epsilon_clean(&mut self) {
        while self.epsilon_clean_inner() {
            continue;
        }
    }

    fn epsilon_clean_inner(&mut self) -> bool {
        let mut changed = false;

        for i in 0..self.nodes.len() {
            self.nodes[i].epsilon.sort_unstable();
            self.nodes[i].epsilon.dedup();

            let mut old = self.nodes[i].epsilon.len();
            self.nodes[i].epsilon.retain(|&x| x != i);

            if old != self.nodes[i].epsilon.len() {
                changed = true;
            }
        }

        // two states are duplicates if they dont have any edges and have the same epsilons.
        //let mut map = HashMap::new();
        //let mut eps = vec![];
        let mut fixup = HashMap::new();

        let mut cl1 = vec![];
        let mut cl2 = vec![];

        for i in 0..self.nodes.len() {
            cl1.clear();

            if i == self.start || i == self.accept {
                continue;
            }

            for j in (i + 1)..self.nodes.len() {
                cl2.clear();

                if j == self.start || j == self.accept {
                    //continue;
                }

                if i == j {
                    continue;
                }

                if !self.nodes[i].edges.is_empty() || !self.nodes[j].edges.is_empty() {
                    continue;
                }

                self.calculate_epsilon_closure(i, &mut cl1);
                self.calculate_epsilon_closure(j, &mut cl2);

                if self.nodes[i].epsilon.contains(&j) {
                    cl2.push(j);
                }

                if self.nodes[j].epsilon.contains(&i) {
                    cl1.push(i);
                }

                cl1.sort_unstable();
                cl1.dedup();

                cl2.sort_unstable();
                cl2.dedup();

                println!("i: {}, j: {}", i, j);
                println!("cl1: {:?}", cl1);
                println!("cl2: {:?}", cl2);

                if cl1 == cl2 {
                    fixup.insert(i, j);
                    println!("fixing up {} -> {}", i, j);
                }
                println!();
            }
        }

        /*
        for i in 0..self.nodes.len() {
            eps.clear();

            if !self.nodes[i].edges.is_empty() {
                continue;
            }

            if i == self.accept || i == self.start {
                continue;
            }

            eps.clear();

            eps.extend(self.nodes[i].epsilon.iter().copied());

            eps.sort_unstable();
            eps.dedup();

            if let Some(&index) = map.get(&eps) {
                fixup.insert(i, index);
            } else {
                map.insert(eps.clone(), i);
            }
        }
        */

        println!("fixup: {:?}", fixup);

        for i in 0..self.nodes.len() {
            if let Some(_) = fixup.get(&i) {
                if self.nodes[i].edges.is_empty() && self.nodes[i].epsilon.is_empty() {
                    continue;
                }

                self.nodes[i].edges.clear();
                self.nodes[i].epsilon.clear();

                changed = true;
                continue;
            }

            for (_, _, to, _) in &mut self.nodes[i].edges {
                if let Some(&index) = fixup.get(to) {
                    if *to == index {
                        continue;
                    }

                    println!("fixing up edge {} -> {} to {}", i, to, index);
                    *to = index;
                    changed = true;
                }
            }

            for to in &mut self.nodes[i].epsilon {
                if let Some(&index) = fixup.get(to) {
                    if *to == index {
                        continue;
                    }

                    println!("fixing up epsilon {} -> {} to {}", i, to, index);
                    *to = index;
                    changed = true;
                }
            }
        }

        for i in 0..self.nodes.len() {
            if i == self.start || i == self.accept {
                continue;
            }

            if !self.nodes[i].edges.is_empty() || !self.nodes[i].epsilon.is_empty() {
                continue;
            }

            for j in 0..self.nodes.len() {
                if j == i {
                    continue;
                }

                //self.nodes[j].edges.retain(|&(_, _, to, _)| to != i);
                let old = self.nodes[j].epsilon.len();
                self.nodes[j].epsilon.retain(|&to| to != i);
                if old != self.nodes[j].epsilon.len() {
                    changed = true;
                }
            }
        }

        changed
    }

    fn epsilon_closure(&mut self) {
        //self.epsilon_clean();

        while self.epsilon_closure_1() {
            continue;
        }
    }

    fn epsilon_closure_1(&mut self) -> bool {
        let mut changed = false;

        let mut additional = vec![];

        for a in 0..self.nodes.len() {
            for &b in self.nodes[a].epsilon.iter() {
                for &c in self.nodes[b].epsilon.iter() {
                    if !self.nodes[a].epsilon.contains(&c) {
                        additional.push(c);
                        changed = true;
                    }
                }
            }

            self.nodes[a].epsilon.extend(additional.drain(..));
        }

        changed
    }

    fn calculate_epsilon_closure(&self, from: usize, closure: &mut Vec<usize>) {
        closure.clear();

        let mut stack = vec![from];
        let mut visited = vec![false; self.nodes.len()];

        while let Some(node) = stack.pop() {
            visited[node] = true;

            closure.extend(self.nodes[node].epsilon.iter().copied());
            for &next in &self.nodes[node].epsilon {
                if !visited[next] {
                    stack.push(next);
                }
            }
        }
    }

    fn thompson(r: &Regex, output: &T) -> Self {
        let mut nfa: Nfa<T> = Nfa::new();

        match r.node.as_ref() {
            RegexNode::Empty => {}

            RegexNode::Epsilon => {
                nfa.add_epsilon(nfa.get_start(), nfa.get_accept());
            }

            RegexNode::Range(lo, hi) => {
                nfa.add_edge(nfa.get_start(), nfa.get_accept(), *lo, *hi, output.clone());
            }

            RegexNode::Concat(left, right) => {
                let left = Self::thompson(left, output);
                let right = Self::thompson(right, output);

                let (ls, la) = nfa.merge(&left);
                let (rs, ra) = nfa.merge(&right);

                nfa.add_epsilon(nfa.get_start(), ls);
                nfa.add_epsilon(la, rs);
                nfa.add_epsilon(ra, nfa.get_accept());
            }

            RegexNode::Union(left, right) => {
                let left = Self::thompson(left, output);
                let right = Self::thompson(right, output);

                let (ls, la) = nfa.merge(&left);
                let (rs, ra) = nfa.merge(&right);

                nfa.add_epsilon(nfa.get_start(), ls);
                nfa.add_epsilon(nfa.get_start(), rs);
                nfa.add_epsilon(la, nfa.get_accept());
                nfa.add_epsilon(ra, nfa.get_accept());
            }

            RegexNode::Intersect(left, right) => {
                let left = Self::thompson(left, output);
                let right = Self::thompson(right, output);

                let (ls, la) = nfa.merge(&left);
                let (rs, ra) = nfa.merge(&right);

                nfa.add_epsilon(nfa.get_start(), ls);
                nfa.add_epsilon(nfa.get_start(), rs);
                nfa.add_epsilon(la, nfa.get_accept());
                nfa.add_epsilon(ra, nfa.get_accept());

                todo!("does intersect work? hm.");
            }

            RegexNode::Star(inner) => {
                let inner = Self::thompson(inner, output);

                let (is, ia) = nfa.merge(&inner);

                nfa.add_epsilon(nfa.get_start(), nfa.get_accept());
                nfa.add_epsilon(nfa.get_accept(), nfa.get_start());
                nfa.add_epsilon(nfa.get_start(), is);
                nfa.add_epsilon(ia, nfa.get_accept());
            }

            RegexNode::Not(inner) => {
                let inner = Self::thompson(inner, output);

                let (is, ia) = nfa.merge(&inner);

                nfa.add_epsilon(nfa.get_start(), is);
                nfa.add_epsilon(ia, nfa.get_accept());

                todo!("does 'not' work? hm.");
            }
        }

        nfa
    }

    fn merge(&mut self, other: &Self) -> (usize, usize) {
        let mut map = HashMap::new();

        let start = *map.entry(other.start).or_insert_with(|| self.create_node());

        let accept = *map
            .entry(other.accept)
            .or_insert_with(|| self.create_node());

        for (from, node) in other.nodes.iter().enumerate() {
            let from = *map.entry(from).or_insert_with(|| self.create_node());

            for &(lo, hi, to, ref data) in &node.edges {
                let to = *map.entry(to).or_insert_with(|| self.create_node());
                self.add_edge(from, to, lo, hi, data.clone());
            }

            for &to in &node.epsilon {
                let to = *map.entry(to).or_insert_with(|| self.create_node());
                self.add_epsilon(from, to);
            }
        }

        (start, accept)
    }

    fn print_dot(&self) {
        println!("--------------------");
        self.write_dot(std::io::stdout()).unwrap();
        println!("--------------------");
    }

    fn write_dot_to_file(&self, path: &str) {
        let file = std::fs::File::create(path).unwrap();
        self.write_dot(file).unwrap();
    }

    fn write_dot<W: std::io::Write>(&self, mut io: W) -> std::io::Result<()> {
        writeln!(io, "digraph NFA {{")?;
        writeln!(io, "  rankdir=LR;")?;
        writeln!(io, "  node [shape=circle];")?;

        writeln!(io, "  _start_point [shape=point];")?;
        writeln!(io, "  _start_point -> {};", self.get_start())?;

        writeln!(io, "  {} [shape=doublecircle];", self.get_accept())?;

        for (i, node) in self.nodes.iter().enumerate() {
            for &(lo, hi, to, _) in &node.edges {
                writeln!(io, "  {} -> {} [label=\"{}-{}\"];", i, to, lo, hi)?;
            }

            for &to in &node.epsilon {
                writeln!(io, "  {} -> {} [label=\"ε\"];", i, to)?;
            }
        }

        writeln!(io, "}}")?;

        Ok(())
    }
}

pub struct NfaRunner<'a, T> {
    nfa: &'a Nfa<T>,
    current: Vec<usize>,
    next: Vec<usize>,
}

impl<'a, T> NfaRunner<'a, T>
where
    T: Clone + Debug,
{
    pub fn new(nfa: &'a Nfa<T>) -> Self {
        let mut runner = NfaRunner {
            nfa,
            current: Vec::new(),
            next: Vec::new(),
        };

        runner.reset();

        runner
    }

    pub fn reset(&mut self) {
        self.current.clear();
        self.current.push(self.nfa.get_start());

        self.current
            .extend(self.nfa.nodes[self.nfa.get_start()].epsilon.iter().cloned());
        self.current.sort_unstable();
        self.current.dedup();

        self.next.clear();
    }

    pub fn step(&mut self, c: char) {
        println!("step: {}", c);
        println!("  current: {:?}", self.current);

        self.next.clear();

        for &state in &self.current {
            for &(lo, hi, to, _) in &self.nfa.nodes[state].edges {
                if lo <= c && c <= hi {
                    self.next.push(to);
                    self.next.extend(self.nfa.nodes[to].epsilon.iter().cloned());
                }
            }
        }

        self.next.sort_unstable();
        self.next.dedup();

        println!("  next: {:?}", self.next);

        std::mem::swap(&mut self.current, &mut self.next);
    }

    pub fn is_accept(&self) -> bool {
        self.current.binary_search(&self.nfa.get_accept()).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfa_empty() {
        let regex = Regex::empty();
        let nfa = Nfa::thompson(&regex, &());
        let mut runner = NfaRunner::new(&nfa);

        runner.step('a');
        assert!(!runner.is_accept());
    }

    #[test]
    fn test_nfa_epsilon() {
        let regex = Regex::epsilon();
        let nfa = Nfa::thompson(&regex, &());
        let mut runner = NfaRunner::new(&nfa);

        nfa.print_dot();

        assert!(runner.is_accept());
    }

    #[test]
    fn test_nfa_char() {
        let regex = char('a');
        let nfa = Nfa::thompson(&regex, &());

        nfa.print_dot();

        let mut runner = NfaRunner::new(&nfa);

        runner.step('a');
        assert!(runner.is_accept(), "current states: {:?}", runner.current);

        runner.reset();
        runner.step('b');
        assert!(!runner.is_accept());
    }

    #[test]
    fn test_nfa_range() {
        let regex = range('a', 'c');
        let nfa = Nfa::thompson(&regex, &());
        let mut runner = NfaRunner::new(&nfa);

        runner.step('b');
        assert!(runner.is_accept());

        runner.reset();
        runner.step('d');
        assert!(!runner.is_accept());
    }

    #[test]
    fn test_nfa_union() {
        let regex = one_of("ab");
        let mut nfa = Nfa::thompson(&regex, &());

        nfa.write_dot_to_file("a.dot");
        nfa.epsilon_clean();
        nfa.epsilon_clean();
        nfa.epsilon_clean();

        nfa.write_dot_to_file("b.dot");

        let mut runner = NfaRunner::new(&nfa);

        runner.step('a');
        assert!(runner.is_accept());

        runner.reset();
        runner.step('b');
        assert!(runner.is_accept());

        runner.reset();
        runner.step('c');
        assert!(!runner.is_accept());
    }

    #[test]
    fn test_nfa_concat() {
        let regex = char('a').concat(&char('b'));
        let nfa = Nfa::thompson(&regex, &());
        let mut runner = NfaRunner::new(&nfa);

        runner.step('a');
        assert!(!runner.is_accept());

        runner.step('b');
        assert!(runner.is_accept());

        runner.reset();
        runner.step('b');
        assert!(!runner.is_accept());
    }

    #[test]
    fn test_nfa_star() {
        let regex = char('a').star();
        let nfa = Nfa::thompson(&regex, &());
        let mut runner = NfaRunner::new(&nfa);

        assert!(runner.is_accept());

        runner.step('a');
        assert!(runner.is_accept());

        runner.step('a');
        assert!(runner.is_accept());

        runner.step('b');
        assert!(!runner.is_accept());
    }

    #[test]
    fn test_nfa_complex() {
        let regex = char('a').concat(&char('b')).union(&char('c').star());
        let nfa = Nfa::thompson(&regex, &());
        let mut runner = NfaRunner::new(&nfa);

        runner.step('a');
        assert!(!runner.is_accept());

        runner.step('b');
        assert!(runner.is_accept());

        runner.reset();
        runner.step('c');
        assert!(runner.is_accept());

        runner.step('c');
        assert!(runner.is_accept());

        runner.step('a');
        assert!(!runner.is_accept());
    }
}
