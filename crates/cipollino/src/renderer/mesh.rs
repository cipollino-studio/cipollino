use std::sync::Arc;

use glow::{Context, HasContext, NativeVertexArray, NativeBuffer};

fn to_byte_slice<'a, T>(floats: &'a [T]) -> &'a [u8] {
    unsafe {
        std::slice::from_raw_parts(floats.as_ptr() as *const _, floats.len() * 4)
    }
}

#[derive(Clone)]
pub struct Mesh {

    vbo: NativeBuffer,
    ebo: NativeBuffer,
    vao: NativeVertexArray,

    tris: u32,

    attribs: Vec<u32>,
    vals_per_vert: u32
}

impl Mesh {

    pub fn new(attribs: Vec<u32>, gl: &Arc<Context>) -> Mesh {
        unsafe {
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));
            let vbo = gl.create_buffer().unwrap();
            let ebo = gl.create_buffer().unwrap();
            
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &[], glow::STATIC_DRAW);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &[], glow::STATIC_DRAW);
            
            Mesh {
                vbo,
                ebo,
                vao,
                tris: 0,
                attribs: attribs.clone(),
                vals_per_vert: attribs.iter().sum() 
            }   
        }
    }

    pub fn upload(&mut self, verts: &Vec<f32>, tris: &Vec<u32>, gl: &Arc<Context>) {
        if tris.len() == 0 || verts.len() == 0 {
            return;
        }
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, to_byte_slice(verts.as_slice()), glow::STATIC_DRAW);
            self.config_attribs(gl);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, to_byte_slice(tris.as_slice()), glow::STATIC_DRAW);
        } 
        self.tris = (tris.len() / 3) as u32;
    }

    pub fn render(&self, gl: &Arc<Context>) {
        self.render_with_mode(gl, glow::TRIANGLES); 
    }

    pub fn render_lines(&self, gl: &Arc<Context>) {
        self.render_with_mode(gl, glow::LINES); 
    }
    
    pub fn render_with_mode(&self, gl: &Arc<Context>, mode: u32) {
        if self.tris == 0 {
            return;
        }
        unsafe {
            gl.bind_vertex_array(Some(self.vao)); 
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            self.config_attribs(gl);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
            gl.draw_elements(mode, (self.tris * 3) as i32, glow::UNSIGNED_INT, 0);
        }
    }

    fn config_attribs(&self, gl: &Arc<Context>) {
        self.attribs.iter().fold((0, 0), |(i, offset), attrib| {
            unsafe {
                gl.vertex_attrib_pointer_f32(i, *attrib as i32, glow::FLOAT, false, (self.vals_per_vert * 4) as i32, offset * 4);
                gl.enable_vertex_attrib_array(i);
            }
            (i + 1, offset + (*attrib as i32)) 
        });
    }

    pub fn delete(&self, gl: &Arc<Context>) {
        unsafe {
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
            gl.delete_buffer(self.ebo);
        }
    }

}
