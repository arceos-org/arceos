#include <stdio.h>
#include <time.h>
#include <string.h>

void print_test_result(const char* test_name, int passed) {
    printf("[%s] %s\n", passed ? "PASS" : "FAIL", test_name);
}

int main() {
    struct tm t;
    time_t result;
    int test_passed;

    printf("=== mktime() Implementation Tests ===\n\n");

    // Test 1: Epoch
    printf("Test 1: Epoch (1970-01-01 00:00:00)\n");
    memset(&t, 0, sizeof(t));
    t.tm_year = 70;
    t.tm_mon = 0;
    t.tm_mday = 1;
    t.tm_isdst = -1;
    result = mktime(&t);
    test_passed = (result == 0 && t.tm_wday == 4 && t.tm_yday == 0);
    printf("  time_t: %ld (expected: 0)\n", (long)result);
    printf("  wday: %d (expected: 4/Thursday)\n", t.tm_wday);
    printf("  yday: %d (expected: 0)\n", t.tm_yday);
    print_test_result("Epoch test", test_passed);
    printf("\n");

    // Test 2: Leap year date
    printf("Test 2: Leap year date (2020-02-29 06:15:30)\n");
    memset(&t, 0, sizeof(t));
    t.tm_year = 120;
    t.tm_mon = 1;
    t.tm_mday = 29;
    t.tm_hour = 6;
    t.tm_min = 15;
    t.tm_sec = 30;
    t.tm_isdst = -1;
    result = mktime(&t);
    test_passed = (result == 1582956930 && t.tm_wday == 6 && t.tm_yday == 59);
    printf("  time_t: %ld (expected: 1582956930)\n", (long)result);
    printf("  wday: %d (expected: 6/Saturday)\n", t.tm_wday);
    printf("  yday: %d (expected: 59)\n", t.tm_yday);
    print_test_result("Leap year test", test_passed);
    printf("\n");

    // Test 3: Normalization - overflow seconds
    printf("Test 3: Normalization - overflow seconds (60 sec -> next minute)\n");
    memset(&t, 0, sizeof(t));
    t.tm_year = 124;
    t.tm_mon = 0;
    t.tm_mday = 1;
    t.tm_sec = 60;
    t.tm_isdst = -1;
    result = mktime(&t);
    test_passed = (t.tm_sec == 0 && t.tm_min == 1 && result == 1704067260);
    printf("  Normalized: %04d-%02d-%02d %02d:%02d:%02d\n", 
           t.tm_year + 1900, t.tm_mon + 1, t.tm_mday, t.tm_hour, t.tm_min, t.tm_sec);
    printf("  Expected:   2024-01-01 00:01:00\n");
    printf("  time_t: %ld (expected: 1704067260)\n", (long)result);
    print_test_result("Seconds overflow normalization", test_passed);
    printf("\n");

    // Test 4: Normalization - month overflow
    printf("Test 4: Normalization - month overflow (month 12 -> next year)\n");
    memset(&t, 0, sizeof(t));
    t.tm_year = 124;
    t.tm_mon = 12;
    t.tm_mday = 15;
    t.tm_hour = 12;
    t.tm_isdst = -1;
    result = mktime(&t);
    test_passed = (t.tm_year == 125 && t.tm_mon == 0 && t.tm_mday == 15 && result == 1736942400);
    printf("  Normalized: %04d-%02d-%02d %02d:%02d:%02d\n", 
           t.tm_year + 1900, t.tm_mon + 1, t.tm_mday, t.tm_hour, t.tm_min, t.tm_sec);
    printf("  Expected:   2025-01-15 12:00:00\n");
    printf("  time_t: %ld (expected: 1736942400)\n", (long)result);
    print_test_result("Month overflow normalization", test_passed);
    printf("\n");

    // Test 5: Normalization - negative month
    printf("Test 5: Normalization - negative month (month -1 -> prev year)\n");
    memset(&t, 0, sizeof(t));
    t.tm_year = 124;
    t.tm_mon = -1;
    t.tm_mday = 15;
    t.tm_hour = 12;
    t.tm_isdst = -1;
    result = mktime(&t);
    test_passed = (t.tm_year == 123 && t.tm_mon == 11 && t.tm_mday == 15 && result == 1702641600);
    printf("  Normalized: %04d-%02d-%02d %02d:%02d:%02d\n", 
           t.tm_year + 1900, t.tm_mon + 1, t.tm_mday, t.tm_hour, t.tm_min, t.tm_sec);
    printf("  Expected:   2023-12-15 12:00:00\n");
    printf("  time_t: %ld (expected: 1702641600)\n", (long)result);
    print_test_result("Negative month normalization", test_passed);
    printf("\n");

    // Test 6: Normalization - day overflow
    printf("Test 6: Normalization - day overflow (Jan 32 -> Feb 1)\n");
    memset(&t, 0, sizeof(t));
    t.tm_year = 124;
    t.tm_mon = 0;
    t.tm_mday = 32;
    t.tm_isdst = -1;
    result = mktime(&t);
    test_passed = (t.tm_mon == 1 && t.tm_mday == 1 && result == 1706745600);
    printf("  Normalized: %04d-%02d-%02d %02d:%02d:%02d\n", 
           t.tm_year + 1900, t.tm_mon + 1, t.tm_mday, t.tm_hour, t.tm_min, t.tm_sec);
    printf("  Expected:   2024-02-01 00:00:00\n");
    printf("  time_t: %ld (expected: 1706745600)\n", (long)result);
    print_test_result("Day overflow normalization", test_passed);
    printf("\n");

    // Test 7: Normalization - day underflow
    printf("Test 7: Normalization - day underflow (Jan 0 -> Dec 31 prev year)\n");
    memset(&t, 0, sizeof(t));
    t.tm_year = 124;
    t.tm_mon = 0;
    t.tm_mday = 0;
    t.tm_isdst = -1;
    result = mktime(&t);
    test_passed = (t.tm_year == 123 && t.tm_mon == 11 && t.tm_mday == 31 && result == 1703980800);
    printf("  Normalized: %04d-%02d-%02d %02d:%02d:%02d\n", 
           t.tm_year + 1900, t.tm_mon + 1, t.tm_mday, t.tm_hour, t.tm_min, t.tm_sec);
    printf("  Expected:   2023-12-31 00:00:00\n");
    printf("  time_t: %ld (expected: 1703980800)\n", (long)result);
    print_test_result("Day underflow normalization", test_passed);
    printf("\n");

    // Test 8: Large year (performance test - should not hang)
    printf("Test 8: Large year value (year 10000) - performance test\n");
    memset(&t, 0, sizeof(t));
    t.tm_year = 8100;
    t.tm_mon = 0;
    t.tm_mday = 1;
    t.tm_isdst = -1;
    result = mktime(&t);
    test_passed = (result == 253402300800 && t.tm_wday == 6);
    printf("  time_t: %ld (expected: 253402300800)\n", (long)result);
    printf("  wday: %d (expected: 6/Saturday)\n", t.tm_wday);
    print_test_result("Large year performance (O(1) algorithm)", test_passed);
    printf("\n");

    // Test 9: Pre-epoch date
    printf("Test 9: Pre-epoch date (1969-12-31 23:59:59)\n");
    memset(&t, 0, sizeof(t));
    t.tm_year = 69;
    t.tm_mon = 11;
    t.tm_mday = 31;
    t.tm_hour = 23;
    t.tm_min = 59;
    t.tm_sec = 59;
    t.tm_isdst = -1;
    result = mktime(&t);
    test_passed = (result == -1 && t.tm_wday == 3 && t.tm_yday == 364);
    printf("  time_t: %ld (expected: -1)\n", (long)result);
    printf("  wday: %d (expected: 3/Wednesday)\n", t.tm_wday);
    printf("  yday: %d (expected: 364)\n", t.tm_yday);
    print_test_result("Pre-epoch test", test_passed);
    printf("\n");

    printf("=== All tests completed ===\n");
    return 0;
}
