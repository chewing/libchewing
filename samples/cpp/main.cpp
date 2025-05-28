#include "chewing-simplified.h"
#include <cstdio>
#include <iostream>
#include <string_view>
#include <termios.h>
#include <unistd.h>
#include <vector>

constexpr char escapeKey = 27;

/// helper to enter raw mode
void enableRawMode(struct termios *orig)
{
    struct termios raw = *orig;
    raw.c_lflag &= ~(ECHO | ICANON);
    tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw);
}

/// restore cooked mode
void disableRawMode()
{
    struct termios orig;
    tcgetattr(STDIN_FILENO, &orig);
    tcsetattr(STDIN_FILENO, TCSAFLUSH, &orig);
}

int main()
{
    // prepare terminal
    struct termios orig;
    tcgetattr(STDIN_FILENO, &orig);
    atexit(disableRawMode);
    enableRawMode(&orig);

    cs_context_t csCtx{};
    csCtx.config.data_path = "../build/cpp/lib/libchewing/data";
    csCtx.config.cand_per_page = 10;
    csCtx.config.max_chi_symbol_len = 18;

    csCtx.callbacks.candidate_info = [](int pageSize, int numPages,
                                        int candidateOnPage, int totalChoices,
                                        const char **candidates) {
        std::cout << "Candidates [" << candidateOnPage << "/" << totalChoices
                  << "]\n";
        std::vector<std::string_view> options(candidates,
                                              candidates + totalChoices);
        for (int i = 0; i < totalChoices; ++i) {
            std::cout << "  " << i << ": " << options[i] << "\n";
        }
    };
    csCtx.callbacks.buffer = [](const char *buf) {
        std::cout << "Buffer:   " << buf << "\n";
    };
    csCtx.callbacks.bopomofo = [](const char *buf) {
        std::cout << "Preedit:  " << buf << "\n";
    };
    csCtx.callbacks.commit = [](const char *buf) {
        std::cout << "Commit:   " << buf << "\n";
    };
    csCtx.callbacks.logger = [](int level, const char *msg) {
        if (level == CHEWING_LOG_VERBOSE) {
            return;
        }
        const char *lvl = level == CHEWING_LOG_VERBOSE ? "VERBOSE"
                          : level == CHEWING_LOG_DEBUG ? "DEBUG"
                          : level == CHEWING_LOG_INFO  ? "INFO"
                          : level == CHEWING_LOG_WARN  ? "WARN"
                          : level == CHEWING_LOG_ERROR ? "ERROR"
                                                       : "UNKNOWN";
        std::cout << "[" << lvl << "] " << msg << "\n";
    };

    // initialize
    if (!cs_init(&csCtx)) {
        std::cerr << "Failed to initialize libchewing\n";
        return 1;
    }

    // event loop
    char ch;
    while ((ch = std::getchar()) != escapeKey) {
        if (ch == '`') {
            cs_select_candidate(5);
            continue;
        }
        std::cout << "\n---------------------------\n";
        std::cout << "You pressed: " << ch << std::endl;
        cs_process_key(ch);
        std::cout << "---------------------------\n";
    }

    std::cout << "Program terminated.\n";
    cs_terminate();
    return 0;
}