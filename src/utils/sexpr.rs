use std::collections::HashMap;
use std::rc::Rc;

pub struct SExpr<T>(Rc<SExprNode<T>>);

impl<T> Clone for SExpr<T> {
    fn clone(&self) -> Self {
        SExpr(self.0.clone())
    }
}

pub enum SExprNode<T> {
    Nil,
    Var(u64),
    Atom(T),
    Cons(SExpr<T>, SExpr<T>),
}

impl<T> SExpr<T> {
    pub fn nil() -> Self {
        SExpr(Rc::new(SExprNode::Nil))
    }

    pub fn var(id: u64) -> Self {
        SExpr(Rc::new(SExprNode::Var(id)))
    }

    pub fn atom(value: T) -> Self {
        SExpr(Rc::new(SExprNode::Atom(value)))
    }

    pub fn cons(car: &SExpr<T>, cdr: &SExpr<T>) -> Self {
        SExpr(Rc::new(SExprNode::Cons(car.clone(), cdr.clone())))
    }

    pub fn list(xs: &[SExpr<T>]) -> Self {
        let mut list = SExpr::nil();

        for x in xs.iter().rev() {
            list = SExpr::cons(x, &list);
        }

        list
    }

    pub fn decons(&self) -> Option<(SExpr<T>, SExpr<T>)> {
        match &*self.0 {
            SExprNode::Cons(car, cdr) => Some((car.clone(), cdr.clone())),
            _ => None,
        }
    }

    pub fn is_cons(&self) -> bool {
        match &*self.0 {
            SExprNode::Cons(_, _) => true,
            _ => false,
        }
    }

    pub fn car(&self) -> Option<SExpr<T>> {
        self.decons().map(|(car, _)| car)
    }

    pub fn cdr(&self) -> Option<SExpr<T>> {
        self.decons().map(|(_, cdr)| cdr)
    }

    pub fn is_nil(&self) -> bool {
        match &*self.0 {
            SExprNode::Nil => true,
            _ => false,
        }
    }

    pub fn is_atom(&self) -> bool {
        match &*self.0 {
            SExprNode::Atom(_) => true,
            _ => false,
        }
    }

    pub fn get_atom(&self) -> Option<&T> {
        match &*self.0 {
            SExprNode::Atom(value) => Some(value),
            _ => None,
        }
    }

    pub fn get_var(&self) -> Option<u64> {
        match &*self.0 {
            SExprNode::Var(id) => Some(*id),
            _ => None,
        }
    }

    pub fn is_var(&self) -> bool {
        match &*self.0 {
            SExprNode::Var(_) => true,
            _ => false,
        }
    }

    pub fn occurs(&self, id: u64) -> bool {
        let mut stack = vec![self.clone()];

        while let Some(sexpr) = stack.pop() {
            match &*sexpr.0 {
                SExprNode::Var(var_id) => {
                    if *var_id == id {
                        return true;
                    }
                }

                SExprNode::Cons(car, cdr) => {
                    stack.push(car.clone());
                    stack.push(cdr.clone());
                }

                _ => {}
            }
        }

        false
    }

    fn apply_bindings(&self, bindings: &HashMap<u64, SExpr<T>>) -> SExpr<T> {
        match &*self.0 {
            SExprNode::Var(id) => match bindings.get(id) {
                Some(sexpr) => sexpr.clone(),
                None => self.clone(),
            },

            SExprNode::Cons(car, cdr) => {
                let new_car = car.apply_bindings(bindings);
                let new_cdr = cdr.apply_bindings(bindings);

                SExpr::cons(&new_car, &new_cdr)
            }

            _ => self.clone(),
        }
    }

    pub fn unify(&self, other: &SExpr<T>) -> HashMap<u64, SExpr<T>> {
        use SExprNode::*;

        let mut stack = vec![(self.clone(), other.clone())];
        let mut bindings = HashMap::new();

        while let Some((lhs, rhs)) = stack.pop() {
            match (&*lhs.0, &*rhs.0) {
                (Var(l), Var(r)) => {
                    if l != r {
                        bindings.insert(*l, rhs.clone());
                    }
                }

                (Var(l), _) => {
                    if rhs.occurs(*l) {
                        panic!("occurs check failed");
                    }

                    bindings.insert(*l, rhs.clone());
                }

                (_, Var(..)) => {
                    stack.push((rhs, lhs));
                }

                (Cons(lcar, lcdr), Cons(rcar, rcdr)) => {
                    let lcar = lcar.apply_bindings(&bindings);
                    let rcar = rcar.apply_bindings(&bindings);
                    let lcdr = lcdr.apply_bindings(&bindings);
                    let rcdr = rcdr.apply_bindings(&bindings);

                    stack.push((lcar, rcar));
                    stack.push((lcdr, rcdr));
                }

                _ => {}
            }
        }

        bindings
    }

    pub fn zipper(&self) -> Zipper<T> {
        Zipper {
            stack: Vec::new(),
            focus: self.clone(),
        }
    }
}

pub enum ZipperDirection {
    Left,
    Right,
}

