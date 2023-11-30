#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <unistd.h>
#include <ctype.h>

void die(const char *msg) {
    fputs(msg, stderr);
    exit(-1);
}

unsigned vis[1024], max_p;

void check() {
    FILE *f = fopen("/proc/interrupts", "r");
    if (!f) {
        die("fopen failed\n");
    }

    unsigned p = 0, inc = 0;
    char buf[1024];
    while (fgets(buf, sizeof(buf), f)) {
        const char *s = buf;
        while (*s == ' ' && *s) {
            ++s;
        }
        if (!isdigit(*s)) {
            continue;
        }

        unsigned irq, cnt;
        if (sscanf(s, " %u: %u", &irq, &cnt) != 2) {
            die("parse error\n");
        }

        if (irq < p) {
            die("irq decreased\n");
        }
        for (; p < irq; ++p) {
            if (vis[p]) {
                die("count vanished\n");
            }
        }
        if (vis[irq] > cnt) {
            die("count decreased\n");
        } else if (vis[irq] < cnt) {
            inc = 1;
            vis[irq] = cnt;
        }
        ++p;
    }
    if (p == 0) {
        die("no records\n");
    }
    if (p < max_p) {
        die("maximum irq decreased\n");
    } else if (p > max_p) {
        max_p = p;
    }
    if (!inc) {
        die("not changed\n");
    }
    fclose(f);
}

int main() {
    check();
    usleep(100000);
    check();
    usleep(100000);
    check();
    puts("interrupts-test: passed case 2");
    return 0;
}
