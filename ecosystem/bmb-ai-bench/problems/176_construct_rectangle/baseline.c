#include <stdio.h>
#include <math.h>
int main() {
    long long area; scanf("%lld", &area);
    long long w = (long long)sqrt((double)area);
    while (area % w != 0) w--;
    printf("%lld %lld\n", area / w, w);
    return 0;
}
