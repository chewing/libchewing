/*
 * plat_posix.h
 *
 * Copyright (c) 2010, 2011, 2012
 *      libchewing Core Team.
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

#        define PLAT_SEPARATOR "\\"
#    endif                      /* defined(_WIN32) || defined(_WIN64) || defined(_WIN32_WCE) */
#endif                          /* PLAT_WINDOWS_H */
