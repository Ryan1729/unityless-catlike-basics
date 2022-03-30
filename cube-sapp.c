//------------------------------------------------------------------------------
//  shadows-sapp.c
//  Render to an offscreen rendertarget texture, and use this texture
//  for rendering shadows to the screen.
//------------------------------------------------------------------------------
#define SOKOL_IMPL
#include "sokol_app.h"
#include "sokol_gfx.h"
#include "sokol_glue.h"
#define HANDMADE_MATH_IMPLEMENTATION
#define HANDMADE_MATH_NO_SSE
#include "HandmadeMath.h"

#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <stddef.h>
#if !defined(SOKOL_SHDC_ALIGN)
  #if defined(_MSC_VER)
    #define SOKOL_SHDC_ALIGN(a) __declspec(align(a))
  #else
    #define SOKOL_SHDC_ALIGN(a) __attribute__((aligned(a)))
  #endif
#endif
#define ATTR_shadowVS_position (0)
#define ATTR_colorVS_position (0)
#define ATTR_colorVS_normal (1)
#define SLOT_shadowMap (0)
#define SLOT_vs_shadow_params (0)
#pragma pack(push,1)
SOKOL_SHDC_ALIGN(16) typedef struct vs_shadow_params_t {
    hmm_mat4 mvp;
} vs_shadow_params_t;
#pragma pack(pop)
#define SLOT_vs_light_params (0)
#pragma pack(push,1)
SOKOL_SHDC_ALIGN(16) typedef struct vs_light_params_t {
    hmm_mat4 model;
    hmm_mat4 mvp;
    hmm_mat4 lightMVP;
    hmm_vec3 diffColor;
    uint8_t _pad_204[4];
} vs_light_params_t;
#pragma pack(pop)
#define SLOT_fs_light_params (0)
#pragma pack(push,1)
SOKOL_SHDC_ALIGN(16) typedef struct fs_light_params_t {
    hmm_vec2 shadowMapSize;
    uint8_t _pad_8[8];
    hmm_vec3 lightDir;
    uint8_t _pad_28[4];
    hmm_vec3 eyePos;
    uint8_t _pad_44[4];
} fs_light_params_t;
#pragma pack(pop)
/*
    #version 330
    
    uniform vec4 vs_shadow_params[4];
    layout(location = 0) in vec4 position;
    out vec2 projZW;
    
    void main()
    {
        gl_Position = mat4(vs_shadow_params[0], vs_shadow_params[1], vs_shadow_params[2], vs_shadow_params[3]) * position;
        projZW = gl_Position.zw;
    }
    
*/
static const char shadowVS_source_glsl330[271] = {
    0x23,0x76,0x65,0x72,0x73,0x69,0x6f,0x6e,0x20,0x33,0x33,0x30,0x0a,0x0a,0x75,0x6e,
    0x69,0x66,0x6f,0x72,0x6d,0x20,0x76,0x65,0x63,0x34,0x20,0x76,0x73,0x5f,0x73,0x68,
    0x61,0x64,0x6f,0x77,0x5f,0x70,0x61,0x72,0x61,0x6d,0x73,0x5b,0x34,0x5d,0x3b,0x0a,
    0x6c,0x61,0x79,0x6f,0x75,0x74,0x28,0x6c,0x6f,0x63,0x61,0x74,0x69,0x6f,0x6e,0x20,
    0x3d,0x20,0x30,0x29,0x20,0x69,0x6e,0x20,0x76,0x65,0x63,0x34,0x20,0x70,0x6f,0x73,
    0x69,0x74,0x69,0x6f,0x6e,0x3b,0x0a,0x6f,0x75,0x74,0x20,0x76,0x65,0x63,0x32,0x20,
    0x70,0x72,0x6f,0x6a,0x5a,0x57,0x3b,0x0a,0x0a,0x76,0x6f,0x69,0x64,0x20,0x6d,0x61,
    0x69,0x6e,0x28,0x29,0x0a,0x7b,0x0a,0x20,0x20,0x20,0x20,0x67,0x6c,0x5f,0x50,0x6f,
    0x73,0x69,0x74,0x69,0x6f,0x6e,0x20,0x3d,0x20,0x6d,0x61,0x74,0x34,0x28,0x76,0x73,
    0x5f,0x73,0x68,0x61,0x64,0x6f,0x77,0x5f,0x70,0x61,0x72,0x61,0x6d,0x73,0x5b,0x30,
    0x5d,0x2c,0x20,0x76,0x73,0x5f,0x73,0x68,0x61,0x64,0x6f,0x77,0x5f,0x70,0x61,0x72,
    0x61,0x6d,0x73,0x5b,0x31,0x5d,0x2c,0x20,0x76,0x73,0x5f,0x73,0x68,0x61,0x64,0x6f,
    0x77,0x5f,0x70,0x61,0x72,0x61,0x6d,0x73,0x5b,0x32,0x5d,0x2c,0x20,0x76,0x73,0x5f,
    0x73,0x68,0x61,0x64,0x6f,0x77,0x5f,0x70,0x61,0x72,0x61,0x6d,0x73,0x5b,0x33,0x5d,
    0x29,0x20,0x2a,0x20,0x70,0x6f,0x73,0x69,0x74,0x69,0x6f,0x6e,0x3b,0x0a,0x20,0x20,
    0x20,0x20,0x70,0x72,0x6f,0x6a,0x5a,0x57,0x20,0x3d,0x20,0x67,0x6c,0x5f,0x50,0x6f,
    0x73,0x69,0x74,0x69,0x6f,0x6e,0x2e,0x7a,0x77,0x3b,0x0a,0x7d,0x0a,0x0a,0x00,
};
/*
    #version 330
    
    in vec2 projZW;
    layout(location = 0) out vec4 fragColor;
    
    vec4 encodeDepth(float v)
    {
        vec4 _25 = fract(vec4(1.0, 255.0, 65025.0, 16581375.0) * v);
        return _25 - (_25.yzww * vec4(0.0039215688593685626983642578125, 0.0039215688593685626983642578125, 0.0039215688593685626983642578125, 0.0));
    }
    
    void main()
    {
        float param = projZW.x / projZW.y;
        fragColor = encodeDepth(param);
    }
    
*/
static const char shadowFS_source_glsl330[407] = {
    0x23,0x76,0x65,0x72,0x73,0x69,0x6f,0x6e,0x20,0x33,0x33,0x30,0x0a,0x0a,0x69,0x6e,
    0x20,0x76,0x65,0x63,0x32,0x20,0x70,0x72,0x6f,0x6a,0x5a,0x57,0x3b,0x0a,0x6c,0x61,
    0x79,0x6f,0x75,0x74,0x28,0x6c,0x6f,0x63,0x61,0x74,0x69,0x6f,0x6e,0x20,0x3d,0x20,
    0x30,0x29,0x20,0x6f,0x75,0x74,0x20,0x76,0x65,0x63,0x34,0x20,0x66,0x72,0x61,0x67,
    0x43,0x6f,0x6c,0x6f,0x72,0x3b,0x0a,0x0a,0x76,0x65,0x63,0x34,0x20,0x65,0x6e,0x63,
    0x6f,0x64,0x65,0x44,0x65,0x70,0x74,0x68,0x28,0x66,0x6c,0x6f,0x61,0x74,0x20,0x76,
    0x29,0x0a,0x7b,0x0a,0x20,0x20,0x20,0x20,0x76,0x65,0x63,0x34,0x20,0x5f,0x32,0x35,
    0x20,0x3d,0x20,0x66,0x72,0x61,0x63,0x74,0x28,0x76,0x65,0x63,0x34,0x28,0x31,0x2e,
    0x30,0x2c,0x20,0x32,0x35,0x35,0x2e,0x30,0x2c,0x20,0x36,0x35,0x30,0x32,0x35,0x2e,
    0x30,0x2c,0x20,0x31,0x36,0x35,0x38,0x31,0x33,0x37,0x35,0x2e,0x30,0x29,0x20,0x2a,
    0x20,0x76,0x29,0x3b,0x0a,0x20,0x20,0x20,0x20,0x72,0x65,0x74,0x75,0x72,0x6e,0x20,
    0x5f,0x32,0x35,0x20,0x2d,0x20,0x28,0x5f,0x32,0x35,0x2e,0x79,0x7a,0x77,0x77,0x20,
    0x2a,0x20,0x76,0x65,0x63,0x34,0x28,0x30,0x2e,0x30,0x30,0x33,0x39,0x32,0x31,0x35,
    0x36,0x38,0x38,0x35,0x39,0x33,0x36,0x38,0x35,0x36,0x32,0x36,0x39,0x38,0x33,0x36,
    0x34,0x32,0x35,0x37,0x38,0x31,0x32,0x35,0x2c,0x20,0x30,0x2e,0x30,0x30,0x33,0x39,
    0x32,0x31,0x35,0x36,0x38,0x38,0x35,0x39,0x33,0x36,0x38,0x35,0x36,0x32,0x36,0x39,
    0x38,0x33,0x36,0x34,0x32,0x35,0x37,0x38,0x31,0x32,0x35,0x2c,0x20,0x30,0x2e,0x30,
    0x30,0x33,0x39,0x32,0x31,0x35,0x36,0x38,0x38,0x35,0x39,0x33,0x36,0x38,0x35,0x36,
    0x32,0x36,0x39,0x38,0x33,0x36,0x34,0x32,0x35,0x37,0x38,0x31,0x32,0x35,0x2c,0x20,
    0x30,0x2e,0x30,0x29,0x29,0x3b,0x0a,0x7d,0x0a,0x0a,0x76,0x6f,0x69,0x64,0x20,0x6d,
    0x61,0x69,0x6e,0x28,0x29,0x0a,0x7b,0x0a,0x20,0x20,0x20,0x20,0x66,0x6c,0x6f,0x61,
    0x74,0x20,0x70,0x61,0x72,0x61,0x6d,0x20,0x3d,0x20,0x70,0x72,0x6f,0x6a,0x5a,0x57,
    0x2e,0x78,0x20,0x2f,0x20,0x70,0x72,0x6f,0x6a,0x5a,0x57,0x2e,0x79,0x3b,0x0a,0x20,
    0x20,0x20,0x20,0x66,0x72,0x61,0x67,0x43,0x6f,0x6c,0x6f,0x72,0x20,0x3d,0x20,0x65,
    0x6e,0x63,0x6f,0x64,0x65,0x44,0x65,0x70,0x74,0x68,0x28,0x70,0x61,0x72,0x61,0x6d,
    0x29,0x3b,0x0a,0x7d,0x0a,0x0a,0x00,
};
/*
    #version 330
    
    uniform vec4 vs_light_params[13];
    layout(location = 0) in vec4 position;
    out vec4 lightProjPos;
    out vec4 P;
    out vec3 N;
    layout(location = 1) in vec3 normal;
    out vec3 color;
    
    void main()
    {
        gl_Position = mat4(vs_light_params[4], vs_light_params[5], vs_light_params[6], vs_light_params[7]) * position;
        lightProjPos = mat4(vs_light_params[8], vs_light_params[9], vs_light_params[10], vs_light_params[11]) * position;
        mat4 _41 = mat4(vs_light_params[0], vs_light_params[1], vs_light_params[2], vs_light_params[3]);
        P = _41 * position;
        N = (_41 * vec4(normal, 0.0)).xyz;
        color = vs_light_params[12].xyz;
    }
    
*/
static const char colorVS_source_glsl330[640] = {
    0x23,0x76,0x65,0x72,0x73,0x69,0x6f,0x6e,0x20,0x33,0x33,0x30,0x0a,0x0a,0x75,0x6e,
    0x69,0x66,0x6f,0x72,0x6d,0x20,0x76,0x65,0x63,0x34,0x20,0x76,0x73,0x5f,0x6c,0x69,
    0x67,0x68,0x74,0x5f,0x70,0x61,0x72,0x61,0x6d,0x73,0x5b,0x31,0x33,0x5d,0x3b,0x0a,
    0x6c,0x61,0x79,0x6f,0x75,0x74,0x28,0x6c,0x6f,0x63,0x61,0x74,0x69,0x6f,0x6e,0x20,
    0x3d,0x20,0x30,0x29,0x20,0x69,0x6e,0x20,0x76,0x65,0x63,0x34,0x20,0x70,0x6f,0x73,
    0x69,0x74,0x69,0x6f,0x6e,0x3b,0x0a,0x6f,0x75,0x74,0x20,0x76,0x65,0x63,0x34,0x20,
    0x6c,0x69,0x67,0x68,0x74,0x50,0x72,0x6f,0x6a,0x50,0x6f,0x73,0x3b,0x0a,0x6f,0x75,
    0x74,0x20,0x76,0x65,0x63,0x34,0x20,0x50,0x3b,0x0a,0x6f,0x75,0x74,0x20,0x76,0x65,
    0x63,0x33,0x20,0x4e,0x3b,0x0a,0x6c,0x61,0x79,0x6f,0x75,0x74,0x28,0x6c,0x6f,0x63,
    0x61,0x74,0x69,0x6f,0x6e,0x20,0x3d,0x20,0x31,0x29,0x20,0x69,0x6e,0x20,0x76,0x65,
    0x63,0x33,0x20,0x6e,0x6f,0x72,0x6d,0x61,0x6c,0x3b,0x0a,0x6f,0x75,0x74,0x20,0x76,
    0x65,0x63,0x33,0x20,0x63,0x6f,0x6c,0x6f,0x72,0x3b,0x0a,0x0a,0x76,0x6f,0x69,0x64,
    0x20,0x6d,0x61,0x69,0x6e,0x28,0x29,0x0a,0x7b,0x0a,0x20,0x20,0x20,0x20,0x67,0x6c,
    0x5f,0x50,0x6f,0x73,0x69,0x74,0x69,0x6f,0x6e,0x20,0x3d,0x20,0x6d,0x61,0x74,0x34,
    0x28,0x76,0x73,0x5f,0x6c,0x69,0x67,0x68,0x74,0x5f,0x70,0x61,0x72,0x61,0x6d,0x73,
    0x5b,0x34,0x5d,0x2c,0x20,0x76,0x73,0x5f,0x6c,0x69,0x67,0x68,0x74,0x5f,0x70,0x61,
    0x72,0x61,0x6d,0x73,0x5b,0x35,0x5d,0x2c,0x20,0x76,0x73,0x5f,0x6c,0x69,0x67,0x68,
    0x74,0x5f,0x70,0x61,0x72,0x61,0x6d,0x73,0x5b,0x36,0x5d,0x2c,0x20,0x76,0x73,0x5f,
    0x6c,0x69,0x67,0x68,0x74,0x5f,0x70,0x61,0x72,0x61,0x6d,0x73,0x5b,0x37,0x5d,0x29,
    0x20,0x2a,0x20,0x70,0x6f,0x73,0x69,0x74,0x69,0x6f,0x6e,0x3b,0x0a,0x20,0x20,0x20,
    0x20,0x6c,0x69,0x67,0x68,0x74,0x50,0x72,0x6f,0x6a,0x50,0x6f,0x73,0x20,0x3d,0x20,
    0x6d,0x61,0x74,0x34,0x28,0x76,0x73,0x5f,0x6c,0x69,0x67,0x68,0x74,0x5f,0x70,0x61,
    0x72,0x61,0x6d,0x73,0x5b,0x38,0x5d,0x2c,0x20,0x76,0x73,0x5f,0x6c,0x69,0x67,0x68,
    0x74,0x5f,0x70,0x61,0x72,0x61,0x6d,0x73,0x5b,0x39,0x5d,0x2c,0x20,0x76,0x73,0x5f,
    0x6c,0x69,0x67,0x68,0x74,0x5f,0x70,0x61,0x72,0x61,0x6d,0x73,0x5b,0x31,0x30,0x5d,
    0x2c,0x20,0x76,0x73,0x5f,0x6c,0x69,0x67,0x68,0x74,0x5f,0x70,0x61,0x72,0x61,0x6d,
    0x73,0x5b,0x31,0x31,0x5d,0x29,0x20,0x2a,0x20,0x70,0x6f,0x73,0x69,0x74,0x69,0x6f,
    0x6e,0x3b,0x0a,0x20,0x20,0x20,0x20,0x6d,0x61,0x74,0x34,0x20,0x5f,0x34,0x31,0x20,
    0x3d,0x20,0x6d,0x61,0x74,0x34,0x28,0x76,0x73,0x5f,0x6c,0x69,0x67,0x68,0x74,0x5f,
    0x70,0x61,0x72,0x61,0x6d,0x73,0x5b,0x30,0x5d,0x2c,0x20,0x76,0x73,0x5f,0x6c,0x69,
    0x67,0x68,0x74,0x5f,0x70,0x61,0x72,0x61,0x6d,0x73,0x5b,0x31,0x5d,0x2c,0x20,0x76,
    0x73,0x5f,0x6c,0x69,0x67,0x68,0x74,0x5f,0x70,0x61,0x72,0x61,0x6d,0x73,0x5b,0x32,
    0x5d,0x2c,0x20,0x76,0x73,0x5f,0x6c,0x69,0x67,0x68,0x74,0x5f,0x70,0x61,0x72,0x61,
    0x6d,0x73,0x5b,0x33,0x5d,0x29,0x3b,0x0a,0x20,0x20,0x20,0x20,0x50,0x20,0x3d,0x20,
    0x5f,0x34,0x31,0x20,0x2a,0x20,0x70,0x6f,0x73,0x69,0x74,0x69,0x6f,0x6e,0x3b,0x0a,
    0x20,0x20,0x20,0x20,0x4e,0x20,0x3d,0x20,0x28,0x5f,0x34,0x31,0x20,0x2a,0x20,0x76,
    0x65,0x63,0x34,0x28,0x6e,0x6f,0x72,0x6d,0x61,0x6c,0x2c,0x20,0x30,0x2e,0x30,0x29,
    0x29,0x2e,0x78,0x79,0x7a,0x3b,0x0a,0x20,0x20,0x20,0x20,0x63,0x6f,0x6c,0x6f,0x72,
    0x20,0x3d,0x20,0x76,0x73,0x5f,0x6c,0x69,0x67,0x68,0x74,0x5f,0x70,0x61,0x72,0x61,
    0x6d,0x73,0x5b,0x31,0x32,0x5d,0x2e,0x78,0x79,0x7a,0x3b,0x0a,0x7d,0x0a,0x0a,0x00,

};
/*
    #version 330
    
    uniform vec4 fs_light_params[3];
    uniform sampler2D shadowMap;
    
    in vec3 N;
    in vec4 lightProjPos;
    in vec4 P;
    layout(location = 0) out vec4 fragColor;
    in vec3 color;
    
    float decodeDepth(vec4 rgba)
    {
        return dot(rgba, vec4(1.0, 0.0039215688593685626983642578125, 1.5378700481960549950599670410156e-05, 6.0308629201699659461155533790588e-08));
    }
    
    float sampleShadow(sampler2D shadowMap_1, vec2 uv, float compare)
    {
        vec4 param = texture(shadowMap_1, vec2(uv.x, uv.y));
        return step(compare, decodeDepth(param) + 0.001000000047497451305389404296875);
    }
    
    float sampleShadowPCF(sampler2D shadowMap_1, vec2 uv, vec2 smSize, float compare)
    {
        float result = 0.0;
        for (int x = -2; x <= 2; x++)
        {
            for (int y = -2; y <= 2; y++)
            {
                vec2 param = uv + (vec2(float(x), float(y)) / smSize);
                float param_1 = compare;
                result += sampleShadow(shadowMap_1, param, param_1);
            }
        }
        return result * 0.039999999105930328369140625;
    }
    
    vec4 gamma(vec4 c)
    {
        return vec4(pow(c.xyz, vec3(0.4545454680919647216796875)), c.w);
    }
    
    void main()
    {
        vec3 _149 = normalize(fs_light_params[1].xyz);
        vec3 _154 = normalize(N);
        float _158 = dot(_154, _149);
        if (_158 > 0.0)
        {
            vec3 _172 = lightProjPos.xyz / vec3(lightProjPos.w);
            vec2 param = (_172.xy + vec2(1.0)) * 0.5;
            vec2 param_1 = fs_light_params[0].xy;
            float param_2 = _172.z;
            float _195 = sampleShadowPCF(shadowMap, param, param_1, param_2);
            fragColor = vec4(vec3((pow(max(dot(reflect(-_149, _154), normalize(fs_light_params[2].xyz - P.xyz)), 0.0), 2.2000000476837158203125) * _158) * _195) + (color * (max(_158 * _195, 0.0) + 0.25)), 1.0);
        }
        else
        {
            fragColor = vec4(color * 0.25, 1.0);
        }
        vec4 param_3 = fragColor;
        fragColor = gamma(param_3);
    }
    
*/
static const char colorFS_source_glsl330[1861] = {
    0x23,0x76,0x65,0x72,0x73,0x69,0x6f,0x6e,0x20,0x33,0x33,0x30,0x0a,0x0a,0x75,0x6e,
    0x69,0x66,0x6f,0x72,0x6d,0x20,0x76,0x65,0x63,0x34,0x20,0x66,0x73,0x5f,0x6c,0x69,
    0x67,0x68,0x74,0x5f,0x70,0x61,0x72,0x61,0x6d,0x73,0x5b,0x33,0x5d,0x3b,0x0a,0x75,
    0x6e,0x69,0x66,0x6f,0x72,0x6d,0x20,0x73,0x61,0x6d,0x70,0x6c,0x65,0x72,0x32,0x44,
    0x20,0x73,0x68,0x61,0x64,0x6f,0x77,0x4d,0x61,0x70,0x3b,0x0a,0x0a,0x69,0x6e,0x20,
    0x76,0x65,0x63,0x33,0x20,0x4e,0x3b,0x0a,0x69,0x6e,0x20,0x76,0x65,0x63,0x34,0x20,
    0x6c,0x69,0x67,0x68,0x74,0x50,0x72,0x6f,0x6a,0x50,0x6f,0x73,0x3b,0x0a,0x69,0x6e,
    0x20,0x76,0x65,0x63,0x34,0x20,0x50,0x3b,0x0a,0x6c,0x61,0x79,0x6f,0x75,0x74,0x28,
    0x6c,0x6f,0x63,0x61,0x74,0x69,0x6f,0x6e,0x20,0x3d,0x20,0x30,0x29,0x20,0x6f,0x75,
    0x74,0x20,0x76,0x65,0x63,0x34,0x20,0x66,0x72,0x61,0x67,0x43,0x6f,0x6c,0x6f,0x72,
    0x3b,0x0a,0x69,0x6e,0x20,0x76,0x65,0x63,0x33,0x20,0x63,0x6f,0x6c,0x6f,0x72,0x3b,
    0x0a,0x0a,0x66,0x6c,0x6f,0x61,0x74,0x20,0x64,0x65,0x63,0x6f,0x64,0x65,0x44,0x65,
    0x70,0x74,0x68,0x28,0x76,0x65,0x63,0x34,0x20,0x72,0x67,0x62,0x61,0x29,0x0a,0x7b,
    0x0a,0x20,0x20,0x20,0x20,0x72,0x65,0x74,0x75,0x72,0x6e,0x20,0x64,0x6f,0x74,0x28,
    0x72,0x67,0x62,0x61,0x2c,0x20,0x76,0x65,0x63,0x34,0x28,0x31,0x2e,0x30,0x2c,0x20,
    0x30,0x2e,0x30,0x30,0x33,0x39,0x32,0x31,0x35,0x36,0x38,0x38,0x35,0x39,0x33,0x36,
    0x38,0x35,0x36,0x32,0x36,0x39,0x38,0x33,0x36,0x34,0x32,0x35,0x37,0x38,0x31,0x32,
    0x35,0x2c,0x20,0x31,0x2e,0x35,0x33,0x37,0x38,0x37,0x30,0x30,0x34,0x38,0x31,0x39,
    0x36,0x30,0x35,0x34,0x39,0x39,0x35,0x30,0x35,0x39,0x39,0x36,0x37,0x30,0x34,0x31,
    0x30,0x31,0x35,0x36,0x65,0x2d,0x30,0x35,0x2c,0x20,0x36,0x2e,0x30,0x33,0x30,0x38,
    0x36,0x32,0x39,0x32,0x30,0x31,0x36,0x39,0x39,0x36,0x35,0x39,0x34,0x36,0x31,0x31,
    0x35,0x35,0x35,0x33,0x33,0x37,0x39,0x30,0x35,0x38,0x38,0x65,0x2d,0x30,0x38,0x29,
    0x29,0x3b,0x0a,0x7d,0x0a,0x0a,0x66,0x6c,0x6f,0x61,0x74,0x20,0x73,0x61,0x6d,0x70,
    0x6c,0x65,0x53,0x68,0x61,0x64,0x6f,0x77,0x28,0x73,0x61,0x6d,0x70,0x6c,0x65,0x72,
    0x32,0x44,0x20,0x73,0x68,0x61,0x64,0x6f,0x77,0x4d,0x61,0x70,0x5f,0x31,0x2c,0x20,
    0x76,0x65,0x63,0x32,0x20,0x75,0x76,0x2c,0x20,0x66,0x6c,0x6f,0x61,0x74,0x20,0x63,
    0x6f,0x6d,0x70,0x61,0x72,0x65,0x29,0x0a,0x7b,0x0a,0x20,0x20,0x20,0x20,0x76,0x65,
    0x63,0x34,0x20,0x70,0x61,0x72,0x61,0x6d,0x20,0x3d,0x20,0x74,0x65,0x78,0x74,0x75,
    0x72,0x65,0x28,0x73,0x68,0x61,0x64,0x6f,0x77,0x4d,0x61,0x70,0x5f,0x31,0x2c,0x20,
    0x76,0x65,0x63,0x32,0x28,0x75,0x76,0x2e,0x78,0x2c,0x20,0x75,0x76,0x2e,0x79,0x29,
    0x29,0x3b,0x0a,0x20,0x20,0x20,0x20,0x72,0x65,0x74,0x75,0x72,0x6e,0x20,0x73,0x74,
    0x65,0x70,0x28,0x63,0x6f,0x6d,0x70,0x61,0x72,0x65,0x2c,0x20,0x64,0x65,0x63,0x6f,
    0x64,0x65,0x44,0x65,0x70,0x74,0x68,0x28,0x70,0x61,0x72,0x61,0x6d,0x29,0x20,0x2b,
    0x20,0x30,0x2e,0x30,0x30,0x31,0x30,0x30,0x30,0x30,0x30,0x30,0x30,0x34,0x37,0x34,
    0x39,0x37,0x34,0x35,0x31,0x33,0x30,0x35,0x33,0x38,0x39,0x34,0x30,0x34,0x32,0x39,
    0x36,0x38,0x37,0x35,0x29,0x3b,0x0a,0x7d,0x0a,0x0a,0x66,0x6c,0x6f,0x61,0x74,0x20,
    0x73,0x61,0x6d,0x70,0x6c,0x65,0x53,0x68,0x61,0x64,0x6f,0x77,0x50,0x43,0x46,0x28,
    0x73,0x61,0x6d,0x70,0x6c,0x65,0x72,0x32,0x44,0x20,0x73,0x68,0x61,0x64,0x6f,0x77,
    0x4d,0x61,0x70,0x5f,0x31,0x2c,0x20,0x76,0x65,0x63,0x32,0x20,0x75,0x76,0x2c,0x20,
    0x76,0x65,0x63,0x32,0x20,0x73,0x6d,0x53,0x69,0x7a,0x65,0x2c,0x20,0x66,0x6c,0x6f,
    0x61,0x74,0x20,0x63,0x6f,0x6d,0x70,0x61,0x72,0x65,0x29,0x0a,0x7b,0x0a,0x20,0x20,
    0x20,0x20,0x66,0x6c,0x6f,0x61,0x74,0x20,0x72,0x65,0x73,0x75,0x6c,0x74,0x20,0x3d,
    0x20,0x30,0x2e,0x30,0x3b,0x0a,0x20,0x20,0x20,0x20,0x66,0x6f,0x72,0x20,0x28,0x69,
    0x6e,0x74,0x20,0x78,0x20,0x3d,0x20,0x2d,0x32,0x3b,0x20,0x78,0x20,0x3c,0x3d,0x20,
    0x32,0x3b,0x20,0x78,0x2b,0x2b,0x29,0x0a,0x20,0x20,0x20,0x20,0x7b,0x0a,0x20,0x20,
    0x20,0x20,0x20,0x20,0x20,0x20,0x66,0x6f,0x72,0x20,0x28,0x69,0x6e,0x74,0x20,0x79,
    0x20,0x3d,0x20,0x2d,0x32,0x3b,0x20,0x79,0x20,0x3c,0x3d,0x20,0x32,0x3b,0x20,0x79,
    0x2b,0x2b,0x29,0x0a,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x7b,0x0a,0x20,0x20,
    0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x76,0x65,0x63,0x32,0x20,0x70,
    0x61,0x72,0x61,0x6d,0x20,0x3d,0x20,0x75,0x76,0x20,0x2b,0x20,0x28,0x76,0x65,0x63,
    0x32,0x28,0x66,0x6c,0x6f,0x61,0x74,0x28,0x78,0x29,0x2c,0x20,0x66,0x6c,0x6f,0x61,
    0x74,0x28,0x79,0x29,0x29,0x20,0x2f,0x20,0x73,0x6d,0x53,0x69,0x7a,0x65,0x29,0x3b,
    0x0a,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x66,0x6c,0x6f,
    0x61,0x74,0x20,0x70,0x61,0x72,0x61,0x6d,0x5f,0x31,0x20,0x3d,0x20,0x63,0x6f,0x6d,
    0x70,0x61,0x72,0x65,0x3b,0x0a,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,
    0x20,0x20,0x72,0x65,0x73,0x75,0x6c,0x74,0x20,0x2b,0x3d,0x20,0x73,0x61,0x6d,0x70,
    0x6c,0x65,0x53,0x68,0x61,0x64,0x6f,0x77,0x28,0x73,0x68,0x61,0x64,0x6f,0x77,0x4d,
    0x61,0x70,0x5f,0x31,0x2c,0x20,0x70,0x61,0x72,0x61,0x6d,0x2c,0x20,0x70,0x61,0x72,
    0x61,0x6d,0x5f,0x31,0x29,0x3b,0x0a,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x7d,
    0x0a,0x20,0x20,0x20,0x20,0x7d,0x0a,0x20,0x20,0x20,0x20,0x72,0x65,0x74,0x75,0x72,
    0x6e,0x20,0x72,0x65,0x73,0x75,0x6c,0x74,0x20,0x2a,0x20,0x30,0x2e,0x30,0x33,0x39,
    0x39,0x39,0x39,0x39,0x39,0x39,0x31,0x30,0x35,0x39,0x33,0x30,0x33,0x32,0x38,0x33,
    0x36,0x39,0x31,0x34,0x30,0x36,0x32,0x35,0x3b,0x0a,0x7d,0x0a,0x0a,0x76,0x65,0x63,
    0x34,0x20,0x67,0x61,0x6d,0x6d,0x61,0x28,0x76,0x65,0x63,0x34,0x20,0x63,0x29,0x0a,
    0x7b,0x0a,0x20,0x20,0x20,0x20,0x72,0x65,0x74,0x75,0x72,0x6e,0x20,0x76,0x65,0x63,
    0x34,0x28,0x70,0x6f,0x77,0x28,0x63,0x2e,0x78,0x79,0x7a,0x2c,0x20,0x76,0x65,0x63,
    0x33,0x28,0x30,0x2e,0x34,0x35,0x34,0x35,0x34,0x35,0x34,0x36,0x38,0x30,0x39,0x31,
    0x39,0x36,0x34,0x37,0x32,0x31,0x36,0x37,0x39,0x36,0x38,0x37,0x35,0x29,0x29,0x2c,
    0x20,0x63,0x2e,0x77,0x29,0x3b,0x0a,0x7d,0x0a,0x0a,0x76,0x6f,0x69,0x64,0x20,0x6d,
    0x61,0x69,0x6e,0x28,0x29,0x0a,0x7b,0x0a,0x20,0x20,0x20,0x20,0x76,0x65,0x63,0x33,
    0x20,0x5f,0x31,0x34,0x39,0x20,0x3d,0x20,0x6e,0x6f,0x72,0x6d,0x61,0x6c,0x69,0x7a,
    0x65,0x28,0x66,0x73,0x5f,0x6c,0x69,0x67,0x68,0x74,0x5f,0x70,0x61,0x72,0x61,0x6d,
    0x73,0x5b,0x31,0x5d,0x2e,0x78,0x79,0x7a,0x29,0x3b,0x0a,0x20,0x20,0x20,0x20,0x76,
    0x65,0x63,0x33,0x20,0x5f,0x31,0x35,0x34,0x20,0x3d,0x20,0x6e,0x6f,0x72,0x6d,0x61,
    0x6c,0x69,0x7a,0x65,0x28,0x4e,0x29,0x3b,0x0a,0x20,0x20,0x20,0x20,0x66,0x6c,0x6f,
    0x61,0x74,0x20,0x5f,0x31,0x35,0x38,0x20,0x3d,0x20,0x64,0x6f,0x74,0x28,0x5f,0x31,
    0x35,0x34,0x2c,0x20,0x5f,0x31,0x34,0x39,0x29,0x3b,0x0a,0x20,0x20,0x20,0x20,0x69,
    0x66,0x20,0x28,0x5f,0x31,0x35,0x38,0x20,0x3e,0x20,0x30,0x2e,0x30,0x29,0x0a,0x20,
    0x20,0x20,0x20,0x7b,0x0a,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x76,0x65,0x63,
    0x33,0x20,0x5f,0x31,0x37,0x32,0x20,0x3d,0x20,0x6c,0x69,0x67,0x68,0x74,0x50,0x72,
    0x6f,0x6a,0x50,0x6f,0x73,0x2e,0x78,0x79,0x7a,0x20,0x2f,0x20,0x76,0x65,0x63,0x33,
    0x28,0x6c,0x69,0x67,0x68,0x74,0x50,0x72,0x6f,0x6a,0x50,0x6f,0x73,0x2e,0x77,0x29,
    0x3b,0x0a,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x76,0x65,0x63,0x32,0x20,0x70,
    0x61,0x72,0x61,0x6d,0x20,0x3d,0x20,0x28,0x5f,0x31,0x37,0x32,0x2e,0x78,0x79,0x20,
    0x2b,0x20,0x76,0x65,0x63,0x32,0x28,0x31,0x2e,0x30,0x29,0x29,0x20,0x2a,0x20,0x30,
    0x2e,0x35,0x3b,0x0a,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x76,0x65,0x63,0x32,
    0x20,0x70,0x61,0x72,0x61,0x6d,0x5f,0x31,0x20,0x3d,0x20,0x66,0x73,0x5f,0x6c,0x69,
    0x67,0x68,0x74,0x5f,0x70,0x61,0x72,0x61,0x6d,0x73,0x5b,0x30,0x5d,0x2e,0x78,0x79,
    0x3b,0x0a,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x66,0x6c,0x6f,0x61,0x74,0x20,
    0x70,0x61,0x72,0x61,0x6d,0x5f,0x32,0x20,0x3d,0x20,0x5f,0x31,0x37,0x32,0x2e,0x7a,
    0x3b,0x0a,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x66,0x6c,0x6f,0x61,0x74,0x20,
    0x5f,0x31,0x39,0x35,0x20,0x3d,0x20,0x73,0x61,0x6d,0x70,0x6c,0x65,0x53,0x68,0x61,
    0x64,0x6f,0x77,0x50,0x43,0x46,0x28,0x73,0x68,0x61,0x64,0x6f,0x77,0x4d,0x61,0x70,
    0x2c,0x20,0x70,0x61,0x72,0x61,0x6d,0x2c,0x20,0x70,0x61,0x72,0x61,0x6d,0x5f,0x31,
    0x2c,0x20,0x70,0x61,0x72,0x61,0x6d,0x5f,0x32,0x29,0x3b,0x0a,0x20,0x20,0x20,0x20,
    0x20,0x20,0x20,0x20,0x66,0x72,0x61,0x67,0x43,0x6f,0x6c,0x6f,0x72,0x20,0x3d,0x20,
    0x76,0x65,0x63,0x34,0x28,0x76,0x65,0x63,0x33,0x28,0x28,0x70,0x6f,0x77,0x28,0x6d,
    0x61,0x78,0x28,0x64,0x6f,0x74,0x28,0x72,0x65,0x66,0x6c,0x65,0x63,0x74,0x28,0x2d,
    0x5f,0x31,0x34,0x39,0x2c,0x20,0x5f,0x31,0x35,0x34,0x29,0x2c,0x20,0x6e,0x6f,0x72,
    0x6d,0x61,0x6c,0x69,0x7a,0x65,0x28,0x66,0x73,0x5f,0x6c,0x69,0x67,0x68,0x74,0x5f,
    0x70,0x61,0x72,0x61,0x6d,0x73,0x5b,0x32,0x5d,0x2e,0x78,0x79,0x7a,0x20,0x2d,0x20,
    0x50,0x2e,0x78,0x79,0x7a,0x29,0x29,0x2c,0x20,0x30,0x2e,0x30,0x29,0x2c,0x20,0x32,
    0x2e,0x32,0x30,0x30,0x30,0x30,0x30,0x30,0x34,0x37,0x36,0x38,0x33,0x37,0x31,0x35,
    0x38,0x32,0x30,0x33,0x31,0x32,0x35,0x29,0x20,0x2a,0x20,0x5f,0x31,0x35,0x38,0x29,
    0x20,0x2a,0x20,0x5f,0x31,0x39,0x35,0x29,0x20,0x2b,0x20,0x28,0x63,0x6f,0x6c,0x6f,
    0x72,0x20,0x2a,0x20,0x28,0x6d,0x61,0x78,0x28,0x5f,0x31,0x35,0x38,0x20,0x2a,0x20,
    0x5f,0x31,0x39,0x35,0x2c,0x20,0x30,0x2e,0x30,0x29,0x20,0x2b,0x20,0x30,0x2e,0x32,
    0x35,0x29,0x29,0x2c,0x20,0x31,0x2e,0x30,0x29,0x3b,0x0a,0x20,0x20,0x20,0x20,0x7d,
    0x0a,0x20,0x20,0x20,0x20,0x65,0x6c,0x73,0x65,0x0a,0x20,0x20,0x20,0x20,0x7b,0x0a,
    0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x20,0x66,0x72,0x61,0x67,0x43,0x6f,0x6c,0x6f,
    0x72,0x20,0x3d,0x20,0x76,0x65,0x63,0x34,0x28,0x63,0x6f,0x6c,0x6f,0x72,0x20,0x2a,
    0x20,0x30,0x2e,0x32,0x35,0x2c,0x20,0x31,0x2e,0x30,0x29,0x3b,0x0a,0x20,0x20,0x20,
    0x20,0x7d,0x0a,0x20,0x20,0x20,0x20,0x76,0x65,0x63,0x34,0x20,0x70,0x61,0x72,0x61,
    0x6d,0x5f,0x33,0x20,0x3d,0x20,0x66,0x72,0x61,0x67,0x43,0x6f,0x6c,0x6f,0x72,0x3b,
    0x0a,0x20,0x20,0x20,0x20,0x66,0x72,0x61,0x67,0x43,0x6f,0x6c,0x6f,0x72,0x20,0x3d,
    0x20,0x67,0x61,0x6d,0x6d,0x61,0x28,0x70,0x61,0x72,0x61,0x6d,0x5f,0x33,0x29,0x3b,
    0x0a,0x7d,0x0a,0x0a,0x00,
};

