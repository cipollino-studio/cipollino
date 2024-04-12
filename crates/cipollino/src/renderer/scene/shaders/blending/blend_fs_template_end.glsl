
uniform sampler2D uTopLayer;
uniform sampler2D uBottomLayer;
varying mediump vec2 pUv;
uniform mediump float uLayerAlpha;

void main() {
    mediump vec4 bottomColor = texture2D(uBottomLayer, pUv); 
    mediump vec4 topColor = texture2D(uTopLayer, pUv); 
    mediump float alpha = topColor.w * uLayerAlpha;
    mediump vec4 color = blend(bottomColor, topColor);
    gl_FragColor = bottomColor * (1.0 - alpha) + color * alpha;
}
