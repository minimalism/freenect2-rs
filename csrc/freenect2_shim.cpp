#include "freenect2_shim.h"

#include <libfreenect2/frame_listener_impl.h>
#include <libfreenect2/libfreenect2.hpp>

#include <cstring>

namespace {

using libfreenect2::Freenect2;
using libfreenect2::Freenect2Device;
using libfreenect2::Frame;
using libfreenect2::FrameMap;
using libfreenect2::SyncMultiFrameListener;

struct ListenerCtx {
  SyncMultiFrameListener listener;
  FrameMap frames;

  ListenerCtx()
      : listener(Frame::Color | Frame::Ir | Frame::Depth), frames() {}
};

void frame_data_clear(FrameData *out) {
  if (out != nullptr) {
    std::memset(out, 0, sizeof(FrameData));
  }
}

} /* namespace */

extern "C" Freenect2Handle fn2_create(void) { return new Freenect2(); }

extern "C" void fn2_destroy(Freenect2Handle handle) {
  if (handle == nullptr) {
    return;
  }
  delete static_cast<Freenect2 *>(handle);
}

extern "C" int fn2_enumerate_devices(Freenect2Handle f2) {
  if (f2 == nullptr) {
    return 0;
  }
  return static_cast<Freenect2 *>(f2)->enumerateDevices();
}

extern "C" DeviceHandle fn2_open_device(Freenect2Handle f2, int index) {
  if (f2 == nullptr) {
    return nullptr;
  }
  return static_cast<Freenect2 *>(f2)->openDevice(index);
}

extern "C" void fn2_start_device(DeviceHandle dev) {
  if (dev == nullptr) {
    return;
  }
  static_cast<Freenect2Device *>(dev)->start();
}

extern "C" void fn2_stop_device(DeviceHandle dev) {
  if (dev == nullptr) {
    return;
  }
  static_cast<Freenect2Device *>(dev)->stop();
}

extern "C" void fn2_close_device(DeviceHandle dev) {
  if (dev == nullptr) {
    return;
  }
  static_cast<Freenect2Device *>(dev)->close();
}

extern "C" ListenerHandle fn2_create_sync_listener(void) {
  return new ListenerCtx();
}

extern "C" void fn2_destroy_listener(ListenerHandle listener) {
  if (listener == nullptr) {
    return;
  }
  auto *ctx = static_cast<ListenerCtx *>(listener);
  if (!ctx->frames.empty()) {
    ctx->listener.release(ctx->frames);
  }
  ctx->frames.clear();
  delete ctx;
}

extern "C" void fn2_set_listeners(DeviceHandle dev, ListenerHandle listener) {
  if (dev == nullptr || listener == nullptr) {
    return;
  }
  auto *d = static_cast<Freenect2Device *>(dev);
  auto *ctx = static_cast<ListenerCtx *>(listener);
  d->setColorFrameListener(&ctx->listener);
  d->setIrAndDepthFrameListener(&ctx->listener);
}

extern "C" int fn2_wait_for_frame(ListenerHandle listener, FrameData *out,
                                  int timeout_ms) {
  if (listener == nullptr || out == nullptr) {
    return 0;
  }
  frame_data_clear(out);
  auto *ctx = static_cast<ListenerCtx *>(listener);
  if (!ctx->frames.empty()) {
    ctx->listener.release(ctx->frames);
    ctx->frames.clear();
  }
  if (!ctx->listener.waitForNewFrame(ctx->frames, timeout_ms)) {
    return 0;
  }
  Frame *color = nullptr;
  Frame *depth = nullptr;
  auto itc = ctx->frames.find(Frame::Color);
  auto itd = ctx->frames.find(Frame::Depth);
  if (itc != ctx->frames.end()) {
    color = itc->second;
  }
  if (itd != ctx->frames.end()) {
    depth = itd->second;
  }
  if (color == nullptr || depth == nullptr) {
    ctx->listener.release(ctx->frames);
    ctx->frames.clear();
    return 0;
  }
  out->color_data = color->data;
  out->color_width = static_cast<int>(color->width);
  out->color_height = static_cast<int>(color->height);
  out->color_bpp = static_cast<int>(color->bytes_per_pixel);
  out->depth_data = depth->data;
  out->depth_width = static_cast<int>(depth->width);
  out->depth_height = static_cast<int>(depth->height);
  out->depth_bpp = static_cast<int>(depth->bytes_per_pixel);
  return 1;
}

extern "C" void fn2_release_frame(ListenerHandle listener, FrameData *out) {
  if (listener == nullptr) {
    return;
  }
  auto *ctx = static_cast<ListenerCtx *>(listener);
  if (!ctx->frames.empty()) {
    ctx->listener.release(ctx->frames);
  }
  ctx->frames.clear();
  frame_data_clear(out);
}
