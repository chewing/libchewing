#include "chewing-simplified.h"

#ifdef __cplusplus
#include <algorithm>
#include <functional>
#include <memory>
#include <string>
#include <vector>

// Helper to wrap nullable C callbacks into std::function with no-op fallback
template <typename R, typename... Args>
static std::function<R(Args...)>
make_callback(cs_callbacks_t &ctx, R (*cs_callbacks_t::*mem)(Args...))
{
    if (ctx.*mem) {
        return std::function<R(Args...)>(ctx.*mem);
    }
    return std::function<R(Args...)>([](Args...) { /* no-op */ });
}

class CallbacksWrapper
{
  public:
    explicit CallbacksWrapper(cs_callbacks_t ctx)
        : logger_{make_callback(ctx, &cs_callbacks_t::logger)},
          candidateInfo_{make_callback(ctx, &cs_callbacks_t::candidate_info)},
          bufferCallback_{make_callback(ctx, &cs_callbacks_t::buffer)},
          bopomofoCallback_{make_callback(ctx, &cs_callbacks_t::bopomofo)},
          commitCallback_{make_callback(ctx, &cs_callbacks_t::commit)}
    {
    }

    void log(const int level, const char *msg) const { logger_(level, msg); }
    void onCandidateInfo(const int pageSize, const int numPages,
                         const int candidateOnPage, const int totalChoices,
                         const char **candidate) const
    {
        candidateInfo_(pageSize, numPages, candidateOnPage, totalChoices,
                       candidate);
    }
    void onBuffer(const char *buffer) const { bufferCallback_(buffer); }
    void onBopomofo(const char *buffer) const { bopomofoCallback_(buffer); }
    void onCommit(const char *buffer) const { commitCallback_(buffer); }

  private:
    std::function<void(const int, const char *)> logger_;
    std::function<void(const int, const int, const int, const int,
                       const char **)>
        candidateInfo_;
    std::function<void(const char *)> bufferCallback_;
    std::function<void(const char *)> bopomofoCallback_;
    std::function<void(const char *)> commitCallback_;
};

#endif // __cplusplus

