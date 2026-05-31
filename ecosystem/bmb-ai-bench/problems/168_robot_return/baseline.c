#include <stdio.h>
int main() {
    char buf[10001];
    scanf("%s", buf);
    long long ud = 0, lr = 0;
    for (int i = 0; buf[i]; i++) {
        if (buf[i]=='U') ud++;
        else if (buf[i]=='D') ud--;
        else if (buf[i]=='L') lr--;
        else if (buf[i]=='R') lr++;
    }
    printf("%lld\n", (ud==0 && lr==0) ? 1LL : 0LL);
    return 0;
}
