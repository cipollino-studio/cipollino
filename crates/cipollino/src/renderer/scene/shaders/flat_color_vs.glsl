
#version 100

attribute vec2 aPos;

uniform mat4 uTrans;

void main() {
    gl_Position = uTrans * vec4(aPos, 0.0, 1.0);                
} 
