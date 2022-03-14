#define SOKOL_IMPL
#include "sokol_gfx.h"
#define SOKOL_NO_ENTRY
#include "sokol_app.h"
#include "sokol_glue.h"

#define ATTR_vs_pos (0)
#define ATTR_vs_color0 (1)
#define ATTR_vs_texcoord0 (2)
#define SLOT_tex (0)
#define SLOT_vs_params (0)

static inline const sg_shader_desc* cube_shader_desc(sg_backend backend) {
  if (backend == SG_BACKEND_GLCORE33) {
    static sg_shader_desc desc;
    static bool valid;
    if (!valid) {
      valid = true;
      desc.attrs[ATTR_vs_pos].name = "pos";
      desc.attrs[ATTR_vs_color0].name = "color0";
      desc.attrs[ATTR_vs_texcoord0].name = "texcoord0";
      desc.vs.source = "#version 330\n"
"\n"
"uniform vec4 vs_params[4];\n"
"layout(location = 0) in vec4 pos;\n"
"out vec4 color;\n"
"layout(location = 1) in vec4 color0;\n"
"out vec2 uv;\n"
"layout(location = 2) in vec2 texcoord0;\n"
"\n"
"void main()\n"
"{\n"
"    gl_Position = mat4(vs_params[0], vs_params[1], vs_params[2], vs_params[3]) * pos;\n"
"    color = color0;\n"
"    uv = texcoord0 * 5.0;\n"
"}\n";
      desc.vs.uniform_blocks[0].size = 64;
      desc.vs.uniform_blocks[0].layout = SG_UNIFORMLAYOUT_STD140;
      desc.vs.uniform_blocks[0].uniforms[0].name = "vs_params";
      desc.vs.uniform_blocks[0].uniforms[0].type = SG_UNIFORMTYPE_FLOAT4;
      desc.vs.uniform_blocks[0].uniforms[0].array_count = 4;
      desc.vs.entry = "main";
      desc.fs.source = "#version 330\n"
"\n"
"uniform sampler2D tex;\n"
"\n"
"layout(location = 0) out vec4 frag_color;\n"
"in vec2 uv;\n"
"in vec4 color;\n"
"\n"
"void main()\n"
"{\n"
"    frag_color = texture(tex, uv) * color;\n"
"}\n";
      desc.fs.entry = "main";
      desc.fs.images[0].name = "tex";
      desc.fs.images[0].image_type = SG_IMAGETYPE_2D;
      desc.fs.images[0].sampler_type = SG_SAMPLERTYPE_FLOAT;
      desc.label = "cube_shader";
    }
    return &desc;
  }
  return 0;
}

static struct {
    sg_pipeline pip;
    sg_bindings bind;
} state;

typedef struct {
    float x, y, z;
    uint32_t color;
    int16_t u, v;
} vertex_t;

