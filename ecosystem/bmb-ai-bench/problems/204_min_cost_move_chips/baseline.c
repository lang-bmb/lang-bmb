#include <stdio.h>
int main() {
    int n;
    scanf("%d", &n);
    int even = 0, odd = 0;
    for (int i = 0; i < n; i++) {
        int pos;
        scanf("%d", &pos);
        if (pos % 2 == 0) even++; else odd++;
    }
    printf("%d\n", even < odd ? even : odd);
    return 0;
}
