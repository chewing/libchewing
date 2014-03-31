#ifndef CHEWING_PRIVATE_H
#define CHEWING_PRIVATE_H

#ifdef HAVE_CONFIG_H
#include <config.h>
#endif

/* Platform-dependent declaration */
#include "plat_types.h"

#define LOG_VERBOSE(fmt, ...) \
    do { \
        pgdata->logger(pgdata->loggerData, CHEWING_LOG_VERBOSE, \
                       "[%s:%d %s] " fmt "\n", \
                       __FILE__, __LINE__, __func__, ##__VA_ARGS__); \
    } while (0)

#define LOG_DEBUG(fmt, ...) \
    do { \
        pgdata->logger(pgdata->loggerData, CHEWING_LOG_DEBUG, \
                       "[%s:%d %s] " fmt "\n", \
                       __FILE__, __LINE__, __func__, ##__VA_ARGS__); \
    } while (0)

#define LOG_INFO(fmt, ...) \
    do { \
        pgdata->logger(pgdata->loggerData, CHEWING_LOG_INFO, \
                       "[%s:%d %s] " fmt "\n", \
                       __FILE__, __LINE__, __func__, ##__VA_ARGS__); \
    } while (0)

#define LOG_WARN(fmt, ...) \
    do { \
        pgdata->logger(pgdata->loggerData, CHEWING_LOG_WARN, \
                       "[%s:%d %s] " fmt "\n", \
                       __FILE__, __LINE__, __func__, ##__VA_ARGS__); \
    } while (0)

#define LOG_ERROR(fmt, ...) \
    do { \
        pgdata->logger(pgdata->loggerData, CHEWING_LOG_ERROR, \
                       "[%s:%d %s] " fmt "\n", \
                       __FILE__, __LINE__, __func__, ##__VA_ARGS__); \
    } while (0)

#define DEBUG_OUT( fmt, ... ) \
    do { \
        pgdata->logger(pgdata->loggerData, CHEWING_LOG_INFO, fmt, \
        ##__VA_ARGS__ ); \
    } while (0)

#define DEBUG_CHECKPOINT() \
    do { \
        pgdata->logger(pgdata->loggerData, CHEWING_LOG_VERBOSE, \
                       "[ File: %s  Func: %s  Line: %d ]\n", \
                       __FILE__, __FUNCTION__, __LINE__ ); \
    } while (0)

/* Use LOG_API to log all public API call. */
#define LOG_API(fmt, ...) \
    do { \
        LOG_INFO("API call: " fmt, ##__VA_ARGS__); \
    } while (0)

#define ALC(type, size) \
    (type *) calloc(size, sizeof(type))

#define ASSERT_CONCAT_(a, b) a##b
#define ASSERT_CONCAT(a, b) ASSERT_CONCAT_(a, b)

#ifdef __COUNTER__
#define STATIC_ASSERT(e) \
    { enum { ASSERT_CONCAT(static_assert_, __COUNTER__) = 1/(!!(e)) }; }
#else
  /* This can't be used twice on the same line so ensure if using in headers
   * that the headers are not included twice (by wrapping in #ifndef...#endif)
   * Note it doesn't cause an issue when used on same line of separate modules
   * compiled with gcc -combine -fwhole-program.  */
#define STATIC_ASSERT(e) \
    { enum { ASSERT_CONCAT(assert_line_, __LINE__) = 1/(!!(e)) }; }
#endif

#ifdef __GNUC__
#define ARRAY_SIZE(array) (sizeof(array) / sizeof(((typeof(array)){})[0]))
#else
#define ARRAY_SIZE(array) (sizeof(array) / sizeof(array[0]))
#endif

typedef int (*CompFuncType)(const void *, const void *);

#define CEIL_DIV(a, b) (((a) + (b) - 1) / (b))

#define __stringify(x)  #x

#endif
