
#version 100

mediump float luminance(mediump vec4 color) {
    return dot(color.xyz, vec3(0.299, 0.587, 0.114));
}

mediump vec3 rbgToHsl(mediump vec3 color) {
    mediump vec3 hsl;
    mediump float fmin = min(min(color.x, color.y), color.z);
    mediump float fmax = max(max(color.x, color.y), color.z);
    mediump float delta = fmax - fmin;
    hsl.z = (fmin + fmax) / 2.0;

    if(delta == 0.0) {
        hsl.x = 0.0;
        hsl.y = 0.0;
        return hsl;
    } 

    if(hsl.z < 0.5)
        hsl.y = delta / (fmax + fmin);
    else
        hsl.y = delta / (2.0 - fmax - fmin);
    
    mediump float deltaR = (((fmax - color.r) / 6.0) + (delta / 2.0)) / delta;
    mediump float deltaG = (((fmax - color.g) / 6.0) + (delta / 2.0)) / delta;
    mediump float deltaB = (((fmax - color.b) / 6.0) + (delta / 2.0)) / delta;

    if(color.r == fmax)
        hsl.x = deltaB - deltaG;
    else if(color.g == fmax)
        hsl.x = (1.0 / 3.0) + deltaR - deltaB;
    else if(color.b == fmax)
        hsl.x = (2.0 / 3.0) + deltaG - deltaR;
    
    if(hsl.x < 0.0)
        hsl.x += 1.0;
    else if(hsl.x > 1.0)
        hsl.x -= 1.0;
    
    return hsl;
}

mediump float hueToRgb(mediump float f1, mediump float f2, mediump float hue) {
    if(hue < 0.0)
        hue += 1.0;
    else if(hue > 1.0)
        hue -= 1.0;

    mediump float res;
    if((6.0 * hue) < 1.0)
        res = f1 + (f2 - f1) * 6.0 * hue;
    else if((2.0 * hue) < 1.0)
        res = f2;
    else if((3.0 * hue) < 2.0)
        res = f1 + (f2 - f1) * ((2.0 / 3.0) - hue) * 6.0;
    else
        res = f1;
    return res;
}

mediump vec3 hslToRgb(mediump vec3 hsl) {

    if(hsl.y == 0.0)
        return vec3(hsl.z);

    mediump float f2;
    if(hsl.z < 0.5)
        f2 = hsl.z * (1.0 + hsl.y);
    else
        f2 = (hsl.z + hsl.y) - (hsl.y * hsl.z);
    mediump float f1 = 2.0 * hsl.z - f2;

    return vec3(
        hueToRgb(f1, f2, hsl.x + (1.0 / 3.0)),
        hueToRgb(f1, f2, hsl.x),
        hueToRgb(f1, f2, hsl.x - (1.0 / 3.0))
    );
}

#define COMPONENT_BLEND(bottomColor, topColor, func) vec4(func(bottomColor.x, topColor.x), func(bottomColor.y, topColor.y), func(bottomColor.z, topColor.z), func(bottomColor.w, topColor.w))
