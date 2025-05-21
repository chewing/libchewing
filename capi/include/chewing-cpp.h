#ifndef chewing_public_cpp_bindings_h
#define chewing_public_cpp_bindings_h

#pragma once

#include "chewing.h"

#ifdef __cplusplus
extern "C" {
#endif

// typedefs for pointers to callbacks that can take and proccess
// candidates/buffers
typedef void (*candidate_info_callback_t)(const int pageCnt, int numPages,
                                          int cntPerPage);
typedef void (*candidate_callback_t)(const char *candidate);
typedef void (*buffer_callback_t)(const char *buffer);
typedef void (*bopomofo_callback_t)(const char *buffer);

typedef void (*commit_callback_t)(const char *buffer);

typedef void (*print_func_t)(const char *buf, const char *prefix);

typedef void (*logger_func_t)(void *data, int level, const char *message);

// a struct that holds the data path and pointers to callbacks that can take and
// proccess candidates/buffers
typedef struct CallbacksContext {
    char *data_path;
    candidate_info_callback_t candidate_info_callback;
    candidate_callback_t candidate_callback;
    buffer_callback_t buffer_callback;
    bopomofo_callback_t bopomofo_callback;
    commit_callback_t commit_callback;
    print_func_t print_func;
    logger_func_t logger_func;
    void *logger_data;
} CallbacksContext;

int display_candidates(ChewingContext *ctx);

bool display_text_buffer(ChewingContext *ctx);

bool display_preedit_buffer(ChewingContext *ctx);

bool display_commit_buffer(ChewingContext *ctx);



void chewing_init(ChewingContext **ctx, CallbacksContext *callbacks_context);

void chewing_terminate(ChewingContext **ctx);

#ifdef __cplusplus
}
#endif

#endif // chewing_public_cpp_bindings_h