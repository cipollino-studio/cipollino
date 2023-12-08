use std::sync::Arc;

use glow::{Context, HasContext, NativeShader, NativeProgram};

pub struct Shader {
    program: NativeProgram 
}

fn make_shader(src: &str, frag: bool, gl: &Arc<Context>) -> NativeShader {
    unsafe {
        let shader = gl.create_shader(if frag { glow::FRAGMENT_SHADER } else { glow::VERTEX_SHADER }).unwrap();
        gl.shader_source(shader, src); 
        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            println!("{} shader error:\n{}", if frag { "Fragment" } else { "Vertex" }, gl.get_shader_info_log(shader));
            panic!("Shader failed to compile.");
        }
        shader
    }
}

impl Shader {

    pub fn new(vert_src: &str, frag_src: &str, gl: &Arc<Context>) -> Shader {
        unsafe {
            let program = gl.create_program().unwrap();
            let vert_shader = make_shader(vert_src, false, gl);
            let frag_shader = make_shader(frag_src, true, gl);
            gl.attach_shader(program, vert_shader);
            gl.attach_shader(program, frag_shader);
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("Shader linking error.\n{}", gl.get_program_info_log(program));
            }
            gl.delete_shader(vert_shader);
            gl.delete_shader(frag_shader);
            Shader {
                program: program
            }
        }
    }

    pub fn enable(&self, gl: &Arc<Context>) {
        unsafe {
            gl.use_program(Some(self.program));
        }
    }

    pub fn set_mat4(&mut self, name: &str, mat: &glam::f32::Mat4, gl: &Arc<Context>) {
        unsafe {
            if let Some(uniform_loc) = gl.get_uniform_location(self.program, name) {
                gl.uniform_matrix_4_f32_slice(Some(&uniform_loc), false, mat.to_cols_array().as_slice()); 
            }
        }
    }

    pub fn set_int(&mut self, name: &str, val: i32, gl: &Arc<Context>) {
        unsafe {
            if let Some(uniform_loc) = gl.get_uniform_location(self.program, name) {
                gl.uniform_1_i32(Some(&uniform_loc), val); 
            }
        }
    }
    
    pub fn set_vec4(&mut self, name: &str, val: glam::Vec4, gl: &Arc<Context>) {
        unsafe {
            if let Some(uniform_loc) = gl.get_uniform_location(self.program, name) {
                gl.uniform_4_f32_slice(Some(&uniform_loc), &[val.x, val.y, val.z, val.w]); 
            }
        }
    }

    pub fn delete(&self, gl: &Arc<Context>) {
        unsafe {
            gl.delete_program(self.program);
        }
    }

}

