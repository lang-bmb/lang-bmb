#include <stdio.h>
#include <stdlib.h>

void merge(long long *arr, int lo, int mid, int hi, long long *tmp) {
    int i = lo, j = mid + 1, k = lo;
    while (i <= mid && j <= hi) {
        if (arr[i] <= arr[j]) tmp[k++] = arr[i++];
        else tmp[k++] = arr[j++];
    }
    while (i <= mid) tmp[k++] = arr[i++];
    while (j <= hi) tmp[k++] = arr[j++];
    for (int x = lo; x <= hi; x++) arr[x] = tmp[x];
}

void merge_sort(long long *arr, int lo, int hi, long long *tmp) {
    if (lo >= hi) return;
    int mid = lo + (hi - lo) / 2;
    merge_sort(arr, lo, mid, tmp);
    merge_sort(arr, mid + 1, hi, tmp);
    merge(arr, lo, mid, hi, tmp);
}

int main(void) {
    int n;
    scanf("%d", &n);
    long long *arr = NULL, *tmp = NULL;
    if (n > 0) {
        arr = (long long *)malloc(n * sizeof(long long));
        tmp = (long long *)malloc(n * sizeof(long long));
        for (int i = 0; i < n; i++) scanf("%lld", &arr[i]);
        merge_sort(arr, 0, n - 1, tmp);
    }
    for (int i = 0; i < n; i++) {
        if (i > 0) printf(" ");
        printf("%lld", arr[i]);
    }
    printf("\n");
    free(arr);
    free(tmp);
    return 0;
}
