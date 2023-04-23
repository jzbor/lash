use crate::lambda::*;

pub trait Strategy {
    fn reduce(&self, term: LambdaTree) -> Option<(LambdaTree, u32)>;

    fn normalize(&self, term: LambdaTree) -> LambdaTree {
        let mut current = term;
        loop {
            if let Some((next, _)) = self.reduce(current.clone()) {
                current = next;
            } else {
                return current;
            }
        }
    }
}

struct ApplicativeStrategy {}

impl Strategy for ApplicativeStrategy {
    fn reduce(&self, term: LambdaTree) -> Option<(LambdaTree, u32)> {
        use LambdaNode::*;
        match term.node() {
            Abstraction(_, term) => self.reduce(term.clone()).map(|(term, depth)| (term, depth + 1)),
            Application(left_term, right_term) => {
                let left_option = self.reduce(left_term.clone());
                let right_option = self.reduce(right_term.clone());

                if let Some((left_reduced, left_depth)) = left_option {
                    if let Some((right_reduced, right_depth)) = right_option {
                        if left_depth >= right_depth {
                            Some((LambdaTree::new_application(left_reduced, right_term.clone()), left_depth + 1))
                        } else {
                            Some((LambdaTree::new_application(left_term.clone(), right_reduced), right_depth + 1))
                        }
                    } else {
                        Some((LambdaTree::new_application(left_reduced, right_term.clone()), left_depth + 1))
                    }
                } else if let Some((right_reduced, right_depth)) = right_option {
                    Some((LambdaTree::new_application(left_term.clone(), right_reduced), right_depth + 1))
                } else if let Application(left_term, right_term) = term.node() {
                    if let Abstraction(var, inner) = left_term.node() {
                        Some((inner.substitute(var, right_term.clone()), 0))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            Variable(_) => None,
            Macro(..) => panic!(),
            Named(named) => self.reduce(named.term()),
        }
    }
}

pub fn default_strategy() -> impl Strategy {
    ApplicativeStrategy {}
}
