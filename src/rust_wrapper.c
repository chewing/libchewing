#include "chewing_internal.h"

void force_linkage()
{
    rust_link_io();
    rust_link_key2pho();
    rust_link_path();
    rust_link_utf8();
}