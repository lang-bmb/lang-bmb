// BMB Event Loop - Platform-specific async I/O implementation
// v0.98: IOCP (Windows) and epoll (Linux) backends
//
// Design:
// - Registration table maps fd → {callback, user_data, events}
// - Platform backend polls for ready events
// - Dispatches callbacks for ready fds

#include "bmb_event_loop.h"
#include <stdlib.h>
#include <string.h>

// Maximum number of registered fds (simple fixed array for now)
#define BMB_MAX_FDS 1024

// Per-fd registration entry
typedef struct {
    int64_t fd;
    int events;
    bmb_event_callback callback;
    void* user_data;
    int active;
} BmbEventEntry;

// ============================================================================
// Windows IOCP Backend
// ============================================================================
#ifdef _WIN32

#include <winsock2.h>
#include <windows.h>

struct BmbEventLoop {
    BmbEventEntry entries[BMB_MAX_FDS];
    int entry_count;
    volatile int stopped;
    // For Windows, we use WSAPoll (similar to poll) as a simpler first step
    // before full IOCP. WSAPoll works with sockets.
    WSAPOLLFD poll_fds[BMB_MAX_FDS];
};

static int ensure_wsa_init(void) {
    static int initialized = 0;
    if (!initialized) {
        WSADATA wsa;
        if (WSAStartup(MAKEWORD(2, 2), &wsa) != 0) return -1;
        initialized = 1;
    }
    return 0;
}

BmbEventLoop* bmb_event_loop_create(void) {
    if (ensure_wsa_init() != 0) return NULL;
    BmbEventLoop* loop = (BmbEventLoop*)calloc(1, sizeof(BmbEventLoop));
    if (!loop) return NULL;
    loop->entry_count = 0;
    loop->stopped = 0;
    return loop;
}

void bmb_event_loop_destroy(BmbEventLoop* loop) {
    if (loop) free(loop);
}

static int find_entry(BmbEventLoop* loop, int64_t fd) {
    for (int i = 0; i < loop->entry_count; i++) {
        if (loop->entries[i].active && loop->entries[i].fd == fd) return i;
    }
    return -1;
}

int bmb_event_loop_register(BmbEventLoop* loop, int64_t fd, int events,
                            bmb_event_callback callback, void* user_data) {
    if (!loop || !callback) return BMB_EL_ERROR;
    if (loop->entry_count >= BMB_MAX_FDS) return BMB_EL_ERROR;

    // Check if already registered
    int idx = find_entry(loop, fd);
    if (idx >= 0) {
        // Update existing
        loop->entries[idx].events = events;
        loop->entries[idx].callback = callback;
        loop->entries[idx].user_data = user_data;
        return BMB_EL_OK;
    }

    // Find free slot
    idx = loop->entry_count;
    for (int i = 0; i < loop->entry_count; i++) {
        if (!loop->entries[i].active) { idx = i; break; }
    }
    if (idx == loop->entry_count) loop->entry_count++;

    loop->entries[idx].fd = fd;
    loop->entries[idx].events = events;
    loop->entries[idx].callback = callback;
    loop->entries[idx].user_data = user_data;
    loop->entries[idx].active = 1;
    return BMB_EL_OK;
}

int bmb_event_loop_unregister(BmbEventLoop* loop, int64_t fd) {
    if (!loop) return BMB_EL_ERROR;
    int idx = find_entry(loop, fd);
    if (idx < 0) return BMB_EL_ERROR;
    loop->entries[idx].active = 0;
    return BMB_EL_OK;
}

int bmb_event_loop_run_once(BmbEventLoop* loop, int timeout_ms) {
    if (!loop || loop->stopped) return 0;

    // Build WSAPoll array from active entries
    int poll_count = 0;
    int fd_map[BMB_MAX_FDS]; // maps poll index → entry index
    for (int i = 0; i < loop->entry_count; i++) {
        if (!loop->entries[i].active) continue;
        loop->poll_fds[poll_count].fd = (SOCKET)loop->entries[i].fd;
        loop->poll_fds[poll_count].events = 0;
        if (loop->entries[i].events & BMB_EVENT_READ)
            loop->poll_fds[poll_count].events |= POLLIN;
        if (loop->entries[i].events & BMB_EVENT_WRITE)
            loop->poll_fds[poll_count].events |= POLLOUT;
        loop->poll_fds[poll_count].revents = 0;
        fd_map[poll_count] = i;
        poll_count++;
    }

    if (poll_count == 0) return 0;

    int result = WSAPoll(loop->poll_fds, poll_count, timeout_ms);
    if (result <= 0) return result == 0 ? 0 : BMB_EL_ERROR;

    int dispatched = 0;
    for (int i = 0; i < poll_count; i++) {
        if (loop->poll_fds[i].revents == 0) continue;
        int entry_idx = fd_map[i];
        BmbEventEntry* entry = &loop->entries[entry_idx];
        int ready_events = 0;
        if (loop->poll_fds[i].revents & POLLIN) ready_events |= BMB_EVENT_READ;
        if (loop->poll_fds[i].revents & POLLOUT) ready_events |= BMB_EVENT_WRITE;
        if (loop->poll_fds[i].revents & (POLLERR | POLLHUP | POLLNVAL))
            ready_events |= BMB_EVENT_ERROR;
        entry->callback(entry->user_data, entry->fd, ready_events);
        dispatched++;
    }
    return dispatched;
}

