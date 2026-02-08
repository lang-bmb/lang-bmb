// BMB Event Loop - Platform-specific async I/O abstraction
// v0.98: Foundation for non-blocking async runtime
//
// Windows: IOCP (I/O Completion Ports)
// Linux: epoll
// macOS: kqueue (future)

#ifndef BMB_EVENT_LOOP_H
#define BMB_EVENT_LOOP_H

#include <stdint.h>

// Event types for registration
#define BMB_EVENT_READ  1
#define BMB_EVENT_WRITE 2
#define BMB_EVENT_ERROR 4

// Event loop status codes
#define BMB_EL_OK       0
#define BMB_EL_ERROR   -1
#define BMB_EL_TIMEOUT -2

// Callback signature: (user_data, fd, events)
typedef void (*bmb_event_callback)(void* user_data, int64_t fd, int events);

// Opaque event loop handle
typedef struct BmbEventLoop BmbEventLoop;

// Create a new event loop instance
// Returns NULL on failure
BmbEventLoop* bmb_event_loop_create(void);

// Destroy an event loop instance
void bmb_event_loop_destroy(BmbEventLoop* loop);

// Register a file descriptor for events
// fd: socket or file descriptor
// events: BMB_EVENT_READ | BMB_EVENT_WRITE
// callback: called when events are ready
// user_data: passed to callback
// Returns BMB_EL_OK on success
int bmb_event_loop_register(BmbEventLoop* loop, int64_t fd, int events,
                            bmb_event_callback callback, void* user_data);

// Unregister a file descriptor
// Returns BMB_EL_OK on success
int bmb_event_loop_unregister(BmbEventLoop* loop, int64_t fd);

// Run the event loop once, dispatching ready events
// timeout_ms: max wait time in milliseconds (-1 = block indefinitely, 0 = non-blocking)
// Returns number of events dispatched, or BMB_EL_ERROR on failure
int bmb_event_loop_run_once(BmbEventLoop* loop, int timeout_ms);

// Signal the event loop to stop (from another thread)
void bmb_event_loop_stop(BmbEventLoop* loop);

// Check if the event loop has been stopped
int bmb_event_loop_is_stopped(BmbEventLoop* loop);

#endif // BMB_EVENT_LOOP_H
