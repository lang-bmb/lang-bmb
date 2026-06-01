#include <stdio.h>
int main() {
    int num; scanf("%d", &num);
    int dg[4] = { num/1000, (num/100)%10, (num/10)%10, num%10 };
    for (int i = 0; i < 4; i++) for (int j = i+1; j < 4; j++)
        if (dg[j] < dg[i]) { int t = dg[i]; dg[i] = dg[j]; dg[j] = t; }
    int ans = (dg[0]*10 + dg[2]) + (dg[1]*10 + dg[3]);
    printf("%d\n", ans);
    return 0;
}
