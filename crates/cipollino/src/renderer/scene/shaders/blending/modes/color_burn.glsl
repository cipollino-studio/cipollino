mediump float blendf(mediump float bottom, mediump float top) {
    return 1.0 - (1.0 - bottom) / (0.0001 + top);
}

mediump vec4 blend(mediump vec4 bottomColor, mediump vec4 topColor) {
    return COMPONENT_BLEND(bottomColor, topColor, blendf);
}