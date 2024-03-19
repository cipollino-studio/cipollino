
#version 100

uniform sampler2D uTex;
varying mediump vec2 pUv;

void main() {
    gl_FragColor = texture2D(uTex, pUv); 
}
