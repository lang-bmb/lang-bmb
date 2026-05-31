#include <stdio.h>
int main() {
    int ranks[5];
    char suits[5];
    for (int i = 0; i < 5; i++) scanf("%d", &ranks[i]);
    for (int i = 0; i < 5; i++) {
        char buf[3]; scanf("%s", buf); suits[i] = buf[0];
    }
    int flush = 1;
    for (int i = 1; i < 5; i++) if (suits[i] != suits[0]) { flush = 0; break; }
    int freq[14] = {0};
    for (int i = 0; i < 5; i++) freq[ranks[i]]++;
    int max_freq = 0;
    for (int i = 1; i <= 13; i++) if (freq[i] > max_freq) max_freq = freq[i];
    if (flush) printf("Flush\n");
    else if (max_freq >= 3) printf("Three of a Kind\n");
    else if (max_freq == 2) printf("Pair\n");
    else printf("High Card\n");
    return 0;
}
