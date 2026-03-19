#ifndef FREENECT2_SHIM_H
#define FREENECT2_SHIM_H

#ifdef __cplusplus
extern "C" {
#endif

void *fn2_create(void);
void fn2_destroy(void *handle);

#ifdef __cplusplus
}
#endif

#endif /* FREENECT2_SHIM_H */
