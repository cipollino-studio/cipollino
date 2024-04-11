use std::{cell::RefCell, ops::{Deref, DerefMut}, rc::Rc, sync::Arc};

use crate::renderer::fb::Framebuffer;

pub struct FramebufferManager {
    framebuffers: Rc<RefCell<Vec<Framebuffer>>>
}

impl FramebufferManager {

    pub fn new() -> Self {
        Self {
            framebuffers: Rc::new(RefCell::new(Vec::new())) 
        }
    }

    pub fn alloc(&mut self, w: u32, h: u32, gl: &Arc<glow::Context>) -> FramebufferRef {
        let fb = if let Some(mut fb) = self.framebuffers.borrow_mut().pop() {
            fb.resize(w, h, gl);
            fb
        } else {
            let fb = Framebuffer::new(w, h, gl);
            fb
        };
        FramebufferRef::new(self.framebuffers.clone(), fb)
    }

}

pub struct FramebufferRef {
    fb: Option<Framebuffer>, 
    framebuffers: Rc<RefCell<Vec<Framebuffer>>>
}

impl FramebufferRef {

    pub fn new(framebuffers: Rc<RefCell<Vec<Framebuffer>>>, fb: Framebuffer) -> Self {
        Self {
            fb: Some(fb),
            framebuffers
        }
    }

}

impl Drop for FramebufferRef {

    fn drop(&mut self) {
        self.framebuffers.borrow_mut().push(std::mem::replace(&mut self.fb, None).unwrap())
    }

}

impl Deref for FramebufferRef {
    type Target = Framebuffer;
    
    fn deref(&self) -> &Self::Target {
        self.fb.as_ref().unwrap()
    }

}

impl DerefMut for FramebufferRef {

    fn deref_mut(&mut self) -> &mut Self::Target {
        self.fb.as_mut().unwrap()
    }

}
