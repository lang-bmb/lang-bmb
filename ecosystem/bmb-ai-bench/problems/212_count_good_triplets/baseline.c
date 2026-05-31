#include <stdio.h>
#include <stdlib.h>
int abs_diff(int a, int b) { return a >= b ? a - b : b - a; }
int main() {
    int n;
    scanf("%d", &n);
    int *arr = (int*)malloc(n*sizeof(int));
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    int a, b, c;
    scanf("%d %d %d", &a, &b, &c);
    int count = 0;
    for (int i = 0; i < n-2; i++)
      for (int j = i+1; j < n-1; j++)
        if (abs_diff(arr[i], arr[j]) <= a)
          for (int k = j+1; k < n; k++)
            if (abs_diff(arr[j], arr[k]) <= b && abs_diff(arr[i], arr[k]) <= c)
              count++;
    printf("%d\n", count);
    free(arr);
    return 0;
}
