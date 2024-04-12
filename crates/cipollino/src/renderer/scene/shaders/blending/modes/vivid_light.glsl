mediump float blendf(mediump float bottom, mediump float top) {
    if(top < 0.5) {
        return 1.0 - (1.0 - bottom) / (0.001 + top);
    } else {
        return bottom / (1.0001 - top);
    }
}

mediump vec4 blend(mediump vec4 bottomColor, mediump vec4 topColor) {
    return COMPONENT_BLEND(bottomColor, topColor, blendf); 
}