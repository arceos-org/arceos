#include <errno.h>
#include <pthread.h>
#include <pwd.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/uio.h>

#define FIX(x) (pw->pw_##x = pw->pw_##x - line + buf)

#ifdef AX_CONFIG_ALLOC

static unsigned atou(char **s)
{
    unsigned x;
    for (x = 0; **s - '0' < 10U; ++*s) x = 10 * x + (**s - '0');
    return x;
}

int __getpwent_a(FILE *f, struct passwd *pw, char **line, size_t *size, struct passwd **res)
{
    ssize_t l;
    char *s;
    int rv = 0;
    int cs;
    pthread_setcancelstate(PTHREAD_CANCEL_DISABLE, &cs);
    for (;;) {
        if ((l = getline(line, size, f)) < 0) {
            rv = ferror(f) ? errno : 0;
            free(*line);
            *line = 0;
            pw = 0;
            break;
        }
        line[0][l - 1] = 0;

        s = line[0];
        pw->pw_name = s++;
        if (!(s = strchr(s, ':')))
            continue;

        *s++ = 0;
        pw->pw_passwd = s;
        if (!(s = strchr(s, ':')))
            continue;

        *s++ = 0;
        pw->pw_uid = atou(&s);
        if (*s != ':')
            continue;

        *s++ = 0;
        pw->pw_gid = atou(&s);
        if (*s != ':')
            continue;

        *s++ = 0;
        pw->pw_gecos = s;
        if (!(s = strchr(s, ':')))
            continue;

        *s++ = 0;
        pw->pw_dir = s;
        if (!(s = strchr(s, ':')))
            continue;

        *s++ = 0;
        pw->pw_shell = s;
        break;
    }
    pthread_setcancelstate(cs, 0);
    *res = pw;
    if (rv)
        errno = rv;
    return rv;
}
#endif // AX_CONFIG_ALLOC

static __inline uint32_t bswap_32(uint32_t __x)
{
    return __x >> 24 | (__x >> 8 & 0xff00) | (__x << 8 & 0xff0000) | __x << 24;
}

#if defined(AX_CONFIG_NET) && defined(AX_CONFIG_FS)

static const struct {
    short sun_family;
    char sun_path[21];
} addr = {AF_UNIX, "/var/run/nscd/socket"};

FILE *__nscd_query(int32_t req, const char *key, int32_t *buf, size_t len, int *swap)
{
    size_t i;
    int fd;
    FILE *f = 0;
    int32_t req_buf[REQ_LEN] = {NSCDVERSION, req, strnlen(key, LOGIN_NAME_MAX) + 1};
    struct msghdr msg = {
        .msg_iov = (struct iovec[]){{&req_buf, sizeof(req_buf)}, {(char *)key, strlen(key) + 1}},
        .msg_iovlen = 2};
    int errno_save = errno;

    *swap = 0;
retry:
    memset(buf, 0, len);
    buf[0] = NSCDVERSION;

    fd = socket(PF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0);
    if (fd < 0) {
        if (errno == EAFNOSUPPORT) {
            f = fopen("/dev/null", "re");
            if (f)
                errno = errno_save;
            return f;
        }
        return 0;
    }

    if (!(f = fdopen(fd, "r"))) {
        close(fd);
        return 0;
    }

    if (req_buf[2] > LOGIN_NAME_MAX)
        return f;

    if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        /* If there isn't a running nscd we simulate a "not found"
         * result and the caller is responsible for calling
         * fclose on the (unconnected) socket. The value of
         * errno must be left unchanged in this case.  */
        if (errno == EACCES || errno == ECONNREFUSED || errno == ENOENT) {
            errno = errno_save;
            return f;
        }
        goto error;
    }

    if (sendmsg(fd, &msg, MSG_NOSIGNAL) < 0)
        goto error;

    if (!fread(buf, len, 1, f)) {
        /* If the VERSION entry mismatches nscd will disconnect. The
         * most likely cause is that the endianness mismatched. So, we
         * byteswap and try once more. (if we already swapped, just
         * fail out)
         */
        if (ferror(f))
            goto error;
        if (!*swap) {
            fclose(f);
            for (i = 0; i < sizeof(req_buf) / sizeof(req_buf[0]); i++) {
                req_buf[i] = bswap_32(req_buf[i]);
            }
            *swap = 1;
            goto retry;
        } else {
            errno = EIO;
            goto error;
        }
    }

    if (*swap) {
        for (i = 0; i < len / sizeof(buf[0]); i++) {
            buf[i] = bswap_32(buf[i]);
        }
    }

    /* The first entry in every nscd response is the version number. This
     * really shouldn't happen, and is evidence of some form of malformed
     * response.
     */
    if (buf[0] != NSCDVERSION) {
        errno = EIO;
        goto error;
    }

    return f;
error:
    fclose(f);
    return 0;
}

static char *itoa(char *p, unsigned x)
{
    p += 3 * sizeof(int);
    *--p = 0;
    do {
        *--p = '0' + x % 10;
        x /= 10;
    } while (x);
    return p;
}

int __getpw_a(const char *name, uid_t uid, struct passwd *pw, char **buf, size_t *size,
              struct passwd **res)
{
    FILE *f;
    int cs;
    int rv = 0;

