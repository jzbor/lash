extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::rc::Rc;

use crate::lambda::NamedTerm;


pub type ContextFrame = Vec<Rc<NamedTerm>>;


pub struct ContextTracker {
    current_terms: BTreeMap<String, Rc<NamedTerm>>,
    frames: Vec<ContextFrame>,
}


impl ContextTracker {
    pub fn new() -> Self {
        ContextTracker {
            current_terms: BTreeMap::new(),
            frames: Vec::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<Rc<NamedTerm>> {
        self.current_terms.get(name).cloned()
    }

    pub fn push_frame(&mut self, frame: ContextFrame) {
        self.frames.push(frame);
        self.rebuild_current_terms();
    }

    pub fn pop_frame(&mut self) {
        self.frames.pop();
        self.rebuild_current_terms();
    }

    fn rebuild_current_terms(&mut self) {
        self.current_terms = BTreeMap::new();
        for frame in &self.frames {
            for named in frame {
                self.current_terms.insert(named.name().to_owned(), named.clone());
            }
        }
    }
}