static inline const sg_shader_desc* color_shader_desc(sg_backend backend) {
  if (backend == SG_BACKEND_GLCORE33) {
    static sg_shader_desc desc;
    static bool valid;
    if (!valid) {
      valid = true;
      desc.attrs[0].name = "position";
      desc.attrs[1].name = "normal";
      desc.vs.source = colorVS_source_glsl330;
      desc.vs.entry = "main";
      desc.vs.uniform_blocks[0].size = 208;
      desc.vs.uniform_blocks[0].layout = SG_UNIFORMLAYOUT_STD140;
      desc.vs.uniform_blocks[0].uniforms[0].name = "vs_light_params";
      desc.vs.uniform_blocks[0].uniforms[0].type = SG_UNIFORMTYPE_FLOAT4;
      desc.vs.uniform_blocks[0].uniforms[0].array_count = 13;
      desc.fs.source = colorFS_source_glsl330;
      desc.fs.entry = "main";
      desc.fs.uniform_blocks[0].size = 48;
      desc.fs.uniform_blocks[0].layout = SG_UNIFORMLAYOUT_STD140;
      desc.fs.uniform_blocks[0].uniforms[0].name = "fs_light_params";
      desc.fs.uniform_blocks[0].uniforms[0].type = SG_UNIFORMTYPE_FLOAT4;
      desc.fs.uniform_blocks[0].uniforms[0].array_count = 3;
      desc.fs.images[0].name = "shadowMap";
      desc.fs.images[0].image_type = SG_IMAGETYPE_2D;
      desc.fs.images[0].sampler_type = SG_SAMPLERTYPE_FLOAT;
      desc.label = "color_shader";
    }
    return &desc;
  }
  return 0;
}
static inline const sg_shader_desc* shadow_shader_desc(sg_backend backend) {
  if (backend == SG_BACKEND_GLCORE33) {
    static sg_shader_desc desc;
    static bool valid;
    if (!valid) {
      valid = true;
      desc.attrs[0].name = "position";
      desc.vs.source = shadowVS_source_glsl330;
      desc.vs.entry = "main";
      desc.vs.uniform_blocks[0].size = 64;
      desc.vs.uniform_blocks[0].layout = SG_UNIFORMLAYOUT_STD140;
      desc.vs.uniform_blocks[0].uniforms[0].name = "vs_shadow_params";
      desc.vs.uniform_blocks[0].uniforms[0].type = SG_UNIFORMTYPE_FLOAT4;
      desc.vs.uniform_blocks[0].uniforms[0].array_count = 4;
      desc.fs.source = shadowFS_source_glsl330;
      desc.fs.entry = "main";
      desc.label = "shadow_shader";
    }
    return &desc;
  }
  return 0;
}

