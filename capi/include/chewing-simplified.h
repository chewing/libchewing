/**
 * @file chewing-simplified.h
 * @brief Simplified C API for libchewing with Chewing Simplified (CS) prefix.
 *
 * Provides functions and callbacks for simplified integration of libchewing.
 */
#ifndef CHEWING_SIMPLIFIED_H
#define CHEWING_SIMPLIFIED_H

#pragma once

#include "chewing.h"

#ifdef __cplusplus
extern "C" {
#endif

static const char CHEWING_KEY_Enter = 10;      // Enter key
static const char CHEWING_KEY_Space = ' ';     // Space key
static const char CHEWING_KEY_Backspace = 127; // Backspace key

/** @brief Callback invoked with candidate list details.
 *  @param pageSize Number of candidates per page.
 *  @param numPages Total pages available.
 *  @param candidateOnPage Index of current candidate on the page.
 *  @param totalChoices Total candidate count.
 *  @param candidates Null-terminated array of candidate strings.
 */
typedef void (*cs_candidate_info_callback_t)(const int pageSize,
                                             const int numPages,
                                             const int candidateOnPage,
                                             const int totalChoices,
                                             const char **candidates);

/** @brief Callback invoked when edit buffer changes.
 *  @param buffer Null-terminated pre-edit buffer string.
 */
typedef void (*cs_buffer_callback_t)(const char *buffer);

/** @brief Callback invoked when bopomofo (pre-edit) buffer changes.
 *  @param buffer Null-terminated bopomofo buffer string.
 */
typedef void (*cs_bopomofo_callback_t)(const char *buffer);

/** @brief Callback invoked when text is committed.
 *  @param buffer Null-terminated committed text string.
 */
typedef void (*cs_commit_callback_t)(const char *buffer);

/** @brief Logger callback for CS events.
 *  @param level Log level (CHEWING_LOG_VERBOSE, CHEWING_LOG_DEBUG,
 * CHEWING_LOG_INFO, CHEWING_LOG_WARN, CHEWING_LOG_ERROR).
 *  @param message Null-terminated log message.
 */
typedef void (*cs_logger_callback_t)(const int level, const char *message);

/** @brief Configuration for CS integration.
 *  @param data_path Filesystem path to chewing data files.
 *  @param cand_per_page Number of candidates to fetch per page.
 *  @param max_chi_symbol_len Maximum length of a Chinese symbol sequence.
 */
typedef struct cs_config_s {
    char *data_path;
    int cand_per_page;
    int max_chi_symbol_len;
} cs_config_t;

/** @brief Collection of CS callback functions for UI integration. */
typedef struct cs_callbacks_s {
    cs_candidate_info_callback_t candidate_info;
    cs_buffer_callback_t buffer;
    cs_bopomofo_callback_t bopomofo;
    cs_commit_callback_t commit;
    cs_logger_callback_t logger;
} cs_callbacks_t;

/** @brief Context object holding CS configuration and callbacks. */
typedef struct cs_context_s {
    cs_config_t config;
    cs_callbacks_t callbacks;
} cs_context_t;

/** @brief Initialize CS context.
 *  @param ctx Pointer to cs_context_t.
 *  @return true on success, false on failure.
 */
bool cs_init(const cs_context_t *ctx);

/** @brief Terminate CS context and release resources.
 *  @return true on success, false on failure.
 */
bool cs_terminate(void);

/** @brief Process a single key input through CS.
 *  @param key Input key character.
 */
void cs_process_key(const char key);

/** @brief Select a candidate from the current list.
 *  @param index Zero-based index of the candidate to select.
 */
void cs_select_candidate(const int index);

#ifdef __cplusplus
}
#endif

#endif // CHEWING_SIMPLIFIED_H