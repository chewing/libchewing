#ifndef CHEWING_PRIVATE_H
#define CHEWING_PRIVATE_H

#ifdef HAVE_CONFIG_H
  #include <config.h>
#endif

#ifdef ENABLE_DEBUG
extern FILE *fp_g;
#endif

#define ALC(type, size) \
	(type *) calloc( size, sizeof( type ) )

typedef int (*CompFuncType)( const void *, const void * );

#define TerminateServicesNUM 5
extern int addTerminateService( void (*callback)() );
#endif
