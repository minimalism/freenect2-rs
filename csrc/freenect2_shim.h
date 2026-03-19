#ifndef FREENECT2_SHIM_H
#define FREENECT2_SHIM_H

#ifdef __cplusplus
extern "C" {
#endif

typedef void *Freenect2Handle;
typedef void *DeviceHandle;
typedef void *ListenerHandle;

typedef struct {
  void *color_data;
  int color_width, color_height, color_bpp;
  /** `libfreenect2::Frame::Format` for the color frame (e.g. BGRX = 4, RGBX = 5). */
  int color_format;
  void *depth_data;
  int depth_width, depth_height, depth_bpp;
  /** `libfreenect2::Frame::Format` for the depth frame (typically Float = 2). */
  int depth_format;
} FrameData;

Freenect2Handle fn2_create(void);
void fn2_destroy(Freenect2Handle handle);

int fn2_enumerate_devices(Freenect2Handle f2);
DeviceHandle fn2_open_device(Freenect2Handle f2, int index);
void fn2_start_device(DeviceHandle dev);
void fn2_stop_device(DeviceHandle dev);
void fn2_close_device(DeviceHandle dev);

ListenerHandle fn2_create_sync_listener(void);
void fn2_destroy_listener(ListenerHandle listener);
void fn2_set_listeners(DeviceHandle dev, ListenerHandle listener);

/** Returns 1 on success, 0 on timeout or incomplete color/depth pair. */
int fn2_wait_for_frame(ListenerHandle listener, FrameData *out, int timeout_ms);
void fn2_release_frame(ListenerHandle listener, FrameData *out);

#ifdef __cplusplus
}
#endif

#endif /* FREENECT2_SHIM_H */
