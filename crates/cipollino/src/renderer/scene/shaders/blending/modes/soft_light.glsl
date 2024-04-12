mediump float blendf(mediump float bottom, mediump float top) {
    if(bottom < 0.5) {
        return 2.0 * top * bottom + top * top * (1.0 - 2.0 * bottom);
    } else {
        return sqrt(top) * (2.0 * bottom - 1.0) + 2.0 * top * (1.0 - bottom);
    }
}

mediump vec4 blend(mediump vec4 bottomColor, mediump vec4 topColor) {
    return COMPONENT_BLEND(bottomColor, topColor, blendf);
}