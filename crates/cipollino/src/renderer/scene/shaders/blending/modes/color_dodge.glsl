
mediump float blendf(mediump float bottom, mediump float top) {
    return top / (1.0001 - bottom);
}

mediump vec4 blend(mediump vec4 bottomColor, mediump vec4 topColor) {
    return COMPONENT_BLEND(bottomColor, topColor, blendf);
}