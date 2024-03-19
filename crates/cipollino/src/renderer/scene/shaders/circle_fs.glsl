
#version 100

uniform highp vec4 uColor;

varying mediump vec2 pUv;

void main() {
    gl_FragColor = uColor;
    if(length(pUv) > 0.5) {
        gl_FragColor = vec4(0.0);
    }
}
