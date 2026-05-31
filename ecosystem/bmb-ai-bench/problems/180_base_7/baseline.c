#include <stdio.h>
#include <string.h>
void to_base7(long long n, char *buf, int *pos) {
    if (n >= 7) to_base7(n/7, buf, pos);
    buf[(*pos)++] = '0' + (int)(n % 7);
}
int main() {
    long long num; scanf("%lld", &num);
    if (num == 0) { printf("0\n"); return 0; }
    char buf[64]; int pos = 0;
    if (num < 0) { buf[pos++] = '-'; to_base7(-num, buf, &pos); }
    else { to_base7(num, buf, &pos); }
    buf[pos] = '\0';
    printf("%s\n", buf);
    return 0;
}
