package io.bmb.text;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class BmbTextTest {

    // Search
    @Test void testKmpFound()      { assertEquals(2,  BmbText.kmpSearch("hello", "llo")); }
    @Test void testKmpNotFound()   { assertEquals(-1, BmbText.kmpSearch("hello", "xyz")); }
    @Test void testFind()          { assertEquals(1,  BmbText.find("abcabc", "bc")); }
    @Test void testRfind()         { assertEquals(4,  BmbText.rfind("abcabc", "bc")); }
    @Test void testCount()         { assertEquals(2,  BmbText.count("abcabc", "bc")); }
    @Test void testContainsTrue()  { assertTrue(BmbText.contains("hello world", "world")); }
    @Test void testContainsFalse() { assertFalse(BmbText.contains("hello", "xyz")); }
    @Test void testStartsWith()    { assertTrue(BmbText.startsWith("hello", "hel")); }
    @Test void testEndsWith()      { assertTrue(BmbText.endsWith("hello", "llo")); }
    @Test void testFindByte()      { assertEquals(1, BmbText.findByte("abc", 'b')); }
    @Test void testCountByte()     { assertEquals(2, BmbText.countByte("abab", 'a')); }
    @Test void testHamming()       { assertEquals(2, BmbText.hamming("karolin", "kathrin")); }

    // Metrics
    @Test void testIsPalindromeTrue()  { assertTrue(BmbText.isPalindrome("racecar")); }
    @Test void testIsPalindromeFalse() { assertFalse(BmbText.isPalindrome("hello")); }
    @Test void testTokenCount()    { assertEquals(3, BmbText.tokenCount("a,b,c", ',')); }
    @Test void testWordCount()     { assertEquals(3, BmbText.wordCount("hello world foo")); }
    @Test void testLen()           { assertEquals(5, BmbText.len("hello")); }
    @Test void testCharAt()        { assertEquals('e', BmbText.charAt("hello", 1)); }
    @Test void testCompare()       {
        assertTrue(BmbText.compare("abc", "abd") < 0);
        assertEquals(0, BmbText.compare("abc", "abc"));
    }

    // Transformations
    @Test void testReverse()       { assertEquals("olleh", BmbText.reverse("hello")); }
    @Test void testReverseEmpty()  { assertEquals("", BmbText.reverse("")); }
    @Test void testReplace()       { assertEquals("hXllo", BmbText.replace("hello", "e", "X")); }
    @Test void testReplaceAll()    { assertEquals("XbXb", BmbText.replaceAll("abab", "a", "X")); }
    @Test void testToUpper()       { assertEquals("HELLO", BmbText.toUpper("hello")); }
    @Test void testToLower()       { assertEquals("hello", BmbText.toLower("HELLO")); }
    @Test void testTrim()          { assertEquals("hello", BmbText.trim("  hello  ")); }
    @Test void testTrimEmpty()     { assertEquals("", BmbText.trim("")); }
    @Test void testRepeat()        { assertEquals("abab", BmbText.repeat("ab", 2)); }
    @Test void testRepeatEmpty()   { assertEquals("", BmbText.repeat("", 5)); }
}
