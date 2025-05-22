#ifndef chewing_public_cpp_bindings_h
#define chewing_public_cpp_bindings_h

#pragma once

#include "chewing.h"

#ifdef __cplusplus
extern "C" {
#endif

// typedefs for pointers to callbacks that can take and proccess
// candidates/buffers
typedef void (*candidate_info_callback_t)(const int pageSize, int numPages,
                                          int candidateOnPage, int totalChoices,
                                          const char **candidates);
typedef void (*buffer_callback_t)(const char *buffer);
typedef void (*bopomofo_callback_t)(const char *buffer);
typedef void (*commit_callback_t)(const char *buffer);
typedef void (*logger_func_t)(int level, const char *message);

// a struct that holds the data path and pointers to callbacks that can take and
// proccess candidates/buffers
typedef struct Callbacks {
    candidate_info_callback_t candidate_info_callback; // the candidates that are
                                           // going to be displayed
    buffer_callback_t buffer_callback;
    bopomofo_callback_t bopomofo_callback; // preedit buffer the sounds that are
                                           // going to be converted
    commit_callback_t commit_callback; // the text that should be
                                       // in the input field
    logger_func_t logger_func;
} Callbacks;

typedef struct ConfigData {
    char *data_path;
    int candPerPage = 10;
    int maxChiSymbolLen = 18;
} ConfigData;

typedef struct ApplicationContext {
    ConfigData config_data;
    Callbacks callbacks;
} ApplicationContext;

///
void process_key(char key);

bool chewing_init(ApplicationContext *ctx);

void select_candidate(int index);

bool chewing_terminate();

#ifdef __cplusplus
}
#endif

#endif // chewing_public_cpp_bindings_h