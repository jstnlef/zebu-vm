#ifndef __MU_IMPL_FAST_H__
#define __MU_IMPL_FAST_H__

#include "muapi.h"

#ifdef __cplusplus
extern "C" {
#endif

MuVM *mu_fastimpl_new();
MuVM *mu_fastimpl_new_with_opts(const char*);

#ifdef __cplusplus
}
#endif

#endif // __MU_IMPL_FAST_H__
