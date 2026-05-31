#include <stdio.h>
int main() {
    int num; scanf("%d", &num);
    int d[4] = {num/1000, (num/100)%10, (num/10)%10, num%10};
    for (int i = 0; i < 3; i++)
        for (int j = i+1; j < 4; j++)
            if (d[i] > d[j]) { int tmp = d[i]; d[i] = d[j]; d[j] = tmp; }
    printf("%d\n", (d[0]+d[1])*10 + d[2]+d[3]);
    return 0;
}
