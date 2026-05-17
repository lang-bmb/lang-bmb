package io.bmb.compute;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class BmbComputeTest {

    // Scalar math
    @Test void testAbs()       { assertEquals(5,  BmbCompute.abs(-5)); assertEquals(5, BmbCompute.abs(5)); }
    @Test void testMin()       { assertEquals(3,  BmbCompute.min(3, 7)); }
    @Test void testMax()       { assertEquals(7,  BmbCompute.max(3, 7)); }
    @Test void testClamp()     { assertEquals(5,  BmbCompute.clamp(10, 0, 5)); assertEquals(2, BmbCompute.clamp(2, 0, 5)); }
    @Test void testSign()      { assertEquals(1,  BmbCompute.sign(42)); assertEquals(-1, BmbCompute.sign(-3)); assertEquals(0, BmbCompute.sign(0)); }
    @Test void testIpow()      { assertEquals(8,  BmbCompute.ipow(2, 3)); assertEquals(1, BmbCompute.ipow(5, 0)); }
    @Test void testSqrt()      { assertEquals(4,  BmbCompute.sqrt(16)); assertEquals(3, BmbCompute.sqrt(9)); }
    @Test void testFactorial() { assertEquals(120, BmbCompute.factorial(5)); assertEquals(1, BmbCompute.factorial(0)); }
    @Test void testPow2()      { assertTrue(BmbCompute.isPowerOfTwo(8)); assertFalse(BmbCompute.isPowerOfTwo(6)); }
    @Test void testNextPow2()  { assertEquals(8, BmbCompute.nextPowerOfTwo(5)); assertEquals(4, BmbCompute.nextPowerOfTwo(4)); }

    // RNG (deterministic with fixed seed)
    @Test void testRngSeeded() {
        long s = BmbCompute.randSeed(42);
        long n = BmbCompute.randPos(s);
        assertTrue(n > 0);
    }
    @Test void testRandRange() {
        long s = BmbCompute.randSeed(7);
        long n = BmbCompute.randRange(s, 100);
        assertTrue(n >= 0 && n < 100);
    }

    // Array statistics
    @Test void testSum()      { assertEquals(15, BmbCompute.sum(new long[]{1,2,3,4,5})); }
    @Test void testMean()     { assertEquals(3000, BmbCompute.meanScaled(new long[]{1,2,3,4,5})); } // 3.000 × 1000
    @Test void testMinVal()   { assertEquals(1,  BmbCompute.minVal(new long[]{3,1,4,1,5})); }
    @Test void testMaxVal()   { assertEquals(9,  BmbCompute.maxVal(new long[]{3,1,4,1,5,9})); }
    @Test void testRange()    { assertEquals(8,  BmbCompute.rangeVal(new long[]{1,9})); }
    @Test void testVariance() { assertTrue(BmbCompute.varianceScaled(new long[]{1,2,3,4,5}) > 0); }
    @Test void testMagnitude(){ assertEquals(30, BmbCompute.magnitudeSquared(new long[]{1,2,3,4})); } // 1+4+9+16=30

    // Two-array operations
    @Test void testDotProduct() { assertEquals(32, BmbCompute.dotProduct(new long[]{1,2,3}, new long[]{4,5,6})); }
    @Test void testWeightedSum(){ assertEquals(14, BmbCompute.weightedSum(new long[]{1,2,3}, new long[]{1,2,3})); } // 1+4+9
    @Test void testDistSquared() {
        assertEquals(3, BmbCompute.distSquared(new long[]{0,0,0}, new long[]{1,1,1}));
    }

    // Output-array operations
    @Test void testCumsum() {
        assertArrayEquals(new long[]{1,3,6,10,15}, BmbCompute.cumsum(new long[]{1,2,3,4,5}));
    }
    @Test void testVecAdd() {
        assertArrayEquals(new long[]{5,7,9}, BmbCompute.vecAdd(new long[]{1,2,3}, new long[]{4,5,6}));
    }
    @Test void testVecSub() {
        assertArrayEquals(new long[]{3,3,3}, BmbCompute.vecSub(new long[]{4,5,6}, new long[]{1,2,3}));
    }
    @Test void testVecScale() {
        assertArrayEquals(new long[]{2,4,6}, BmbCompute.vecScale(new long[]{1,2,3}, 2));
    }
    @Test void testMapSquare() {
        assertArrayEquals(new long[]{1,4,9,16}, BmbCompute.mapSquare(new long[]{1,2,3,4}));
    }
}
