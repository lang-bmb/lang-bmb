package io.bmb.json;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class BmbJsonTest {

    // Validation
    @Test void testValidTrue()    { assertTrue(BmbJson.validate("{\"a\":1}")); }
    @Test void testValidFalse()   { assertFalse(BmbJson.validate("{invalid}")); }
    @Test void testValidEmpty()   { assertFalse(BmbJson.validate("")); }
    @Test void testValidArray()   { assertTrue(BmbJson.validate("[1,2,3]")); }

    // Stringify
    @Test void testStringify()    { assertEquals("{\"x\":1}", BmbJson.stringify("{\"x\":1}")); }

    // Type
    @Test void testTypeNull()     { assertEquals("null",   BmbJson.type("null")); }
    @Test void testTypeBool()     { assertEquals("bool",   BmbJson.type("true")); }
    @Test void testTypeNumber()   { assertEquals("number", BmbJson.type("42")); }
    @Test void testTypeString()   { assertEquals("string", BmbJson.type("\"hello\"")); }
    @Test void testTypeArray()    { assertEquals("array",  BmbJson.type("[1,2]")); }
    @Test void testTypeObject()   { assertEquals("object", BmbJson.type("{\"a\":1}")); }

    // Object access
    @Test void testGetNumber()    { assertEquals(42, BmbJson.getNumber("{\"n\":42}", "n")); }
    @Test void testGetString()    { assertEquals("hello", BmbJson.getString("{\"s\":\"hello\"}", "s")); }
    @Test void testGetBoolTrue()  { assertEquals(1, BmbJson.getBool("{\"b\":true}", "b")); }
    @Test void testGetBoolFalse() { assertEquals(0, BmbJson.getBool("{\"b\":false}", "b")); }
    @Test void testGetBoolMiss()  { assertEquals(-1, BmbJson.getBool("{\"a\":1}", "b")); }
    @Test void testHasKeyTrue()   { assertTrue(BmbJson.hasKey("{\"a\":1}", "a")); }
    @Test void testHasKeyFalse()  { assertFalse(BmbJson.hasKey("{\"a\":1}", "z")); }
    @Test void testObjectLen()    { assertEquals(2, BmbJson.objectLen("{\"a\":1,\"b\":2}")); }
    @Test void testObjectLenNonObj() { assertEquals(-1, BmbJson.objectLen("[1,2]")); }

    // Array access
    @Test void testArrayLen()     { assertEquals(3, BmbJson.arrayLen("[1,2,3]")); }
    @Test void testArrayLenNonArr() { assertEquals(-1, BmbJson.arrayLen("{\"a\":1}")); }
    @Test void testArrayGet()     { assertEquals("2", BmbJson.arrayGet("[1,2,3]", 1)); }
    @Test void testArrayGetOob()  { assertEquals("", BmbJson.arrayGet("[1,2,3]", 5)); }

    // Count
    @Test void testCount()        { assertEquals(3, BmbJson.count("[1,2,3]")); }
}
