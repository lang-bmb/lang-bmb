#include <stdio.h>
int main(void) {
    int days[] = {31,28,31,30,31,30,31,31,30,31,30,31};
    int t; scanf("%d", &t);
    while (t--) {
        int month, day; scanf("%d %d", &month, &day);
        int total = 0;
        for (int m = 0; m < month - 1; m++) total += days[m];
        total += day;
        printf("%d\n", total);
    }
    return 0;
}