#define SCREEN_SAMPLE_COUNT (4)

static struct {
    struct {
        sg_pass_action pass_action;
        sg_pass pass;
        sg_pipeline pip;
        sg_bindings bind;
    } shadows;
    struct {
        sg_pass_action pass_action;
        sg_pipeline pip;
        sg_bindings bind;
    } deflt;
    float ry;
} state;

void init(void) {
    sg_setup(&(sg_desc){
        .context = sapp_sgcontext()
    });

    /* default pass action: clear to blue-ish */
    state.deflt.pass_action = (sg_pass_action) {
        .colors[0] = { .action = SG_ACTION_CLEAR, .value = { 0.0f, 0.25f, 1.0f, 1.0f } }
    };

    /* shadow pass action: clear to white */
    state.shadows.pass_action = (sg_pass_action) {
        .colors[0] = { .action = SG_ACTION_CLEAR, .value = { 1.0f, 1.0f, 1.0f, 1.0f } }
    };

    /* a render pass with one color- and one depth-attachment image */
    sg_image_desc img_desc = {
        .render_target = true,
        .width = 2048,
        .height = 2048,
        .pixel_format = SG_PIXELFORMAT_RGBA8,
        .min_filter = SG_FILTER_LINEAR,
        .mag_filter = SG_FILTER_LINEAR,
        .sample_count = 1,
        .label = "shadow-map-color-image"
    };
    sg_image color_img = sg_make_image(&img_desc);
    img_desc.pixel_format = SG_PIXELFORMAT_DEPTH;
    img_desc.label = "shadow-map-depth-image";
    sg_image depth_img = sg_make_image(&img_desc);
    state.shadows.pass = sg_make_pass(&(sg_pass_desc){
        .color_attachments[0].image = color_img,
        .depth_stencil_attachment.image = depth_img,
        .label = "shadow-map-pass"
    });

    /* cube vertex buffer with positions & normals */
    float vertices[] = {
        /* pos                  normals             */
        -1.0f, -1.0f, -1.0f,    0.0f, 0.0f, -1.0f,  //CUBE BACK FACE
         1.0f, -1.0f, -1.0f,    0.0f, 0.0f, -1.0f,
         1.0f,  1.0f, -1.0f,    0.0f, 0.0f, -1.0f,
        -1.0f,  1.0f, -1.0f,    0.0f, 0.0f, -1.0f,

        -1.0f, -1.0f,  1.0f,    0.0f, 0.0f, 1.0f,   //CUBE FRONT FACE
         1.0f, -1.0f,  1.0f,    0.0f, 0.0f, 1.0f,
         1.0f,  1.0f,  1.0f,    0.0f, 0.0f, 1.0f,
        -1.0f,  1.0f,  1.0f,    0.0f, 0.0f, 1.0f,

        -1.0f, -1.0f, -1.0f,    -1.0f, 0.0f, 0.0f,  //CUBE LEFT FACE
        -1.0f,  1.0f, -1.0f,    -1.0f, 0.0f, 0.0f,
        -1.0f,  1.0f,  1.0f,    -1.0f, 0.0f, 0.0f,
        -1.0f, -1.0f,  1.0f,    -1.0f, 0.0f, 0.0f,

         1.0f, -1.0f, -1.0f,    1.0f, 0.0f, 0.0f,   //CUBE RIGHT FACE
         1.0f,  1.0f, -1.0f,    1.0f, 0.0f, 0.0f,
         1.0f,  1.0f,  1.0f,    1.0f, 0.0f, 0.0f,
         1.0f, -1.0f,  1.0f,    1.0f, 0.0f, 0.0f,

        -1.0f, -1.0f, -1.0f,    0.0f, -1.0f, 0.0f,  //CUBE BOTTOM FACE
        -1.0f, -1.0f,  1.0f,    0.0f, -1.0f, 0.0f,
         1.0f, -1.0f,  1.0f,    0.0f, -1.0f, 0.0f,
         1.0f, -1.0f, -1.0f,    0.0f, -1.0f, 0.0f,

        -1.0f,  1.0f, -1.0f,    0.0f, 1.0f, 0.0f,   //CUBE TOP FACE
        -1.0f,  1.0f,  1.0f,    0.0f, 1.0f, 0.0f,
         1.0f,  1.0f,  1.0f,    0.0f, 1.0f, 0.0f,
         1.0f,  1.0f, -1.0f,    0.0f, 1.0f, 0.0f,

        -1.0f,  0.0f, -1.0f,    0.0f, 1.0f, 0.0f,   //PLANE GEOMETRY
        -1.0f,  0.0f,  1.0f,    0.0f, 1.0f, 0.0f,
         1.0f,  0.0f,  1.0f,    0.0f, 1.0f, 0.0f,
         1.0f,  0.0f, -1.0f,    0.0f, 1.0f, 0.0f,
    };
    sg_buffer vbuf = sg_make_buffer(&(sg_buffer_desc){
        .data = SG_RANGE(vertices),
        .label = "cube-vertices"
    });

    /* an index buffer for the cube */
    uint16_t indices[] = {
        0, 1, 2,  0, 2, 3,
        6, 5, 4,  7, 6, 4,
        8, 9, 10,  8, 10, 11,
        14, 13, 12,  15, 14, 12,
        16, 17, 18,  16, 18, 19,
        22, 21, 20,  23, 22, 20,
        26, 25, 24,  27, 26, 24
    };
    sg_buffer ibuf = sg_make_buffer(&(sg_buffer_desc){
        .type = SG_BUFFERTYPE_INDEXBUFFER,
        .data = SG_RANGE(indices),
        .label = "cube-indices"
    });

    /* pipeline-state-object for shadows-rendered cube, don't need texture coord here */
    state.shadows.pip = sg_make_pipeline(&(sg_pipeline_desc){
        .layout = {
            /* need to provide stride, because the buffer's normal vector is skipped */
            .buffers[0].stride = 6 * sizeof(float),
            /* but don't need to provide attr offsets, because pos and normal are continuous */
            .attrs = {
                [ATTR_shadowVS_position].format = SG_VERTEXFORMAT_FLOAT3
            }
        },
        .shader = sg_make_shader(shadow_shader_desc(sg_query_backend())),
        .index_type = SG_INDEXTYPE_UINT16,
        /* Cull front faces in the shadow map pass */
        .cull_mode = SG_CULLMODE_FRONT,
        .sample_count = 1,
        .depth = {
            .pixel_format = SG_PIXELFORMAT_DEPTH,
            .compare = SG_COMPAREFUNC_LESS_EQUAL,
            .write_enabled = true,
        },
        .colors[0].pixel_format = SG_PIXELFORMAT_RGBA8,
        .label = "shadow-map-pipeline"
    });

    /* and another pipeline-state-object for the default pass */
    state.deflt.pip = sg_make_pipeline(&(sg_pipeline_desc){
        .layout = {
            /* don't need to provide buffer stride or attr offsets, no gaps here */
            .attrs = {
                [ATTR_colorVS_position].format = SG_VERTEXFORMAT_FLOAT3,
                [ATTR_colorVS_normal].format = SG_VERTEXFORMAT_FLOAT3
            }
        },
        .shader = sg_make_shader(color_shader_desc(sg_query_backend())),
        .index_type = SG_INDEXTYPE_UINT16,
        /* Cull back faces when rendering to the screen */
        .cull_mode = SG_CULLMODE_BACK,
        .depth = {
            .compare = SG_COMPAREFUNC_LESS_EQUAL,
            .write_enabled = true
        },
        .label = "default-pipeline"
    });

    /* the resource bindings for rendering the cube into the shadow map render target */
    state.shadows.bind = (sg_bindings){
        .vertex_buffers[0] = vbuf,
        .index_buffer = ibuf
    };

    /* resource bindings to render the cube, using the shadow map render target as texture */
    state.deflt.bind = (sg_bindings){
        .vertex_buffers[0] = vbuf,
        .index_buffer = ibuf,
        .fs_images[SLOT_shadowMap] = color_img
    };

    state.ry = 0.0f;
}

