'use strict';
const { test } = require('node:test');
const assert = require('node:assert/strict');
const j = require('..');

test('validate — valid inputs', () => {
  assert.equal(j.validate('{"a":1}'), true);
  assert.equal(j.validate('{}'), true);
  assert.equal(j.validate('[1,2,3]'), true);
  assert.equal(j.validate('[]'), true);
  assert.equal(j.validate('"hello"'), true);
  assert.equal(j.validate('42'), true);
  assert.equal(j.validate('null'), true);
  assert.equal(j.validate('true'), true);
  assert.equal(j.validate('false'), true);
});

test('validate — invalid inputs', () => {
  assert.equal(j.validate(''), false);
  assert.equal(j.validate('hello'), false);
  assert.equal(j.validate("'hello'"), false);
  assert.equal(j.validate('None'), false);
});

test('validate — BMB lenient parser', () => {
  // BMB accepts trailing commas in arrays and unmatched delimiters
  assert.equal(j.validate('[1,2,]'), true);
  assert.equal(j.validate('{"a":1'), true);
  assert.equal(j.validate('[1,2'), true);
  assert.equal(j.validate('   '), true);
});

test('get_type', () => {
  assert.equal(j.get_type('null'), 'null');
  assert.equal(j.get_type('true'), 'bool');
  assert.equal(j.get_type('false'), 'bool');
  assert.equal(j.get_type('42'), 'number');
  assert.equal(j.get_type('-7'), 'number');
  assert.equal(j.get_type('"hello"'), 'string');
  assert.equal(j.get_type('""'), 'string');
  assert.equal(j.get_type('[1,2,3]'), 'array');
  assert.equal(j.get_type('[]'), 'array');
  assert.equal(j.get_type('{"a":1}'), 'object');
  assert.equal(j.get_type('{}'), 'object');
});

test('stringify — normalises whitespace', () => {
  assert.equal(j.stringify('{"a":1}'), '{"a":1}');
  assert.equal(j.stringify('[1,2,3]'), '[1,2,3]');
  assert.equal(j.stringify('null'), 'null');
  assert.equal(j.stringify('true'), 'true');
  assert.equal(j.stringify('"hello"'), '"hello"');
  assert.equal(j.stringify('{}'), '{}');
  assert.equal(j.stringify('[]'), '[]');
  // Spaced input → minified output
  const minified = j.stringify('{ "a" : 1 , "b" : 2 }');
  assert.equal(JSON.parse(minified).a, 1);
  assert.equal(JSON.parse(minified).b, 2);
});

test('array_len', () => {
  assert.equal(j.array_len('[10,20,30,40,50]'), 5);
  assert.equal(j.array_len('[42]'), 1);
  assert.equal(j.array_len('[]'), 0);
  assert.equal(j.array_len('[[1,2],[3,4],[5]]'), 3);
  // Non-arrays return -1
  assert.equal(j.array_len('42'), -1);
  assert.equal(j.array_len('"hello"'), -1);
  assert.equal(j.array_len('{"a":1}'), -1);
  assert.equal(j.array_len('null'), -1);
});

test('object_len', () => {
  assert.equal(j.object_len('{"a":1,"b":2,"c":3}'), 3);
  assert.equal(j.object_len('{}'), 0);
  // Non-objects return -1
  assert.equal(j.object_len('[1,2]'), -1);
  assert.equal(j.object_len('42'), -1);
});

test('count', () => {
  // count = total nodes (object/array + all children)
  assert.equal(j.count('{"a":1,"b":2}'), 5);  // obj + 2 keys + 2 values
  assert.equal(j.count('[1,2,3]'), 4);          // arr + 3 elements
  assert.equal(j.count('42'), 1);
});

test('get_number', () => {
  assert.equal(j.get_number('{"version":97}', 'version'), 97);
  assert.equal(j.get_number('{"x":-42}', 'x'), -42);
  assert.equal(j.get_number('{"n":0}', 'n'), 0);
  // Missing key returns 0
  assert.equal(j.get_number('{"a":1}', 'missing'), 0);
  assert.equal(j.get_number('{}', 'key'), 0);
});

test('has_key', () => {
  assert.equal(j.has_key('{"a":1,"b":2}', 'a'), true);
  assert.equal(j.has_key('{"a":1}', 'b'), false);
  assert.equal(j.has_key('[1,2]', 'a'), false);
  assert.equal(j.has_key('{}', 'key'), false);
});

test('get_bool', () => {
  assert.equal(j.get_bool('{"ok":true}', 'ok'), 1);
  assert.equal(j.get_bool('{"ok":false}', 'ok'), 0);
  assert.equal(j.get_bool('{"a":1}', 'ok'), -1);   // missing key
  assert.equal(j.get_bool('{"a":1}', 'a'), -1);     // non-bool value
});

test('get — raw JSON value at key', () => {
  const obj = '{"name":"BMB","version":97,"tags":["fast","safe"],"active":true,"score":null}';
  assert.equal(j.get(obj, 'name'), '"BMB"');
  assert.equal(j.get(obj, 'version'), '97');
  assert.equal(j.get(obj, 'tags'), '["fast","safe"]');
  assert.equal(j.get(obj, 'active'), 'true');
  assert.equal(j.get(obj, 'score'), 'null');
  assert.equal(j.get(obj, 'missing'), '');
});

test('get_string — unquoted string at key', () => {
  assert.equal(j.get_string('{"name":"BMB"}', 'name'), 'BMB');
  assert.equal(j.get_string('{"msg":"hello world"}', 'msg'), 'hello world');
  assert.equal(j.get_string('{"name":"BMB"}', 'other'), '');
  const obj = '{"first":"Alice","last":"Smith"}';
  assert.equal(j.get_string(obj, 'first'), 'Alice');
  assert.equal(j.get_string(obj, 'last'), 'Smith');
});

test('array_get — element by index', () => {
  const arr = '[10,20,30]';
  assert.equal(j.array_get(arr, 0), '10');
  assert.equal(j.array_get(arr, 1), '20');
  assert.equal(j.array_get(arr, 2), '30');
  assert.equal(j.array_get(arr, 3), '');  // out of bounds
  assert.equal(j.array_get('[]', 0), ''); // empty array

  const strArr = '["a","b","c"]';
  assert.equal(j.array_get(strArr, 0), '"a"');
  assert.equal(j.array_get(strArr, 2), '"c"');

  const mixed = '[1,true,null,"x"]';
  assert.equal(j.array_get(mixed, 0), '1');
  assert.equal(j.array_get(mixed, 1), 'true');
  assert.equal(j.array_get(mixed, 2), 'null');
  assert.equal(j.array_get(mixed, 3), '"x"');
});

test('integration — nested extraction', () => {
  const doc = '{"items":[1,2,3],"meta":{"count":3}}';
  assert.equal(j.validate(doc), true);
  assert.equal(j.get_type(doc), 'object');

  const rawArr = j.get(doc, 'items');
  assert.equal(j.get_type(rawArr), 'array');
  assert.equal(j.array_len(rawArr), 3);
  assert.equal(j.array_get(rawArr, 0), '1');

  const rawMeta = j.get(doc, 'meta');
  assert.equal(j.get_number(rawMeta, 'count'), 3);
});

test('integration — stringify then query', () => {
  const raw = '{ "x" : 1 , "y" : 2 }';
  const min = j.stringify(raw);
  assert.equal(j.get_number(min, 'x'), 1);
  assert.equal(j.get_number(min, 'y'), 2);
  assert.equal(j.has_key(min, 'x'), true);
  assert.equal(j.has_key(min, 'z'), false);
});
