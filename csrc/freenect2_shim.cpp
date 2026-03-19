#include "freenect2_shim.h"

#include <libfreenect2/libfreenect2.hpp>

extern "C" void *fn2_create(void) {
  return new libfreenect2::Freenect2();
}

extern "C" void fn2_destroy(void *handle) {
  delete static_cast<libfreenect2::Freenect2 *>(handle);
}
