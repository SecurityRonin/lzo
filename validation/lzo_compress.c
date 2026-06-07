/* Compress a file with a chosen lzo1x variant, emit the raw LZO1X block.
 * Usage: lzo_compress <1|15|999> <infile> <outfile.lzo>
 * Links the reference C liblzo2 — the output is exactly what lzop/dar -zlzo
 * would embed for these bytes. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <lzo/lzo1x.h>

static unsigned char *slurp(const char *p, lzo_uint *n) {
    FILE *f = fopen(p, "rb");
    if (!f) { perror(p); exit(2); }
    fseek(f, 0, SEEK_END); long sz = ftell(f); fseek(f, 0, SEEK_SET);
    unsigned char *b = malloc(sz ? sz : 1);
    if (sz && fread(b, 1, sz, f) != (size_t)sz) { perror("read"); exit(2); }
    fclose(f); *n = (lzo_uint)sz; return b;
}

int main(int argc, char **argv) {
    if (argc != 4) { fprintf(stderr, "usage: %s <1|15|999> in out\n", argv[0]); return 2; }
    if (lzo_init() != LZO_E_OK) { fprintf(stderr, "lzo_init failed\n"); return 2; }
    lzo_uint in_len; unsigned char *in = slurp(argv[2], &in_len);
    lzo_uint out_cap = in_len + in_len / 16 + 64 + 3;
    unsigned char *out = malloc(out_cap);
    lzo_uint out_len = 0; int r; void *wrk;
    if (!strcmp(argv[1], "1")) {
        wrk = malloc(LZO1X_1_MEM_COMPRESS);
        r = lzo1x_1_compress(in, in_len, out, &out_len, wrk);
    } else if (!strcmp(argv[1], "15")) {
        wrk = malloc(LZO1X_1_15_MEM_COMPRESS);
        r = lzo1x_1_15_compress(in, in_len, out, &out_len, wrk);
    } else if (!strcmp(argv[1], "999")) {
        wrk = malloc(LZO1X_999_MEM_COMPRESS);
        r = lzo1x_999_compress(in, in_len, out, &out_len, wrk);
    } else { fprintf(stderr, "bad algo %s\n", argv[1]); return 2; }
    if (r != LZO_E_OK) { fprintf(stderr, "compress failed: %d\n", r); return 2; }
    FILE *o = fopen(argv[3], "wb");
    if (!o || fwrite(out, 1, out_len, o) != out_len) { perror(argv[3]); return 2; }
    fclose(o);
    fprintf(stderr, "%-4s %-28s %8lu -> %8lu\n", argv[1], argv[2], (unsigned long)in_len, (unsigned long)out_len);
    return 0;
}
