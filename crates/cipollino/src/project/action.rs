
use super::Project;

pub struct ObjAction {
    redo_func: Box<dyn Fn(&mut Project) + Send + Sync>,
    undo_func: Box<dyn Fn(&mut Project) + Send + Sync>
}

impl ObjAction {

    pub fn new<T, G>(redo: T, undo: G) -> Self where T: Fn(&mut Project) + Send + Sync + 'static, G: Fn(&mut Project) + Send + Sync + 'static {
        Self {
            redo_func: Box::new(redo),
            undo_func: Box::new(undo)
        }
    }

    pub fn redo(&self, project: &mut Project) {
        let func = &self.redo_func;
        func(project) 
    }

    pub fn undo(&self, project: &mut Project) {
        let func = &self.undo_func;
        func(project); 
    }

}

pub struct Action {
    pub actions: Vec<ObjAction>
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

    pub fn add_list(&mut self, mut acts: Vec<ObjAction>) {
        self.actions.append(&mut acts);
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
