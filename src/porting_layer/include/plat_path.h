/**
 * plat_path.h
 *
 * Copyright (c) 2012
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef PATH_PRIVATE_H
#    define PATH_PRIVATE_H

#    ifdef HAVE_CONFIG_H
#        include <config.h>
#    endif

#    include <stddef.h>

#    ifdef UNDER_POSIX
#        define SEARCH_PATH_SEP ":"

#    elif defined(_WIN32) || defined(_WIN64) || defined(_WIN32_WCE)
#        define SEARCH_PATH_SEP ";"

#    else
#        error please define SEARCH_PATH_SEP
#    endif

int get_search_path(char *path, size_t path_len);
int find_path_by_files(const char *search_path, const char *const *files, char *output, size_t output_len);

#    ifndef HAVE_ASPRINTF
int asprintf(char **strp, const char *fmt, ...);
#    endif

#endif                          // PATH_PRIVATE_H