void frame(void) {

    const float t = (float)(sapp_frame_duration() * 60.0);
    state.ry += 0.2f * t;

    /* Calculate matrices for shadow pass */
    const hmm_mat4 rym = HMM_Rotate(state.ry, HMM_Vec3(0.0f,1.0f,0.0f));
    const hmm_vec4 light_dir = HMM_MultiplyMat4ByVec4(rym, HMM_Vec4(50.0f,50.0f,-50.0f,0.0f));
    const hmm_mat4 light_view = HMM_LookAt(light_dir.XYZ, HMM_Vec3(0.0f, 0.0f, 0.0f), HMM_Vec3(0.0f, 1.0f, 0.0f));

    /* Configure a bias matrix for converting view-space coordinates into uv coordinates */
    hmm_mat4 light_proj = { {
        { 0.5f, 0.0f, 0.0f, 0 },
        { 0.0f, 0.5f, 0.0f, 0 },
        { 0.0f, 0.0f, 0.5f, 0 },
        { 0.5f, 0.5f, 0.5f, 1 }
    } };
    light_proj = HMM_MultiplyMat4(light_proj, HMM_Orthographic(-4.0f, 4.0f, -4.0f, 4.0f, 0, 200.0f));
    const hmm_mat4 light_view_proj = HMM_MultiplyMat4(light_proj, light_view);

    /* Calculate matrices for camera pass */
    const hmm_mat4 proj = HMM_Perspective(60.0f, sapp_widthf()/sapp_heightf(), 0.01f, 100.0f);
    const hmm_mat4 view = HMM_LookAt(HMM_Vec3(5.0f, 5.0f, 5.0f), HMM_Vec3(0.0f, 0.0f, 0.0f), HMM_Vec3(0.0f, 1.0f, 0.0f));
    const hmm_mat4 view_proj = HMM_MultiplyMat4(proj, view);

    /* Calculate transform matrices for plane and cube */
    const hmm_mat4 scale = HMM_Scale(HMM_Vec3(5,0,5));
    const hmm_mat4 translate = HMM_Translate(HMM_Vec3(0,1.5f,0));

    /* Initialise fragment uniforms for light shader */
    const fs_light_params_t fs_light_params = {
        .lightDir = HMM_NormalizeVec3(light_dir.XYZ),
        .shadowMapSize = HMM_Vec2(2048,2048),
        .eyePos = HMM_Vec3(5.0f,5.0f,5.0f)
    };

    /* the shadow map pass, render the vertices into the depth image */
    sg_begin_pass(state.shadows.pass, &state.shadows.pass_action);
    sg_apply_pipeline(state.shadows.pip);
    sg_apply_bindings(&state.shadows.bind);
    {
        /* Render the cube into the shadow map */
        const vs_shadow_params_t vs_shadow_params = {
            .mvp = HMM_MultiplyMat4(light_view_proj,translate)
        };
        sg_apply_uniforms(SG_SHADERSTAGE_VS, SLOT_vs_shadow_params, &SG_RANGE(vs_shadow_params));
        sg_draw(0, 36, 1);
    }
    sg_end_pass();

    /* and the display-pass, rendering the scene, using the previously rendered shadow map as a texture */
    sg_begin_default_pass(&state.deflt.pass_action, sapp_width(), sapp_height());
    sg_apply_pipeline(state.deflt.pip);
    sg_apply_bindings(&state.deflt.bind);
    sg_apply_uniforms(SG_SHADERSTAGE_FS, SLOT_fs_light_params, &SG_RANGE(fs_light_params));
    {
        /* Render the plane in the light pass */
        const vs_light_params_t vs_light_params = {
            .mvp = HMM_MultiplyMat4(view_proj,scale),
            .lightMVP = HMM_MultiplyMat4(light_view_proj,scale),
            .model = HMM_Mat4d(1.0f),
            .diffColor = HMM_Vec3(0.5f,0.5f,0.5f)
        };
        sg_apply_uniforms(SG_SHADERSTAGE_VS, SLOT_vs_light_params, &SG_RANGE(vs_light_params));
        sg_draw(36, 6, 1);
    }
    {
        /* Render the cube in the light pass */
        const vs_light_params_t vs_light_params = {
            .lightMVP = HMM_MultiplyMat4(light_view_proj,translate),
            .model = translate,
            .mvp = HMM_MultiplyMat4(view_proj,translate),
            .diffColor = HMM_Vec3(1.0f,1.0f,1.0f)
        };
        sg_apply_uniforms(SG_SHADERSTAGE_VS, SLOT_vs_light_params, &SG_RANGE(vs_light_params));
        sg_draw(0, 36, 1);
    }

    sg_end_pass();

    sg_commit();
}

void cleanup(void) {
    sg_shutdown();
}

sapp_desc sokol_main(int argc, char* argv[]) {
    (void)argc; (void)argv;
    return (sapp_desc){
        .init_cb = init,
        .frame_cb = frame,
        .cleanup_cb = cleanup,
        .width = 800,
        .height = 600,
        .sample_count = SCREEN_SAMPLE_COUNT,
        .window_title = "Shadow Rendering (sokol-app)",
        .icon.sokol_default = true,
    };
}
