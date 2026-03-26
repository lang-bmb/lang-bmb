#include <stdio.h>
#include <stdlib.h>

void swap(long long *a, long long *b) {
    long long t = *a;
    *a = *b;
    *b = t;
}

int partition(long long *arr, int lo, int hi) {
    long long pivot = arr[hi];
    int i = lo - 1;
    for (int j = lo; j < hi; j++) {
        if (arr[j] <= pivot) {
            i++;
            swap(&arr[i], &arr[j]);
        }
    }
    swap(&arr[i + 1], &arr[hi]);
    return i + 1;
}

void quicksort(long long *arr, int lo, int hi) {
    if (lo < hi) {
        int p = partition(arr, lo, hi);
        quicksort(arr, lo, p - 1);
        quicksort(arr, p + 1, hi);
    }
}

int main(void) {
    int n;
    scanf("%d", &n);

    long long *arr = NULL;
    if (n > 0) {
        arr = (long long *)malloc(n * sizeof(long long));
        for (int i = 0; i < n; i++) {
            scanf("%lld", &arr[i]);
        }
        quicksort(arr, 0, n - 1);
    }

    for (int i = 0; i < n; i++) {
        if (i > 0) printf(" ");
        printf("%lld", arr[i]);
    }
    printf("\n");

    free(arr);
    return 0;
}
