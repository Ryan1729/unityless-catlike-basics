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

#if !defined(SOKOL_SHDC_ALIGN)
  #if defined(_MSC_VER)
    #define SOKOL_SHDC_ALIGN(a) __declspec(align(a))
  #else
    #define SOKOL_SHDC_ALIGN(a) __attribute__((aligned(a)))
  #endif
#endif
#define ATTR_colorVS_position (0)
#define ATTR_colorVS_normal (1)
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
    hmm_vec3 lightDir;
    uint8_t _pad_28[4];
    hmm_vec3 eyePos;
    uint8_t _pad_44[4];
} fs_light_params_t;
#pragma pack(pop)

static inline const sg_shader_desc* color_shader_desc(sg_backend backend) {
  if (backend == SG_BACKEND_GLCORE33) {
    static sg_shader_desc desc;
    static bool valid;
    if (!valid) {
      valid = true;
      desc.attrs[0].name = "position";
      desc.attrs[1].name = "normal";
      desc.vs.source = "#version 330\n"
"\n"
"uniform vec4 vs_light_params[13];\n"
"layout(location = 0) in vec4 position;\n"
"out vec4 lightProjPos;\n"
"out vec4 P;\n"
"out vec3 N;\n"
"layout(location = 1) in vec3 normal;\n"
"out vec3 color;\n"
"\n"
"void main()\n"
"{\n"
"    mat4 mvp = mat4(vs_light_params[4], vs_light_params[5], vs_light_params[6], vs_light_params[7]);\n"
"    gl_Position = mvp * position;\n"
"    mat4 lightMVP = mat4(vs_light_params[8], vs_light_params[9], vs_light_params[10], vs_light_params[11]);\n"
"    lightProjPos = lightMVP * position;\n"
"    mat4 model = mat4(vs_light_params[0], vs_light_params[1], vs_light_params[2], vs_light_params[3]);\n"
"    P = model * position;\n"
"    N = (model * vec4(normal, 0.0)).xyz;\n"
"    color = vs_light_params[12].xyz;\n"
"}\n"
      ;
      desc.vs.entry = "main";
      desc.vs.uniform_blocks[0].size = 208;
      desc.vs.uniform_blocks[0].layout = SG_UNIFORMLAYOUT_STD140;
      desc.vs.uniform_blocks[0].uniforms[0].name = "vs_light_params";
      desc.vs.uniform_blocks[0].uniforms[0].type = SG_UNIFORMTYPE_FLOAT4;
      desc.vs.uniform_blocks[0].uniforms[0].array_count = 13;
      desc.fs.source = "#version 330\n"
"\n"
"uniform vec4 fs_light_params[2];\n"
"\n"
"in vec3 N;\n"
"in vec4 lightProjPos;\n"
"in vec4 P;\n"
"layout(location = 0) out vec4 fragColor;\n"
"in vec3 color;\n"
"\n"
"vec4 linearToGamma(vec4 c)\n"
"{\n"
"    return vec4(pow(c.xyz, vec3(1/2.2)), c.w);\n"
"}\n"
"\n"
"float gammaToLinear(float c)\n"
"{\n"
"    return pow(c, 2.2);\n"
"}\n"
"\n"
"void main()\n"
"{\n"
"    vec3 lightDir = normalize(fs_light_params[0].xyz);\n"
"    vec3 normal = normalize(N);\n"
"    float angle = dot(normal, lightDir);\n"
"    if (angle > 0.0)\n"
"    {\n"
"        vec3 eye = fs_light_params[1].xyz;"
"        fragColor = vec4("
"            ("
"                gammaToLinear("
"                    max(dot(reflect(-lightDir, normal), normalize(eye - P.xyz)), 0.0)"
"                )"
"                * angle"
"            )"
"            + (color * (angle + 0.25)),"
"            1.0"
"         );\n"
"    }\n"
"    else\n"
"    {\n"
"        fragColor = vec4(color * 0.25, 1.0);\n"
"    }\n"
"    fragColor = linearToGamma(fragColor);\n"
"}\n"
    ;
      desc.fs.entry = "main";
      desc.fs.uniform_blocks[0].size = 32;
      desc.fs.uniform_blocks[0].layout = SG_UNIFORMLAYOUT_STD140;
      desc.fs.uniform_blocks[0].uniforms[0].name = "fs_light_params";
      desc.fs.uniform_blocks[0].uniforms[0].type = SG_UNIFORMTYPE_FLOAT4;
      desc.fs.uniform_blocks[0].uniforms[0].array_count = 2;
      desc.label = "color_shader";
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

    /* resource bindings to render the cube */
    state.deflt.bind = (sg_bindings){
        .vertex_buffers[0] = vbuf,
        .index_buffer = ibuf,
    };

    state.ry = 0.0f;
}

void frame(void) {
    const float t = (float)(sapp_frame_duration() * 60.0);
    state.ry += t;

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
        .eyePos = HMM_Vec3(5.0f,5.0f,5.0f)
    };

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
            .diffColor = HMM_Vec3(0.25f,0.5f,0.75f)
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
            .diffColor = HMM_Vec3(1.0f,1.0f,0.0f)
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
