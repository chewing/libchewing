#ifndef CHEWING_PRIVATE_H
#define CHEWING_PRIVATE_H

#ifdef HAVE_CONFIG_H
  #include <config.h>
#endif

/* Platform-dependent declaration */
#include "plat_types.h"

#ifdef ENABLE_DEBUG
#include <stdarg.h>
#include <stdio.h>
extern FILE *fp_g;
#define DEBUG_OUT( ... ) \
	do { \
		if ( fp_g ) { \
			fprintf( fp_g, __VA_ARGS__ ); \
		} \
		else { \
			fprintf( stderr, __VA_ARGS__ ); \
		} \
	} while (0)
#define DEBUG_CHECKPOINT() \
	DEBUG_OUT( "[ File: %s  Func: %s  Line: %d ]\n", __FILE__, __FUNCTION__, __LINE__ )
#define DEBUG_FLUSH \
	do { \
		if ( fp_g ) { \
			fflush( fp_g ); \
		} \
	} while (0)
#define EMPHASIZE(real_string) \
	"\033[44;37m"real_string"\033[m"

#else /* ! ENABLE_DEBUG */
#if _MSC_VER > 1000     // Vsual C++ compiler
__forceinline void DEBUG_OUT( char* str, ... ){ }
#else
#define DEBUG_OUT( ... )
#endif /* _MSC_VER > 1000 */
#define DEBUG_FLUSH
#define DEBUG_CHECKPOINT()
#endif

#define ALC(type, size) \
	(type *) calloc( size, sizeof( type ) )

#define ASSERT_CONCAT_(a, b) a##b
#define ASSERT_CONCAT(a, b) ASSERT_CONCAT_(a, b)
#ifdef __COUNTER__
  #define STATIC_ASSERT(e,m) \
    { enum { ASSERT_CONCAT(static_assert_, __COUNTER__) = 1/(!!(e)) }; }
#else
  /* This can't be used twice on the same line so ensure if using in headers
   * that the headers are not included twice (by wrapping in #ifndef...#endif)
   * Note it doesn't cause an issue when used on same line of separate modules
   * compiled with gcc -combine -fwhole-program.  */
  #define STATIC_ASSERT(e,m) \
    { enum { ASSERT_CONCAT(assert_line_, __LINE__) = 1/(!!(e)) }; }
#endif

#ifdef __GNUC__
#define ARRAY_SIZE( array ) ( sizeof(array) / sizeof(((typeof(array)){})[0]) )
#else
#define ARRAY_SIZE( array ) ( sizeof(array) / sizeof(array[0] ) )
#endif

typedef int (*CompFuncType)( const void *, const void * );

#endif
