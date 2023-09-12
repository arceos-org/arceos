#ifndef __SYSINFO_H__
#define __SYSINFO_H__

struct sysinfo {
    long uptime;             /* Seconds since boot */
    unsigned long loads[3];  /* 1, 5, and 15 minute load averages */
    unsigned long totalram;  /* Total usable main memory size */
    unsigned long freeram;   /* Available memory size */
    unsigned long sharedram; /* Amount of shared memory */
    unsigned long bufferram; /* Memory used by buffers */
    unsigned long totalswap; /* Total swap space size */
    unsigned long freeswap;  /* swap space still available */
    unsigned short procs;    /* Number of current processes */
    unsigned short pad;      /* Explicit padding for m68k */
    unsigned long totalhigh; /* Total high memory size */
    unsigned long freehigh;  /* Available high memory size */
    unsigned int mem_unit;   /* Memory unit size in bytes */
    char _f[20 - 2 * sizeof(unsigned long) - sizeof(unsigned int)]; /* Padding: libc5 uses this.. */
};

extern int sys_sysinfo(struct sysinfo *__info);

#endif // __SYSINFO_H__
