#include <stdio.h>
#include <string.h>
int main() {
    char s[100], digits[100]; int di = 0;
    fgets(s, 100, stdin); s[strcspn(s, "\n")] = 0;
    for (int i = 0; s[i]; i++)
        if (s[i] >= '0' && s[i] <= '9') digits[di++] = s[i];
    digits[di] = 0;
    char out[200]; int oi = 0;
    int pos = 0;
    while (pos < di) {
        int rem = di - pos;
        if (oi > 0) out[oi++] = '-';
        if (rem == 4) {
            out[oi++] = digits[pos]; out[oi++] = digits[pos+1];
            out[oi++] = '-';
            out[oi++] = digits[pos+2]; out[oi++] = digits[pos+3];
            pos += 4;
        } else if (rem <= 3) {
            for (int k = 0; k < rem; k++) out[oi++] = digits[pos+k];
            pos += rem;
        } else {
            out[oi++] = digits[pos]; out[oi++] = digits[pos+1]; out[oi++] = digits[pos+2];
            pos += 3;
        }
    }
    out[oi] = 0;
    printf("%s\n", out);
    return 0;
}
