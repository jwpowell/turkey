use std::collections::HashMap;
use std::fmt;
use std::io;

pub struct Graph<V, E> {
    vertices: Vec<Vertex<V, E>>,
    map: HashMap<u64, usize>,
    next_id: u64,
}

struct Vertex<V, E> {
    data: V,
    edges: Vec<Edge<E>>,
}

struct Edge<E> {
    data: E,
    target: u64,
}

impl<V, E> Graph<V, E> {
    pub fn new() -> Self {
        Graph {
            vertices: Vec::new(),
            map: HashMap::new(),
            next_id: 1,
        }
    }

    fn get_index(&self, key: u64) -> Option<usize> {
        self.map.get(&key).copied()
    }

    fn genid(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add_vertex(&mut self, data: V) -> u64 {
        let id = self.genid();
        let index = self.vertices.len();

        self.vertices.push(Vertex {
            data,
            edges: Vec::new(),
        });

        self.map.insert(id, index);

        id
    }

    pub fn add_edge(&mut self, src: u64, dst: u64, data: E) {
        let src_index = self.get_index(src).expect("source vertex not found");

        self.vertices[src_index]
            .edges
            .push(Edge { data, target: dst });
    }

    pub fn remove_vertex(&mut self, deleted: u64) {
        let old_index = self.vertices.len() - 1;
        let deleted_index = *self.map.get(&deleted).expect("vertex not found");
        let new_index = deleted_index;

        self.vertices.swap_remove(deleted_index);

        for index in &mut self.map.values_mut() {
            if *index == old_index {
                *index = new_index;
                break;
            }
        }
    }

    pub fn remove_edge(&mut self, src: u64, dst: u64) {
        let src_index = self.get_index(src).expect("source vertex not found");
        self.vertices[src_index]
            .edges
            .retain(|edge| edge.target != dst);
    }

    pub fn visit<Vi: Visitor<V, E>>(&self, visitor: &mut Vi) {
        for (&vertex, &vertex_index) in self.map.iter() {
            let vertex_data = &self.vertices[vertex_index];
            visitor.visit_vertex(vertex, &vertex_data.data);

            for edge in &vertex_data.edges {
                visitor.visit_edge(vertex, edge.target, &edge.data);
            }
        }
    }

    pub fn visit_mut<Vi: VisitorMut<V, E>>(&mut self, visitor: &mut Vi) {
        for (&vertex, &vertex_index) in self.map.iter() {
            let vertex_data = &mut self.vertices[vertex_index];
            visitor.visit_mut_vertex(vertex, &mut vertex_data.data);

            for edge in &mut vertex_data.edges {
                visitor.visit_mut_edge(vertex, edge.target, &mut edge.data);
            }
        }
    }

    pub fn visit_from<Vi: Visitor<V, E>>(&self, start: &[u64], visitor: &mut Vi) {
        let mut stack = start.to_vec();
        let mut visited = vec![false; self.vertices.len()];

        while let Some(vertex) = stack.pop() {
            let vertex_index = self.get_index(vertex).expect("vertex not found");
            if visited[vertex_index] {
                continue;
            }

            visited[vertex_index] = true;

            let vertex_data = &self.vertices[vertex_index];

            visitor.visit_vertex(vertex, &vertex_data.data);

            for edge in &vertex_data.edges {
                visitor.visit_edge(vertex, edge.target, &edge.data);

                stack.push(edge.target);
            }
        }
    }

    pub fn visit_to<Vi: Visitor<V, E>>(&self, end: &[u64], visitor: &mut Vi) {
        let mut stack = end.to_vec();
        let mut visited = vec![false; self.vertices.len()];

        while let Some(vertex) = stack.pop() {
            let vertex_index = self.get_index(vertex as u64).expect("vertex not found");

            if visited[vertex_index] {
                continue;
            }

            visited[vertex_index] = true;

            let vertex_data = &self.vertices[vertex_index];

            visitor.visit_vertex(vertex, &vertex_data.data);

            for (&src, &src_index) in self.map.iter() {
                for edge in &self.vertices[src_index].edges {
                    if edge.target == vertex {
                        visitor.visit_edge(src, vertex, &edge.data);

                        stack.push(src);
                    }
                }
            }
        }
    }

    pub fn visit_mut_from<Vi: VisitorMut<V, E>>(&mut self, start: &[u64], visitor: &mut Vi) {
        let mut stack = start.to_vec();
        let mut visited = vec![false; self.vertices.len()];

        while let Some(vertex) = stack.pop() {
            let vertex_index = self.get_index(vertex).expect("vertex not found");
            if visited[vertex_index] {
                continue;
            }

            visited[vertex_index] = true;

            let vertex_data = &mut self.vertices[vertex_index];

            visitor.visit_mut_vertex(vertex, &mut vertex_data.data);

            for edge in &mut vertex_data.edges {
                visitor.visit_mut_edge(vertex, edge.target, &mut edge.data);

                stack.push(edge.target);
            }
        }
    }

    pub fn visit_mut_to<Vi: VisitorMut<V, E>>(&mut self, end: &[u64], visitor: &mut Vi) {
        let mut stack = end.to_vec();
        let mut visited = vec![false; self.vertices.len()];

        while let Some(vertex) = stack.pop() {
            let vertex_index = self.get_index(vertex as u64).expect("vertex not found");

            if visited[vertex_index] {
                continue;
            }

            visited[vertex_index] = true;

            let vertex_data = &mut self.vertices[vertex_index];

            visitor.visit_mut_vertex(vertex, &mut vertex_data.data);

            for (&src, &src_index) in self.map.iter() {
                for edge in &mut self.vertices[src_index].edges {
                    if edge.target == vertex {
                        visitor.visit_mut_edge(src, vertex, &mut edge.data);

                        stack.push(src);
                    }
                }
            }
        }
    }

    pub fn fmt_dot<W: fmt::Write>(&self, mut f: W) -> fmt::Result
    where
        V: fmt::Display,
        E: fmt::Display,
    {
        write!(f, "digraph {{\n")?;
        write!(f, "  rankdir=LR;\n")?;
        write!(f, "  node [shape=circle];\n")?;

        let mut visitor = DotVisitor {
            buffer: f,
            result: Ok(()),
        };

        self.visit(&mut visitor);

        let DotVisitor {
            buffer: mut f,
            result,
        } = visitor;

        result?;

        write!(f, "}}\n")
    }

    pub fn write_dot<W: io::Write>(&self, mut io: W) -> io::Result<()>
    where
        V: fmt::Display,
        E: fmt::Display,
    {
        let mut buffer = String::new();

        self.fmt_dot(&mut buffer).unwrap();

        write!(io, "{}", buffer)
    }
}

struct DotVisitor<W> {
    buffer: W,
    result: fmt::Result,
}

impl<V, E, W> Visitor<V, E> for DotVisitor<W>
where
    V: fmt::Display,
    E: fmt::Display,
    W: fmt::Write,
{
    fn visit_vertex(&mut self, vertex: u64, data: &V) {
        self.result = self
            .result
            .and_then(|_| write!(self.buffer, "  {} [label=\"{}\"];\n", vertex, data));
    }

    fn visit_edge(&mut self, src: u64, dst: u64, data: &E) {
        self.result = self
            .result
            .and_then(|_| write!(self.buffer, "  {} -> {} [label=\"{}\"];\n", src, dst, data));
    }
}

#[allow(unused_variables)]
pub trait Visitor<V, E> {
    fn visit_vertex(&mut self, vertex: u64, data: &V) {}
    fn visit_edge(&mut self, src: u64, dst: u64, data: &E) {}
}

#[allow(unused_variables)]
pub trait VisitorMut<V, E> {
    fn visit_mut_vertex(&mut self, vertex: u64, data: &mut V) {}
    fn visit_mut_edge(&mut self, src: u64, dst: u64, data: &mut E) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_graph_a() -> (Graph<u32, u32>, u64, u64, u64, u64) {
        let mut graph = Graph::new();

        let a = graph.add_vertex(0);
        let b = graph.add_vertex(1);
        let c = graph.add_vertex(2);
        let d = graph.add_vertex(3);

        graph.add_edge(a, b, 0);
        graph.add_edge(b, c, 1);
        graph.add_edge(c, a, 2);
        graph.add_edge(c, c, 3);
        graph.add_edge(d, d, 4);

        (graph, a, b, c, d)
    }

    struct ReachableVisitor {
        reachable: Vec<u64>,
    }

    impl Visitor<u32, u32> for ReachableVisitor {
        fn visit_vertex(&mut self, vertex: u64, _: &u32) {
            self.reachable.push(vertex);
        }
    }

    #[test]
    fn test_graph_visit() {
        let (graph, a, b, c, d) = test_graph_a();

        let mut visitor = ReachableVisitor {
            reachable: Vec::new(),
        };

        graph.visit_from(&[a], &mut visitor);

        visitor.reachable.sort_unstable();
        visitor.reachable.dedup();

        assert_eq!(visitor.reachable, vec![a, b, c]);

        visitor.reachable.clear();

        graph.visit_from(&[d], &mut visitor);

        visitor.reachable.sort_unstable();
        visitor.reachable.dedup();

        assert_eq!(visitor.reachable, vec![d]);
    }
}
