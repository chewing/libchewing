/*
 * plat_posix.h
 *
 * Copyright (c) 2010, 2011, 2012
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef PLAT_WINDOWS_H
#    define PLAT_WINDOWS_H

#    if defined(_WIN32) || defined(_WIN64) || defined(_WIN32_WCE)

#        include <windows.h>
#        include <stdio.h>
#        include <io.h>

#        if _MSC_VER > 1000
#            include <direct.h>
#            define F_OK	00
#            define W_OK	02
#            define R_OK	04
#        endif

#        ifndef PATH_MAX
#            ifdef MAX_PATH
#                define PATH_MAX MAX_PATH
#            else
#                define PATH_MAX (260)
#            endif
#        endif

#        define PLAT_SEPARATOR "\\"
#        define PLAT_TMPDIR "C:\\Windows\\TEM\\"
#        define PLAT_MKDIR(dir) \
	mkdir(dir)
#        define PLAT_RENAME(oldpath, newpath) \
	MoveFile(oldpath, newpath)
#        define PLAT_UNLINK(path) \
	_unlink(path)

/* strtok_s is simply the Windows version of strtok_r which is standard
   everywhere else.
   FIXME: use strtok_s instead of our own implementation.
 */
#        if !HAVE_STRTOK_R
char *strtok_r(char *s, const char *delim, char **save_ptr);
#        endif

#        ifdef __cplusplus
extern "C" {
#        endif                  /* __cplusplus */

/* plat_mmap.h */
    typedef struct plat_mmap {
        HANDLE fd_file, fd_map;
        void *address;
        int fAccessAttr;
    } plat_mmap;

#        ifdef __cplusplus
}
#        endif                  /* __cplusplus */
#    endif                      /* defined(_WIN32) || defined(_WIN64) || defined(_WIN32_WCE) */
#endif                          /* PLAT_WINDOWS_H */
