
#version 100

attribute vec2 aPos;

uniform mat4 uTrans;

void main() {
    if(abs(aPos.x) > 0.75 || abs(aPos.y) > 0.75) {
        gl_Position = vec4(aPos, 0.0, 1.0);
    } else {
        gl_Position = uTrans * vec4(aPos, 0.0, 1.0);                
    }
}
