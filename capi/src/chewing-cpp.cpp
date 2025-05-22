#include <stdarg.h>

#include "chewing-cpp.h"
#include <cstdio> // for printf

#ifdef __cplusplus
#include <functional>
#include <string>

// Helper to wrap nullable C callbacks into std::function with no-op fallback
template <typename R, typename... Args>
static std::function<R(Args...)> make_callback(Callbacks *ctx,
                                               R (*Callbacks::*mem)(Args...))
{
    if (ctx && ctx->*mem) {
        return std::function<R(Args...)>(ctx->*mem);
    }
    return std::function<R(Args...)>([](Args...) { /* no-op */ });
}

class CallbacksWrapper
{
  public:
    explicit CallbacksWrapper(Callbacks *ctx)
        : logger_{make_callback(ctx, &Callbacks::logger_func)},
          candidateInfo_{
              make_callback(ctx, &Callbacks::candidate_info_callback)},
          bufferCallback_{make_callback(ctx, &Callbacks::buffer_callback)},
          bopomofoCallback_{make_callback(ctx, &Callbacks::bopomofo_callback)},
          commitCallback_{make_callback(ctx, &Callbacks::commit_callback)}
    {
    }

    void log(int level, const char *msg) const { logger_(level, msg); }
    void onCandidateInfo(const int pageSize, int numPages, int candidateOnPage,
                         int totalChoices, const char **candidate) const
    {
        candidateInfo_(pageSize, numPages, candidateOnPage, totalChoices,
                       candidate);
    }
    void onBuffer(const char *buffer) const { bufferCallback_(buffer); }
    void onBopomofo(const char *buffer) const { bopomofoCallback_(buffer); }
    void onCommit(const char *buffer) const { commitCallback_(buffer); }

  private:
    std::function<void(int, const char *)> logger_;
    std::function<void(const int, int, int, int, const char **)> candidateInfo_;
    std::function<void(const char *)> bufferCallback_;
    std::function<void(const char *)> bopomofoCallback_;
    std::function<void(const char *)> commitCallback_;
};

#endif // __cplusplus

