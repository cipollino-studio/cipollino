
use super::{ObjData, Project};

enum ObjActionKind {
    Addition(Box<dyn ObjData>),
    Deletion(Box<dyn ObjData>),
    Modification(Box<dyn ObjData>, Box<dyn ObjData>) 
}

pub struct ObjAction {
    kind: ObjActionKind,
    key: u64
}

impl ObjAction {

    pub fn addition<T>(key: u64, data: T) -> Self where T: ObjData + 'static {
        Self {
            kind: ObjActionKind::Addition(Box::new(data)),
            key
        }
    }

    pub fn deletion<T>(key: u64, data: T) -> Self where T: ObjData + 'static {
        Self {
            kind: ObjActionKind::Deletion(Box::new(data)),
            key
        }
    }

    pub fn modification<T>(key: u64, old_data: T, new_data: T) -> Self where T: ObjData + 'static {
        Self {
            kind: ObjActionKind::Modification(Box::new(old_data), Box::new(new_data)),
            key
        }
    }

    pub fn redo(&self, project: &mut Project) {
        match &self.kind {
            ObjActionKind::Addition(data) => data.add(self.key, project),
            ObjActionKind::Deletion(data) => data.delete(self.key, project),
            ObjActionKind::Modification(_old, new) => new.set(self.key, project)
        }
    }

    pub fn undo(&self, project: &mut Project) {
        match &self.kind {
            ObjActionKind::Addition(data) => data.delete(self.key, project),
            ObjActionKind::Deletion(data) => data.add(self.key, project),
            ObjActionKind::Modification(old, _new) => old.set(self.key, project)
        }
    }

}

pub struct Action {
    actions: Vec<ObjAction>
}

impl Action {

    pub fn new() -> Self {
        Self {
            actions: Vec::new()
        }
    }

    pub fn from_single(act: ObjAction) -> Self {
        Self {
            actions: vec![act]
        }
    }

    pub fn from_list(acts: Vec<ObjAction>) -> Self {
        Self {
            actions: acts 
        }
    }

    pub fn redo(&self, project: &mut Project) {
        for action in &self.actions {
            action.redo(project);
        }
    }

    pub fn undo(&self, project: &mut Project) {
        for action in self.actions.iter().rev() {
            action.undo(project);
        }
    }

    pub fn add(&mut self, act: ObjAction) {
        self.actions.push(act);
    }

}

impl Default for Action {

    fn default() -> Self {
        Action::new() 
    }

}

pub struct ActionManager {
    actions: Vec<Action>,
    curr: i32 
}

impl ActionManager {

    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            curr: -1
        }
    }

    pub fn add(&mut self, act: Action) {
        self.actions.truncate((self.curr + 1) as usize);
        self.actions.push(act);
        self.curr += 1;
    }

    pub fn can_redo(&self) -> bool {
        ((self.curr + 1) as usize) < self.actions.len()
    }

    pub fn redo(&mut self, project: &mut Project) {
        if self.can_redo() { 
            self.curr += 1;
            self.actions[self.curr as usize].redo(project);
        }
    }

    pub fn can_undo(&self) -> bool {
        self.curr > -1
    }

    pub fn undo(&mut self, project: &mut Project) {
        if self.can_undo() {
            self.actions[self.curr as usize].undo(project);
            self.curr -= 1;
        }
    }

}
