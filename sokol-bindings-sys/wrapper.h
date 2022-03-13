#include <stdio.h>

#ifdef SOKOL_BINGDINGS_DEBUG
#define SOKOL_LOG(s) { SOKOL_ASSERT(s); fprintf(stderr, "%s\n", s); }
#endif

#define SOKOL_IMPL
#include "../third-party/sokol_gfx.h"
#define SOKOL_NO_ENTRY
#include "../third-party/sokol_app.h"
#include "../third-party/sokol_glue.h"
