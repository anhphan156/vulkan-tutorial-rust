#version 450

layout(location = 0) out vec4 out_color;
layout(location = 0) in vec3 fragColor;

void main(){
  out_color = vec4(fragColor, 1.0);
}
