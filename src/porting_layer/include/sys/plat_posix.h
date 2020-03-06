/*
 * plat_posix.h
 *
 * Copyright (c) 2005, 2006, 2008, 2012
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef PLAT_POSIX_H
#    define PLAT_POSIX_H

#    ifdef UNDER_POSIX

#        include <unistd.h>
#        include <sys/types.h>
#        include <sys/stat.h>
#        include <fcntl.h>
#        include <limits.h>

#        include <sys/types.h>
#        include <errno.h>

#        define PLAT_SEPARATOR "/"
#        define PLAT_TMPDIR "/tmp"
#        define PLAT_MKDIR(dir) \
	mkdir(dir, S_IRWXU)
#        define PLAT_RENAME(oldpath, newpath) do { \
             int ret = rename(oldpath, newpath); \
             if (ret == -1) { \
                 LOG_ERROR("rename fails. errno = %d", errno); \
             } \
         } while (0)

#        define PLAT_UNLINK(path) \
	unlink(path)

/* GNU Hurd doesn't define PATH_MAX */
#        ifndef PATH_MAX
#            define PATH_MAX 4096
#        endif

#        ifdef __cplusplus
extern "C" {
#        endif                  /* __cplusplus */

/* plat_mmap.h */
    typedef struct plat_mmap {
        int fd;
        void *address;
        size_t sizet;
        int fAccessAttr;
    } plat_mmap;

#        ifdef __cplusplus
}
#        endif                  /* __cplusplus */
#    endif                      /* UNDER_POSIX */
#endif                          /* PLAT_POSIX_H */
