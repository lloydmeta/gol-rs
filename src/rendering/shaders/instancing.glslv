#version 150 core

in vec2 a_Position;
in vec2 a_Translate;
in vec4 a_Color;

uniform mat2 u_Scale;

out vec4 v_Color;

void main() {
    gl_Position = vec4((u_Scale * a_Position) + a_Translate, 0.0, 1.0);
    v_Color = a_Color;
}
