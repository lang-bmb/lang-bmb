#include <stdio.h>
int main(void) {
    long long keys[1024], vals[1024]; int sz=0;
    int n; scanf("%d", &n);
    while (n--) {
        int op; scanf("%d", &op);
        if (op==1) {
            long long idx, val; scanf("%lld %lld", &idx, &val);
            int found=0;
            for(int j=0;j<sz;j++) if(keys[j]==idx){vals[j]=val;found=1;break;}
            if(!found){keys[sz]=idx;vals[sz]=val;sz++;}
        } else {
            long long idx; scanf("%lld", &idx);
            int found=0;
            for(int j=0;j<sz;j++) if(keys[j]==idx){printf("%lld\n",vals[j]);found=1;break;}
            if(!found) printf("0\n");
        }
    }
    return 0;
}