#ifdef __cplusplus
extern "C" {
#endif

static CallbacksWrapper s_callbacks{nullptr};
static ChewingContext *s_context{nullptr};

constexpr char enterKey = 10;

// shim: format variadic logs into a single message, then call the simple logger
static void chewing_cpp_logger_shim(void *data, int level, const char *fmt, ...)
{
    va_list args{};
    va_start(args, fmt);
    int len = std::vsnprintf(nullptr, 0, fmt, args);
    va_end(args);

    std::string buf;
    buf.resize(len + 1);
    va_start(args, fmt);
    std::vsnprintf(&buf[0], buf.size(), fmt, args);
    va_end(args);
    s_callbacks.log(level, buf.c_str());
}

//////////////////////////////////////////////////////////////////////////////////////////////
/// This function is used to signal to libchewing to bring up the candidates
/// menu, iterates over the candidates and displays them. Displays additional
/// information about the candidates.
///
/// @param[in] ctx - the context which holds the candidates
/// @retval int - the number of canidates
//////////////////////////////////////////////////////////////////////////////////////////////
int fetch_candidates()
{
    chewing_handle_Down(s_context);
    // chewing_cand_open(s_context);
    const int totalChoices{chewing_cand_TotalChoice(s_context)};
    int pageSize{chewing_get_candPerPage(s_context)};
    int numPages{chewing_cand_TotalPage(s_context)};
    int choicePerPage{chewing_cand_ChoicePerPage(s_context)};

    std::vector<const char *> v;
    v.reserve(totalChoices);
    chewing_cand_Enumerate(s_context);
    while (chewing_cand_hasNext(s_context)) {
        auto *cand = chewing_cand_String(s_context);
        if (cand == nullptr) {
            continue;
        }
        v.emplace_back(cand);
    }
    s_callbacks.onCandidateInfo(pageSize, numPages, choicePerPage, totalChoices,
                                v.data());
    for (auto item : v) {
        chewing_free((void *)item);
    }
    // chewing_cand_close(s_context);
    

    chewing_handle_Up(s_context);
    return totalChoices;
}

void select_candidate(int index)
{
    printf("Selecting candidate %d\n", index);
    chewing_handle_Down(s_context);
    chewing_cand_Enumerate(s_context);
    chewing_cand_choose_by_index(s_context, index);
    chewing_handle_Up(s_context);;
        // Trigger callbacks based on updated context
    if (chewing_bopomofo_Check(s_context)) {
    printf("chewing_bopomofo_Check\n");
        s_callbacks.onBopomofo(chewing_bopomofo_String_static(s_context));
    }
    if (chewing_buffer_Check(s_context)) {
    printf("chewing_buffer_Check\n");
        s_callbacks.onBuffer(chewing_buffer_String_static(s_context));
    }
    if (chewing_commit_Check(s_context)) {
    printf("chewing_commit_Check\n");
        s_callbacks.onCommit(chewing_commit_String_static(s_context));
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////
/// Processses the keyboard input. Updates the internal state of the library and
/// the UI of the app. For more information about the each key's functionaly
/// read the README.md in the same directory.
///
/// @param[in] ctx - the context which holds the buffer.
/// @param[in] key - the keyboard key that was inputted.
//////////////////////////////////////////////////////////////////////////////////////////////
void process_key(char key)
{
    switch (key) {
    case enterKey:
        chewing_handle_Enter(s_context);
        break;
    case ' ':
        chewing_handle_Space(s_context);
        break;
    case 127: // Backspace key
        chewing_handle_Backspace(s_context);
        break;
    default:
        chewing_handle_Default(s_context, key);
        fetch_candidates();
        break;
    }

    // Trigger callbacks based on updated context
    if (chewing_bopomofo_Check(s_context)) {
    printf("chewing_bopomofo_Check\n");
        s_callbacks.onBopomofo(chewing_bopomofo_String_static(s_context));
    }
    if (chewing_buffer_Check(s_context)) {
    printf("chewing_buffer_Check\n");
        s_callbacks.onBuffer(chewing_buffer_String_static(s_context));
    }
    if (chewing_commit_Check(s_context)) {
    printf("chewing_commit_Check\n");
        s_callbacks.onCommit(chewing_commit_String_static(s_context));
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////
/// Creates a new instance of ChewingContext and configures it.
///
/// @param[in] ctx - the context to be initialized.
//////////////////////////////////////////////////////////////////////////////////////////////
bool chewing_init(ApplicationContext *ctx)
{
    if (ctx == nullptr) {
        return false;
    }
    s_callbacks = CallbacksWrapper(&ctx->callbacks);

    const char *data_path = ctx->config_data.data_path;
    if (data_path == nullptr) {
        s_callbacks.log(CHEWING_LOG_ERROR, "Error: data_path is null");
        return false;
    }

    s_context = chewing_new2(data_path, nullptr, nullptr, nullptr);
    if (s_context == nullptr) {
        s_callbacks.log(CHEWING_LOG_ERROR,
                        "Error: chewing_new2 failed to initialize context");
        return false;
    }
    // Register logger if provided
    if (ctx->callbacks.logger_func) {
        chewing_set_logger(s_context, chewing_cpp_logger_shim, nullptr);
    }
    // Only configure if initialization succeeded
    chewing_set_candPerPage(s_context, ctx->config_data.candPerPage);
    chewing_set_maxChiSymbolLen(s_context, ctx->config_data.maxChiSymbolLen);
    chewing_set_KBType(s_context, chewing_KBStr2Num("KB_DEFAULT"));

    return true;
}

bool chewing_terminate()
{
    if (s_context == nullptr) {
        s_callbacks.log(CHEWING_LOG_ERROR,
                        "Error: chewing_terminate called with null context");
        return false;
    }
    s_callbacks = CallbacksWrapper(nullptr);

    chewing_delete(s_context);
    s_context = nullptr;

    return true;
}

#ifdef __cplusplus
}
#endif