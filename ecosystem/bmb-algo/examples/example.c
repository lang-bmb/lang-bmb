/**
 * example.c — Using bmb-algo from C
 *
 * Compile (Windows):
 *   gcc example.c -I../include -L../.. -lbmb_algo -o example.exe
 *
 * Compile (Linux/macOS):
 *   gcc example.c -I../include -L../.. -lbmb_algo -o example
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../include/bmb_algo.h"

int main(void) {
    printf("=== BMB Algo C Example ===\n\n");

    /* Knapsack */
    int64_t weights[] = {2, 3, 4};
    int64_t values[]  = {3, 4, 5};
    int64_t result = bmb_knapsack((int64_t)weights, (int64_t)values, 3, 7);
    printf("knapsack([2,3,4], [3,4,5], capacity=7) = %lld\n", (long long)result);

    /* GCD */
    printf("gcd(12, 8) = %lld\n", (long long)bmb_gcd(12, 8));

    /* Fibonacci */
    printf("fibonacci(10) = %lld\n", (long long)bmb_fibonacci(10));

    /* Prime count */
    printf("prime_count(100) = %lld\n", (long long)bmb_prime_count(100));

    /* Is prime */
    printf("is_prime(97) = %lld\n", (long long)bmb_is_prime(97));

    /* Sort */
    int64_t arr[] = {5, 3, 1, 4, 2};
    bmb_quicksort((int64_t)arr, 5);
    printf("quicksort([5,3,1,4,2]) = [");
    for (int i = 0; i < 5; i++) {
        printf("%lld%s", (long long)arr[i], i < 4 ? "," : "");
    }
    printf("]\n");

    printf("\nAll examples passed!\n");
    return 0;
}
