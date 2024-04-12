
mediump vec4 blend(mediump vec4 bottomColor, mediump vec4 topColor) {
    mediump vec3 topHSL = rbgToHsl(topColor.xyz);
    mediump vec3 bottomHSL = rbgToHsl(bottomColor.xyz);
    mediump vec3 blendHSL = vec3(topHSL.x, topHSL.y, bottomHSL.z);
    return vec4(hslToRgb(blendHSL), bottomColor.w);
}