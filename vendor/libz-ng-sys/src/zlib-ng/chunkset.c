/* chunkset.c -- inline functions to copy small data chunks.
 * For conditions of distribution and use, see copyright notice in zlib.h
 */

#include "zbuild.h"
#include "zutil.h"

// We need sizeof(chunk_t) to be 8, no matter what.
#if defined(UNALIGNED64_OK)
typedef uint64_t chunk_t;
#elif defined(UNALIGNED_OK)
typedef struct chunk_t { uint32_t u32[2]; } chunk_t;
#else
typedef struct chunk_t { uint8_t u8[8]; } chunk_t;
#endif

#define CHUNK_SIZE 8

#define HAVE_CHUNKMEMSET_4
#define HAVE_CHUNKMEMSET_8

static inline void chunkmemset_4(uint8_t *from, chunk_t *chunk) {
    uint8_t *dest = (uint8_t *)chunk;
    memcpy(dest, from, sizeof(uint32_t));
    memcpy(dest+4, from, sizeof(uint32_t));
}

static inline void chunkmemset_8(uint8_t *from, chunk_t *chunk) {
    memcpy(chunk, from, sizeof(uint64_t));
}

static inline void loadchunk(uint8_t const *s, chunk_t *chunk) {
    memcpy(chunk, (uint8_t *)s, sizeof(uint64_t));
}

static inline void storechunk(uint8_t *out, chunk_t *chunk) {
    memcpy(out, chunk, sizeof(uint64_t));
}

#define CHUNKSIZE        chunksize_c
#define CHUNKCOPY        chunkcopy_c
#define CHUNKCOPY_SAFE   chunkcopy_safe_c
#define CHUNKUNROLL      chunkunroll_c
#define CHUNKMEMSET      chunkmemset_c
#define CHUNKMEMSET_SAFE chunkmemset_safe_c

#include "chunkset_tpl.h"