pub struct Zipper<T> {
    stack: Vec<(SExpr<T>, ZipperDirection)>,
    focus: SExpr<T>,
}

impl<T> Zipper<T> {
    pub fn focus(&self) -> &SExpr<T> {
        &self.focus
    }

    pub fn left(&mut self) -> bool {
        if let Some((l, r)) = self.focus.decons() {
            self.stack.push((r, ZipperDirection::Left));
            self.focus = l;
            true
        } else {
            false
        }
    }

    pub fn right(&mut self) -> bool {
        if let Some((l, r)) = self.focus.decons() {
            self.stack.push((l, ZipperDirection::Right));
            self.focus = r;
            true
        } else {
            false
        }
    }

    pub fn up(&mut self) -> Option<ZipperDirection> {
        if let Some((sibling, direction)) = self.stack.pop() {
            match direction {
                ZipperDirection::Left => {
                    let l = self.focus.clone();
                    let r = sibling;

                    self.focus = SExpr::cons(&l, &r);
                }

                ZipperDirection::Right => {
                    let l = sibling;
                    let r = self.focus.clone();

                    self.focus = SExpr::cons(&l, &r);
                }
            }
            Some(direction)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unify_atoms() {
        let a1 = SExpr::atom(1);
        let a2 = SExpr::atom(1);
        let bindings = a1.unify(&a2);
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_unify_vars() {
        let v1 = SExpr::<()>::var(1);
        let v2 = SExpr::<()>::var(2);

        let bindings = v1.unify(&v2);
        assert_eq!(bindings.len(), 1);
        assert!(bindings.contains_key(&1));
        assert_eq!(bindings[&1].get_var(), Some(2));
    }

    #[test]
    fn test_unify_var_with_atom() {
        let v = SExpr::var(1);
        let a = SExpr::atom(42);
        let bindings = v.unify(&a);
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[&1].get_atom(), Some(&42));
    }

    #[test]
    fn test_unify_complex_expression() {
        let expr1 = SExpr::cons(&SExpr::var(1), &SExpr::cons(&SExpr::atom(2), &SExpr::nil()));
        let expr2 = SExpr::cons(
            &SExpr::atom(42),
            &SExpr::cons(&SExpr::atom(2), &SExpr::nil()),
        );
        let bindings = expr1.unify(&expr2);
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[&1].get_atom(), Some(&42));
    }

    #[test]
    #[should_panic(expected = "occurs check failed")]
    fn test_unify_occurs_check() {
        let v = SExpr::<()>::var(1);
        let expr = SExpr::cons(&v, &SExpr::nil());
        v.unify(&expr); // Should panic due to occurs check
    }

    #[test]
    fn test_zipper_navigation() {
        // Create a nested structure: (1 (2 3))
        let inner = SExpr::cons(&SExpr::atom(2), &SExpr::atom(3));
        let expr = SExpr::cons(&SExpr::atom(1), &inner);

        let mut z = expr.zipper();
        assert_eq!(z.focus().car().unwrap().get_atom(), Some(&1));

        // Move right to (2 3)
        assert!(z.right());
        assert_eq!(z.focus().car().unwrap().get_atom(), Some(&2));

        // Move back up
        assert!(z.up().is_some());
        assert_eq!(z.focus().car().unwrap().get_atom(), Some(&1));
    }

    #[test]
    fn test_zipper_deep_navigation() {
        // Create (1 (2 (3 4)))
        let leaf = SExpr::cons(&SExpr::atom(3), &SExpr::atom(4));
        let middle = SExpr::cons(&SExpr::atom(2), &leaf);
        let expr = SExpr::cons(&SExpr::atom(1), &middle);

        let mut z = expr.zipper();

        // Navigate to 4
        assert!(z.right()); // Move to (2 (3 4))
        assert!(z.right()); // Move to (3 4)
        assert!(z.right()); // Move to 4
        assert_eq!(z.focus().get_atom(), Some(&4));

        // Navigate back to root
        assert!(z.up().is_some()); // Back to (3 4)
        assert!(z.up().is_some()); // Back to (2 (3 4))
        assert!(z.up().is_some()); // Back to (1 (2 (3 4)))
        assert_eq!(z.focus().car().unwrap().get_atom(), Some(&1));
    }

    #[test]
    fn test_zipper_left_navigation() {
        // Create (1 2)
        let expr = SExpr::cons(&SExpr::atom(1), &SExpr::atom(2));
        let mut z = expr.zipper();

        assert!(z.left()); // Move to 1
        assert_eq!(z.focus().get_atom(), Some(&1));
        assert!(z.up().is_some()); // Back to (1 2)
        assert!(z.right()); // Move to 2
        assert_eq!(z.focus().get_atom(), Some(&2));
    }

    #[test]
    fn test_zipper_boundary_conditions() {
        let atom = SExpr::atom(1);
        let mut z = atom.zipper();

        // Can't navigate on an atom
        assert!(!z.left());
        assert!(!z.right());
        assert!(z.up().is_none());

        // Can't navigate up at root
        let list = SExpr::cons(&SExpr::atom(1), &SExpr::atom(2));
        let mut z = list.zipper();
        assert!(z.up().is_none());
    }
}
