#include <stdarg.h>

#include "chewing-cpp.h"
#include <cstdio> // for printf
#include <vector> // for exit

#ifdef __cplusplus
extern "C" {
#endif

static CallbacksContext *s_callbacks_context = nullptr;

// shim: format variadic logs into a single message, then call the simple logger
static void chewing_cpp_logger_shim(void *data, int level, const char *fmt, ...)
{
    char buf[1024];
    va_list args;
    va_start(args, fmt);
    vsnprintf(buf, sizeof(buf), fmt, args);
    va_end(args);
    if (s_callbacks_context && s_callbacks_context->logger_func) {
        s_callbacks_context->logger_func(data, level, buf);
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////
/// This function is used to signal to libchewing to bring up the candidates
/// menu, iterates over the candidates and displays them. Displays additional
/// information about the candidates.
///
/// @param[in] ctx - the context which holds the candidates
/// @retval int - the number of canidates
//////////////////////////////////////////////////////////////////////////////////////////////
int display_candidates(ChewingContext *ctx)
{
    int pageSize{chewing_get_candPerPage(ctx)};
    int numPages{chewing_cand_TotalPage(ctx)};
    int choicePerPage{chewing_cand_ChoicePerPage(ctx)};
    int index{0};
    printf("Pages: %d, PageSize: %d, ChoicePerPage: %d, \nCandidates:\n",
           numPages, pageSize, choicePerPage);
    if (s_callbacks_context->candidate_info_callback) {
        s_callbacks_context->candidate_info_callback(pageSize, numPages,
                                                     choicePerPage);
    }
    chewing_cand_Enumerate(ctx);
    while (chewing_cand_hasNext(ctx) && index < pageSize) {
        auto buf{const_cast<char *>(chewing_cand_String(ctx))};
        if (s_callbacks_context->print_func) {
            s_callbacks_context->print_func(buf, "   candidate: ");
        }
        if (s_callbacks_context->candidate_callback) {
            s_callbacks_context->candidate_callback(buf);
        }

        index++;
    }

    return index;
}

//////////////////////////////////////////////////////////////////////////////////////////////
/// Display the current contents of a buffer which hold text in Chinese which
/// has been converted up to this point. The text will be moved to the commit
/// buffer if the commit key is pressed. However it is still subject to
/// conversion.
///
/// @param[in] ctx - the context which holds the buffer
/// @retval bool - false if the buffer is empty, true otherwise.
//////////////////////////////////////////////////////////////////////////////////////////////
bool display_text_buffer(ChewingContext *ctx)
{
    if (chewing_buffer_Check(ctx)) {
        auto buf{const_cast<char *>(chewing_buffer_String_static(ctx))};
        if (s_callbacks_context->print_func) {
            s_callbacks_context->print_func(buf, "buffer: ");
        }
        if (s_callbacks_context->buffer_callback) {
            s_callbacks_context->buffer_callback(buf);
        }
        return true;
    }
    return false;
}

//////////////////////////////////////////////////////////////////////////////////////////////
/// Display the current contents of the preedit buffer. The preedit buffer is
/// the buffer that contains the bopomofo text which gets converted to Chinese.
///
/// @param[in] ctx - the context which holds the buffer
/// @retval bool - false if the buffer is empty, true otherwise.
//////////////////////////////////////////////////////////////////////////////////////////////
bool display_preedit_buffer(ChewingContext *ctx)
{
    if (chewing_bopomofo_Check(ctx)) {
        auto bopomofo_buf{
            const_cast<char *>(chewing_bopomofo_String_static(ctx))};
        if (s_callbacks_context->print_func) {
            s_callbacks_context->print_func(bopomofo_buf, "bopomofo: ");
        }
        if (s_callbacks_context->bopomofo_callback) {
            s_callbacks_context->bopomofo_callback(bopomofo_buf);
        }
        return true;
    }
    return false;
}

//////////////////////////////////////////////////////////////////////////////////////////////
/// Display the current contents of the commit buffer. The commit buffer is the
/// buffer that contains the converted text in Chinese which should be written
/// to the screen and will no longer be subject to conversion.
///
/// @param[in] ctx - the context which holds the buffer
/// @retval bool - false if the buffer is empty, true otherwise.
//////////////////////////////////////////////////////////////////////////////////////////////
bool display_commit_buffer(ChewingContext *ctx)
{
    if (chewing_commit_Check(ctx)) {
        auto commit_buf{const_cast<char *>(chewing_commit_String_static(ctx))};
        if (s_callbacks_context->print_func) {
            s_callbacks_context->print_func(commit_buf, "commit: ");
        }
        if (s_callbacks_context->commit_callback) {
            s_callbacks_context->commit_callback(commit_buf);
        }
        return true;
    }
    return false;
}

//////////////////////////////////////////////////////////////////////////////////////////////
/// Creates a new instance of ChewingContext and configures it.
///
/// @param[in] ctx - the context to be initialized.
//////////////////////////////////////////////////////////////////////////////////////////////
void chewing_init(ChewingContext **ctx, CallbacksContext *callbacks_context)
{
    if (ctx == nullptr) {
        return;
    }

    if (callbacks_context == nullptr) {
        printf("Error: callbacks_context is null\n");
        return;
    }
    s_callbacks_context = callbacks_context;

    const char *data_path = callbacks_context->data_path;
    if (data_path == nullptr) {
        printf("Error: data_path is null\n");
        return;
    }
    if (s_callbacks_context->print_func) {
        s_callbacks_context->print_func(data_path, "data_path: ");
    }

    *ctx = chewing_new2(data_path, nullptr, nullptr, nullptr);
    if (*ctx == nullptr) {
        fprintf(stderr, "Error: chewing_new2 failed to initialize context\n");
        return;
    }
    // Register logger if provided
    if (s_callbacks_context->logger_func) {
        chewing_set_logger(*ctx,
                           chewing_cpp_logger_shim,
                           s_callbacks_context->logger_data);
    }
    // Only configure if initialization succeeded
    chewing_set_candPerPage(*ctx, 10);
    chewing_set_maxChiSymbolLen(*ctx, 18);
    chewing_set_KBType(*ctx, chewing_KBStr2Num("KB_DEFAULT"));
}

void chewing_terminate(ChewingContext **ctx)
{
    if (ctx == nullptr || *ctx == nullptr) {
        return;
    }
    s_callbacks_context = nullptr;
    chewing_delete(*ctx);
    *ctx = nullptr;
}

#ifdef __cplusplus
}
#endif