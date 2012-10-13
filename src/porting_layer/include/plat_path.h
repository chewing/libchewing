#ifndef __PLAT_PATH_H__
#define __PLAT_PATH_H__

int find_path_by_files( const char *search_path, const char * const *files, char *output, size_t output_len );
void get_search_path( char * path, size_t path_len );

#endif /* __PLAT_PATH_H__ */