void bmb_event_loop_stop(BmbEventLoop* loop) {
    if (loop) loop->stopped = 1;
}

int bmb_event_loop_is_stopped(BmbEventLoop* loop) {
    return loop ? loop->stopped : 1;
}

// ============================================================================
// Linux epoll Backend
// ============================================================================
#elif defined(__linux__)

#include <sys/epoll.h>
#include <unistd.h>

struct BmbEventLoop {
    BmbEventEntry entries[BMB_MAX_FDS];
    int entry_count;
    volatile int stopped;
    int epoll_fd;
};

BmbEventLoop* bmb_event_loop_create(void) {
    BmbEventLoop* loop = (BmbEventLoop*)calloc(1, sizeof(BmbEventLoop));
    if (!loop) return NULL;
    loop->epoll_fd = epoll_create1(0);
    if (loop->epoll_fd < 0) { free(loop); return NULL; }
    loop->entry_count = 0;
    loop->stopped = 0;
    return loop;
}

void bmb_event_loop_destroy(BmbEventLoop* loop) {
    if (loop) {
        if (loop->epoll_fd >= 0) close(loop->epoll_fd);
        free(loop);
    }
}

static int find_entry(BmbEventLoop* loop, int64_t fd) {
    for (int i = 0; i < loop->entry_count; i++) {
        if (loop->entries[i].active && loop->entries[i].fd == fd) return i;
    }
    return -1;
}

int bmb_event_loop_register(BmbEventLoop* loop, int64_t fd, int events,
                            bmb_event_callback callback, void* user_data) {
    if (!loop || !callback) return BMB_EL_ERROR;
    if (loop->entry_count >= BMB_MAX_FDS) return BMB_EL_ERROR;

    int idx = find_entry(loop, fd);
    int is_update = (idx >= 0);

    if (!is_update) {
        idx = loop->entry_count;
        for (int i = 0; i < loop->entry_count; i++) {
            if (!loop->entries[i].active) { idx = i; break; }
        }
        if (idx == loop->entry_count) loop->entry_count++;
    }

    loop->entries[idx].fd = fd;
    loop->entries[idx].events = events;
    loop->entries[idx].callback = callback;
    loop->entries[idx].user_data = user_data;
    loop->entries[idx].active = 1;

    struct epoll_event ev;
    ev.events = 0;
    if (events & BMB_EVENT_READ) ev.events |= EPOLLIN;
    if (events & BMB_EVENT_WRITE) ev.events |= EPOLLOUT;
    ev.data.fd = (int)fd;

    int op = is_update ? EPOLL_CTL_MOD : EPOLL_CTL_ADD;
    if (epoll_ctl(loop->epoll_fd, op, (int)fd, &ev) < 0) return BMB_EL_ERROR;
    return BMB_EL_OK;
}

int bmb_event_loop_unregister(BmbEventLoop* loop, int64_t fd) {
    if (!loop) return BMB_EL_ERROR;
    int idx = find_entry(loop, fd);
    if (idx < 0) return BMB_EL_ERROR;
    loop->entries[idx].active = 0;
    epoll_ctl(loop->epoll_fd, EPOLL_CTL_DEL, (int)fd, NULL);
    return BMB_EL_OK;
}

int bmb_event_loop_run_once(BmbEventLoop* loop, int timeout_ms) {
    if (!loop || loop->stopped) return 0;

    struct epoll_event events[64];
    int nfds = epoll_wait(loop->epoll_fd, events, 64, timeout_ms);
    if (nfds < 0) return BMB_EL_ERROR;

    int dispatched = 0;
    for (int i = 0; i < nfds; i++) {
        int fd = events[i].data.fd;
        int idx = find_entry(loop, (int64_t)fd);
        if (idx < 0) continue;
        BmbEventEntry* entry = &loop->entries[idx];
        int ready_events = 0;
        if (events[i].events & EPOLLIN) ready_events |= BMB_EVENT_READ;
        if (events[i].events & EPOLLOUT) ready_events |= BMB_EVENT_WRITE;
        if (events[i].events & (EPOLLERR | EPOLLHUP)) ready_events |= BMB_EVENT_ERROR;
        entry->callback(entry->user_data, entry->fd, ready_events);
        dispatched++;
    }
    return dispatched;
}

