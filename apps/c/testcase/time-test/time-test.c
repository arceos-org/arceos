#include<stddef.h>
#include<stdio.h>
#include<stdlib.h>
#include<time.h>

#define MS_PER_SEC      1000ull
#define NS_PER_SEC      1000000000ull
#define LP0             1000000ull
#define TARGET_TIME_MS  500ull

typedef unsigned long long ULL;

ULL now_ns() {
   struct timespec ts = { 0, 0 };
   clock_gettime(CLOCK_REALTIME, &ts);
   return ts.tv_sec * NS_PER_SEC + ts.tv_nsec;
}

ULL iter(ULL n) {
   volatile size_t v = 0;
   for (ULL i = 0; i < n; i++) {
      v += 1;
   }
}

int main(void) {
   ULL t0 = now_ns();
   iter(LP0);
   ULL t1 = now_ns();
   ULL n_lp = TARGET_TIME_MS * (NS_PER_SEC / MS_PER_SEC) / (t1 - t0);

   ULL n = LP0 * n_lp;
   iter(n);
   ULL t2 = now_ns();
   ULL dur_ns = t2 - t1;
   ULL dur_ms = dur_ns / (NS_PER_SEC / MS_PER_SEC);

   double per = (double)n / (double)dur_ns;
   printf("time-test: time/iteration: %.3f ns total time: %llums\n", per, dur_ms);
   return 0;
}