#ifdef __cplusplus
extern "C" {
#endif

static ChewingContext *s_context{nullptr};
static CallbacksWrapper s_callbacks{cs_callbacks_t{}};

/**
 * @brief Formats variadic logs into a single message and invokes the CS logger
 * callback.
 * @param data User-provided logger data pointer.
 * @param level Log level severity.
 * @param fmt printf-style format string.
 * @param ... Arguments for the format string.
 */
static void cs_logger_shim(void *data, int level, const char *fmt, ...)
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

/**
 * @brief Notifies application of the current IME preedit, buffer, and commit
 * state.
 */
static void cs_notify_state_change()
{
    if (s_context == nullptr) {
        s_callbacks.log(CHEWING_LOG_ERROR,
                        "cs_notify_state_change called with null context");
        return;
    }

    if (chewing_bopomofo_Check(s_context)) {
        s_callbacks.onBopomofo(chewing_bopomofo_String_static(s_context));
    }
    if (chewing_buffer_Check(s_context)) {
        s_callbacks.onBuffer(chewing_buffer_String_static(s_context));
    }
    if (chewing_commit_Check(s_context)) {
        s_callbacks.onCommit(chewing_commit_String_static(s_context));
    }
}

/**
 * @brief Fetches candidate list from ChewingContext and invokes the candidate
 * info callback.
 * @return Total number of candidates fetched.
 */
static void cs_fetch_candidates()
{
    if (s_context == nullptr) {
        s_callbacks.log(CHEWING_LOG_ERROR,
                        "cs_fetch_candidates called with null context");
        return;
    }

    chewing_handle_Down(s_context);

    const int totalChoices = chewing_cand_TotalChoice(s_context);
    const int pageSize = chewing_get_candPerPage(s_context);
    const int numPages = chewing_cand_TotalPage(s_context);
    const int choicePerPage = chewing_cand_ChoicePerPage(s_context);

    if (totalChoices <= 0) {
        chewing_handle_Up(s_context);
        return;
    }

    size_t safeCount = static_cast<size_t>(totalChoices);
    std::vector<const char *> rawPointers;
    rawPointers.reserve(safeCount);

    chewing_cand_Enumerate(s_context);
    while (chewing_cand_hasNext(s_context)) {
        if (auto *raw = chewing_cand_String(s_context)) {
            rawPointers.emplace_back(raw);
        }
    }

    s_callbacks.onCandidateInfo(pageSize, numPages, choicePerPage, totalChoices,
                                rawPointers.data());

    // Clean up raw pointers
    for (const auto *ptr : rawPointers) {
        chewing_free(const_cast<char *>(ptr));
    }

    chewing_handle_Up(s_context);
}

/**
 * @brief Selects a candidate at the specified index and updates state.
 * @param index Zero-based index of the candidate to select.
 */
void cs_select_candidate(const int index)
{
    if (s_context == nullptr) {
        s_callbacks.log(CHEWING_LOG_ERROR,
                        "cs_select_candidate called with null context");
        return;
    }
    if (index < 0) {
        std::string msg = "cs_select_candidate called with invalid index " +
                          std::to_string(index);
        s_callbacks.log(CHEWING_LOG_ERROR, msg.c_str());
        return;
    }

    chewing_handle_Down(s_context);
    chewing_cand_Enumerate(s_context);
    chewing_cand_choose_by_index(s_context, index);
    chewing_handle_Up(s_context);

    cs_notify_state_change();
}

/**
 * @brief Processes a keyboard input through CS and updates state.
 * @param key Input key character.
 */
void cs_process_key(const char key)
{
    if (s_context == nullptr) {
        s_callbacks.log(CHEWING_LOG_ERROR,
                        "cs_process_key called with null context");
        return;
    }

    switch (key) {
    case CHEWING_KEY_Enter:
        chewing_handle_Enter(s_context);
        break;
    case CHEWING_KEY_Space:
        chewing_handle_Space(s_context);
        cs_fetch_candidates();
        break;
    case CHEWING_KEY_Backspace: // Backspace key
        chewing_handle_Backspace(s_context);
        cs_fetch_candidates();
        break;
    default:
        chewing_handle_Default(s_context, key);
        cs_fetch_candidates();
        break;
    }

    cs_notify_state_change();
}

/**
 * @brief Initializes the CS context with provided configuration and callbacks.
 * @param ctx Pointer to cs_context_t with config and callbacks.
 * @return true on successful initialization, false otherwise.
 */
bool cs_init(const cs_context_t *ctx)
{
    if (ctx == nullptr) {
        return false;
    }

    const char *data_path = ctx->config.data_path;
    if (data_path == nullptr) {
        return false;
    }

    if (s_context != nullptr) {
        cs_terminate();
    }

    s_callbacks = CallbacksWrapper(ctx->callbacks);

    s_context = chewing_new2(data_path, nullptr, cs_logger_shim, nullptr);
    if (s_context == nullptr) {
        s_callbacks.log(CHEWING_LOG_ERROR,
                        "chewing_new2 failed to initialize context");
        s_callbacks = CallbacksWrapper(cs_callbacks_t{});
        return false;
    }

    // Only configure if initialization succeeded
    chewing_set_candPerPage(s_context, ctx->config.cand_per_page);
    chewing_set_maxChiSymbolLen(s_context, ctx->config.max_chi_symbol_len);
    chewing_set_KBType(s_context, chewing_KBStr2Num("KB_DEFAULT"));

    return true;
}

/**
 * @brief Terminates the CS context, releases resources, and resets state.
 * @return true on success, false if context was null.
 */
bool cs_terminate(void)
{
    if (s_context == nullptr) {
        s_callbacks.log(CHEWING_LOG_ERROR,
                        "cs_terminate called with null context");

        s_callbacks = CallbacksWrapper(cs_callbacks_t{});
        return false;
    }
    s_callbacks = CallbacksWrapper(cs_callbacks_t{});

    chewing_delete(s_context);
    s_context = nullptr;

    return true;
}

#ifdef __cplusplus
}
#endif