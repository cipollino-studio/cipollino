mediump float blendf(mediump float bottom, mediump float top) {
    if(top < 0.5) {
        return 2.0 * top * bottom;
    } else {
        return 1.0 - 2.0 * (1.0 - top) * (1.0 - bottom);
    }
}

mediump vec4 blend(mediump vec4 bottomColor, mediump vec4 topColor) {
    return COMPONENT_BLEND(bottomColor, topColor, blendf); 
}