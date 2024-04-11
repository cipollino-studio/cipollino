mediump float luminance = dot(topColor.xyz, vec3(0.299, 0.587, 0.114));
mediump vec4 color;
if(luminance < 0.5) {
    color = 2.0 * topColor * bottomColor;
} else {
    color = 1.0 - 2.0 * (1.0 - topColor) * (1.0 - bottomColor);
}