#ifndef __PLAT_PATH_H__
#define __PLAT_PATH_H__

#ifdef UNDER_POSIX
#define SEARCH_PATH_SEP ":"

#elif defined(_WIN32) || defined(_WIN64) || defined(_WIN32_WCE)
#define SEARCH_PATH_SEP ";"
char * strtok_r (char *s, const char *delim, char **save_ptr);

#else
#error Please defined SEARCH_PATH_SEP
#endif

int get_search_path( char * path, size_t path_len );

#endif /* __PLAT_PATH_H__ */
