package io.bmb.algo;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class BmbAlgoTest {

    // Scalar
    @Test void testGcd()            { assertEquals(6,  BmbAlgo.gcd(48, 18)); }
    @Test void testLcm()            { assertEquals(24, BmbAlgo.lcm(8, 12)); }
    @Test void testFibonacci()      { assertEquals(55, BmbAlgo.fibonacci(10)); }
    @Test void testIsPrime()        { assertTrue(BmbAlgo.isPrime(97)); assertFalse(BmbAlgo.isPrime(100)); }
    @Test void testPrimeCount()     { assertEquals(25, BmbAlgo.primeCount(100)); }
    @Test void testModPow()         { assertEquals(24, BmbAlgo.modPow(2, 10, 1000)); }
    @Test void testNQueens()        { assertEquals(92, BmbAlgo.nQueens(8)); }
    @Test void testDigitSum()       { assertEquals(6,  BmbAlgo.digitSum(123)); }
    @Test void testBitPopcount()    { assertEquals(3,  BmbAlgo.bitPopcount(7)); }
    @Test void testPowerSetSize()   { assertEquals(8,  BmbAlgo.powerSetSize(3)); }
    @Test void testIsPalindromeNum(){ assertTrue(BmbAlgo.isPalindromeNum(121)); assertFalse(BmbAlgo.isPalindromeNum(123)); }
    @Test void testBitOps() {
        assertEquals(5,  BmbAlgo.bitSet(4, 0));
        assertEquals(4,  BmbAlgo.bitClear(5, 0));
        assertTrue(       BmbAlgo.bitTest(5, 0));
        assertEquals(5,  BmbAlgo.bitToggle(4, 0));
    }

    // Array (read-only)
    @Test void testArraySum()       { assertEquals(15, BmbAlgo.arraySum(new long[]{1, 2, 3, 4, 5})); }
    @Test void testArrayMin()       { assertEquals(1,  BmbAlgo.arrayMin(new long[]{3, 1, 4, 1, 5})); }
    @Test void testArrayMax()       { assertEquals(9,  BmbAlgo.arrayMax(new long[]{3, 1, 4, 1, 5, 9})); }
    @Test void testArrayProduct()   { assertEquals(120, BmbAlgo.arrayProduct(new long[]{1, 2, 3, 4, 5})); }
    @Test void testIsSorted()       { assertTrue(BmbAlgo.isSorted(new long[]{1, 2, 3})); assertFalse(BmbAlgo.isSorted(new long[]{3, 1, 2})); }
    @Test void testBinarySearch()   { assertEquals(2, BmbAlgo.binarySearch(new long[]{1, 3, 5, 7, 9}, 5)); }
    @Test void testMaxSubarray()    { assertEquals(6, BmbAlgo.maxSubarray(new long[]{-2, 1, -3, 4, -1, 2, 1, -5, 4})); }
    @Test void testLis()            { assertEquals(4, BmbAlgo.lis(new long[]{10, 9, 2, 5, 3, 7, 101, 18})); }
    @Test void testCoinChange()     { assertEquals(3, BmbAlgo.coinChange(new long[]{1, 5, 6, 9}, 11)); }
    @Test void testKnapsack()       { assertEquals(220, BmbAlgo.knapsack(new long[]{1, 3, 4, 5}, new long[]{1, 4, 5, 7}, 7)); }
    @Test void testSubsetSum()      { assertTrue(BmbAlgo.subsetSum(new long[]{3, 34, 4, 12, 5, 2}, 9)); }
    @Test void testUniqueCount()    { assertEquals(4, BmbAlgo.uniqueCount(new long[]{1, 2, 2, 3, 3, 4})); }
    @Test void testArrayContains()  { assertTrue(BmbAlgo.arrayContains(new long[]{1, 2, 3}, 2)); }
    @Test void testArrayIndexOf()   { assertEquals(1, BmbAlgo.arrayIndexOf(new long[]{1, 2, 3}, 2)); }
    @Test void testKthSmallest()    { assertEquals(2, BmbAlgo.kthSmallest(new long[]{7, 10, 4, 3, 20, 15}, 3)); }

    // Sorting
    @Test void testQuickSort() {
        assertArrayEquals(new long[]{1, 2, 3, 4, 5}, BmbAlgo.quickSort(new long[]{3, 1, 4, 1, 5}));
    }
    @Test void testMergeSort() {
        assertArrayEquals(new long[]{1, 2, 3, 4, 5}, BmbAlgo.mergeSort(new long[]{5, 3, 4, 1, 2}));
    }
    @Test void testHeapSort() {
        assertArrayEquals(new long[]{1, 2, 3, 4, 5}, BmbAlgo.heapSort(new long[]{4, 2, 5, 1, 3}));
    }
    @Test void testInsertionSort() {
        assertArrayEquals(new long[]{1, 2, 3}, BmbAlgo.insertionSort(new long[]{3, 1, 2}));
    }

    // String functions
    @Test void testLcs()          { assertEquals(4, BmbAlgo.lcs("ABCBDAB", "BDCABA")); }
    @Test void testEditDistance() { assertEquals(3, BmbAlgo.editDistance("kitten", "sitting")); }
    @Test void testDjb2Hash()     { assertNotEquals(0L, BmbAlgo.djb2Hash("hello")); }
}