void init(void) {
    sg_setup(&(sg_desc){
        .context = sapp_sgcontext()
    });

    /*
        Cube vertex buffer with packed vertex formats for color and texture coords.
        Note that a vertex format which must be portable across all
        backends must only use the normalized integer formats
        (BYTE4N, UBYTE4N, SHORT2N, SHORT4N), which can be converted
        to floating point formats in the vertex shader inputs.

        The reason is that D3D11 cannot convert from non-normalized
        formats to floating point inputs (only to integer inputs),
        and WebGL2 / GLES2 don't support integer vertex shader inputs.
    */
    vertex_t vertices[] = {
        /* pos                  color       uvs */
        { -1.0f, -1.0f, -1.0f,  0xFF0000FF,     0,     0 },
        {  1.0f, -1.0f, -1.0f,  0xFF0000FF, 32767,     0 },
        {  1.0f,  1.0f, -1.0f,  0xFF0000FF, 32767, 32767 },
        { -1.0f,  1.0f, -1.0f,  0xFF0000FF,     0, 32767 },

        { -1.0f, -1.0f,  1.0f,  0xFF00FF00,     0,     0 },
        {  1.0f, -1.0f,  1.0f,  0xFF00FF00, 32767,     0 },
        {  1.0f,  1.0f,  1.0f,  0xFF00FF00, 32767, 32767 },
        { -1.0f,  1.0f,  1.0f,  0xFF00FF00,     0, 32767 },

        { -1.0f, -1.0f, -1.0f,  0xFFFF0000,     0,     0 },
        { -1.0f,  1.0f, -1.0f,  0xFFFF0000, 32767,     0 },
        { -1.0f,  1.0f,  1.0f,  0xFFFF0000, 32767, 32767 },
        { -1.0f, -1.0f,  1.0f,  0xFFFF0000,     0, 32767 },

        {  1.0f, -1.0f, -1.0f,  0xFFFF007F,     0,     0 },
        {  1.0f,  1.0f, -1.0f,  0xFFFF007F, 32767,     0 },
        {  1.0f,  1.0f,  1.0f,  0xFFFF007F, 32767, 32767 },
        {  1.0f, -1.0f,  1.0f,  0xFFFF007F,     0, 32767 },

        { -1.0f, -1.0f, -1.0f,  0xFFFF7F00,     0,     0 },
        { -1.0f, -1.0f,  1.0f,  0xFFFF7F00, 32767,     0 },
        {  1.0f, -1.0f,  1.0f,  0xFFFF7F00, 32767, 32767 },
        {  1.0f, -1.0f, -1.0f,  0xFFFF7F00,     0, 32767 },

        { -1.0f,  1.0f, -1.0f,  0xFF007FFF,     0,     0 },
        { -1.0f,  1.0f,  1.0f,  0xFF007FFF, 32767,     0 },
        {  1.0f,  1.0f,  1.0f,  0xFF007FFF, 32767, 32767 },
        {  1.0f,  1.0f, -1.0f,  0xFF007FFF,     0, 32767 },
    };
    state.bind.vertex_buffers[0] = sg_make_buffer(&(sg_buffer_desc){
        .data = SG_RANGE(vertices),
        .label = "cube-vertices"
    });

    /* create an index buffer for the cube */
    uint16_t indices[] = {
        0, 1, 2,  0, 2, 3,
        6, 5, 4,  7, 6, 4,
        8, 9, 10,  8, 10, 11,
        14, 13, 12,  15, 14, 12,
        16, 17, 18,  16, 18, 19,
        22, 21, 20,  23, 22, 20
    };
    state.bind.index_buffer = sg_make_buffer(&(sg_buffer_desc){
        .type = SG_BUFFERTYPE_INDEXBUFFER,
        .data = SG_RANGE(indices),
        .label = "cube-indices"
    });

    /* create a checkerboard texture */
    uint32_t pixels[4*4] = {
        0xFFFFFFFF, 0xFF000000, 0xFFFFFFFF, 0xFF000000,
        0xFF000000, 0xFFFFFFFF, 0xFF000000, 0xFFFFFFFF,
        0xFFFFFFFF, 0xFF000000, 0xFFFFFFFF, 0xFF000000,
        0xFF000000, 0xFFFFFFFF, 0xFF000000, 0xFFFFFFFF,
    };

    state.bind.fs_images[SLOT_tex] = sg_make_image(&(sg_image_desc){
        .width = 4,
        .height = 4,
        .data.subimage[0][0] = SG_RANGE(pixels),
        .label = "cube-texture"
    });

    /* create shader */
    sg_shader shd = sg_make_shader(cube_shader_desc(sg_query_backend()));

    /* create pipeline object */
    state.pip = sg_make_pipeline(&(sg_pipeline_desc){
        .layout = {
            .attrs = {
                [ATTR_vs_pos].format = SG_VERTEXFORMAT_FLOAT3,
                [ATTR_vs_color0].format = SG_VERTEXFORMAT_UBYTE4N,
                [ATTR_vs_texcoord0].format = SG_VERTEXFORMAT_SHORT2N
            }
        },
        .shader = shd,
        .index_type = SG_INDEXTYPE_UINT16,
        .cull_mode = SG_CULLMODE_BACK,
        .depth = {
            .compare = SG_COMPAREFUNC_LESS_EQUAL,
            .write_enabled = true
        },
        .label = "cube-pipeline"
    });
}

typedef struct vs_params_t {
    float elements[4 * 4];
} vs_params_t;

void frame(void) {
    sg_pass_action pass_action = {
        .colors[0] = { .action = SG_ACTION_CLEAR, .value = { 0.25f, 0.5f, 0.75f, 1.0f } }
    };
    vs_params_t vs_params = {
        .elements = {
            0.12511493265628815, 0.10831947781532758, 0.18738500347083775, 0.0,
            -0.21643994748592377, 0.06261498549435007, 0.10831947781532758, 0.0,
            0.0, -0.21643994748592377, 0.12511493265628815, 0.0,
            0.0, 0.0, 0.0, 1.0
        },
    };

    sg_begin_default_pass(&pass_action, sapp_width(), sapp_height());
    sg_apply_pipeline(state.pip);
    sg_apply_bindings(&state.bind);
    sg_apply_uniforms(SG_SHADERSTAGE_VS, SLOT_vs_params, &SG_RANGE(vs_params));
    sg_draw(0, 36, 1);
    sg_end_pass();
    sg_commit();
}

void cleanup(void) {
    sg_shutdown();
}

int main() {
    sapp_run(&(sapp_desc){
        .init_cb = init,
        .frame_cb = frame,
        .cleanup_cb = cleanup,
        .width = 800,
        .height = 600,
        .sample_count = 4,
        .window_title = "Cube (sokol-app)",
        .icon.sokol_default = true,
    });

    return 0;
}
