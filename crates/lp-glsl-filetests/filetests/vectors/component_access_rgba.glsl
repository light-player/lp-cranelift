// test compile

float main() {
    vec4 color = vec4(0.5, 0.6, 0.7, 1.0);
    float red = color.r;
    float alpha = color.a;
    return red + alpha;  // 1.5
}