void bmb_event_loop_stop(BmbEventLoop* loop) {
    if (loop) loop->stopped = 1;
}

int bmb_event_loop_is_stopped(BmbEventLoop* loop) {
    return loop ? loop->stopped : 1;
}

// ============================================================================
// Fallback: poll-based (macOS, other POSIX)
// ============================================================================
#else

#include <poll.h>
#include <unistd.h>

struct BmbEventLoop {
    BmbEventEntry entries[BMB_MAX_FDS];
    int entry_count;
    volatile int stopped;
    struct pollfd poll_fds[BMB_MAX_FDS];
};

BmbEventLoop* bmb_event_loop_create(void) {
    BmbEventLoop* loop = (BmbEventLoop*)calloc(1, sizeof(BmbEventLoop));
    if (!loop) return NULL;
    loop->entry_count = 0;
    loop->stopped = 0;
    return loop;
}

void bmb_event_loop_destroy(BmbEventLoop* loop) {
    if (loop) free(loop);
}

static int find_entry(BmbEventLoop* loop, int64_t fd) {
    for (int i = 0; i < loop->entry_count; i++) {
        if (loop->entries[i].active && loop->entries[i].fd == fd) return i;
    }
    return -1;
}

int bmb_event_loop_register(BmbEventLoop* loop, int64_t fd, int events,
                            bmb_event_callback callback, void* user_data) {
    if (!loop || !callback) return BMB_EL_ERROR;
    if (loop->entry_count >= BMB_MAX_FDS) return BMB_EL_ERROR;

    int idx = find_entry(loop, fd);
    if (idx >= 0) {
        loop->entries[idx].events = events;
        loop->entries[idx].callback = callback;
        loop->entries[idx].user_data = user_data;
        return BMB_EL_OK;
    }

    idx = loop->entry_count;
    for (int i = 0; i < loop->entry_count; i++) {
        if (!loop->entries[i].active) { idx = i; break; }
    }
    if (idx == loop->entry_count) loop->entry_count++;

    loop->entries[idx].fd = fd;
    loop->entries[idx].events = events;
    loop->entries[idx].callback = callback;
    loop->entries[idx].user_data = user_data;
    loop->entries[idx].active = 1;
    return BMB_EL_OK;
}

int bmb_event_loop_unregister(BmbEventLoop* loop, int64_t fd) {
    if (!loop) return BMB_EL_ERROR;
    int idx = find_entry(loop, fd);
    if (idx < 0) return BMB_EL_ERROR;
    loop->entries[idx].active = 0;
    return BMB_EL_OK;
}

int bmb_event_loop_run_once(BmbEventLoop* loop, int timeout_ms) {
    if (!loop || loop->stopped) return 0;

    int poll_count = 0;
    int fd_map[BMB_MAX_FDS];
    for (int i = 0; i < loop->entry_count; i++) {
        if (!loop->entries[i].active) continue;
        loop->poll_fds[poll_count].fd = (int)loop->entries[i].fd;
        loop->poll_fds[poll_count].events = 0;
        if (loop->entries[i].events & BMB_EVENT_READ)
            loop->poll_fds[poll_count].events |= POLLIN;
        if (loop->entries[i].events & BMB_EVENT_WRITE)
            loop->poll_fds[poll_count].events |= POLLOUT;
        loop->poll_fds[poll_count].revents = 0;
        fd_map[poll_count] = i;
        poll_count++;
    }

    if (poll_count == 0) return 0;

    int result = poll(loop->poll_fds, poll_count, timeout_ms);
    if (result <= 0) return result == 0 ? 0 : BMB_EL_ERROR;

    int dispatched = 0;
    for (int i = 0; i < poll_count; i++) {
        if (loop->poll_fds[i].revents == 0) continue;
        int entry_idx = fd_map[i];
        BmbEventEntry* entry = &loop->entries[entry_idx];
        int ready_events = 0;
        if (loop->poll_fds[i].revents & POLLIN) ready_events |= BMB_EVENT_READ;
        if (loop->poll_fds[i].revents & POLLOUT) ready_events |= BMB_EVENT_WRITE;
        if (loop->poll_fds[i].revents & (POLLERR | POLLHUP | POLLNVAL))
            ready_events |= BMB_EVENT_ERROR;
        entry->callback(entry->user_data, entry->fd, ready_events);
        dispatched++;
    }
    return dispatched;
}

void bmb_event_loop_stop(BmbEventLoop* loop) {
    if (loop) loop->stopped = 1;
}

int bmb_event_loop_is_stopped(BmbEventLoop* loop) {
    return loop ? loop->stopped : 1;
}

#endif // platform backends