    *res = 0;

    pthread_setcancelstate(PTHREAD_CANCEL_DISABLE, &cs);

    f = fopen("/etc/passwd", "rbe");
    if (!f) {
        rv = errno;
        goto done;
    }

    while (!(rv = __getpwent_a(f, pw, buf, size, res)) && *res) {
        if ((name && (!strcmp(name, (*res)->pw_name))) || (!name && (*res)->pw_uid == uid))
            break;
    }
    fclose(f);

    if (!*res && (rv == 0 || rv == ENOENT || rv == ENOTDIR)) {
        int32_t req = name ? GETPWBYNAME : GETPWBYUID;
        const char *key;
        int32_t passwdbuf[PW_LEN] = {0};
        size_t len = 0;
        char uidbuf[11] = {0};

        if (name) {
            key = name;
        } else {
            /* uid outside of this range can't be queried with the
             * nscd interface, but might happen if uid_t ever
             * happens to be a larger type (this is not true as of
             * now)
             */
            if (uid < 0 || uid > UINT32_MAX) {
                rv = 0;
                goto done;
            }
            key = itoa(uidbuf, uid);
        }

        f = __nscd_query(req, key, passwdbuf, sizeof passwdbuf, (int[]){0});
        if (!f) {
            rv = errno;
            goto done;
        }

        if (!passwdbuf[PWFOUND]) {
            rv = 0;
            goto cleanup_f;
        }

        /* A zero length response from nscd is invalid. We ignore
         * invalid responses and just report an error, rather than
         * trying to do something with them.
         */
        if (!passwdbuf[PWNAMELEN] || !passwdbuf[PWPASSWDLEN] || !passwdbuf[PWGECOSLEN] ||
            !passwdbuf[PWDIRLEN] || !passwdbuf[PWSHELLLEN]) {
            rv = EIO;
            goto cleanup_f;
        }

        if ((passwdbuf[PWNAMELEN] | passwdbuf[PWPASSWDLEN] | passwdbuf[PWGECOSLEN] |
             passwdbuf[PWDIRLEN] | passwdbuf[PWSHELLLEN]) >= SIZE_MAX / 8) {
            rv = ENOMEM;
            goto cleanup_f;
        }

        len = passwdbuf[PWNAMELEN] + passwdbuf[PWPASSWDLEN] + passwdbuf[PWGECOSLEN] +
              passwdbuf[PWDIRLEN] + passwdbuf[PWSHELLLEN];

        if (len > *size || !*buf) {
            char *tmp = realloc(*buf, len);
            if (!tmp) {
                rv = errno;
                goto cleanup_f;
            }
            *buf = tmp;
            *size = len;
        }

        if (!fread(*buf, len, 1, f)) {
            rv = ferror(f) ? errno : EIO;
            goto cleanup_f;
        }

        pw->pw_name = *buf;
        pw->pw_passwd = pw->pw_name + passwdbuf[PWNAMELEN];
        pw->pw_gecos = pw->pw_passwd + passwdbuf[PWPASSWDLEN];
        pw->pw_dir = pw->pw_gecos + passwdbuf[PWGECOSLEN];
        pw->pw_shell = pw->pw_dir + passwdbuf[PWDIRLEN];
        pw->pw_uid = passwdbuf[PWUID];
        pw->pw_gid = passwdbuf[PWGID];

        /* Don't assume that nscd made sure to null terminate strings.
         * It's supposed to, but malicious nscd should be ignored
         * rather than causing a crash.
         */
        if (pw->pw_passwd[-1] || pw->pw_gecos[-1] || pw->pw_dir[-1] ||
            pw->pw_shell[passwdbuf[PWSHELLLEN] - 1]) {
            rv = EIO;
            goto cleanup_f;
        }

        if ((name && (strcmp(name, pw->pw_name))) || (!name && uid != pw->pw_uid)) {
            rv = EIO;
            goto cleanup_f;
        }

        *res = pw;
    cleanup_f:
        fclose(f);
        goto done;
    }

done:
    pthread_setcancelstate(cs, 0);
    if (rv)
        errno = rv;
    return rv;
}

static int getpw_r(const char *name, uid_t uid, struct passwd *pw, char *buf, size_t size,
                   struct passwd **res)
{
    char *line = 0;
    size_t len = 0;
    int rv = 0;
    int cs;

    pthread_setcancelstate(PTHREAD_CANCEL_DISABLE, &cs);

    rv = __getpw_a(name, uid, pw, &line, &len, res);
    if (*res && size < len) {
        *res = 0;
        rv = ERANGE;
    }
    if (*res) {
        memcpy(buf, line, len);
        FIX(name);
        FIX(passwd);
        FIX(gecos);
        FIX(dir);
        FIX(shell);
    }
    free(line);
    pthread_setcancelstate(cs, 0);
    if (rv)
        errno = rv;
    return rv;
}

int getpwnam_r(const char *name, struct passwd *pw, char *buf, size_t size, struct passwd **res)
{
    return getpw_r(name, 0, pw, buf, size, res);
}

int getpwuid_r(uid_t uid, struct passwd *pw, char *buf, size_t size, struct passwd **res)
{
    return getpw_r(0, uid, pw, buf, size, res);
}

#endif
